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

use nix::sys::ioctl;

mod neopixel_ioctl {
  const NEOPIXEL_IOC_MAGIC: u8 = '0' as u8;
  const NEOPIXEL_IOCTL_GET_VERSION: u8 = 1;

  ioctl_read_buf!(get_version, NEOPIXEL_IOC_MAGIC, NEOPIXEL_IOCTL_GET_VERSION, u8);
}

pub struct NeoPixel {
  driver_fd: Option<RawFd>
}

impl NeoPixel {

  pub fn new() -> Self {
    return NeoPixel { driver_fd: None };
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

    let mut version: Vec<u8> = Vec::new();
    unsafe {
      if let Err(error) = neopixel_ioctl::get_version(self.driver_fd.unwrap(), &mut version) {
        println!("Error get neopixel driver version {}", error);
      }
    }

    println!("NeoPixel driver version {:?} found!", String::from_utf8(version));

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
