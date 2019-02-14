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

pub enum NeoPixelAnimationDirection {
  Normal,
  Backwards,
}

pub struct NeoPixelWipeAnimation {
  direction: NeoPixelAnimationDirection,
  current_pixel: i32,
  repeat: i32,
  infinite: bool,
  red: u8,
  green: u8,
  blue: u8,
  custom: Option<Box<std::any::Any + Send + Sync>>,
}

pub struct NeoPixelSpinnerAnimation {
  current_pixel: i32,
  interaction: i32,
  size: i32,
  size_direction: i32,
  red: u8,
  green: u8,
  blue: u8,
  custom: Option<Box<std::any::Any + Send + Sync>>,
}

impl NeoPixelWipeAnimation {
  fn simple(red: u8, green: u8, blue: u8) -> Self {
    return NeoPixelWipeAnimation {direction: NeoPixelAnimationDirection::Normal, repeat: 0, infinite: false, current_pixel: 0, red: red, green: green, blue: blue, custom: None};
  }

  fn repeat(red: u8, green: u8, blue: u8, repeat: i32) -> Self {
    return NeoPixelWipeAnimation {direction: NeoPixelAnimationDirection::Normal, repeat: repeat, infinite: false, current_pixel: 0, red: red, green: green, blue: blue, custom: None};
  }
}

impl NeoPixelSpinnerAnimation {
  fn new(red: u8, green: u8, blue: u8) -> Self {
    return NeoPixelSpinnerAnimation {current_pixel: 0, interaction: 0, size_direction: 0, size: 2, red: red, green: green, blue: blue, custom: None};
  }
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
                     F1: Fn(bool, &mut P) -> Result<(u64),String> + Send + Sync + Copy + 'static,
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
        let mut wait: u64 = 0;

        loop {
          if Ok(false) == f(&mut *p) {
            next = false;
          }

          if let Some(ref rx) = interface.lock().unwrap().animation_rx {
            if let Ok(msg) = rx.try_recv() {
              match msg {
                NeoPixelThreadCommand::Stop => {
                  println!("Neopixel Animation Thread forced to exit");
                  next = false;
                }
              }
            }
          }

          match finish(next, &mut *p) {
            Ok(timing) => wait = timing,
            Err(err) => return Err(err),
          }

          if next == false  { break; }
	  thread::sleep(Duration::from_millis(wait));
        }
      }
      Ok(())
    }));

    interface_locked.animation = animation;

    Ok(())
  }

  fn animation_spinner<F>(&mut self, info: NeoPixelSpinnerAnimation, finish: F) -> Result<(), String>
                                        where F: Fn(bool, &mut NeoPixelSpinnerAnimation) -> Result<(u64), String> + Send + Sync + Copy + 'static {

    let interface = self.interface.clone();

    let animation_fn = move |animation_info: &mut NeoPixelSpinnerAnimation| {

      let mut interface_locked = interface.lock().unwrap();

      if let Ok(num_leds) = interface_locked.get_num_leds() {

        if animation_info.interaction % 3 == 0 {
          animation_info.current_pixel += 1;
        }

        if animation_info.size < num_leds-4 && animation_info.size_direction == 0 {
          animation_info.size += 1;
        } else if animation_info.size >= num_leds-4 && animation_info.size_direction == 0 {
          animation_info.size_direction = 1;
        } else if animation_info.size > 2 && animation_info.size_direction == 1 {
          animation_info.size -= 1;
          animation_info.current_pixel += 1;
        } else {
          animation_info.size_direction = 0;
        }

        if animation_info.current_pixel >= num_leds { animation_info.current_pixel -= num_leds }
        animation_info.interaction += 1;

        for i in animation_info.current_pixel..(animation_info.current_pixel + animation_info.size) {
          let mut pixel = i;
          if pixel >= num_leds { pixel -= num_leds}
          let pixel_info = neopixel_ioctl::Pixel { pixel: pixel, red: animation_info.red, green: animation_info.green, blue: animation_info.blue};
          interface_locked.set_pixel(pixel_info);
        }

        let mut pixel:i32 = animation_info.current_pixel + animation_info.size;
        while pixel != animation_info.current_pixel {
          let pixel_info = neopixel_ioctl::Pixel { pixel: pixel, red: 0, green: 0, blue: 0};
          interface_locked.set_pixel(pixel_info);
          pixel += 1;
          if pixel >= num_leds { pixel -= num_leds} 
        }

        interface_locked.show();
      }
      return Ok(true);
    };

    self.run_animation(animation_fn, Box::new(info), finish);

    Ok(())
  }

  fn animation_color_wipe<F>(&mut self, info: NeoPixelWipeAnimation, finish: F) -> Result<(), String>
					where F: Fn(bool, &mut NeoPixelWipeAnimation) -> Result<(u64), String> + Send + Sync + Copy + 'static {

    let interface = self.interface.clone();

    let animation_fn = move |animation_info: &mut NeoPixelWipeAnimation| {
      let pixel_info = neopixel_ioctl::Pixel { pixel: animation_info.current_pixel, red: animation_info.red, green: animation_info.green, blue: animation_info.blue};
      let mut interface_locked = interface.lock().unwrap();

      interface_locked.set_pixel(pixel_info);
      interface_locked.show();
      animation_info.current_pixel += 1;

      if animation_info.current_pixel >= interface_locked.get_num_leds().unwrap() {
        if animation_info.repeat > 0 {
          animation_info.repeat -= 1;
          animation_info.current_pixel = 0;
        } else {
          return Ok(false);
        }
      }
      return Ok(true);
    };
    
    self.run_animation(animation_fn, Box::new(info), finish);

    Ok(())
  }

  fn test_hardware(&mut self) -> Result<(), String> {
    //let mut animation_info = NeoPixelWipeAnimation::repeat(255,0,0,3);
    let mut animation_info = NeoPixelSpinnerAnimation::new(255,0,0);

    self.animation_spinner(animation_info, |next: bool, params: &mut NeoPixelSpinnerAnimation| {
      Ok(100)
    });

    //self.animation_color_wipe(animation_info, |next: bool, params:&mut NeoPixelWipeAnimation| {
    //  if next == true {
    //    match params.repeat {
    //      3 => {
    //        params.red = 255;
    //        params.green = 0;
    //        params.blue = 0;
    //      },
    //      2 => {
    //        params.red = 0;
    //        params.green = 255;
    //        params.blue = 0;
    //      },
    //      1 => {
    //        params.red = 0;
    //        params.green = 0;
    //        params.blue = 255;
    //      },
    //      _ => {
    //        params.red = 0;
    //        params.green = 0;
    //        params.blue = 0;
    //      }
    //    }
    //  }

    //  Ok(150)
    //});

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
