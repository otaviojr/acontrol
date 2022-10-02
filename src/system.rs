
/**
 * @file   system.rs
 * @author Otavio Ribeiro
 * @date   24 Dec 2017
 * @brief  System operation/logic
 *
 * Copyright (c) 2022 Otávio Ribeiro <otavio.ribeiro@gmail.com>
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
use crate::nfc::CardType;
use crate::log::{Log, LogType};
use crate::bt::{Bluetooth, BluetoothDevice};
use crate::fingerprint::{Fingerprint, FingerprintState, FingerprintData};
use crate::nfc::{NfcReader};
use crate::audio::{Audio};
use crate::persist::{Persist};
use crate::display::{Display, Animation, AnimationType, AnimationColor};

use std::sync::{Mutex, Arc};
use std::collections::HashMap;

use std::process::Command;

#[derive(PartialEq)]
#[allow(dead_code)]
pub enum NFCSystemState {
  READ,
  WRITE,
  AUTHORIZE,
  RESTORE
}

#[derive(PartialEq)]
#[allow(dead_code)]
pub enum BluetoothSystemState {
  READ,
  AUTHORIZE,
}

#[macro_export]
macro_rules! acontrol_system_log {
  ($type:expr, $message:literal $(,$args:expr)*) => {{
    let asystem = crate::system::acontrol_system_get();
    let log_drv = asystem.log_drv.clone();
    if let Ok(ref mut drv_locked) = log_drv.lock() {
      if let Some(ref mut drv) = **drv_locked {
        let _ = drv.log($type, format!($message $(,$args)*));
      } else {
        println!($message $(,$args)*);
      }
    };
  }};
}

pub struct AControlSystem {
  bt_drv: Mutex<Option<Box<dyn Bluetooth + Send + Sync>>>,
  fingerprint_drv: Mutex<Option<Box<dyn Fingerprint + Send + Sync>>>,
  nfc_drv: Mutex<Option<Box<dyn NfcReader + Send + Sync>>>,
  audio_drv: Mutex<Option<Box<dyn Audio + Send + Sync>>>,
  persist_drv:  Mutex<Option<Box<dyn Persist + Send + Sync>>>,
  display_drv: Mutex<Option<Box<dyn Display + Send + Sync>>>,
  pub log_drv: Arc<Mutex<Option<Box<dyn Log + Send + Sync>>>>,
  nfc_state: Mutex<NFCSystemState>,
  nfc_state_params: Mutex<HashMap<String,String>>,
  fingerprint_data: Mutex<FingerprintData>,
  fingerprint_last_state: Mutex<Option<FingerprintState>>,
  bt_state: Mutex<BluetoothSystemState>,
  bt_state_params: Mutex<HashMap<String,String>>,
}

impl AControlSystem {
}

lazy_static!{
  pub static ref ACONTROL_SYSTEM: AControlSystem = AControlSystem {
    bt_drv: Mutex::new(Option::None),
    fingerprint_drv: Mutex::new(Option::None),
    nfc_drv: Mutex::new(Option::None),
    audio_drv: Mutex::new(Option::None),
    persist_drv:  Mutex::new(Option::None),
    display_drv: Mutex::new(Option::None),
    log_drv: Arc::new(Mutex::new(Option::None)),
    nfc_state: Mutex::new(NFCSystemState::READ),
    nfc_state_params: Mutex::new(HashMap::new()),
    fingerprint_data: Mutex::new(FingerprintData::empty()),
    fingerprint_last_state: Mutex::new(None),
    bt_state: Mutex::new(BluetoothSystemState::READ),
    bt_state_params: Mutex::new(HashMap::new()),      
  };
  
  static ref NFC_CARD_SIGNATURE: &'static str = &"ACONTROL_CARD\0\0\0";
  static ref NFC_CARD_SIGNATURE_BLOCK: u8 = 1;
}

pub fn acontrol_system_end() -> bool {
  let asystem = acontrol_system_get();
  acontrol_system_log!(LogType::Info, "Cleaning all suffs");


  if let Ok(ref mut drv_lock) = asystem.bt_drv.lock() {
    if let Some(ref mut drv) = **drv_lock {
      if let Err(err) = drv.unload() {
        acontrol_system_log!(LogType::Error, "Error unloading bluetooth device (=> {})", err);
        return false;  
      } else {
        *asystem.bt_drv.lock().unwrap() = Option::None;
      }
    };
  }

  if let Ok(ref mut drv_lock) = asystem.audio_drv.lock() {
    if let Some(ref mut drv) = **drv_lock {
      if let Err(err) = drv.unload() {
        acontrol_system_log!(LogType::Error, "Error unloading audio device (=> {})", err);
        return false;  
      } else {
        *asystem.audio_drv.lock().unwrap() = Option::None;
      }
    };
  }

  if let Ok(ref mut drv_lock) = asystem.nfc_drv.lock() {
    if let Some(ref mut drv) = **drv_lock {
      if let Err(err) = drv.unload() {
        acontrol_system_log!(LogType::Error, "Error unloading nfc device (=> {})", err);
        return false;  
      } else {
        *asystem.nfc_drv.lock().unwrap() = Option::None;
      }
    };
  }

  if let Ok(ref mut drv_lock) = asystem.fingerprint_drv.lock() {
    if let Some(ref mut drv) = **drv_lock {
      if let Err(err) = drv.unload() {
        acontrol_system_log!(LogType::Error, "Error unloading fingerprint device (=> {})", err);
        return false;  
      } else {
        *asystem.fingerprint_drv.lock().unwrap() = Option::None;
      }
    };
  }

  if let Ok(ref mut drv_lock) = asystem.persist_drv.lock() {
    if let Some(ref mut drv) = **drv_lock {
      if let Err(err) = drv.unload() {
        acontrol_system_log!(LogType::Error, "Error unloading persistence device (=> {})", err);
        return false;  
      } else {
        *asystem.persist_drv.lock().unwrap() = Option::None;
      }
    };
  }

  if let Ok(ref mut drv_lock) = asystem.log_drv.lock() {
    if let Some(ref mut drv) = **drv_lock {
      if let Err(err) = drv.unload() {
        acontrol_system_log!(LogType::Error, "Error unloading persistence device (=> {})", err);
        return false;  
      } else {
        *asystem.log_drv.lock().unwrap() = Option::None;
      }
    };
  }
  return true;
}

pub fn  acontrol_system_set_mifare_keys(key_a: &Vec<u8>, key_b: &Vec<u8>) -> bool {
  let asystem = acontrol_system_get();
  if let Ok(ref mut drv_lock) = asystem.nfc_drv.lock() {
    if let Some(ref mut drv) = **drv_lock {
      if let Err(err) = drv.set_auth_keys(key_a, key_b) {
        acontrol_system_log!(LogType::Error, "Error setting mifare key: {}", err);
        return false;
      }
      return true;
    };
  }
  return false;
}

fn find_bt_device(device: BluetoothDevice) -> bool {
  let asystem = acontrol_system_get();
  let mut next_bt_system_state: Option<BluetoothSystemState> = None;

  acontrol_system_log!(LogType::Info, "Bluetooth device found: ADDR={:X?}", device.addr);

  if let Ok(ref mut bt_state) = asystem.bt_state.lock() {
    match **bt_state {
      BluetoothSystemState::READ => {
        let _ret = acontrol_system_get_display_drv( |display|{
          let _ret = display.show_animation(Animation::MaterialSpinner, AnimationColor::Orange, AnimationType::Waiting, "Waiting",0);
        });

        let query = Command::new("/acontrol/query")
        .output()
        .expect("failed to execute child");

        if String::from_utf8_lossy(query.stdout.as_slice()).trim_end().to_lowercase().eq(&String::from("close")) {
                    
          let _ret = acontrol_system_get_audio_drv(|audio|{
            let _ret = audio.play_granted();
          });

          let _ret = acontrol_system_get_display_drv(|display|{
            let _ret = display.show_animation(Animation::Blink,AnimationColor::Green,AnimationType::Success, "Done",3);
            let _ret = display.when_animation_ends( || {
              let _ret = acontrol_system_get_display_drv( |display| {
                let _ret = display.show_animation(Animation::MaterialSpinner, AnimationColor::Orange, AnimationType::Waiting, "Waiting",0);
              });
            });
          });

          let granted = Command::new("/acontrol/granted")
          .arg("-f")
          .output()
          .expect("failed to execute child");

          let messages = String::from_utf8_lossy(granted.stdout.as_slice());
          for message in messages.lines() {
            acontrol_system_log!(LogType::Info, "granted: {}", message);
          }

        } else {
          acontrol_system_log!(LogType::Warning, "Device is already open: {}", String::from_utf8_lossy(query.stdout.as_slice()).to_lowercase());
        }
        
        let _ret = acontrol_system_get_display_drv(|display|{
          let _ret = display.clear_and_stop_animations();
        });
      },
      BluetoothSystemState::AUTHORIZE => {
        next_bt_system_state = Some(BluetoothSystemState::READ)
      }
    }
  }

  if let Some(state) = next_bt_system_state {
    acontrol_system_set_bluetooth_state(state,None);
  }

  return true;
}

fn find_finger(state: &FingerprintState, _value: Option<&str>) -> bool {
  let asystem = acontrol_system_get();
  if let Ok(ref mut last_state_locked) = asystem.fingerprint_last_state.lock() {
    if last_state_locked.is_none() || last_state_locked.unwrap() != *state {

      last_state_locked.replace(*state);

      acontrol_system_log!(LogType::Debug, "Fingerprint Current State Changed To: {}", state.name());

      match state {
        FingerprintState::IDLE => {
          let _ret = acontrol_system_get_display_drv( |display|{
            let _ret = display.clear_and_stop_animations();
          });
        },
        FingerprintState::READING => {
          let _ret = acontrol_system_get_display_drv( |display|{
            let _ret = display.show_animation(Animation::MaterialSpinner, AnimationColor::Orange, AnimationType::Waiting, "Waiting",0);
          });
        },
        FingerprintState::WAITING => {
          let _ret = acontrol_system_get_audio_drv(|audio|{
            let _ret = audio.play_alert();
          });
          let _ret = acontrol_system_get_display_drv( |display|{
            let _ret = display.show_animation(Animation::MaterialSpinner, AnimationColor::Orange, AnimationType::Waiting, "Waiting",0);
          });
        },
        FingerprintState::SUCCESS => {
          let _ret = acontrol_system_get_audio_drv(|audio|{
            let _ret = audio.play_success();
          });
          let _ret = acontrol_system_get_display_drv( |display|{
            let _ret = display.show_animation(Animation::BlinkLoop,AnimationColor::Green,AnimationType::Success, "Done",0);
          });
        },
        FingerprintState::ERROR => {
          let _ret = acontrol_system_get_audio_drv(|audio|{
            let _ret = audio.play_error();
          });
          let _ret = acontrol_system_get_display_drv( |display|{
            let _ret = display.show_animation(Animation::Blink,AnimationColor::Red,AnimationType::Error,"Done",3);
            let _ret = display.wait_animation_ends();
          });
        },
        FingerprintState::ENROLL => {
          let data_locked = asystem.fingerprint_data.lock().unwrap();
          if let (&Some(ref name), &Some(ref pos)) = (&data_locked.name, &data_locked.pos){

            let _ret = acontrol_system_get_audio_drv(|audio|{
              let _ret = audio.play_new();
            });

            let _ret = acontrol_system_get_display_drv( |display|{
              let _ret = display.show_animation(Animation::BlinkLoop,AnimationColor::Green,AnimationType::Success, "Done",3);
              let _ret = display.wait_animation_ends();
            });
            acontrol_system_log!(LogType::Info, "User {} added at position {}", name, pos);
          }
        },
        FingerprintState::AUTHORIZED => {
          acontrol_system_log!(LogType::Info, "Fingerprint authorized");

          let _ret = acontrol_system_get_audio_drv(|audio|{
            let _ret = audio.play_granted();
          });

          let _ret = acontrol_system_get_display_drv( |display|{
            let _ret = display.show_animation(Animation::Blink,AnimationColor::Green,AnimationType::Success, "Done",3);
            let _ret = display.when_animation_ends( || {
              let _ret = acontrol_system_get_display_drv( |display|{
                let _ret = display.show_animation(Animation::MaterialSpinner, AnimationColor::Orange, AnimationType::Waiting, "Waiting",0);
              });
            });
          });

          let granted = Command::new("/acontrol/granted")
          .arg("-f")
          .output()
          .expect("failed to execute child");

          let messages = String::from_utf8_lossy(granted.stdout.as_slice());
          for message in messages.lines() {
            acontrol_system_log!(LogType::Info, "granted: {}", message);
          }

          let _ret = acontrol_system_get_display_drv( |display|{
            let _ret = display.clear_and_stop_animations();
          });

        }
        FingerprintState::NOT_AUTHORIZED => {
          acontrol_system_log!(LogType::Info, "Fingerprint not authorized");
          let _ret = acontrol_system_get_audio_drv(|audio|{
            let _ret = audio.play_denied();
          });
          let _ret = acontrol_system_get_display_drv( |display|{
            let _ret = display.show_animation(Animation::BlinkLoop,AnimationColor::Red,AnimationType::Error,"Done",3);
            let _ret = display.when_animation_ends( || {
              let _ret = acontrol_system_get_display_drv( |display|{
                let _ret = display.show_animation(Animation::MaterialSpinner, AnimationColor::Orange, AnimationType::Waiting, "Waiting",0);
              });
            });
          });

          let denieded = Command::new("/acontrol/denieded")
          .arg("-f")
          .output()
          .expect("failed to execute child");

          let messages = String::from_utf8_lossy(denieded.stdout.as_slice());
          for message in messages.lines() {
            acontrol_system_log!(LogType::Info, "denieded: {}", message);
          }

          let _ret = acontrol_system_get_display_drv( |display|{
            let _ret = display.clear_and_stop_animations();
          });
        }
      }
    }
  }
  return true;
}

fn find_tag(_card_type: CardType, uuid: Vec<u8>) -> bool {
  let asystem = acontrol_system_get();
    if let Ok(ref mut drv_lock) = asystem.nfc_drv.lock() {
      if let Some(ref mut nfc_drv) = **drv_lock {
        let mut next_nfc_system_state: Option<NFCSystemState> = None;
      if let Ok(ref mut nfc_state) = asystem.nfc_state.lock() {
        match **nfc_state {
          NFCSystemState::READ => {
            let _ = acontrol_system_get_persist_drv( |persist_drv| {
              match nfc_drv.read_data(&uuid,*NFC_CARD_SIGNATURE_BLOCK,0) {
                Ok(ref val) => {
                  if let Ok(value) = String::from_utf8(val.to_vec()) {
                      if value ==
                        String::from_utf8(NFC_CARD_SIGNATURE.as_bytes().to_vec()).unwrap() {

                        if let Ok(card) = persist_drv.nfc_find(&uuid) {
                          acontrol_system_log!(LogType::Info, "Card {:?} from {} authorized!", uuid, String::from_utf8(card.name).unwrap());

                          let _ret = acontrol_system_get_audio_drv(|audio|{
                            let _ret = audio.play_granted();
                          });
                          let _ret = acontrol_system_get_display_drv( |display|{
                            let _ret = display.show_animation(Animation::Blink,AnimationColor::Green,AnimationType::Success, "Done",3);
                            let _ret = display.when_animation_ends( || {
                              let _ret = acontrol_system_get_display_drv( |display|{
                                let _ret = display.show_animation(Animation::MaterialSpinner, AnimationColor::Orange, AnimationType::Waiting, "Waiting",0);
                              });
                            });
                          });
                          let granted = Command::new("/acontrol/granted")
                          .arg("-f")
                          .output()
                          .expect("failed to execute child");
                
                          let messages = String::from_utf8_lossy(granted.stdout.as_slice());
                          for message in messages.lines() {
                            acontrol_system_log!(LogType::Info, "granted: {}", message);
                          }

                          let _ret = acontrol_system_get_display_drv( |display|{
                            let _ret = display.clear_and_stop_animations();
                          });
                        } else {
                          acontrol_system_log!(LogType::Error, "Card {:?} not found!", uuid);

                          let _ret = acontrol_system_get_audio_drv(|audio|{
                            let _ret = audio.play_denied();
                          });
                          let _ret = acontrol_system_get_display_drv( |display|{
                            let _ret = display.show_animation(Animation::Blink,AnimationColor::Red,AnimationType::Error,"Done",3);
                            let _ret = display.when_animation_ends( || {
                              let _ret = acontrol_system_get_display_drv( |display|{
                                let _ret = display.show_animation(Animation::MaterialSpinner, AnimationColor::Orange, AnimationType::Waiting, "Waiting",0);
                              });
                            });
                          });

                          let denieded = Command::new("/acontrol/denieded")
                          .arg("-f")
                          .output()
                          .expect("failed to execute child");
                
                          let messages = String::from_utf8_lossy(denieded.stdout.as_slice());
                          for message in messages.lines() {
                            acontrol_system_log!(LogType::Info, "denieded: {}", message);
                          }

                          let _ret = acontrol_system_get_display_drv( |display|{
                            let _ret = display.clear_and_stop_animations();
                          });
                        }
                      } else {
                        acontrol_system_log!(LogType::Error, "Invalid card signature: {:?} - {:?}",val, NFC_CARD_SIGNATURE.as_bytes().to_vec());

                        let _ret = acontrol_system_get_audio_drv(|audio|{
                          let _ret = audio.play_denied();
                        });
                        let _ret = acontrol_system_get_display_drv( |display|{
                          let _ret = display.show_animation(Animation::Blink,AnimationColor::Red,AnimationType::Error,"Done",3);
                          let _ret = display.when_animation_ends( || {
                            let _ret = acontrol_system_get_display_drv( |display|{
                              let _ret = display.show_animation(Animation::MaterialSpinner, AnimationColor::Orange, AnimationType::Waiting, "Waiting",0);
                            });
                          });
                        });

                        let denieded = Command::new("/acontrol/denieded")
                        .arg("-f")
                        .output()
                        .expect("failed to execute child");
              
                        let messages = String::from_utf8_lossy(denieded.stdout.as_slice());
                        for message in messages.lines() {
                          acontrol_system_log!(LogType::Info, "denieded: {}", message);
                        }

                        let _ret = acontrol_system_get_display_drv( |display|{
                          let _ret = display.clear_and_stop_animations();
                        });
                      }
                  } else {
                    acontrol_system_log!(LogType::Error, "Error reading card block: {:X?}", val);
                  }
                },
                Err(err) => {
                  acontrol_system_log!(LogType::Error, "Error reading card: {}", err);

                  let _ret = acontrol_system_get_audio_drv(|audio|{
                    let _ret = audio.play_denied();
                  });
                  let _ret = acontrol_system_get_display_drv( |display|{
                    let _ret = display.show_animation(Animation::Blink,AnimationColor::Red,AnimationType::Error,"Done",3);
                    let _ret = display.when_animation_ends( || {
                      let _ret = acontrol_system_get_display_drv( |display|{
                        let _ret = display.show_animation(Animation::MaterialSpinner, AnimationColor::Orange, AnimationType::Waiting, "Waiting",0);
                      });
                    });
                  });

                  let denieded = Command::new("/acontrol/denieded")
                  .arg("-f")
                  .output()
                  .expect("failed to execute child");
        
                  let messages = String::from_utf8_lossy(denieded.stdout.as_slice());
                  for message in messages.lines() {
                    acontrol_system_log!(LogType::Info, "denieded: {}", message);
                  }

                  let _ret = acontrol_system_get_display_drv( |display|{
                    let _ret = display.clear_and_stop_animations();
                  });
                }
              }              
            });
          },
          NFCSystemState::AUTHORIZE => {
            if let Err(err) = nfc_drv.format(&uuid) {
              acontrol_system_log!(LogType::Error, "Error formating. Is this a new card? Let's try to write anyway");
              acontrol_system_log!(LogType::Error, "format return: {}", err);
            }
  
            next_nfc_system_state = Some(NFCSystemState::WRITE)
          }
          NFCSystemState::WRITE => {
            if let Err(_err) = nfc_drv.write_data(&uuid, *NFC_CARD_SIGNATURE_BLOCK, &NFC_CARD_SIGNATURE.as_bytes().to_vec()) {
              acontrol_system_log!(LogType::Error, "No... we really have a problem here. Can't write either.");
              let _ret = acontrol_system_get_audio_drv(|audio|{
                let _ret = audio.play_error();
              });
              let _ret = acontrol_system_get_display_drv( |display|{
                let _ret = display.show_animation(Animation::Blink,AnimationColor::Red,AnimationType::Error,"Done",3);
                let _ret = display.wait_animation_ends();
              });
            } else {
              acontrol_system_log!(LogType::Info, "Ok... signature written successfully!");
              let _ = acontrol_system_get_persist_drv( |persist_drv| {
                if let Ok(ref mut params) = asystem.nfc_state_params.lock() {
                  if let Err(_err) = persist_drv.nfc_find(&uuid) {
                    if let Err(err) = persist_drv.nfc_add(&uuid, &params[&String::from("name")].as_bytes().to_vec()) {
                      acontrol_system_log!(LogType::Error, "Error persisting card info. Card not authorized! => ({})",err);
                      let _ret = acontrol_system_get_audio_drv(|audio|{
                        let _ret = audio.play_error();
                      });
                      let _ret = acontrol_system_get_display_drv( |display|{
                        let _ret = display.show_animation(Animation::Blink,AnimationColor::Red,AnimationType::Error,"Done",3);
                        let _ret = display.wait_animation_ends();
                      });
                    } else {
                      acontrol_system_log!(LogType::Info, "Card successfully added");
                      let _ret = acontrol_system_get_audio_drv(|audio|{
                        let _ret = audio.play_new();
                      });
                      let _ret = acontrol_system_get_display_drv( |display|{
                        let _ret = display.show_animation(Animation::Blink,AnimationColor::Green,AnimationType::Success, "Done",3);
                        let _ret = display.wait_animation_ends();
                      });
                    }
                  } else {
                    acontrol_system_log!(LogType::Warning, "Card already white listed");
                  }
                }
              });
            }
            next_nfc_system_state = Some(NFCSystemState::READ);
          },
          NFCSystemState::RESTORE => {
            if let Err(err) = nfc_drv.restore(&uuid) {
              acontrol_system_log!(LogType::Error, "Error restoring!");
              acontrol_system_log!(LogType::Error, "format return: {}", err);
              let _ret = acontrol_system_get_audio_drv(|audio|{
                let _ret = audio.play_error();
              });
              let _ret = acontrol_system_get_display_drv( |display|{
                let _ret = display.show_animation(Animation::Blink,AnimationColor::Red,AnimationType::Success, "Done",3);
                let _ret = display.wait_animation_ends();
              });
            } else {
              let _ret = acontrol_system_get_audio_drv(|audio|{
                let _ret = audio.play_success();
              });
              let _ret = acontrol_system_get_display_drv( |display|{
                let _ret = display.show_animation(Animation::Blink,AnimationColor::Green,AnimationType::Success, "Done",3);
                let _ret = display.wait_animation_ends();
              });
            }
            next_nfc_system_state = Some(NFCSystemState::READ)
          }
        }
  
        if let Some(state) = next_nfc_system_state {
          acontrol_system_set_nfc_state(state,None);
        }  
      }
    }
  }
  return true;
}

pub async fn acontrol_system_init(_params: &HashMap<String,String>,
        bt_drv: Option<Box<dyn Bluetooth+Sync+Send>>,
        fingerprint_drv: Option<Box<dyn Fingerprint+Sync+Send>>,
				nfc_drv: Option<Box<dyn NfcReader+Sync+Send>>,
				audio_drv: Option<Box<dyn Audio+Sync+Send>>,
				persist_drv: Option<Box<dyn Persist+Sync+Send>>,
        display_drv: Option<Box<dyn Display+Sync+Send>>,
        log_drv: Option<Box<dyn Log+Sync+Send>>) -> bool {

  let mut bt_drv_final = Option::None;
  let mut fingerprint_drv_final = Option::None;
  let mut nfc_drv_final = Option::None;
  let mut audio_drv_final = Option::None;
  let mut persist_drv_final = Option::None;
  let mut display_drv_final = Option::None;
  let mut log_drv_final = Option::None;

  let asystem = &ACONTROL_SYSTEM;

  if let Some(mut drv) = log_drv {
    if let Err(err) = drv.init() {
      eprintln!("Error initializing log module: {}", err);
      return false;      
    }
    log_drv_final = Some(drv);
  }
  *asystem.log_drv.lock().unwrap() = log_drv_final;


  if let Some(mut drv) = bt_drv {
    if let Err(err) = drv.init().await {
      acontrol_system_log!(LogType::Error, "Error initializing bluetooth module: {}", err);
      return false;
    }
    bt_drv_final = Some(drv);
  }
  *asystem.bt_drv.lock().unwrap() = bt_drv_final;

  if let Some(mut drv) = fingerprint_drv {
    if let Err(err) = drv.init() {
      acontrol_system_log!(LogType::Error, "Error initializing fingerprint module: {}", err);
      return false;
    }
    fingerprint_drv_final = Some(drv);
  }
  *asystem.fingerprint_drv.lock().unwrap() = fingerprint_drv_final;

  if let Some(mut drv) = nfc_drv {
    if let Err(err) = drv.init() {
      acontrol_system_log!(LogType::Error, "Error initializing nfc module: {}", err);
      return false;
    }
    nfc_drv_final = Some(drv);
  }
  *asystem.nfc_drv.lock().unwrap() = nfc_drv_final;

  if let Some(mut drv) = audio_drv {
    if let Err(err) = drv.init(){
      acontrol_system_log!(LogType::Error, "Error initializing audio module: {}", err);
      return false;
    }
    audio_drv_final = Some(drv);
  }
  *asystem.audio_drv.lock().unwrap() = audio_drv_final;

  if let Some(mut drv) = display_drv {
    if let Err(err) = drv.init() {
      acontrol_system_log!(LogType::Error, "Error initializing display module: {}", err);
      return false;      
    }
    display_drv_final = Some(drv);
  }
  *asystem.display_drv.lock().unwrap() = display_drv_final;

  if let Some(drv) = persist_drv {
    persist_drv_final = Some(drv);
  }
  *asystem.persist_drv.lock().unwrap() = persist_drv_final;

  if let Ok(ref mut drv_locked) = asystem.bt_drv.lock() {
      if let Some(ref mut drv) = **drv_locked {
        if let Err(err) = drv.find_devices(find_bt_device).await {
          acontrol_system_log!(LogType::Error, "Bluetooth module error: {}", err);
          return false;    
        }
    };
  }

  if let Ok(ref mut drv_locked) = asystem.fingerprint_drv.lock() {
    if let Some(ref mut drv) = **drv_locked {
      if let Err(err) = drv.wait_for_finger(find_finger) {
        acontrol_system_log!(LogType::Error, "Fingerprint module error: {}", err);
        return false;    
      }
    };
  }

  if let Ok(ref mut drv_locked) = asystem.nfc_drv.lock() {
    if let Some(ref mut drv) = **drv_locked {
      if let Err(err) = drv.find_tag(find_tag) {
        acontrol_system_log!(LogType::Error, "Fingerprint module error: {}", err);
        return false;    
      }
    };
  }

  return true;
}

pub fn acontrol_system_set_bluetooth_state(state: BluetoothSystemState, params: Option<HashMap<String,String>>) {
  acontrol_system_log!(LogType::Debug, "Changing Bluetooth System State");
  {
    let asystem = acontrol_system_get();
    if let Some(p) = params {
      if let Ok(ref mut state_params) = asystem.bt_state_params.lock() {
        **state_params = p;
      }
    }  
  }

  {
    let asystem = acontrol_system_get();
    if let Ok(ref mut bt_state) = asystem.bt_state.lock() {
      **bt_state = state;
  
      if **bt_state == BluetoothSystemState::AUTHORIZE {
        let _ret = acontrol_system_get_audio_drv(|audio|{
          let _ret = audio.play_alert();
        });
        let _ret = acontrol_system_get_display_drv( |display|{
          let _ret = display.show_animation(Animation::MaterialSpinner, AnimationColor::Orange, AnimationType::Waiting, "Waiting",0);
        });  
      }
    };
  
  }
}

pub fn acontrol_system_set_nfc_state(state: NFCSystemState, params: Option<HashMap<String,String>>) {
  acontrol_system_log!(LogType::Debug, "Changing NFC System State");

  {
    let asystem = acontrol_system_get();
    if let Some(p) = params {
      if let Ok(ref mut state_params) = asystem.nfc_state_params.lock() {
        **state_params = p;
      }
    }  
  }

  {
    let asystem = acontrol_system_get();
    if let Ok(ref mut nfc_state) = asystem.nfc_state.lock(){
      **nfc_state = state;
  
      if **nfc_state == NFCSystemState::AUTHORIZE {
        let _ret = acontrol_system_get_audio_drv(|audio|{
          let _ret = audio.play_alert();
        });
        let _ret = acontrol_system_get_display_drv( |display|{
          let _ret = display.show_animation(Animation::MaterialSpinner, AnimationColor::Orange, AnimationType::Waiting, "Waiting",0);
        });
      }
    }; 
  }
}

pub fn acontrol_system_fingerprint_delete_all(_params: HashMap<String,String>) -> Result<(), String> {
  let asystem = acontrol_system_get();
  acontrol_system_log!(LogType::Info, "System Delete All");
  
  if let Ok(ref mut drv_locked) = asystem.fingerprint_drv.lock() {
    if let Some(ref mut drv) = **drv_locked {
      if !drv.delete_all() {
        return Err(String::from("Error deleting all fingerprints"));
      }
    }
  } else {
    return Err(String::from("Fingerprint device not found"));
  }
  Ok(())
}


pub fn acontrol_system_fingerprint_start_enroll(params: HashMap<String,String>) -> Result<(), String> {
  let asystem = acontrol_system_get();

  acontrol_system_log!(LogType::Info, "System Start Enroll");
  
  if let Ok(ref mut drv_lock) = asystem.fingerprint_drv.lock() {
    if let Some(ref mut drv) = **drv_lock {
      let pos = (&params[&String::from("pos")]).parse::<u16>().unwrap();
      acontrol_system_log!(LogType::Info, "Adding a fingerprint at pos {} to {}", pos, &params[&String::from("name")]);

      if let Ok(ref mut data_locked) = asystem.fingerprint_data.lock() {

        //*ACONTROL_SYSTEM.fingerprint_data.lock().unwrap() = FingerprintData::new(pos, &params[&String::from("name")]);
        data_locked.pos = Some(pos);
        data_locked.name = Some(params[&String::from("name")].clone());

        //let mut data = *ACONTROL_SYSTEM.fingerprint_data.lock().unwrap();

        if !drv.start_enroll(&*data_locked) {
          return Err(String::from("Error starting enrollment"));
        }
      }
    }
  } else {
    return Err(String::from("Fingerprint device not found"));
  }
  Ok(())
}

pub fn acontrol_system_get_persist_drv<F, T>(f: F) -> Result<(),String>
  where F: FnOnce(&mut Box<dyn Persist + Send + Sync>) -> T, {
    let asystem = acontrol_system_get();

    if let Ok(ref mut drv_lock) = asystem.persist_drv.lock() {
      if let Some(ref mut drv) = **drv_lock {
        f(drv);
      }
    } else {
      return Err(String::from("Display module not found"));
    }

    Ok(())
}

pub fn acontrol_system_get_display_drv<F, T>(f:F) -> Result<(),String>
  where F: FnOnce(&mut Box<dyn Display + Send + Sync>) -> T {
    let asystem = acontrol_system_get();

    acontrol_system_log!(LogType::Debug, "acontrol_system_get_display_drv entered");
    if let Ok(ref mut drv_lock) = asystem.display_drv.lock() {
      if let Some(ref mut drv) = **drv_lock {
        f(drv);
      }
    } else {
      return Err(String::from("Display module not found"));
    }
    acontrol_system_log!(LogType::Debug, "acontrol_system_get_display_drv exited");

    Ok(())
}

pub fn acontrol_system_get_audio_drv<F, T>(f:F) -> Result<(),String>
  where F: FnOnce(&mut Box<dyn Audio + Send + Sync>) -> T {
    let asystem = acontrol_system_get();

    if let Ok(ref mut drv_locked) = asystem.audio_drv.lock() {
      if let Some(ref mut drv) = **drv_locked {
        f(drv);
      }
    } else {
      return Err(String::from("Audio module not found"));
    }
    Ok(())
}

pub fn acontrol_system_get() -> &'static ACONTROL_SYSTEM {
  return &ACONTROL_SYSTEM;
}
