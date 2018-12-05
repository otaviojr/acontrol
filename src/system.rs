use fingerprint::{Fingerprint};
use nfc::{NfcReader};
use audio::{Audio};

use std::sync::Mutex;

#[allow(dead_code)]
enum NFCSystemState {
  READING,
//  FORMATING,
//  WRITING,
  AUTHORIZE,
  RESTORE
}

struct ASystemState {
  nfc_state: Mutex<NFCSystemState>
}

impl ASystemState {
  fn set_system_nfc_state(&mut self, state: NFCSystemState) {
    *self.nfc_state.lock().unwrap() = state;
  }
}

pub struct AControlSystem {
  fingerprint_drv: Option<Mutex<Box<Fingerprint + Send + Sync>>>,
  nfc_drv: Option<Mutex<Box<NfcReader + Send + Sync>>>,
  audio_drv: Option<Mutex<Box<Audio + Send + Sync>>>,
}

impl AControlSystem {
  pub fn set_fingerprint_drv(&mut self, drv: Box<Fingerprint+Sync+Send>){
    unsafe {
      self.fingerprint_drv = Some(Mutex::new(Box::from_raw(Box::into_raw(drv))));
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
  static ref ACONTROL_SYSTEM: Mutex<AControlSystem> = Mutex::new(AControlSystem {
    fingerprint_drv: None, 
    nfc_drv: None, audio_drv: None,
  });

  static ref ACONTROL_SYSTEM_STATE: ASystemState = ASystemState {nfc_state: Mutex::new(NFCSystemState::READING) };
}

pub fn acontrol_system_end() -> bool {
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

pub fn  acontrol_system_set_mifare_keys(key_a: &Vec<u8>, key_b: &Vec<u8>) -> bool {
  let asystem = ACONTROL_SYSTEM.lock().unwrap();
  match asystem.nfc_drv {
    Some(ref drv) => {
      let mut drv_inner = drv.lock().unwrap();
      if let Err(_err) = drv_inner.set_auth_keys(key_a, key_b) {
        return false;
      }
      return true;
    },
    None => return false
  }
}

pub fn acontrol_system_init(fingerprint_drv: Box<Fingerprint+Sync+Send>, nfc_drv: Box<NfcReader+Sync+Send>, audio_drv: Box<Audio+Sync+Send>) -> bool {
  ACONTROL_SYSTEM.lock().unwrap().set_fingerprint_drv(fingerprint_drv);
  ACONTROL_SYSTEM.lock().unwrap().set_nfc_drv(nfc_drv);
  ACONTROL_SYSTEM.lock().unwrap().set_audio_drv(audio_drv);

  let asystem = ACONTROL_SYSTEM.lock().unwrap();

  //initializing audio device
  match asystem.audio_drv  {
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
  match asystem.nfc_drv {
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

            match *ACONTROL_SYSTEM_STATE.nfc_state.lock().unwrap() {
              NFCSystemState::READING => {
                //TODO: Check tag and allow access
              },
//              NFCSystemState::WRITING => {
                //TODO: Write the security mark
//              },
//              NFCSystemState::FORMATING => {
                //TODO: Write security blocks with our key
//              },
              NFCSystemState::AUTHORIZE => {
                //TODO: Persist card to check on reading state
              },
              NFCSystemState::RESTORE => {
                //TODO: Restore authentication's blocks to the original key_a and key_b
              }
            }

            //if let Ok(blocks) = drv.lock().unwrap().write_data(&uuid,1,&"VALÉRIA".as_bytes().to_vec()) {
            //  println!("Data written with success. Used {} blocks", blocks);
            //}

            //match drv.lock().unwrap().read_data(&uuid,1,0) {
	      //Ok(val) => {
              //  println!("Card's read value is: {}", String::from_utf8(val).unwrap());
              //},
              //Err(err) => {
                //println!("Error reading card: {}", err);
              //}
            //}

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
