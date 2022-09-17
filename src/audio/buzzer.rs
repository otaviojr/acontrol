/**
 * @file   buzzer/mod.rs
 * @author Otavio Ribeiro
 * @date   24 Dec 2017
 * @brief  Buzzer driver. Depends on the buzzer/pcmlinux kernel driver
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

mod sounds;

use super::{Audio};

use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::sync::mpsc;
use std::time::Duration;

use std::fs::OpenOptions;                                                                                                                         
use std::os::unix::io::{RawFd,AsRawFd};                                                                                                           
use std::mem;

use super::buzzer::sounds::{Sounds, Tone};

#[allow(dead_code)]
#[allow(non_camel_case_types)]
mod buzzer_ioctl {
  const BUZZER_IOC_MAGIC: u8 = b'B';
  const BUZZER_IOCTL_GET_VERSION: u8 = 1;
  const BUZZER_IOCTL_PLAY_TONE: u8 = 2;

  #[repr(C)]
  pub struct BuzzerTone {
      pub freq: i32,
      pub period: i32
  }

  nix::ioctl_read_buf!(get_version, BUZZER_IOC_MAGIC, BUZZER_IOCTL_GET_VERSION, u8);
  nix::ioctl_write_ptr!(play_tone, BUZZER_IOC_MAGIC, BUZZER_IOCTL_PLAY_TONE, libc::c_long);
}

#[derive(Clone, Copy, Debug)]
pub enum AudioThreadCommand {
  Stop,
}

struct BuzzerThreadSafe {
  driver_fd: Mutex<Option<RawFd>>,
  sound_worker_tx: Mutex<mpsc::Sender<AudioThreadCommand>>,
  sound_worker_rx: Mutex<mpsc::Receiver<AudioThreadCommand>>,
}

impl BuzzerThreadSafe {

  fn play_tone_async(&self, tone: &buzzer_ioctl::BuzzerTone) -> Result<(), String> {
    unsafe {
      let tone_param: *mut libc::c_long = mem::transmute(tone);
      if let Some(ref driver_fd) = self.get_driver_fd() {
        if let Err(err) = buzzer_ioctl::play_tone(*driver_fd, tone_param) {
          return Err(format!("Error playing tone: {}", err));
        }
      }
    }
    Ok(())
  }

  pub fn play_tone(&self, freq: Tone, period: i32) -> Result<(), String> {
    let tone: buzzer_ioctl::BuzzerTone = buzzer_ioctl::BuzzerTone { freq: freq.value(), period: period };
    let ret = self.play_tone_async(&tone);
    thread::sleep(Duration::from_millis(period as u64));
    ret
  }

  fn set_driver_fd(&self, devfile: Option<RawFd>) {
    let mut driver_fd_locked = self.driver_fd.lock().unwrap();
    *driver_fd_locked = devfile;
  }

  fn get_driver_fd(&self) -> Option<RawFd> {
    let driver_fd_locked = self.driver_fd.lock().unwrap();
    return *driver_fd_locked;
  }
}

unsafe impl Sync for BuzzerThreadSafe {}

unsafe impl Send for BuzzerThreadSafe {}

pub struct Buzzer {
  devfile: Option<std::fs::File>,
  buzzer: Arc<Mutex<BuzzerThreadSafe>>,
  sound_worker: Option<std::thread::JoinHandle<Result<(), String>>>,
}

impl Buzzer {
  pub fn new() -> Self {
    let (tx,rx):(mpsc::Sender<AudioThreadCommand>, mpsc::Receiver<AudioThreadCommand>) = mpsc::channel::<AudioThreadCommand>();
    return Buzzer {sound_worker: None, devfile: None, buzzer: Arc::new(Mutex::new(BuzzerThreadSafe {sound_worker_rx: Mutex::new(rx), sound_worker_tx: Mutex::new(tx), driver_fd: Mutex::new(None)}))};
  }
}

impl Drop for Buzzer {
  fn drop(&mut self) {
    println!("Unloading audio driver");
    let _res = self.unload();
  }
}

impl Audio for Buzzer {
  fn init(&mut self) -> Result<(), String> {
    let buzzer = self.buzzer.clone();

    if let Ok(buzzer_locked) = buzzer.lock() {
      self.devfile = match OpenOptions::new()
  				.read(true)
  				.write(true)
 				.create(false)
				.open("/dev/buzzer") {
        Ok(file) => {
          buzzer_locked.set_driver_fd(Some(file.as_raw_fd()));
         Some(file)
        }, 
        Err(err) => return Err(format!("Error opening buzzer kernel driver: {}", err))
      };

      let mut version:[u8;6] = [0;6];
      unsafe {
        if let Some(ref driver_fd) = buzzer_locked.get_driver_fd() {
          if let Err(error) = buzzer_ioctl::get_version(*driver_fd, &mut version) {
            println!("Error get buzzer driver version {}", error);
            return Err(format!("{}","Buzzer device driver not found!"));
          }
        }
      }
      println!("Buzzer driver version {} found!", String::from_utf8(version.to_vec()).unwrap());
    }

    let _ret = Sounds::play_piratescaribean(self);
    
    Ok(())
  }

  fn play_new(&mut self) -> Result<(), String> {
    Sounds::play_supermario(self)
  }

  fn play_granted(&mut self) -> Result<(), String> {
    Sounds::play_supermario(self)
  }

  fn play_denied(&mut self) -> Result<(), String> {
    Sounds::play_starwars(self)
  }

  fn play_success(&mut self) -> Result<(), String> {
    Sounds::play_supermario(self)
  }

  fn play_error(&mut self) -> Result<(), String> {
    Sounds::play_starwars(self)
  }

  fn play_alert(&mut self) -> Result<(), String> {
   Sounds::play_bip(self)
  }


  fn unload(&mut self) -> Result<(), String>{
    println!("Audio driver unloading");
    Ok(())
  }

  fn signature(&self) -> String {
    return String::from("Buzzer Audio Module");
  }
}

unsafe impl Send for Buzzer {}

unsafe impl Sync for Buzzer {}
