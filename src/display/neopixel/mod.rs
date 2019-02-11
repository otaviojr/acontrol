use display::{Display, DisplayState};

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;

use std::thread;
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
      pub pixel: u32,
      pub red: u8,
      pub green: u8,
      pub blue: u8
  }

  ioctl_read_buf!(get_version, NEOPIXEL_IOC_MAGIC, NEOPIXEL_IOCTL_GET_VERSION, u8);
  ioctl_read!(get_num_pixels, NEOPIXEL_IOC_MAGIC, NEOPIXEL_IOCTL_GET_NUM_LEDS, libc::c_long);
  ioctl_write_ptr!(set_pixel, NEOPIXEL_IOC_MAGIC, NEOPIXEL_IOCTL_SET_PIXEL, libc::c_long);
  ioctl_read!(show, NEOPIXEL_IOC_MAGIC, NEOPIXEL_IOCTL_SHOW, libc::c_long);
  ioctl_read!(hardware_test, NEOPIXEL_IOC_MAGIC, NEOPIXEL_IOCTL_HARDWARE_TEST, libc::c_long);
}

pub struct NeoPixel {
  driver_fd: Option<RawFd>
}

impl NeoPixel {

  pub fn new() -> Self {
    return NeoPixel { driver_fd: None };
  }

  fn color_wipe(&mut self, red: u8, green: u8, blue: u8) -> Result<(), String> {
    let mut ret: i32 = 0;
    for i in 0..16 {
      unsafe {
        let pixel: *mut libc::c_long = mem::transmute(&mut neopixel_ioctl::Pixel { pixel: i, red: red, green: green, blue: blue});
        neopixel_ioctl::set_pixel(self.driver_fd.unwrap(), pixel);
        neopixel_ioctl::show(self.driver_fd.unwrap(), &mut ret);
	thread::sleep(Duration::from_millis(1000)); 
      }
    }
    Ok(())
  }

  fn test_hardware(&mut self) -> Result<(), String> {
    let stage:u8 = 0;

//    let _handler = thread::spawn( move || {      
      for stage in 0..3 {
        self.color_wipe( if stage == 0 { 255 } else { 0 }, if stage == 1 { 255 } else {0}, if stage == 2 { 255 } else { 0 } );
      }
//    });

    Ok(())
  }
}

impl Display for NeoPixel {
  fn init(&mut self) -> Result<(), String> {    

    let devfile = match OpenOptions::new()
                           .read(true)
                           .write(true)
                           .create(false)
                           .open("/dev/neopixel") {
      Ok(file) => file,
      Err(err) => return Err(format!("Error opening neopixel kernel driver: {}", err))
    };

    self.driver_fd = Some(devfile.as_raw_fd());

    let mut version:[u8;6] = [0;6];

    unsafe {
      if let Err(error) = neopixel_ioctl::get_version(self.driver_fd.unwrap(), &mut version) {
        println!("Error get neopixel driver version {}", error);
        return Err(format!("{}","NeoPixel device driver not found!"));
      }
    }

    println!("NeoPixel driver version {} found!", String::from_utf8(version.to_vec()).unwrap());

    unsafe {
      let mut ret: libc::c_long = 0;
      if let Err(error) = neopixel_ioctl::hardware_test(self.driver_fd.unwrap(), &mut ret) {
        println!("Error executing neopixel hardware test: {}", error);
        return Err(format!("{}","Error executing hardware test!"));
      }
    }

    //self.test_hardware();
 
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
