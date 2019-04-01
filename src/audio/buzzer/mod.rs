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

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum Tone {
  NOTE_B0  =  31,
  NOTE_C1  =  33,
  NOTE_CS1 =  35,
  NOTE_D1  =  37,
  NOTE_DS1 =  39,
  NOTE_E1  =  41,
  NOTE_F1  =  44,
  NOTE_FS1 =  46,
  NOTE_G1  =  49,
  NOTE_GS1 =  52,
  NOTE_A1  =  55,
  NOTE_AS1 =  58,
  NOTE_B1  =  62,
  NOTE_C2  =  65,
  NOTE_CS2 =  69,
  NOTE_D2  =  73,
  NOTE_DS2 =  78,
  NOTE_E2  =  82,
  NOTE_F2  =  87,
  NOTE_FS2 =  93,
  NOTE_G2  =  98,
  NOTE_GS2 =  104,
  NOTE_A2  =  110,
  NOTE_AS2 =  117,
  NOTE_B2  =  123,
  NOTE_C3  =  131,
  NOTE_CS3 =  139,
  NOTE_D3  =  147,
  NOTE_DS3 =  156,
  NOTE_E3  =  165,
  NOTE_F3  =  175,
  NOTE_FS3 =  185,
  NOTE_G3  =  196,
  NOTE_GS3 =  208,
  NOTE_A3  =  220,
  NOTE_AS3 =  233,
  NOTE_B3  =  247,
  NOTE_C4  =  262,
  NOTE_CS4 =  277,
  NOTE_D4  =  294,
  NOTE_DS4 =  311,
  NOTE_E4  =  330,
  NOTE_F4  =  349,
  NOTE_FS4 =  370,
  NOTE_G4  =  392,
  NOTE_GS4 =  415,
  NOTE_A4  =  440,
  NOTE_AS4 =  466,
  NOTE_B4  =  494,
  NOTE_C5  =  523,
  NOTE_CS5 =  554,
  NOTE_D5  =  587,
  NOTE_DS5 =  622,
  NOTE_E5  =  659,
  NOTE_F5  =  698,
  NOTE_FS5 =  740,
  NOTE_G5  =  784,
  NOTE_GS5 =  831,
  NOTE_A5  =  880,
  NOTE_AS5 =  932,
  NOTE_B5  =  988,
  NOTE_C6  =  1047,
  NOTE_CS6 =  1109,
  NOTE_D6  =  1175,
  NOTE_DS6 =  1245,
  NOTE_E6  =  1319,
  NOTE_F6  =  1397,
  NOTE_FS6 =  1480,
  NOTE_G6  =  1568,
  NOTE_GS6 =  1661,
  NOTE_A6  =  1760,
  NOTE_AS6 =  1865,
  NOTE_B6  =  1976,
  NOTE_C7  =  2093,
  NOTE_CS7 =  2217,
  NOTE_D7  =  2349,
  NOTE_DS7 =  2489,
  NOTE_E7  =  2637,
  NOTE_F7  =  2794,
  NOTE_FS7 =  2960,
  NOTE_G7  =  3136,
  NOTE_GS7 =  3322,
  NOTE_A7  =  3520,
  NOTE_AS7 =  3729,
  NOTE_B7  =  3951,
  NOTE_C8  =  4186,
  NOTE_CS8 =  4435,
  NOTE_D8  =  4699,
  NOTE_DS8 =  4978,  
}

impl Tone {
  fn value(&self) -> i32 {
    return (*self) as i32;
  }
}

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
    let mut tone: buzzer_ioctl::BuzzerTone = buzzer_ioctl::BuzzerTone { freq: freq.value(), period: period };
    let ret = self.play_tone_async(&tone);
    thread::sleep(Duration::from_millis(period as u64));
    ret
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

  pub fn play_supermario(&self) -> Result<(),String> {

    let buzzer = self.buzzer.clone();

    let _handler = thread::spawn( move || {
      if let Ok(ref mut buzzer_locked) = buzzer.lock() {
        (*buzzer_locked).play_tone(Tone::NOTE_E7,12);
        (*buzzer_locked).play_tone(Tone::NOTE_E7,12);
        (*buzzer_locked).play_tone(Tone::NOTE_E7,12);    
        thread::sleep(Duration::from_millis(12));
        (*buzzer_locked).play_tone(Tone::NOTE_C7,12);
        (*buzzer_locked).play_tone(Tone::NOTE_E7,12);
        thread::sleep(Duration::from_millis(12));
        (*buzzer_locked).play_tone(Tone::NOTE_G7,12);
        thread::sleep(Duration::from_millis(36));
        (*buzzer_locked).play_tone(Tone::NOTE_G6,12);
        thread::sleep(Duration::from_millis(36));

        (*buzzer_locked).play_tone(Tone::NOTE_C7,12);
        thread::sleep(Duration::from_millis(24));
        (*buzzer_locked).play_tone(Tone::NOTE_G6,12);
        thread::sleep(Duration::from_millis(24));
        (*buzzer_locked).play_tone(Tone::NOTE_E6,12);
        thread::sleep(Duration::from_millis(24));
        (*buzzer_locked).play_tone(Tone::NOTE_A6,12);
        thread::sleep(Duration::from_millis(12));
        (*buzzer_locked).play_tone(Tone::NOTE_B6,12);
        thread::sleep(Duration::from_millis(12));
        (*buzzer_locked).play_tone(Tone::NOTE_AS6,12);
        (*buzzer_locked).play_tone(Tone::NOTE_A6,12);
        thread::sleep(Duration::from_millis(12));

        (*buzzer_locked).play_tone(Tone::NOTE_G6,9);
        (*buzzer_locked).play_tone(Tone::NOTE_E7,9);
        (*buzzer_locked).play_tone(Tone::NOTE_G7,9);    
        (*buzzer_locked).play_tone(Tone::NOTE_A7,12);
        thread::sleep(Duration::from_millis(12));
        (*buzzer_locked).play_tone(Tone::NOTE_F7,12);
        (*buzzer_locked).play_tone(Tone::NOTE_G7,12);    
        thread::sleep(Duration::from_millis(12));
        (*buzzer_locked).play_tone(Tone::NOTE_E7,12);
        thread::sleep(Duration::from_millis(12));
        (*buzzer_locked).play_tone(Tone::NOTE_C7,12);
        (*buzzer_locked).play_tone(Tone::NOTE_D7,12);
        (*buzzer_locked).play_tone(Tone::NOTE_B6,12);
        thread::sleep(Duration::from_millis(24));

        (*buzzer_locked).play_tone(Tone::NOTE_C7,12);
        thread::sleep(Duration::from_millis(24));
        (*buzzer_locked).play_tone(Tone::NOTE_G6,12);
        thread::sleep(Duration::from_millis(24));
        (*buzzer_locked).play_tone(Tone::NOTE_E6,9);    
        thread::sleep(Duration::from_millis(24));
        (*buzzer_locked).play_tone(Tone::NOTE_A6,12);
        thread::sleep(Duration::from_millis(12));
        (*buzzer_locked).play_tone(Tone::NOTE_B6,12);
        thread::sleep(Duration::from_millis(12));
        (*buzzer_locked).play_tone(Tone::NOTE_AS6,12);
        (*buzzer_locked).play_tone(Tone::NOTE_A6,12);
        thread::sleep(Duration::from_millis(12));


        (*buzzer_locked).play_tone(Tone::NOTE_G6,9);
        (*buzzer_locked).play_tone(Tone::NOTE_E7,9);
        (*buzzer_locked).play_tone(Tone::NOTE_G7,9);    
        (*buzzer_locked).play_tone(Tone::NOTE_A7,12);
        thread::sleep(Duration::from_millis(12));
        (*buzzer_locked).play_tone(Tone::NOTE_F7,12);
        (*buzzer_locked).play_tone(Tone::NOTE_G7,12);    
        thread::sleep(Duration::from_millis(12));
        (*buzzer_locked).play_tone(Tone::NOTE_E7,12);
        thread::sleep(Duration::from_millis(12));
        (*buzzer_locked).play_tone(Tone::NOTE_C7,12);
        (*buzzer_locked).play_tone(Tone::NOTE_D7,12);
        (*buzzer_locked).play_tone(Tone::NOTE_B6,12);
        thread::sleep(Duration::from_millis(24));
      };
    });

    Ok(())
  }

  fn play_starwars_first_session(buzzer_locked: &BuzzerThreadSafe) -> Result<(), String> {
    (*buzzer_locked).play_tone(Tone::NOTE_A4, 500);
    (*buzzer_locked).play_tone(Tone::NOTE_A4, 500);    
    (*buzzer_locked).play_tone(Tone::NOTE_A4, 500);
    (*buzzer_locked).play_tone(Tone::NOTE_F4, 350);
    (*buzzer_locked).play_tone(Tone::NOTE_C5, 150);
    (*buzzer_locked).play_tone(Tone::NOTE_A4, 500);
    (*buzzer_locked).play_tone(Tone::NOTE_F4, 350);
    (*buzzer_locked).play_tone(Tone::NOTE_C5, 150);
    (*buzzer_locked).play_tone(Tone::NOTE_A4, 650);

    thread::sleep(Duration::from_millis(500));

    (*buzzer_locked).play_tone(Tone::NOTE_E5, 500);
    (*buzzer_locked).play_tone(Tone::NOTE_E5, 500);
    (*buzzer_locked).play_tone(Tone::NOTE_E5, 500);  
    (*buzzer_locked).play_tone(Tone::NOTE_F5, 350);
    (*buzzer_locked).play_tone(Tone::NOTE_C5, 150);
    (*buzzer_locked).play_tone(Tone::NOTE_GS4, 500);
    (*buzzer_locked).play_tone(Tone::NOTE_F4, 350);
    (*buzzer_locked).play_tone(Tone::NOTE_C5, 150);
    (*buzzer_locked).play_tone(Tone::NOTE_A4, 650);

    thread::sleep(Duration::from_millis(500));

    Ok(())
  }

  fn play_starwars_second_session(buzzer_locked: &BuzzerThreadSafe) -> Result<(), String> {
    (*buzzer_locked).play_tone(Tone::NOTE_A5, 500);
    (*buzzer_locked).play_tone(Tone::NOTE_A4, 300);
    (*buzzer_locked).play_tone(Tone::NOTE_A4, 150);
    (*buzzer_locked).play_tone(Tone::NOTE_A5, 500);
    (*buzzer_locked).play_tone(Tone::NOTE_GS5, 325);
    (*buzzer_locked).play_tone(Tone::NOTE_G5, 175);
    (*buzzer_locked).play_tone(Tone::NOTE_FS5, 125);
    (*buzzer_locked).play_tone(Tone::NOTE_F5, 125);    
    (*buzzer_locked).play_tone(Tone::NOTE_FS5, 250);

    thread::sleep(Duration::from_millis(325));

    (*buzzer_locked).play_tone(Tone::NOTE_AS4, 250);
    (*buzzer_locked).play_tone(Tone::NOTE_DS5, 500);
    (*buzzer_locked).play_tone(Tone::NOTE_D5, 325);  
    (*buzzer_locked).play_tone(Tone::NOTE_CS5, 175);  
    (*buzzer_locked).play_tone(Tone::NOTE_C5, 125);  
    (*buzzer_locked).play_tone(Tone::NOTE_B4, 125);  
    (*buzzer_locked).play_tone(Tone::NOTE_C5, 250);  

    thread::sleep(Duration::from_millis(350));

    Ok(())
  }

  fn play_starwars_variant1(buzzer_locked: &BuzzerThreadSafe) -> Result<(), String> {
    (*buzzer_locked).play_tone(Tone::NOTE_F4, 250);  
    (*buzzer_locked).play_tone(Tone::NOTE_GS4, 500);  
    (*buzzer_locked).play_tone(Tone::NOTE_F4, 350);  
    (*buzzer_locked).play_tone(Tone::NOTE_A4, 125);
    (*buzzer_locked).play_tone(Tone::NOTE_C5, 500);
    (*buzzer_locked).play_tone(Tone::NOTE_A4, 375);  
    (*buzzer_locked).play_tone(Tone::NOTE_C5, 125);
    (*buzzer_locked).play_tone(Tone::NOTE_E5, 650);
  
    thread::sleep(Duration::from_millis(500));

    Ok(())
  }

  fn play_starwars_variant2(buzzer_locked: &BuzzerThreadSafe) -> Result<(), String> {
    (*buzzer_locked).play_tone(Tone::NOTE_F4, 250);  
    (*buzzer_locked).play_tone(Tone::NOTE_GS4, 500);  
    (*buzzer_locked).play_tone(Tone::NOTE_F4, 375);  
    (*buzzer_locked).play_tone(Tone::NOTE_C5, 125);
    (*buzzer_locked).play_tone(Tone::NOTE_A4, 500);  
    (*buzzer_locked).play_tone(Tone::NOTE_F4, 375);  
    (*buzzer_locked).play_tone(Tone::NOTE_C5, 125);
    (*buzzer_locked).play_tone(Tone::NOTE_A4, 650);  
  
    thread::sleep(Duration::from_millis(650));
    Ok(())
  }

  pub fn play_starwars(&self) -> Result<(), String>  {
    let buzzer = self.buzzer.clone();

    let _handler = thread::spawn( move || {
      if let Ok(ref mut buzzer_locked) = buzzer.lock() {

        Buzzer::play_starwars_first_session(buzzer_locked);
        Buzzer::play_starwars_second_session(buzzer_locked);

        Buzzer::play_starwars_variant1(buzzer_locked);
        Buzzer::play_starwars_second_session(buzzer_locked);
        Buzzer::play_starwars_variant2(buzzer_locked);      
      };
    });

    Ok(())
  }

  pub fn play_doremifa(&self) -> Result<(), String> {
    let buzzer = self.buzzer.clone();

    let _handler = thread::spawn( move || {
      if let Ok(ref mut buzzer_locked) = buzzer.lock() {
        (*buzzer_locked).play_tone(Tone::NOTE_C4, 600);
        (*buzzer_locked).play_tone(Tone::NOTE_D4, 600);
        (*buzzer_locked).play_tone(Tone::NOTE_E4, 600);
        (*buzzer_locked).play_tone(Tone::NOTE_F4, 600);

        thread::sleep(Duration::from_millis(100));
        (*buzzer_locked).play_tone(Tone::NOTE_F4, 400);
        thread::sleep(Duration::from_millis(100));
        (*buzzer_locked).play_tone(Tone::NOTE_F4, 400);

        (*buzzer_locked).play_tone(Tone::NOTE_C4, 600);
        (*buzzer_locked).play_tone(Tone::NOTE_D4, 600);
        (*buzzer_locked).play_tone(Tone::NOTE_C4, 600);
        (*buzzer_locked).play_tone(Tone::NOTE_D4, 600);

        thread::sleep(Duration::from_millis(100));
        (*buzzer_locked).play_tone(Tone::NOTE_D4, 400);
        thread::sleep(Duration::from_millis(100));
        (*buzzer_locked).play_tone(Tone::NOTE_D4, 400);

        (*buzzer_locked).play_tone(Tone::NOTE_C4, 600);
        (*buzzer_locked).play_tone(Tone::NOTE_G4, 600);
        (*buzzer_locked).play_tone(Tone::NOTE_F4, 600);
        (*buzzer_locked).play_tone(Tone::NOTE_E4, 600);

        thread::sleep(Duration::from_millis(100));
        (*buzzer_locked).play_tone(Tone::NOTE_E4, 400);
        thread::sleep(Duration::from_millis(100));
        (*buzzer_locked).play_tone(Tone::NOTE_E4, 400);

        (*buzzer_locked).play_tone(Tone::NOTE_C4, 600);
        (*buzzer_locked).play_tone(Tone::NOTE_D4, 600);
        (*buzzer_locked).play_tone(Tone::NOTE_E4, 600);
        (*buzzer_locked).play_tone(Tone::NOTE_F4, 600);

        thread::sleep(Duration::from_millis(100));
        (*buzzer_locked).play_tone(Tone::NOTE_F4, 400);
        thread::sleep(Duration::from_millis(100));
        (*buzzer_locked).play_tone(Tone::NOTE_F4, 400);
      };
    });

    Ok(())
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
    self. play_supermario()
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
