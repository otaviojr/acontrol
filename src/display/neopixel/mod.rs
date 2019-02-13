use display::{Display, DisplayState, ErrorType};

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;

use std::thread;
use std::sync::mpsc;
use std::time::{Duration,Instant};

use std::io::prelude::*;

use std::io;
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::{RawFd,AsRawFd};
use std::ptr;
use std::mem;

use nix::sys::ioctl;

mod neopixel_ioctl {
  const NEOPIXEL_IOC_MAGIC: u8 = b'N';
  const NEOPIXEL_IOCTL_GET_VERSION: u8 = 1;
  const NEOPIXEL_IOCTL_GET_NUM_LEDS: u8 = 2;
  const NEOPIXEL_IOCTL_SET_PIXEL: u8 = 3;
  const NEOPIXEL_IOCTL_SHOW: u8 = 4;  
  const NEOPIXEL_IOCTL_HARDWARE_TEST: u8 = 5;  

  #[repr(C)]
  pub struct Pixel {
      pub pixel: i32,
      pub red: u8,
      pub green: u8,
      pub blue: u8
  }

  ioctl_read_buf!(get_version, NEOPIXEL_IOC_MAGIC, NEOPIXEL_IOCTL_GET_VERSION, u8);
  ioctl_read!(get_num_leds, NEOPIXEL_IOC_MAGIC, NEOPIXEL_IOCTL_GET_NUM_LEDS, libc::c_long);
  ioctl_write_ptr!(set_pixel, NEOPIXEL_IOC_MAGIC, NEOPIXEL_IOCTL_SET_PIXEL, libc::c_long);
  ioctl_read!(show, NEOPIXEL_IOC_MAGIC, NEOPIXEL_IOCTL_SHOW, libc::c_long);
  ioctl_read!(hardware_test, NEOPIXEL_IOC_MAGIC, NEOPIXEL_IOCTL_HARDWARE_TEST, libc::c_long);
}

pub enum NeoPixelThreadCommand {
  Stop,
}

pub struct NeoPixelInterface {
  animation: Option<std::thread::JoinHandle<Result<(), String>>>,
  animation_tx: Option<mpsc::Sender<NeoPixelThreadCommand>>,
  animation_rx: Option<mpsc::Receiver<NeoPixelThreadCommand>>,
  driver_fd: Option<RawFd>,
  num_leds: Option<i32>
}

pub struct NeoPixel {
  devfile: Option<std::fs::File>,
  interface: Arc<Mutex<NeoPixelInterface>>,
}

impl NeoPixelInterface {

  fn get_num_leds(&mut self) -> Result<i32, String> {

    if let Some(num_leds) = self.num_leds {
      return Ok(num_leds);
    }

    let mut ret: i32 = 0;
    
    unsafe {
      if let Err(err) = neopixel_ioctl::get_num_leds(self.driver_fd.unwrap(), &mut ret){
        return Err(format!("Error getting num of leds: {}", err));
      } else {
        self.num_leds = Some(ret);
      }
      println!("get_num_leds {}", ret);
    }

    Ok(ret)
  }

  fn set_pixel(&mut self, pixel_info: neopixel_ioctl::Pixel) -> Result<(), String> {
    unsafe {
      let pixel: *mut libc::c_long = mem::transmute(&pixel_info);
      if let Err(err) = neopixel_ioctl::set_pixel(self.driver_fd.unwrap(), pixel) {
        return Err(format!("Error getting num of leds: {}", err));
      }
    }
    Ok(())
  }

  fn show(&mut self) -> Result<i32, String> {
    let mut ret: i32 = 0;
    unsafe {
      if let Err(err) = neopixel_ioctl::show(self.driver_fd.unwrap(), &mut ret){
        return Err(format!("Error getting num of leds: {}", err));
      }
    }
    Ok(ret)
  }

//  fn color_wipe(&mut self, red: u8, green: u8, blue: u8) -> Result<(), String> {
//    let mut ret: i32 = 0;
//    for i in 0..16 {
//      unsafe {
//        let pixel: *mut libc::c_long = mem::transmute(&mut neopixel_ioctl::Pixel { pixel: i, red: red, green: green, blue: blue});
//        neopixel_ioctl::set_pixel(self.driver_fd.unwrap(), pixel);
//        neopixel_ioctl::show(self.driver_fd.unwrap(), &mut ret);
//        thread::sleep(Duration::from_millis(50));
//      }
//    }
//    Ok(())
//  }
}

impl NeoPixel {

  pub fn new() -> Self {
    return NeoPixel { devfile:  None, interface: Arc::new( Mutex::new( NeoPixelInterface { animation: None, driver_fd: None, animation_tx: None, animation_rx: None, num_leds: None } ) )  };
  }

  fn set_driver_fd(&mut self, devfile: Option<RawFd>) {
    let interface = self.interface.clone();
    interface.lock().unwrap().driver_fd = devfile;
  }

  fn get_driver_fd(&mut self) -> Option<RawFd> {
    let interface = self.interface.clone();
    return interface.lock().unwrap().driver_fd;
  }

  fn stop_animation(&mut self) -> Result<(), String> {
    let interface = self.interface.clone();
    let mut interface_locked = interface.lock().unwrap();

    if let Some(tx) = interface_locked.animation_tx.take() {
      tx.send(NeoPixelThreadCommand::Stop);
      drop(tx);
    }

    if let Some(animation) = interface_locked.animation.take() {
      println!("Waiting current animation ends");
      animation.join();
    }

    Ok(())
  }

  fn run_animation<F, P, F1>(&mut self, f: F, params: Box<P>, finish: F1) -> Result<(), String> where
                     F: Fn(&mut P) -> Result<(bool), String> + Send + Sync + 'static,
                     F1: Fn(bool) -> Result<(),String> + Send + Sync + 'static,
                     P: Sync + Send + 'static {

    self.stop_animation();

    let interface = self.interface.clone();
    let mut interface_locked = interface.lock().unwrap();

    let (tx,rx) = mpsc::channel::<NeoPixelThreadCommand>();

    interface_locked.animation_tx = Some(tx);
    interface_locked.animation_rx = Some(rx);

    let interface = self.interface.clone();

    let animation = Some(thread::spawn( move || {
      unsafe {
        let mut next = true;
        let mut p = Box::from_raw(Box::into_raw(params));

        loop {
          if Ok(false) == f(&mut *p) {
            break;
          }

          if let Some(ref rx) = interface.lock().unwrap().animation_rx {
            if let Ok(msg) = rx.try_recv() {
              match msg {
                NeoPixelThreadCommand::Stop => {
                  println!("Neopixel Animation Thread forced to exit");
                  next = false;
                  break;
                }
              }
            }
          }
        }
        if let Err(err) = finish(next) {
          return Err(err);
        }
      }
      Ok(())
    }));

    interface_locked.animation = animation;

    Ok(())
  }

  fn animation_color_wipe<F>(&mut self, red: u8, green: u8, blue: u8, finish: F) -> Result<(), String> 
				where F: Fn(bool) -> Result<(), String> + Send + Sync + 'static {
    let interface = self.interface.clone();

    let animation_fn = move |pixel: &mut i32| {
      let pixel_info = neopixel_ioctl::Pixel { pixel: *pixel, red: red, green: green, blue: blue};
      let mut interface_locked = interface.lock().unwrap();

      interface_locked.set_pixel(pixel_info);
      interface_locked.show();
      *pixel += 1;
      Ok( if *pixel >= interface_locked.get_num_leds().unwrap() { false } else { true } )
    };

    let mut pixel: i32 = 0;
    self.run_animation(animation_fn, Box::new(pixel), finish);

    Ok(())
  }

  fn test_hardware(&mut self) -> Result<(), String> {
      self.animation_color_wipe(255, 0, 0, move |next: bool| {
        Ok(())
      });
    Ok(())
  }
}

impl Display for NeoPixel {
  fn init(&mut self) -> Result<(), String> {

    self.devfile = match OpenOptions::new()
                           .read(true)
                           .write(true)
                           .create(false)
                           .open("/dev/neopixel") {
      Ok(file) => {
        self.set_driver_fd(Some(file.as_raw_fd()));
        Some(file)
      },
      Err(err) => return Err(format!("Error opening neopixel kernel driver: {}", err))
    };

    let mut version:[u8;6] = [0;6];
    unsafe {
      if let Err(error) = neopixel_ioctl::get_version(self.get_driver_fd().unwrap(), &mut version) {
        println!("Error get neopixel driver version {}", error);
        return Err(format!("{}","NeoPixel device driver not found!"));
      }
    }

    println!("NeoPixel driver version {} found!", String::from_utf8(version.to_vec()).unwrap());

    self.test_hardware();
 
    Ok(())
  }

  fn show_success(&mut self, message: &str, dismiss: i32) -> Result<(), String>{
    Ok(())
  }

  fn show_error(&mut self, message: &str, error_type: ErrorType, dismiss: i32) -> Result<(), String> {
    Ok(())
  }

  fn show_waiting(&mut self, message: &str, dismiss: i32) -> Result<(), String> {
    Ok(())
  }


  fn unload(&mut self) -> Result<(), String> {
    Ok(())
  }

  fn signature(&self) -> String {
    return String::from("NeoPixel display module");
  }
}

unsafe impl Send for NeoPixel {}
unsafe impl Sync for NeoPixel {}
