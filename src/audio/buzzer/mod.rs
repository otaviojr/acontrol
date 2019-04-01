use audio::{Audio};

use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use std::io;                                                                                                                                      
use std::fs::OpenOptions;                                                                                                                         
use std::os::unix::io::{RawFd,AsRawFd};                                                                                                           
use std::ptr;                                                                                                                                     
use std::mem;

#[allow(dead_code)]                                                                                                                               
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

struct BuzzerThreadSafe {
  driver_fd: Mutex<Option<RawFd>>
}

impl BuzzerThreadSafe {

  fn play_tone(&self, tone: &buzzer_ioctl::BuzzerTone) -> Result<(), String> {
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

  fn set_driver_fd(&self, devfile: Option<RawFd>) {
    let mut driver_fd_locked = self.driver_fd.lock().unwrap();
    *driver_fd_locked = devfile;
  }

  fn get_driver_fd(&self) -> Option<RawFd> {
    let mut driver_fd_locked = self.driver_fd.lock().unwrap();
    return *driver_fd_locked;
  }
}

unsafe impl Sync for BuzzerThreadSafe {}

unsafe impl Send for BuzzerThreadSafe {}

pub struct Buzzer {
  devfile: Option<std::fs::File>,
  buzzer: Arc<Mutex<BuzzerThreadSafe>>
}

impl Buzzer {
  pub fn new() -> Self {
    return Buzzer {devfile: None, buzzer: Arc::new(Mutex::new(BuzzerThreadSafe {driver_fd: Mutex::new(None)}))};
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

    self.play_error();
    
    Ok(())
  }

  fn play_success(&mut self) -> Result<(), String> {
    let buzzer = self.buzzer.clone();
    let _handler = thread::spawn(move || {
      //buzzer.lock().unwrap().set_buzz(true);
      //thread::sleep(Duration::from_millis(1000));
      //buzzer.lock().unwrap().set_buzz(false);
    });

    Ok(())
  }

  fn play_error(&mut self) -> Result<(), String> {
    let buzzer = self.buzzer.clone();
    let _handler = thread::spawn(move || {

      let mut tone: buzzer_ioctl::BuzzerTone = buzzer_ioctl::BuzzerTone { freq: 261, period: 600 };
      buzzer.lock().unwrap().play_tone(&tone);
      thread::sleep(Duration::from_millis(600));

      tone = buzzer_ioctl::BuzzerTone { freq: 294, period: 600 };
      buzzer.lock().unwrap().play_tone(&tone);
      thread::sleep(Duration::from_millis(600));

      tone = buzzer_ioctl::BuzzerTone { freq: 329, period: 600 };
      buzzer.lock().unwrap().play_tone(&tone);
      thread::sleep(Duration::from_millis(600));

      tone = buzzer_ioctl::BuzzerTone { freq: 349, period: 600 };
      buzzer.lock().unwrap().play_tone(&tone);
      thread::sleep(Duration::from_millis(600));

      tone = buzzer_ioctl::BuzzerTone { freq: 349, period: 400 };
      buzzer.lock().unwrap().play_tone(&tone);
      thread::sleep(Duration::from_millis(500));

      tone = buzzer_ioctl::BuzzerTone { freq: 349, period: 400 };
      buzzer.lock().unwrap().play_tone(&tone);
      thread::sleep(Duration::from_millis(500));

      tone = buzzer_ioctl::BuzzerTone { freq: 261, period: 600 };
      buzzer.lock().unwrap().play_tone(&tone);
      thread::sleep(Duration::from_millis(600));

      tone = buzzer_ioctl::BuzzerTone { freq: 294, period: 600 };
      buzzer.lock().unwrap().play_tone(&tone);
      thread::sleep(Duration::from_millis(600));

      tone = buzzer_ioctl::BuzzerTone { freq: 261, period: 600 };
      buzzer.lock().unwrap().play_tone(&tone);
      thread::sleep(Duration::from_millis(600));

      tone = buzzer_ioctl::BuzzerTone { freq: 294, period: 600 };
      buzzer.lock().unwrap().play_tone(&tone);
      thread::sleep(Duration::from_millis(600));

      tone = buzzer_ioctl::BuzzerTone { freq: 294, period: 400 };
      buzzer.lock().unwrap().play_tone(&tone);
      thread::sleep(Duration::from_millis(500));

      tone = buzzer_ioctl::BuzzerTone { freq: 294, period: 400 };
      buzzer.lock().unwrap().play_tone(&tone);
      thread::sleep(Duration::from_millis(500));

      tone = buzzer_ioctl::BuzzerTone { freq: 261, period: 600 };
      buzzer.lock().unwrap().play_tone(&tone);
      thread::sleep(Duration::from_millis(600));

      tone = buzzer_ioctl::BuzzerTone { freq: 392, period: 600 };
      buzzer.lock().unwrap().play_tone(&tone);
      thread::sleep(Duration::from_millis(600));

      tone = buzzer_ioctl::BuzzerTone { freq: 349, period: 600 };
      buzzer.lock().unwrap().play_tone(&tone);
      thread::sleep(Duration::from_millis(600));

      tone = buzzer_ioctl::BuzzerTone { freq: 329, period: 600 };
      buzzer.lock().unwrap().play_tone(&tone);
      thread::sleep(Duration::from_millis(600));

      tone = buzzer_ioctl::BuzzerTone { freq: 329, period: 400 };
      buzzer.lock().unwrap().play_tone(&tone);
      thread::sleep(Duration::from_millis(500));

      tone = buzzer_ioctl::BuzzerTone { freq: 329, period: 400 };
      buzzer.lock().unwrap().play_tone(&tone);
      thread::sleep(Duration::from_millis(500));

      tone = buzzer_ioctl::BuzzerTone { freq: 261, period: 600 };
      buzzer.lock().unwrap().play_tone(&tone);
      thread::sleep(Duration::from_millis(600));

      tone = buzzer_ioctl::BuzzerTone { freq: 294, period: 600 };
      buzzer.lock().unwrap().play_tone(&tone);
      thread::sleep(Duration::from_millis(600));

      tone = buzzer_ioctl::BuzzerTone { freq: 329, period: 600 };
      buzzer.lock().unwrap().play_tone(&tone);
      thread::sleep(Duration::from_millis(600));

      tone = buzzer_ioctl::BuzzerTone { freq: 349, period: 600 };
      buzzer.lock().unwrap().play_tone(&tone);
      thread::sleep(Duration::from_millis(600));

      tone = buzzer_ioctl::BuzzerTone { freq: 349, period: 400 };
      buzzer.lock().unwrap().play_tone(&tone);
      thread::sleep(Duration::from_millis(500));

      tone = buzzer_ioctl::BuzzerTone { freq: 349, period: 400 };
      buzzer.lock().unwrap().play_tone(&tone);
      thread::sleep(Duration::from_millis(500));
    });

    Ok(())
  }

  fn play_alert(&mut self) -> Result<(), String> {
    let buzzer = self.buzzer.clone();
    let _handler = thread::spawn(move || {
      //buzzer.lock().unwrap().set_buzz(true);
      //thread::sleep(Duration::from_millis(300));
      //buzzer.lock().unwrap().set_buzz(false);
    });

    Ok(())
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
