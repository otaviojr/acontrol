/**
 * @file   system.rs
 * @author Otavio Ribeiro
 * @date   24 Dec 2017
 * @brief  System operation/logic
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
use fingerprint::{Fingerprint, FingerprintState, FingerprintData};
use nfc::{NfcReader};
use audio::{Audio};
use persist::{Persist};
use display::{Display, Animation, AnimationType, AnimationColor};

use std::sync::Mutex;
use std::collections::HashMap;

use std::process::Command;
use std::process::Stdio;

#[derive(PartialEq)]
#[allow(dead_code)]
pub enum NFCSystemState {
  READ,
  WRITE,
  AUTHORIZE,
  RESTORE
}

struct AControlSystem {
  fingerprint_drv: Mutex<Option<Mutex<Box<dyn Fingerprint + Send + Sync>>>>,
  nfc_drv: Mutex<Option<Mutex<Box<dyn NfcReader + Send + Sync>>>>,
  audio_drv: Mutex<Option<Mutex<Box<dyn Audio + Send + Sync>>>>,
  persist_drv:  Mutex<Option<Mutex<Box<dyn Persist + Send + Sync>>>>,
  display_drv: Mutex<Option<Mutex<Box<dyn Display + Send + Sync>>>>,
  nfc_state: Mutex<NFCSystemState>,
  nfc_state_params: Mutex<HashMap<String,String>>,
  fingerprint_data: Mutex<FingerprintData>,
  fingerprint_last_state: Mutex<Option<FingerprintState>>,
}

impl AControlSystem {
}

lazy_static!{
  static ref ACONTROL_SYSTEM: AControlSystem = AControlSystem {
    fingerprint_drv: Mutex::new(None),
    nfc_drv: Mutex::new(None),
    audio_drv: Mutex::new(None),
    persist_drv:  Mutex::new(None),
    display_drv: Mutex::new(None),
    nfc_state: Mutex::new(NFCSystemState::READ),
    nfc_state_params: Mutex::new(HashMap::new()),
    fingerprint_data: Mutex::new(FingerprintData::empty()),
    fingerprint_last_state: Mutex::new(None),
  };

  static ref NFC_CARD_SIGNATURE: &'static str = &"ACONTROL_CARD\0\0\0";
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

  match *ACONTROL_SYSTEM.fingerprint_drv.lock().unwrap() {
    Some(ref drv) => {
      let mut drv_inner = drv.lock().unwrap();
      if let Err(err) = drv_inner.unload() {
        eprintln!("Error unloading fingerprint device (=> {})", err);
        return false;
      }
    },
    None => {
      eprintln!("Fingerprint device unloaded");
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

  match *ACONTROL_SYSTEM.persist_drv.lock().unwrap() {
    Some(ref drv) => {
      if let Err(err) = drv.lock().unwrap().unload() {
        eprintln!("Error unload persistence driver (=> {})", err);
        return false;
      }
    },
    None => {
      eprintln!("Persistence driver not found");
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

pub fn acontrol_system_init(params: &HashMap<String,String>,
				fingerprint_drv: Box<dyn Fingerprint+Sync+Send>,
				nfc_drv: Box<dyn NfcReader+Sync+Send>,
				audio_drv: Box<dyn Audio+Sync+Send>,
				persist_drv: Box<dyn Persist+Sync+Send>,
        display_drv: Box<dyn Display+Sync+Send>) -> bool {

  let asystem = &ACONTROL_SYSTEM;
  unsafe {
    *asystem.fingerprint_drv.lock().unwrap() = Some(Mutex::new(Box::from_raw(Box::into_raw(fingerprint_drv))));
    *asystem.nfc_drv.lock().unwrap() = Some(Mutex::new(Box::from_raw(Box::into_raw(nfc_drv))));
    *asystem.audio_drv.lock().unwrap() = Some(Mutex::new(Box::from_raw(Box::into_raw(audio_drv))));
    *asystem.persist_drv.lock().unwrap() = Some(Mutex::new(Box::from_raw(Box::into_raw(persist_drv))));
    *asystem.display_drv.lock().unwrap() = Some(Mutex::new(Box::from_raw(Box::into_raw(display_drv))));
  }

  //initializing persistence driver
  match *asystem.persist_drv.lock().unwrap() {
    Some(ref drv) => {
      if let Err(err) = drv.lock().unwrap().init(params)  {
        eprintln!("Error initializing persistence driver: {}", err);
        return false;
      }
    },
    None => {
      eprintln!("Persistence driver not found");
      return false;
    }
  }

  //initializing display device
  match *asystem.display_drv.lock().unwrap()  {
    Some(ref drv) => {
      if let Err(err) = drv.lock().unwrap().init() {
        eprintln!("Error initializing display device (=> {})", err);
        return false;
      }
    },
    None => {
      eprintln!("Display device not found");
      return false;
    }
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

  match *asystem.fingerprint_drv.lock().unwrap() {
    Some(ref drv) => {
      let mut drv_inner = drv.lock().unwrap();

      if let Err(err) = drv_inner.init() {
        eprintln!("Error initializing fingerprint device (=> {})", err);
        return false;
      }

      let _ret = drv_inner.wait_for_finger( |state, _value| {
        if let Ok(ref mut last_state_locked) = (*ACONTROL_SYSTEM).fingerprint_last_state.lock() {
          if last_state_locked.is_none() || last_state_locked.unwrap() != *state {

            last_state_locked.replace(*state);

            println!("Fingerprint Current State Changed To: {}", state.name());

            match state {
              FingerprintState::IDLE => {
                let _ret = acontrol_system_get_display_drv(|display|{
                  let _ret = display.clear_and_stop_animations();
                });
              },
              FingerprintState::READING => {
                let _ret = acontrol_system_get_display_drv(|display|{
                  let _ret = display.show_animation(Animation::MaterialSpinner, AnimationColor::Orange, AnimationType::Waiting, "Waiting",0);
                });
              },
              FingerprintState::WAITING => {
                let _ret = acontrol_system_get_audio_drv(|audio|{
                  let _ret = audio.play_alert();
                });
                let _ret = acontrol_system_get_display_drv(|display|{
                  let _ret = display.show_animation(Animation::MaterialSpinner, AnimationColor::Orange, AnimationType::Waiting, "Waiting",0);
                });
              },
              FingerprintState::SUCCESS => {
                let _ret = acontrol_system_get_audio_drv(|audio|{
                  let _ret = audio.play_success();
                });
                let _ret = acontrol_system_get_display_drv(|display|{
                  let _ret = display.show_animation(Animation::BlinkLoop,AnimationColor::Green,AnimationType::Success, "Done",0);
                });
              },
              FingerprintState::ERROR => {
                let _ret = acontrol_system_get_audio_drv(|audio|{
                  let _ret = audio.play_error();
                });
                let _ret = acontrol_system_get_display_drv(|display|{
                  let _ret = display.show_animation(Animation::Blink,AnimationColor::Red,AnimationType::Error,"Done",3);
                  let _ret = display.wait_animation_ends();
                });
              },
              FingerprintState::ENROLL => {
                let data_locked = (*ACONTROL_SYSTEM).fingerprint_data.lock().unwrap();
                if let (&Some(ref name), &Some(ref pos)) = (&data_locked.name, &data_locked.pos){

                  let _ret = acontrol_system_get_audio_drv(|audio|{
                    let _ret = audio.play_new();
                  });

                  let _ret = acontrol_system_get_display_drv(|display|{
                    let _ret = display.show_animation(Animation::BlinkLoop,AnimationColor::Green,AnimationType::Success, "Done",3);
                    let _ret = display.wait_animation_ends();
                  });
                  println!("User {} added at position {}", name, pos);
                }
              },
              FingerprintState::AUTHORIZED => {

                let mut process = Command::new("/acontrol/granted")
                    .arg("-f")
                    .stdout(Stdio::inherit())
                    .spawn()
                    .expect("failed to execute child");

                let _ret = acontrol_system_get_audio_drv(|audio|{
                  let _ret = audio.play_granted();
                });

                let _ret = acontrol_system_get_display_drv(|display|{
                  let _ret = display.show_animation(Animation::Blink,AnimationColor::Green,AnimationType::Success, "Done",3);
                  let _ret = display.wait_animation_ends();
                });

                let _ret = acontrol_system_get_display_drv(|display|{
                  let _ret = display.show_animation(Animation::MaterialSpinner, AnimationColor::Orange, AnimationType::Waiting, "Waiting",0);
                });

                let _ret = process.wait();

                let _ret = acontrol_system_get_display_drv(|display|{
                  let _ret = display.clear_and_stop_animations();
                });

              }
              FingerprintState::NOT_AUTHORIZED => {

                let mut process = Command::new("/acontrol/denieded")
                    .arg("-f")
                    .stdout(Stdio::inherit())
                    .spawn()
                    .expect("failed to execute child");

                let _ret = acontrol_system_get_audio_drv(|audio|{
                  let _ret = audio.play_denied();
                });
                let _ret = acontrol_system_get_display_drv(|display|{
                  let _ret = display.show_animation(Animation::BlinkLoop,AnimationColor::Red,AnimationType::Error,"Done",3);
                  let _ret = display.wait_animation_ends();
                });

                let _ret = acontrol_system_get_display_drv(|display|{
                  let _ret = display.show_animation(Animation::MaterialSpinner, AnimationColor::Orange, AnimationType::Waiting, "Waiting",0);
                });

                let _ret = process.wait();

                let _ret = acontrol_system_get_display_drv(|display|{
                  let _ret = display.clear_and_stop_animations();
                });
              }
            }
          }
        }
        return true;
      });
    },
    None => {
      eprintln!("Fingerprint device not found");
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

      drv_inner.find_tag(|_card_type, uuid|{

        match *ACONTROL_SYSTEM.nfc_drv.lock().unwrap() {

          Some(ref mut drv) => {

            let mut next_nfc_system_state: Option<NFCSystemState> = None;
            let mut drv_inner = drv.lock().unwrap();

            println!("Card Found: UUID={:X?}", uuid);

            match *ACONTROL_SYSTEM.nfc_state.lock().unwrap() {
              NFCSystemState::READ => {
                match *ACONTROL_SYSTEM.persist_drv.lock().unwrap() {
                  Some(ref mut drv) => {
                    match drv_inner.read_data(&uuid,*NFC_CARD_SIGNATURE_BLOCK,0) {
                      Ok(ref val) => {
                        if let Ok(value) = String::from_utf8(val.to_vec()) {
                            if value ==
                               String::from_utf8(NFC_CARD_SIGNATURE.as_bytes().to_vec()).unwrap() {

                              if let Ok(card) = drv.lock().unwrap().nfc_find(&uuid) {
                                println!("Card {:?} from {} authorized!", uuid, String::from_utf8(card.name).unwrap());

                                let mut process = Command::new("/acontrol/granted")
                                    .arg("-c")
                                    .stdout(Stdio::inherit())
                                    .spawn()
                                    .expect("failed to execute child");

                                let _ret = acontrol_system_get_audio_drv(|audio|{
                                  let _ret = audio.play_granted();
                                });
                                let _ret = acontrol_system_get_display_drv(|display|{
                                  let _ret = display.show_animation(Animation::Blink,AnimationColor::Green,AnimationType::Success, "Done",3);
                                  let _ret = display.wait_animation_ends();
                                });

                                let _ret = acontrol_system_get_display_drv(|display|{
                                  let _ret = display.show_animation(Animation::MaterialSpinner, AnimationColor::Orange, AnimationType::Waiting, "Waiting",0);
                                });

                                let _ret = process.wait();

                                let _ret = acontrol_system_get_display_drv(|display|{
                                  let _ret = display.clear_and_stop_animations();
                                });
                              } else {
                                println!("Card {:?} not found!", uuid);

                                let mut process = Command::new("/acontrol/denieded")
                                    .arg("-c")
                                    .stdout(Stdio::inherit())
                                    .spawn()
                                    .expect("failed to execute child");

                                let _ret = acontrol_system_get_audio_drv(|audio|{
                                  let _ret = audio.play_denied();
                                });
                                let _ret = acontrol_system_get_display_drv(|display|{
                                  let _ret = display.show_animation(Animation::Blink,AnimationColor::Red,AnimationType::Error,"Done",3);
                                  let _ret = display.wait_animation_ends();
                                });

                                let _ret = acontrol_system_get_display_drv(|display|{
                                  let _ret = display.show_animation(Animation::MaterialSpinner, AnimationColor::Orange, AnimationType::Waiting, "Waiting",0);
                                });

                                let _ret = process.wait();

                                let _ret = acontrol_system_get_display_drv(|display|{
                                  let _ret = display.clear_and_stop_animations();
                                });
                              }
                            } else {
                              println!("Invalid card signature: {:?} - {:?}",val, NFC_CARD_SIGNATURE.as_bytes().to_vec());

                              let mut process = Command::new("/acontrol/denieded")
                                  .arg("-c")
                                  .stdout(Stdio::inherit())
                                  .spawn()
                                  .expect("failed to execute child");

                              let _ret = acontrol_system_get_audio_drv(|audio|{
                                let _ret = audio.play_denied();
                              });
                              let _ret = acontrol_system_get_display_drv(|display|{
                                let _ret = display.show_animation(Animation::Blink,AnimationColor::Red,AnimationType::Error,"Done",3);
                                let _ret = display.wait_animation_ends();
                              });

                              let _ret = acontrol_system_get_display_drv(|display|{
                                let _ret = display.show_animation(Animation::MaterialSpinner, AnimationColor::Orange, AnimationType::Waiting, "Waiting",0);
                              });

                              let _ret = process.wait();

                              let _ret = acontrol_system_get_display_drv(|display|{
                                let _ret = display.clear_and_stop_animations();
                              });
                            }
                        } else {
                            println!("Error reading card block: {:X?}", val);
                        }
                      },
                      Err(err) => {
                        println!("Error reading card: {}", err);

                        let mut process = Command::new("/acontrol/denieded")
                            .arg("-c")
                            .stdout(Stdio::inherit())
                            .spawn()
                            .expect("failed to execute child");

                        let _ret = acontrol_system_get_audio_drv(|audio|{
                          let _ret = audio.play_denied();
                        });
                        let _ret = acontrol_system_get_display_drv(|display|{
                          let _ret = display.show_animation(Animation::Blink,AnimationColor::Red,AnimationType::Error,"Done",3);
                          let _ret = display.wait_animation_ends();
                        });

                        let _ret = acontrol_system_get_display_drv(|display|{
                          let _ret = display.show_animation(Animation::MaterialSpinner, AnimationColor::Orange, AnimationType::Waiting, "Waiting",0);
                        });

                        let _ret = process.wait();

                        let _ret = acontrol_system_get_display_drv(|display|{
                          let _ret = display.clear_and_stop_animations();
                        });
                      }
                    }
                  },
                  None => {
                    println!("Persistence driver not found");
                  }
                }
              },
              NFCSystemState::AUTHORIZE => {
                if let Err(err) = drv_inner.format(&uuid) {
                  eprintln!("Error formating. Is this a new card? Let's try to write anyway");
                  eprintln!("format return: {}", err);
                }

                next_nfc_system_state = Some(NFCSystemState::WRITE)
              }
              NFCSystemState::WRITE => {
                if let Err(_err) = drv_inner.write_data(&uuid, *NFC_CARD_SIGNATURE_BLOCK, &NFC_CARD_SIGNATURE.as_bytes().to_vec()) {
                  eprintln!("No... we really have a problem here. Can't write either.");
                  let _ret = acontrol_system_get_audio_drv(|audio|{
                    let _ret = audio.play_error();
                  });
                  let _ret = acontrol_system_get_display_drv(|display|{
                    let _ret = display.show_animation(Animation::Blink,AnimationColor::Red,AnimationType::Error,"Done",3);
                    let _ret = display.wait_animation_ends();
                  });
                } else {
                  println!("Ok... signature written successfully!");
                  match *ACONTROL_SYSTEM.persist_drv.lock().unwrap() {
                    Some(ref drv) => {
                      let params: &HashMap<String,String> = &*ACONTROL_SYSTEM.nfc_state_params.lock().unwrap();
                      let ref mut persist_drv = &mut drv.lock().unwrap();
                      if let Err(_err) = persist_drv.nfc_find(&uuid) {
                        if let Err(err) = persist_drv.nfc_add(&uuid, &params[&String::from("name")].as_bytes().to_vec()) {
                          eprintln!("Error persisting card info. Card not authorized! => ({})",err);
                          let _ret = acontrol_system_get_audio_drv(|audio|{
                            let _ret = audio.play_error();
                          });
                          let _ret = acontrol_system_get_display_drv(|display|{
                            let _ret = display.show_animation(Animation::Blink,AnimationColor::Red,AnimationType::Error,"Done",3);
                            let _ret = display.wait_animation_ends();
                          });
                        } else {
                          println!("Card successfully added");
                          let _ret = acontrol_system_get_audio_drv(|audio|{
                            let _ret = audio.play_new();
                          });
                          let _ret = acontrol_system_get_display_drv(|display|{
                            let _ret = display.show_animation(Animation::Blink,AnimationColor::Green,AnimationType::Success, "Done",3);
                            let _ret = display.wait_animation_ends();
                          });
                        }
                      } else {
                        println!("Card already white listed");
                      }
                    },
                    None => {
                      eprintln!("Persistence driver not found");
                    }
                  }
                }
                next_nfc_system_state = Some(NFCSystemState::READ);
              },
              NFCSystemState::RESTORE => {
                if let Err(err) = drv_inner.restore(&uuid) {
                  eprintln!("Error restoring!");
                  eprintln!("format return: {}", err);
                  let _ret = acontrol_system_get_audio_drv(|audio|{
                    let _ret = audio.play_error();
                  });
                  let _ret = acontrol_system_get_display_drv(|display|{
                    let _ret = display.show_animation(Animation::Blink,AnimationColor::Red,AnimationType::Success, "Done",3);
                    let _ret = display.wait_animation_ends();
                  });
                } else {
                  let _ret = acontrol_system_get_audio_drv(|audio|{
                    let _ret = audio.play_success();
                  });
                  let _ret = acontrol_system_get_display_drv(|display|{
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

pub fn acontrol_system_set_nfc_state(state: NFCSystemState, params: Option<HashMap<String,String>>) {
  println!("Changing NFC System State");
  if let Some(p) = params {
    *ACONTROL_SYSTEM.nfc_state_params.lock().unwrap() = p;
  }
  *ACONTROL_SYSTEM.nfc_state.lock().unwrap() = state;

  if *ACONTROL_SYSTEM.nfc_state.lock().unwrap() == NFCSystemState::AUTHORIZE {
    let _ret = acontrol_system_get_audio_drv(|audio|{
      let _ret = audio.play_alert();
    });
    let _ret = acontrol_system_get_display_drv(|display|{
      let _ret = display.show_animation(Animation::MaterialSpinner, AnimationColor::Orange, AnimationType::Waiting, "Waiting",0);
    });
  }
}

pub fn acontrol_system_fingerprint_delete_all(_params: HashMap<String,String>) -> Result<(), String>{
    println!("System Delete All");
    match *ACONTROL_SYSTEM.fingerprint_drv.lock().unwrap() {
      Some(ref drv) => {
        let mut drv_inner = drv.lock().unwrap();

        if !drv_inner.delete_all() {
            return Err(String::from("Error deleting all fingerprints"));
        }

        return Ok(());
      },
      None => {
        return Err(String::from("Fingerprint device unloaded"));
      }
    }
}

pub fn acontrol_system_fingerprint_start_enroll(params: HashMap<String,String>) -> Result<(), String>{
  println!("System Start Enroll");
  match *ACONTROL_SYSTEM.fingerprint_drv.lock().unwrap() {
    Some(ref drv) => {
      let mut drv_inner = drv.lock().unwrap();

      let pos = (&params[&String::from("pos")]).parse::<u16>().unwrap();
      println!("Adding a fingerprint at pos {} to {}", pos, &params[&String::from("name")]);

      if let Ok(ref mut data_locked) = (*ACONTROL_SYSTEM).fingerprint_data.lock() {

        //*ACONTROL_SYSTEM.fingerprint_data.lock().unwrap() = FingerprintData::new(pos, &params[&String::from("name")]);
        data_locked.pos = Some(pos);
        data_locked.name = Some(params[&String::from("name")].clone());

        //let mut data = *ACONTROL_SYSTEM.fingerprint_data.lock().unwrap();

        if !drv_inner.start_enroll(&*data_locked) {
          return Err(String::from("Error starting enrollment"));
        }
     }
     return Ok(())
    },
    None => {
      return Err(String::from("Fingerprint device unloaded"));
    }
  }
}

pub fn acontrol_system_get_persist_drv<F, T>(f: F) -> Result<(),String>
  where F: FnOnce(&mut dyn Persist) -> T, {

  match *ACONTROL_SYSTEM.persist_drv.lock().unwrap()  {
    Some(ref drv) => {
      let persist = &mut *(*drv.lock().unwrap());
      f(persist);
      Ok(())
    },
    None => {
      println!("Ops! Error getting persistence at this time!");
      Err(String::from("Persistence driver not found"))
    }
  }
}

pub fn acontrol_system_get_display_drv<F, T>(f:F) -> Result<(),String>
  where F: FnOnce(&mut dyn Display) -> T {

  match *ACONTROL_SYSTEM.display_drv.lock().unwrap()  {
    Some(ref drv) => {
      let display = &mut *(*drv.lock().unwrap());
      f(display);
      Ok(())
    },
    None => {
      println!("Ops! Error getting display at this time!");
      Err(String::from("Display device not found"))
    }
  }
}

pub fn acontrol_system_get_audio_drv<F, T>(f:F) -> Result<(),String>
  where F: FnOnce(&mut dyn Audio) -> T {

  match *ACONTROL_SYSTEM.audio_drv.lock().unwrap()  {
    Some(ref drv) => {
      let audio = &mut *(*drv.lock().unwrap());
      f(audio);
      Ok(())
    },
    None => {
      Err(String::from("Audio device not found"))
    }
  }
}
