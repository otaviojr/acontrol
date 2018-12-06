use fingerprint::{Fingerprint};
use nfc::{NfcReader};
use audio::{Audio};

use std::sync::Mutex;
use std::collections::HashMap;

#[allow(dead_code)]
enum NFCSystemState {
  READING,
//  FORMATING,
//  WRITING,
  AUTHORIZE,
  RESTORE
}

pub struct AControlSystem {
  fingerprint_drv: Mutex<Option<Mutex<Box<Fingerprint + Send + Sync>>>>,
  nfc_drv: Mutex<Option<Mutex<Box<NfcReader + Send + Sync>>>>,
  audio_drv: Mutex<Option<Mutex<Box<Audio + Send + Sync>>>>,
  nfc_state: Mutex<NFCSystemState>,
}

impl AControlSystem {
}

lazy_static!{
  static ref ACONTROL_SYSTEM: AControlSystem = AControlSystem {
    fingerprint_drv: Mutex::new(None), 
    nfc_drv: Mutex::new(None), 
    audio_drv: Mutex::new(None),
    nfc_state: Mutex::new(NFCSystemState::READING),
  };

  static ref NFC_CARD_SIGNATURE: &'static str = &"ACONTROL_CARD";
  static ref NFC_CARD_SIGNATURE_BLOCK: u8 = 1;
}

pub fn acontrol_system_end() -> bool {
  println!("Cleaning all suffs");
  match *ACONTROL_SYSTEM.audio_drv.lock().unwrap()  {
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

  match *ACONTROL_SYSTEM.nfc_drv.lock().unwrap()  {
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
  let asystem = &ACONTROL_SYSTEM;
  match *asystem.nfc_drv.lock().unwrap() {
    Some(ref drv) => {
      if let Err(_err) = drv.lock().unwrap().set_auth_keys(key_a, key_b) {
        return false;
      }
      return true;
    },
    None => return false
  }
}

pub fn acontrol_system_init(params: &HashMap<String,String>, fingerprint_drv: Box<Fingerprint+Sync+Send>, nfc_drv: Box<NfcReader+Sync+Send>, audio_drv: Box<Audio+Sync+Send>) -> bool {

  let asystem = &ACONTROL_SYSTEM;
  unsafe {
    *asystem.fingerprint_drv.lock().unwrap() = Some(Mutex::new(Box::from_raw(Box::into_raw(fingerprint_drv))));
    *asystem.nfc_drv.lock().unwrap() = Some(Mutex::new(Box::from_raw(Box::into_raw(nfc_drv))));
    *asystem.audio_drv.lock().unwrap() = Some(Mutex::new(Box::from_raw(Box::into_raw(audio_drv))));
  }

  //initializing audio device
  match *asystem.audio_drv.lock().unwrap()  {
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
  match *asystem.nfc_drv.lock().unwrap() {
    Some(ref drv) => {
      let mut drv_inner = drv.lock().unwrap();

      if let Err(err) = drv_inner.init() {
        eprintln!("Error initializing nfc (=> {})", err);
        return false;
      }

      drv_inner.find_tag(|uuid, sak|{
        match *ACONTROL_SYSTEM.nfc_drv.lock().unwrap() {
          Some(ref mut drv) => {
            let mut drv_inner = drv.lock().unwrap();

            println!("Card Found: UUID={:?}, SAK={:?}", uuid,sak);

            match *ACONTROL_SYSTEM.nfc_state.lock().unwrap() {
              NFCSystemState::READING => {
                //TODO: Check tag and allow access
              },
              NFCSystemState::AUTHORIZE => {
                if let Err(err) = drv_inner.format(&uuid) {
                  eprintln!("Error formating. Is this a new card? Let's try to write anyway");
                  eprintln!("format return: {}", err);
                }

                if let Err(err) = drv_inner.write_data(&uuid, *NFC_CARD_SIGNATURE_BLOCK, &NFC_CARD_SIGNATURE.as_bytes().to_vec()) {
                }
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

fn acontrol_system_set_nfc_state(state: NFCSystemState) {
  *ACONTROL_SYSTEM.nfc_state.lock().unwrap() = state;
}


pub fn get_acontrol_system<'a>() -> &'a AControlSystem {
  return &ACONTROL_SYSTEM;
}
