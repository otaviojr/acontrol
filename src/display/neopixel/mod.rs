/**
 * @file   neopixel/mod.rs
 * @author Otavio Ribeiro
 * @date   24 Dec 2017
 * @brief  Neopixel driver. Depends on the neopixel/pwm linux kernel driver.
 *
 * Copyright (c) 2019 Otávio Ribeiro <otavio.ribeiro@gmail.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
 * THE SOFTWARE.
 *
 */
use display::{Display, Animation, AnimationType, AnimationColor};

use std::sync::Arc;
use std::sync::Mutex;

use std::thread;
use std::sync::mpsc;
use std::time::{Duration,Instant};

use std::fs::OpenOptions;
use std::os::unix::io::{RawFd,AsRawFd};
use std::mem;

#[allow(dead_code)]
#[allow(non_camel_case_types)]
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

  nix::ioctl_read_buf!(get_version, NEOPIXEL_IOC_MAGIC, NEOPIXEL_IOCTL_GET_VERSION, u8);
  nix::ioctl_read!(get_num_leds, NEOPIXEL_IOC_MAGIC, NEOPIXEL_IOCTL_GET_NUM_LEDS, libc::c_long);
  nix::ioctl_write_ptr!(set_pixel, NEOPIXEL_IOC_MAGIC, NEOPIXEL_IOCTL_SET_PIXEL, libc::c_long);
  nix::ioctl_read!(show, NEOPIXEL_IOC_MAGIC, NEOPIXEL_IOCTL_SHOW, libc::c_long);
  nix::ioctl_read!(hardware_test, NEOPIXEL_IOC_MAGIC, NEOPIXEL_IOCTL_HARDWARE_TEST, libc::c_long);
}

#[derive(Clone, Copy, Debug)]
pub enum NeoPixelThreadCommand {
  Stop,
}

unsafe impl Send for NeoPixelThreadCommand {}

pub struct NeoPixelInterface {
  animation: Mutex<Option<std::thread::JoinHandle<Result<(), String>>>>,
  animation_tx: Mutex<mpsc::Sender<NeoPixelThreadCommand>>,
  animation_rx: Mutex<mpsc::Receiver<NeoPixelThreadCommand>>,
  driver_fd: Mutex<Option<RawFd>>,
  num_leds: Mutex<Option<i32>>,
}

pub struct NeoPixel {
  devfile: Option<std::fs::File>,
  interface: Arc<NeoPixelInterface>,
}

#[derive(PartialEq)]
pub enum NeoPixelAnimationDirection {
  Normal,
  Backwards,
}

pub struct NeoPixelWipeAnimation {
  direction: NeoPixelAnimationDirection,
  current_pixel: i32,
  start: Instant,
  dismiss: u64,
  red: u8,
  green: u8,
  blue: u8,
//  custom: Option<Box<std::any::Any + Send + Sync>>,
}

pub struct NeoPixelSpinnerAnimation {
  current_pixel: i32,
  interaction: i32,
  size: i32,
  size_direction: i32,
  red: u8,
  green: u8,
  blue: u8,
  //custom: Option<Box<std::any::Any + Send + Sync>>,
}

pub struct NeoPixelBlinkAnimation {
  repeat: i32,
  infinity: bool,
  red: u8,
  green: u8,
  blue: u8,
//  custom: Option<Box<std::any::Any + Send + Sync>>,
}

impl NeoPixelWipeAnimation {
  fn new(red: u8, green: u8, blue: u8, dismiss: u64) -> Self {
    return NeoPixelWipeAnimation {direction: NeoPixelAnimationDirection::Normal, start: Instant::now(), dismiss: dismiss, current_pixel: 0, red: red, green: green, blue: blue, /*custom: None*/};
  }
}

impl NeoPixelSpinnerAnimation {
  fn new(red: u8, green: u8, blue: u8) -> Self {
    return NeoPixelSpinnerAnimation {current_pixel: 0, interaction: 0, size_direction: 0, size: 2, red: red, green: green, blue: blue, /*custom: None*/};
  }
}

impl NeoPixelBlinkAnimation {
  fn new(red: u8, green: u8, blue: u8, repeat: i32) -> Self {
    return NeoPixelBlinkAnimation {repeat: if repeat <= 0 { 0 } else {repeat*2}, red: red, green: green, blue: blue, /*custom: None,*/ infinity: false };
  }

  fn new_loop(red: u8, green: u8, blue: u8) -> Self  {
    return NeoPixelBlinkAnimation {repeat: 0, red: red, green: green, blue: blue, /*custom: None,*/ infinity: true  };
  }
}

impl NeoPixelInterface {

  fn get_num_leds(&self) -> Result<i32, String> {

    let mut num_leds_locked = self.num_leds.lock().unwrap();

    if let Some(ref num_leds) = *num_leds_locked {
      return Ok(*num_leds);
    }

    let mut ret: i32 = 0;

    unsafe {
      if let Some(ref driver_fd) = self.get_driver_fd(){
        if let Err(err) = neopixel_ioctl::get_num_leds(*driver_fd, &mut ret){
          return Err(format!("Error getting num of leds: {}", err));
        } else {
          *num_leds_locked = Some(ret);
        }
      }
      println!("get_num_leds {}", ret);
    }

    Ok(ret)
  }

  fn set_driver_fd(&self, devfile: Option<RawFd>) {
    let mut driver_fd_locked = self.driver_fd.lock().unwrap();
    *driver_fd_locked = devfile;
  }

  fn get_driver_fd(&self) -> Option<RawFd> {
    let driver_fd_locked = self.driver_fd.lock().unwrap();
    return *driver_fd_locked;
  }

  fn set_animation(&self, animation: Option<std::thread::JoinHandle<Result<(), String>>>) {
    let mut animation_locked = self.animation.lock().unwrap();
    (*animation_locked) = animation;
  }

  fn set_pixel(&self, pixel_info: neopixel_ioctl::Pixel) -> Result<(), String> {
    unsafe {
      let pixel: *mut libc::c_long = mem::transmute(&pixel_info);
      if let Some(ref driver_fd) = self.get_driver_fd() {
        if let Err(err) = neopixel_ioctl::set_pixel(*driver_fd, pixel) {
          return Err(format!("Error getting num of leds: {}", err));
        }
      }
    }
    Ok(())
  }

  fn show(&self) -> Result<i32, String> {
    let mut ret: i32 = 0;
    unsafe {
      if let Some(ref driver_fd) = self.get_driver_fd() {
        if let Err(err) = neopixel_ioctl::show(*driver_fd, &mut ret){
          return Err(format!("Error getting num of leds: {}", err));
        }
      }
    }
    Ok(ret)
  }

  fn clear(&self) -> Result<(), String> {

    for i in 0..self.get_num_leds().unwrap() {
      let pixel_info = neopixel_ioctl::Pixel { pixel: i, red: 0, green: 0, blue: 0};
      let _ret = self.set_pixel(pixel_info);
    }

    if let Err(err) = self.show(){
      return Err(err);
    }
    Ok(())
  }

}

impl NeoPixel {

  pub fn new() -> Self {
    let (tx,rx):(mpsc::Sender<NeoPixelThreadCommand>, mpsc::Receiver<NeoPixelThreadCommand>) = mpsc::channel::<NeoPixelThreadCommand>();
    return NeoPixel { devfile:  None, interface: Arc::new( NeoPixelInterface { animation: Mutex::new(None), driver_fd: Mutex::new(None), animation_tx: Mutex::new(tx), animation_rx: Mutex::new(rx), num_leds: Mutex::new(None) } ) };
  }

  fn stop_animation(&mut self) -> Result<(), String> {

    let interface = self.interface.clone();

    if let Some(animation) = (*interface.animation.lock().unwrap()).take() {
      match interface.animation_tx.lock().unwrap().send(NeoPixelThreadCommand::Stop) {
        Ok(_ret) => {
          let _ret = animation.join();

          // If animation thread has already leaved, nobody will receive this message
          // and it will be pending to the next thread. Clear the queue before
          // starting another thread.
          while let Ok(_ret) = interface.animation_rx.lock().unwrap().try_recv() {
          }

        },
        Err(err) => return Err(format!("Error sending message: {:?}", err))
      }
    }

    Ok(())
  }

  fn run_animation<F, P, F1>(&mut self, f: F, params: Box<P>, finish: F1) -> Result<(), String> where
                     F: Fn(&mut P) -> Result<(bool), String> + Send + Sync + 'static,
                     F1: Fn(bool, &mut P) -> Result<(i64),String> + Send + Sync + Copy + 'static,
                     P: Sync + Send + 'static {

    let _ret = self.stop_animation();

    let interface = self.interface.clone();

    let animation = thread::spawn( move || {
      unsafe {
        let mut next = true;
        let mut p = Box::from_raw(Box::into_raw(params));
        let mut wait: u64 = 0;

        loop {
          if Ok(false) == f(&mut *p) {
            next = false;
          }

          match interface.animation_rx.lock().unwrap().try_recv() {
            Ok(msg) => {
              match msg {
                NeoPixelThreadCommand::Stop => {
                  next = false;
                },
              }
            },
            Err(_) => {},
          }

          match finish(next, &mut *p) {
            Ok(timing) => if timing >= 0 { wait = timing as u64 } else { next = false },
            Err(err) => {
              let _ret = interface.clear();
              return Err(err)
            },
          }

          if next == false  {
            let _ret = interface.clear();
            break;
          }

	        thread::sleep(Duration::from_millis(wait));
        }
      }
      Ok(())
    });

    let interface = self.interface.clone();
    interface.set_animation(Some(animation));

    Ok(())
  }

  fn animation_blink<F>(&mut self, info: NeoPixelBlinkAnimation, finish: F) -> Result<(), String>
        where F: Fn(bool, &mut NeoPixelBlinkAnimation) -> Result<(i64), String> + Send + Sync + Copy + 'static {

    let interface = self.interface.clone();

    let animation_fn = move |animation_info: &mut NeoPixelBlinkAnimation| {

      if let Ok(num_leds) = interface.get_num_leds() {
        for pixel in 0..num_leds {
          let mut pixel_info: neopixel_ioctl::Pixel;

          if animation_info.repeat%2 != 0 {
            pixel_info = neopixel_ioctl::Pixel { pixel: pixel, red: animation_info.red, green: animation_info.green, blue: animation_info.blue};
          } else {
            pixel_info = neopixel_ioctl::Pixel { pixel: pixel, red: 0, green: 0, blue: 0};
          }
          let _ret = interface.set_pixel(pixel_info);
        }

        let _ret = interface.show();

        animation_info.repeat -= 1;
      }

      if animation_info.repeat >= 0 || animation_info.infinity == true {
        return Ok(true);
      } else {
        return Ok(false);
      }
    };

    self.run_animation(animation_fn, Box::new(info), finish)
  }

  fn animation_spinner<F>(&mut self, info: NeoPixelSpinnerAnimation, finish: F) -> Result<(), String>
                                        where F: Fn(bool, &mut NeoPixelSpinnerAnimation) -> Result<(i64), String> + Send + Sync + Copy + 'static {

    let interface = self.interface.clone();

    let animation_fn = move |animation_info: &mut NeoPixelSpinnerAnimation| {

      if let Ok(num_leds) = interface.get_num_leds() {

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
          let _ret = interface.set_pixel(pixel_info);
        }

        let mut pixel:i32 = animation_info.current_pixel + animation_info.size;
        while pixel != animation_info.current_pixel {
          let pixel_info = neopixel_ioctl::Pixel { pixel: pixel, red: 0, green: 0, blue: 0};
          let _ret = interface.set_pixel(pixel_info);
          pixel += 1;
          if pixel >= num_leds { pixel -= num_leds}
        }

        let _ret = interface.show();
      }
      return Ok(true);
    };

    self.run_animation(animation_fn, Box::new(info), finish)
  }

  fn animation_color_wipe<F>(&mut self, info: NeoPixelWipeAnimation, finish: F) -> Result<(), String>
					where F: Fn(bool, &mut NeoPixelWipeAnimation) -> Result<(i64), String> + Send + Sync + Copy + 'static {

    let interface = self.interface.clone();

    let animation_fn = move |animation_info: &mut NeoPixelWipeAnimation| {
      if animation_info.current_pixel >= 0 && animation_info.current_pixel < interface.get_num_leds().unwrap(){
        let pixel_info = neopixel_ioctl::Pixel { pixel: animation_info.current_pixel, red: animation_info.red, green: animation_info.green, blue: animation_info.blue};

        let _ret = interface.set_pixel(pixel_info);
        let _ret = interface.show();

        if animation_info.direction == NeoPixelAnimationDirection::Normal {
          animation_info.current_pixel += 1;
        } else {
          animation_info.current_pixel -= 1;
        }
      }

      if animation_info.direction == NeoPixelAnimationDirection::Normal &&
           animation_info.current_pixel >= interface.get_num_leds().unwrap() &&
           animation_info.start.elapsed().as_secs() > animation_info.dismiss {

        animation_info.direction = NeoPixelAnimationDirection::Backwards;
        animation_info.current_pixel = interface.get_num_leds().unwrap() - 1;
        animation_info.red = 0;
        animation_info.green = 0;
        animation_info.blue = 0;

      } else if animation_info.direction == NeoPixelAnimationDirection::Backwards &&
          animation_info.current_pixel < 0 {

        return Ok(false);

      }
      return Ok(true);
    };

    self.run_animation(animation_fn, Box::new(info), finish)
  }

  fn clear(&mut self) -> Result<(), String> {
    let interface = self.interface.clone();
    interface.clear()
  }

  fn test_hardware(&mut self) -> Result<(), String> {

    let now = Instant::now();
    let dismiss = 5;

    let animation_info = NeoPixelSpinnerAnimation::new(0,0,255);
      self.animation_spinner(animation_info, move |_next: bool, _params: &mut NeoPixelSpinnerAnimation| {

      if now.elapsed().as_secs() > dismiss && dismiss > 0 { return Ok(-1) }

      Ok(100)
    })
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
        self.interface.set_driver_fd(Some(file.as_raw_fd()));
        Some(file)
      },
      Err(err) => return Err(format!("Error opening neopixel kernel driver: {}", err))
    };

    let mut version:[u8;6] = [0;6];
    unsafe {
      if let Some(ref driver_fd) = self.interface.get_driver_fd() {
        if let Err(error) = neopixel_ioctl::get_version(*driver_fd, &mut version) {
          println!("Error get neopixel driver version {}", error);
          return Err(format!("{}","NeoPixel device driver not found!"));
        }
      }
    }

    println!("NeoPixel driver version {} found!", String::from_utf8(version.to_vec()).unwrap());

    let _ret = self.test_hardware();

    Ok(())
  }

  fn show_animation(&mut self, animation: Animation, color: AnimationColor, _animation_type: AnimationType, _message: &str, dismiss: u64) -> Result<(), String> {

    let now = Instant::now();

    match animation {
      Animation::MaterialSpinner => {
        let mut animation_info = NeoPixelSpinnerAnimation::new(((color.value() >> 16) & 0xFF) as u8, ((color.value() >> 8) & 0xFF) as u8 , (color.value() & 0xFF) as u8);
        self.animation_spinner(animation_info, move |_next: bool, _params: &mut NeoPixelSpinnerAnimation| {

          if now.elapsed().as_secs() > dismiss && dismiss > 0 { return Ok(-1) }

          Ok(100)
        })
      },
      Animation::Wipe => {
        let mut animation_info = NeoPixelWipeAnimation::new(((color.value() >> 16) & 0xFF) as u8, ((color.value() >> 8) & 0xFF) as u8 , (color.value() & 0xFF) as u8, dismiss);

        self.animation_color_wipe(animation_info, |_next: bool, _params:&mut NeoPixelWipeAnimation| {
          Ok(100)
        })
      },
      Animation::Blink  => {
        let mut animation_info = NeoPixelBlinkAnimation::new(((color.value() >> 16) & 0xFF) as u8, ((color.value() >> 8) & 0xFF) as u8 , (color.value() & 0xFF) as u8, dismiss as i32);

        self.animation_blink(animation_info, move |_next: bool, _params: &mut NeoPixelBlinkAnimation| {
          Ok(500)
        })
      },
      Animation::BlinkLoop  => {
        let mut animation_info = NeoPixelBlinkAnimation::new_loop(((color.value() >> 16) & 0xFF) as u8, ((color.value() >> 8) & 0xFF) as u8 , (color.value() & 0xFF) as u8);

        self.animation_blink(animation_info, move |_next: bool, params: &mut NeoPixelBlinkAnimation| {
          if dismiss > 0 && now.elapsed().as_secs() > dismiss  && params.repeat%2 != 0 { return Ok(-1) }
          Ok(500)
        })
      },
      _ => Err(String::from("Invalid animation"))
    }
  }

  fn wait_animation_ends(&mut self) -> Result<(), String> {
    let interface = self.interface.clone();
    if let Some(animation) = (*interface.animation.lock().unwrap()).take() {
      let _ret = animation.join();
    }
    Ok(())
  }

  fn clear_and_stop_animations(&mut self) -> Result<(), String> {
    let _ret = self.stop_animation();
    let _ret = self.clear();
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
