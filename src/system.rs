use fingerprint::{Fingerprint};
use nfc::{NfcReader};
use audio::{Audio};

use std::sync::Mutex;

pub struct AControlSystem {
  fingerprint_drv: Option<Box<Fingerprint + Send + Sync>>,
  nfc_drv: Option<Mutex<Box<NfcReader + Send + Sync>>>,
  audio_drv: Option<Mutex<Box<Audio + Send + Sync>>>,
}

impl AControlSystem {
  pub fn set_fingerprint_drv(&mut self, drv: Box<Fingerprint+Sync+Send>){
    unsafe {
      self.fingerprint_drv = Some(Box::from_raw(Box::into_raw(drv)));
    }
  }

  pub fn set_nfc_drv(&mut self, drv: Box<NfcReader+Sync+Send>){
    unsafe {
      self.nfc_drv = Some(Mutex::new(Box::from_raw(Box::into_raw(drv))));
    }
  }

  pub fn set_audio_drv(&mut self, drv: Box<Audio + Sync + Send>) {
    unsafe {
      self.audio_drv = Some(Mutex::new(Box::from_raw(Box::into_raw(drv))));
    }
  }
}

lazy_static!{
  static ref ACONTROL_SYSTEM: Mutex<AControlSystem> = Mutex::new(AControlSystem {fingerprint_drv: None, nfc_drv: None, audio_drv: None});
}

pub fn end_acontrol_system() -> bool {
  println!("Cleaning all suffs");
  match ACONTROL_SYSTEM.lock().unwrap().audio_drv  {
    Some(ref drv) => {
      if let Err(err) = drv.lock().unwrap().unload() {
        eprintln!("Error unloading audio device (=> {})", err);
        return false;
      }
    },
    None => {
      eprintln!("Audio device not found");
      return false;
    }
  }

  match ACONTROL_SYSTEM.lock().unwrap().nfc_drv  {
    Some(ref drv) => {
      if let Err(err) = drv.lock().unwrap().unload() {
        eprintln!("Error unloading nfc device (=> {})", err);
        return false;
      }
    },
    None => {
      eprintln!("NFC device not found");
      return false;
    }
  }

  return true;
}

pub fn init_acontrol_system(fingerprint_drv: Box<Fingerprint+Sync+Send>, nfc_drv: Box<NfcReader+Sync+Send>, audio_drv: Box<Audio+Sync+Send>) -> bool {
  ACONTROL_SYSTEM.lock().unwrap().set_fingerprint_drv(fingerprint_drv);
  ACONTROL_SYSTEM.lock().unwrap().set_nfc_drv(nfc_drv);
  ACONTROL_SYSTEM.lock().unwrap().set_audio_drv(audio_drv);

  //initializing audio device
  match ACONTROL_SYSTEM.lock().unwrap().audio_drv  {
    Some(ref drv) => {
      if let Err(err) = drv.lock().unwrap().init() {
        eprintln!("Error initializing audio device (=> {})", err);
        return false;
      }
    },
    None => {
      eprintln!("Audio device not found");
      return false;
    }
  }

  //initializing nfc device
  match ACONTROL_SYSTEM.lock().unwrap().nfc_drv {
    Some(ref drv) => {
      let mut drv_inner = drv.lock().unwrap();

      if let Err(err) = drv_inner.init() {
        eprintln!("Error initializing nfc (=> {})", err);
        return false;
      }

      drv_inner.find_tag(|uuid, sak|{
        match ACONTROL_SYSTEM.lock().unwrap().nfc_drv {
          Some(ref drv) => {
            println!("Card Found: UUID={:?}, SAK={:?}", uuid,sak);

            match drv.lock().unwrap().read_data(&uuid) {
	      Ok(val) => {
                println!("Card's read value is: {:?}", val);
              },
              Err(err) => {
                println!("Error reading card: {}", err);
              }
            }

            return true;
          },
          None => return false,
        }
      }).unwrap();

    },
    None => {
      eprintln!("Nfc driver not found!");
      return false;
    }
  };

  return true;
}

pub fn get_acontrol_system<'a>() -> &'a Mutex<AControlSystem> {
  return &ACONTROL_SYSTEM;
}
