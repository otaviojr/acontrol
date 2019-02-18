use audio::{Audio};

use std::sync::Arc;
use std::sync::Mutex;
use sysfs_gpio::{Direction, Pin};
use std::thread;
use std::time::Duration;

struct BuzzerThreadSafe {
  pin: Option<Pin>
}

impl BuzzerThreadSafe {
  fn set_buzz(&mut self, val: bool) -> bool {
    if self.pin.is_none() {
      panic!("Audio Driver Panic: No pin");
    }

    let pin = self.pin.unwrap();
    let ret = pin.set_value(if val == true { 1 } else { 0 });

    if let Err(err) = ret {
      println!("Audio Driver: gpio error ({})", err);
      return false;
    }

    return true;
  }
}

unsafe impl Sync for BuzzerThreadSafe {}

unsafe impl Send for BuzzerThreadSafe {}

pub struct Buzzer {
  buzzer: Arc<Mutex<BuzzerThreadSafe>>
}

impl Buzzer {
  pub fn new() -> Self {
    return Buzzer {buzzer: Arc::new(Mutex::new(BuzzerThreadSafe {pin: None}))};
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

    let pin = Pin::new(5);
    if let Err(err) = pin.export() {
      return Err(format!("{}: {}","Error initializing audio drive",err));
    }

    //for non root users, exporting a pin could have a delay to show up at sysfs
    thread::sleep(Duration::from_millis(100));
    pin.set_direction(Direction::Out).unwrap();

    buzzer.lock().unwrap().pin = Some(pin);

    self.play_alert();
    
    Ok( () )
  }

  fn play_success(&mut self) -> Result<(), String> {
    let buzzer = self.buzzer.clone();
    let _handler = thread::spawn(move || {
      buzzer.lock().unwrap().set_buzz(true);
      thread::sleep(Duration::from_millis(1000));
      buzzer.lock().unwrap().set_buzz(false);
    });

    Ok(())
  }

  fn play_error(&mut self) -> Result<(), String> {
    let buzzer = self.buzzer.clone();
    let _handler = thread::spawn(move || {
      for i in 0..3 {
        buzzer.lock().unwrap().set_buzz(true);
        thread::sleep(Duration::from_millis(500));
        buzzer.lock().unwrap().set_buzz(false);
        thread::sleep(Duration::from_millis(500));
      }
    });

    Ok(())
  }

  fn play_alert(&mut self) -> Result<(), String> {
    let buzzer = self.buzzer.clone();
    let _handler = thread::spawn(move || {
      buzzer.lock().unwrap().set_buzz(true);
      thread::sleep(Duration::from_millis(300));
      buzzer.lock().unwrap().set_buzz(false);
    });

    Ok(())
  }


  fn unload(&mut self) -> Result<(), String>{
    println!("Audio driver unloading");
    let buzzer = self.buzzer.clone();
    let pin = buzzer.lock().unwrap().pin.unwrap();
    if let Err(err) = pin.unexport() {
      return Err(format!("{}(=>{})", "Audio driver error",err));
    }
    Ok(())
  }

  fn signature(&self) -> String {
    return String::from("Buzzer Audio Module");
  }
}

unsafe impl Send for Buzzer {}

unsafe impl Sync for Buzzer {}
