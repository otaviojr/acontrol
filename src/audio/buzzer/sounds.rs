/**
 * @file   sounds/mod.rs
 * @author Otavio Ribeiro
 * @date   24 Dec 2017
 * @brief  System sounds
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

use std::thread;
use std::time::Duration;

use super::{Buzzer, AudioThreadCommand};

#[derive(Clone, Copy)]
#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub enum Tone {
  NOTE_NULL = 0,
  NOTE_B0   =  31,
  NOTE_C1   =  33,
  NOTE_CS1  =  35,
  NOTE_D1   =  37,
  NOTE_DS1  =  39,
  NOTE_E1   =  41,
  NOTE_F1   =  44,
  NOTE_FS1  =  46,
  NOTE_G1   =  49,
  NOTE_GS1  =  52,
  NOTE_A1   =  55,
  NOTE_AS1  =  58,
  NOTE_B1   =  62,
  NOTE_C2   =  65,
  NOTE_CS2  =  69,
  NOTE_D2   =  73,
  NOTE_DS2  =  78,
  NOTE_E2   =  82,
  NOTE_F2   =  87,
  NOTE_FS2  =  93,
  NOTE_G2   =  98,
  NOTE_GS2  =  104,
  NOTE_A2   =  110,
  NOTE_AS2  =  117,
  NOTE_B2   =  123,
  NOTE_C3   =  131,
  NOTE_CS3  =  139,
  NOTE_D3   =  147,
  NOTE_DS3  =  156,
  NOTE_E3   =  165,
  NOTE_F3   =  175,
  NOTE_FS3  =  185,
  NOTE_G3   =  196,
  NOTE_GS3  =  208,
  NOTE_A3   =  220,
  NOTE_AS3  =  233,
  NOTE_B3   =  247,
  NOTE_C4   =  262,
  NOTE_CS4  =  277,
  NOTE_D4   =  294,
  NOTE_DS4  =  311,
  NOTE_E4   =  330,
  NOTE_F4   =  349,
  NOTE_FS4  =  370,
  NOTE_G4   =  392,
  NOTE_GS4  =  415,
  NOTE_A4   =  440,
  NOTE_AS4  =  466,
  NOTE_B4   =  494,
  NOTE_C5   =  523,
  NOTE_CS5  =  554,
  NOTE_D5   =  587,
  NOTE_DS5  =  622,
  NOTE_E5   =  659,
  NOTE_F5   =  698,
  NOTE_FS5  =  740,
  NOTE_G5   =  784,
  NOTE_GS5  =  831,
  NOTE_A5   =  880,
  NOTE_AS5  =  932,
  NOTE_B5   =  988,
  NOTE_C6   =  1047,
  NOTE_CS6  =  1109,
  NOTE_D6   =  1175,
  NOTE_DS6  =  1245,
  NOTE_E6   =  1319,
  NOTE_F6   =  1397,
  NOTE_FS6  =  1480,
  NOTE_G6   =  1568,
  NOTE_GS6  =  1661,
  NOTE_A6   =  1760,
  NOTE_AS6  =  1865,
  NOTE_B6   =  1976,
  NOTE_C7   =  2093,
  NOTE_CS7  =  2217,
  NOTE_D7   =  2349,
  NOTE_DS7  =  2489,
  NOTE_E7   =  2637,
  NOTE_F7   =  2794,
  NOTE_FS7  =  2960,
  NOTE_G7   =  3136,
  NOTE_GS7  =  3322,
  NOTE_A7   =  3520,
  NOTE_AS7  =  3729,
  NOTE_B7   =  3951,
  NOTE_C8   =  4186,
  NOTE_CS8  =  4435,
  NOTE_D8   =  4699,
  NOTE_DS8  =  4978,  
}

impl Tone {
  pub fn value(&self) -> i32 {
    return (*self) as i32;
  }
}

pub struct Sounds {
}

impl Sounds {

  pub fn play_sound(buzzer: &mut Buzzer, tones: Vec<Tone>, periods: Vec<i32>) -> Result<(),String> {
    let buzzer_cloned = buzzer.buzzer.clone();

    if tones.len() != periods.len() {
      return Err(String::from("tones and periods differs in length."));
    }

    if let Some(thread) = buzzer.sound_worker.take() {

      // Ask the running thread to exit. 
      // Message will not be received if it's not running already.
      if let Ok(ref mut buzzer_locked) = buzzer_cloned.lock(){
        match buzzer_locked.sound_worker_tx.lock().unwrap().send(AudioThreadCommand::Stop) {
          Ok(_ret) => {

          },
          Err(err) => return Err(format!("Error sending message: {:?}", err))
        }
      }

      //Wait for the last thread to finish.
      //Will exit immediately if the thread have already ended.
      println!("WAITING PREVIOUS SOUND TO LEAVE");
      let _ret = thread.join();
      println!("LAST SOUND LEAVE. STARTING NEW THREAD!");

      //zero out unreceived messages.
      //If thread has already gone, the last exit message will be received by
      //the new thread. This will prevent that behavior.
      if let Ok(ref mut buzzer_locked) = buzzer_cloned.lock(){
        while let Ok(_ret) = buzzer_locked.sound_worker_rx.lock().unwrap().try_recv() {

        }
      }
    }

    let handle = thread::spawn( move || {
      for (i, tone) in tones.iter().enumerate() {
        if periods[i] != 0 {
          if let Ok(ref mut buzzer_locked) = buzzer_cloned.lock() {
            let _ret = (*buzzer_locked).play_tone(*tone, periods[i]);
          }
        } else {
          thread::sleep(Duration::from_millis(periods[i] as u64));
        }

        if let Ok(ref mut buzzer_locked) = buzzer_cloned.lock() {
          match buzzer_locked.sound_worker_rx.lock().unwrap().try_recv() {
            Ok(msg) => {
              match msg {
                AudioThreadCommand::Stop => {
                  println!("THREAD INTERRUPTED! WILL START A NEW SOUND?");
                  return Err(String::from("Interrupted"));
                },
              }
            },
            Err(_) => {},
          }
        }
      };
      Ok(())
    });

    buzzer.sound_worker = Some(handle);

    Ok(())
  }

  pub fn play_piratescaribean(buzzer: &mut Buzzer) -> Result<(), String> {
    let mut tones: Vec<Tone> = vec!(
      Tone::NOTE_E4, Tone::NOTE_G4, Tone::NOTE_A4, Tone::NOTE_A4,
      Tone::NOTE_NULL,
      Tone::NOTE_A4, Tone::NOTE_B4, Tone::NOTE_C5, Tone::NOTE_C5,
      Tone::NOTE_NULL,
      Tone::NOTE_C5, Tone::NOTE_D5, Tone::NOTE_B4, Tone::NOTE_B4,
      Tone::NOTE_NULL,
      Tone::NOTE_A4, Tone::NOTE_G4, Tone::NOTE_A4,
      Tone::NOTE_NULL,
      Tone::NOTE_E4, Tone::NOTE_G4, Tone::NOTE_A4, Tone::NOTE_A4, 
      Tone::NOTE_NULL,
      Tone::NOTE_A4, Tone::NOTE_B4, Tone::NOTE_C5, Tone::NOTE_C5,
      Tone::NOTE_NULL,
      Tone::NOTE_C5, Tone::NOTE_D5, Tone::NOTE_B4, Tone::NOTE_B4,
      Tone::NOTE_NULL,
      Tone::NOTE_A4, Tone::NOTE_G4, Tone::NOTE_A4, 
      Tone::NOTE_NULL,
      Tone::NOTE_E4, Tone::NOTE_G4, Tone::NOTE_A4, Tone::NOTE_A4, 
      Tone::NOTE_NULL,
      Tone::NOTE_A4, Tone::NOTE_C5, Tone::NOTE_D5, Tone::NOTE_D5,
      Tone::NOTE_NULL,
      Tone::NOTE_D5, Tone::NOTE_E5, Tone::NOTE_F5, Tone::NOTE_F5,
      Tone::NOTE_NULL,
      Tone::NOTE_E5, Tone::NOTE_D5, Tone::NOTE_E5, Tone::NOTE_A4,
      Tone::NOTE_NULL,
      Tone::NOTE_A4, Tone::NOTE_B4, Tone::NOTE_C5, Tone::NOTE_C5,
      Tone::NOTE_NULL,
      Tone::NOTE_D5, Tone::NOTE_E5, Tone::NOTE_A4,
      Tone::NOTE_NULL, 
      Tone::NOTE_A4, Tone::NOTE_C5, Tone::NOTE_B4, Tone::NOTE_B4, 
      Tone::NOTE_NULL, 
      Tone::NOTE_C5, Tone::NOTE_A4, Tone::NOTE_B4,
      Tone::NOTE_NULL, 
      Tone::NOTE_A4, Tone::NOTE_A4, Tone::NOTE_E4, Tone::NOTE_G4, Tone::NOTE_A4, Tone::NOTE_A4, 
      Tone::NOTE_NULL, 
      Tone::NOTE_A4, Tone::NOTE_B4, Tone::NOTE_C5, Tone::NOTE_C5, 
      Tone::NOTE_NULL,  
      Tone::NOTE_C5, Tone::NOTE_D5, Tone::NOTE_B4, Tone::NOTE_B4, 
      Tone::NOTE_NULL, 
      Tone::NOTE_A4, Tone::NOTE_G4, Tone::NOTE_A4, 
      Tone::NOTE_NULL, 
      Tone::NOTE_E4, Tone::NOTE_G4, Tone::NOTE_A4, Tone::NOTE_A4, 
      Tone::NOTE_NULL, 
      Tone::NOTE_A4, Tone::NOTE_B4, Tone::NOTE_C5, Tone::NOTE_C5, 
      Tone::NOTE_NULL,  
      Tone::NOTE_C5, Tone::NOTE_D5, Tone::NOTE_B4, Tone::NOTE_B4,
      Tone::NOTE_NULL,  
      Tone::NOTE_A4, Tone::NOTE_G4, Tone::NOTE_A4, 
      Tone::NOTE_NULL, 
      Tone::NOTE_E4, Tone::NOTE_G4, Tone::NOTE_A4, Tone::NOTE_A4, 
      Tone::NOTE_NULL, 
      Tone::NOTE_A4, Tone::NOTE_C5, Tone::NOTE_D5, Tone::NOTE_D5, 
      Tone::NOTE_NULL, 
      Tone::NOTE_D5, Tone::NOTE_E5, Tone::NOTE_F5, Tone::NOTE_F5, 
      Tone::NOTE_NULL, 
      Tone::NOTE_E5, Tone::NOTE_D5, Tone::NOTE_E5, Tone::NOTE_A4, 
      Tone::NOTE_NULL, 
      Tone::NOTE_A4, Tone::NOTE_B4, Tone::NOTE_C5, Tone::NOTE_C5, 
      Tone::NOTE_NULL, 
      Tone::NOTE_D5, Tone::NOTE_E5, Tone::NOTE_A4, 
      Tone::NOTE_NULL, 
      Tone::NOTE_A4, Tone::NOTE_C5, Tone::NOTE_B4, Tone::NOTE_B4, 
      Tone::NOTE_NULL, 
      Tone::NOTE_C5, Tone::NOTE_A4, Tone::NOTE_B4, 
      Tone::NOTE_NULL, 
      Tone::NOTE_E5,
      Tone::NOTE_NULL,
      Tone::NOTE_F5,
      Tone::NOTE_NULL,
      Tone::NOTE_E5, Tone::NOTE_E5, 
      Tone::NOTE_NULL,
      Tone::NOTE_G5,
      Tone::NOTE_NULL,
      Tone::NOTE_E5, Tone::NOTE_D5, 
      Tone::NOTE_NULL,
      Tone::NOTE_D5,
      Tone::NOTE_NULL,
      Tone::NOTE_C5,
      Tone::NOTE_NULL,
      Tone::NOTE_B4, Tone::NOTE_C5, 
      Tone::NOTE_NULL, 
      Tone::NOTE_B4, Tone::NOTE_C5, 
      Tone::NOTE_NULL, 
      Tone::NOTE_B4,
      Tone::NOTE_NULL, 
      Tone::NOTE_A4, Tone::NOTE_E5, 
      Tone::NOTE_NULL, 
      Tone::NOTE_F5,
      Tone::NOTE_NULL, 
      Tone::NOTE_E5, Tone::NOTE_E5, 
      Tone::NOTE_NULL, 
      Tone::NOTE_G5,
      Tone::NOTE_NULL, 
      Tone::NOTE_E5, Tone::NOTE_D5,
      Tone::NOTE_NULL, 
      Tone::NOTE_D5,
      Tone::NOTE_NULL, 
      Tone::NOTE_C5,
      Tone::NOTE_NULL, 
      Tone::NOTE_B4, Tone::NOTE_C5, 
      Tone::NOTE_NULL, 
      Tone::NOTE_B4, 
      Tone::NOTE_NULL, 
      Tone::NOTE_A4
    );

    let mut periods: Vec<i32> = vec!(
      125,125,250,125,
      50,
      125,125,250,125,
      50,
      125,125,250,125,
      50,
      125,125,375,
      50,
      125,125,250,125,
      50,
      125,125,250,125,
      50,
      125,125,250,125,
      50,
      125,125,375,
      50,
      125,125,250,125,
      50,
      125,125,250,125,
      50,
      125,125,250,125,
      50,
      125,125,125,250,
      50,
      125,125,250,125,
      50,
      250,125,250,
      50,
      125,125,250,125,
      50,
      125,125,375,
      50,
      250,125,125,125,250,125,
      50,
      125,125,250,125,
      50,
      125,125,250,125,
      50,
      125,125,375,
      50,
      125,125,250,125,
      50,
      125,125,250,125,
      50,
      125,125,250,125,
      50,
      125,125,375,
      50,
      125,125,250,125,
      50,
      125,125,250,125,
      50,
      125,125,250,125,
      50,
      125,125,125,250,
      50,
      125,125,250,125,
      50,
      250,125,250,
      50,
      125,125,250,125,
      50,
      125,125,375,
      200,
      250,
      400,
      250,
      400,
      125,125,
      50,
      125,
      50,
      125,125,
      400,
      250,
      400,
      250,
      400,
      125,125,
      50,
      125,125,
      50,
      125,
      50,
      500,125,
      50,
      250,
      400,
      125,125,
      50,
      125,
      50,
      125,125,
      400,
      250,
      400,
      250,
      400,
      125,125,
      50,
      125,
      50,
      500
    );

    tones.truncate(37);
    periods.truncate(37);

    Sounds::play_sound(buzzer, tones, periods)
  }

  pub fn play_harrypotter(buzzer: &mut Buzzer) -> Result<(), String> {

    let tones: Vec<Tone> = vec!(
      Tone::NOTE_B4, Tone::NOTE_E5, Tone::NOTE_G5, Tone::NOTE_FS5,Tone::NOTE_E5, 
      Tone::NOTE_B5, Tone::NOTE_A5, Tone::NOTE_FS5,Tone::NOTE_E5, Tone::NOTE_G5, 
      Tone::NOTE_FS5,Tone::NOTE_DS5,Tone::NOTE_F5, Tone::NOTE_B4, Tone::NOTE_B4, 
      Tone::NOTE_E5, Tone::NOTE_G5, Tone::NOTE_FS5,Tone::NOTE_E5, Tone::NOTE_B5, 
      Tone::NOTE_D6, Tone::NOTE_CS6,Tone::NOTE_C6, Tone::NOTE_GS5,Tone::NOTE_C6, 
      Tone::NOTE_B5, Tone::NOTE_AS5,Tone::NOTE_AS4,Tone::NOTE_G5, Tone::NOTE_E5, 
      Tone::NOTE_G5, Tone::NOTE_B5, Tone::NOTE_G5, Tone::NOTE_B5, Tone::NOTE_G5, 
      Tone::NOTE_C6, Tone::NOTE_B5, Tone::NOTE_AS5,Tone::NOTE_FS5,Tone::NOTE_G5, 
      Tone::NOTE_B5, Tone::NOTE_AS5,Tone::NOTE_AS4,Tone::NOTE_B4, Tone::NOTE_B5, 
      Tone::NOTE_G5, Tone::NOTE_B5, Tone::NOTE_G5, Tone::NOTE_B5, Tone::NOTE_G5, 
      Tone::NOTE_D6, Tone::NOTE_CS6,Tone::NOTE_C6, Tone::NOTE_GS5,Tone::NOTE_C6, 
      Tone::NOTE_B5, Tone::NOTE_AS5,Tone::NOTE_AS4,Tone::NOTE_G5, Tone::NOTE_E5, 
    );

    let periods: Vec<i32> = vec!(333,500,166,333,666,333,1000,1000,500,166,333,666,
      333,1666,333,500,166,333,666,333,666,333,666,333,500,166,333,666,333,1666,333,
      666,333,666,333,666,333,666,333,500,166,333,666,333,1666,333,666,333,666,333,
      666,333,666,333,500,166,333,666,333,1666,
    );

    Sounds::play_sound(buzzer, tones, periods)
  }
  
  pub fn play_supermario(buzzer: &mut Buzzer) -> Result<(),String> {

    let mut tones: Vec<Tone> = vec!(
      Tone::NOTE_E5,Tone::NOTE_E5,Tone::NOTE_E5,Tone::NOTE_NULL,Tone::NOTE_C5,Tone::NOTE_E5,
      Tone::NOTE_NULL,Tone::NOTE_G5,Tone::NOTE_NULL,Tone::NOTE_G4,Tone::NOTE_NULL,Tone::NOTE_C5,
      Tone::NOTE_NULL,Tone::NOTE_G4,Tone::NOTE_NULL,Tone::NOTE_E4,Tone::NOTE_NULL,Tone::NOTE_A4,
      Tone::NOTE_NULL,Tone::NOTE_B4,Tone::NOTE_NULL,Tone::NOTE_AS4,Tone::NOTE_A4,Tone::NOTE_NULL,
      Tone::NOTE_G4,Tone::NOTE_E5,Tone::NOTE_G5,Tone::NOTE_A5,Tone::NOTE_NULL,Tone::NOTE_F5,
      Tone::NOTE_G5,Tone::NOTE_NULL,Tone::NOTE_E5,Tone::NOTE_NULL,Tone::NOTE_C5,Tone::NOTE_D5,
      Tone::NOTE_B4,Tone::NOTE_NULL,Tone::NOTE_C5,Tone::NOTE_NULL,Tone::NOTE_G4,Tone::NOTE_NULL,
      Tone::NOTE_E4,Tone::NOTE_NULL,Tone::NOTE_A4,Tone::NOTE_NULL,Tone::NOTE_B4,Tone::NOTE_NULL,
      Tone::NOTE_AS4,Tone::NOTE_A4,Tone::NOTE_NULL,Tone::NOTE_G4,Tone::NOTE_E5,Tone::NOTE_G5,
      Tone::NOTE_A5,Tone::NOTE_NULL,Tone::NOTE_F5,Tone::NOTE_G5,Tone::NOTE_NULL,Tone::NOTE_E5,
      Tone::NOTE_NULL,Tone::NOTE_C5,Tone::NOTE_D5,Tone::NOTE_B4,Tone::NOTE_NULL,
    );

    let mut periods: Vec<i32> = vec!(
      200,200,200,200,200,200,200,200,680,200,680,200,440,200,440,200,
      440,200,200,200,200,200,200,200,140,140,140,200,200,200,200,200,
      200,200,200,200,200,440,200,440,200,440,140,440,200,200,200,200,
      200,200,200,140,140,140,200,200,200,200,200,200,200,200,200,200,
      440,    
    );

    tones.truncate(10);
    periods.truncate(10);

    Sounds::play_sound(buzzer, tones, periods)
  }

  pub fn play_starwars(buzzer: &mut Buzzer) -> Result<(), String>  {

    let tones_1s: Vec<Tone> = vec!(
      Tone::NOTE_A4, Tone::NOTE_A4, Tone::NOTE_A4, Tone::NOTE_F4, 
      Tone::NOTE_C5, Tone::NOTE_A4, Tone::NOTE_F4, Tone::NOTE_C5, 
      Tone::NOTE_A4, Tone::NOTE_NULL, Tone::NOTE_E5, Tone::NOTE_E5,
      Tone::NOTE_E5, Tone::NOTE_F5, Tone::NOTE_C5, Tone::NOTE_GS4, 
      Tone::NOTE_F4, Tone::NOTE_C5, Tone::NOTE_A4, Tone::NOTE_NULL,
    );

    let periods_1s: Vec<i32> = vec!(
      500,500,500,350,150,500,350,150,650,500,500,500,500,350,150,500,350,150,650,500,
    );

    let tones_2s: Vec<Tone> = vec!(
      Tone::NOTE_A5, Tone::NOTE_A4, Tone::NOTE_A4, Tone::NOTE_A5, Tone::NOTE_GS5, Tone::NOTE_G5, 
      Tone::NOTE_FS5, Tone::NOTE_F5, Tone::NOTE_FS5, Tone::NOTE_NULL, Tone::NOTE_AS4, Tone::NOTE_DS5, 
      Tone::NOTE_D5, Tone::NOTE_CS5, Tone::NOTE_C5, Tone::NOTE_B4, Tone::NOTE_C5, Tone::NOTE_NULL,
    );

    let periods_2s: Vec<i32> = vec!(
      500,300,150,500,325,175,125,125,250,325,250,500,325,175,125,125,250,350,
    );

    let tones_1v: Vec<Tone> = vec!(
      Tone::NOTE_F4, Tone::NOTE_GS4, Tone::NOTE_F4, Tone::NOTE_A4, Tone::NOTE_C5, Tone::NOTE_A4, 
      Tone::NOTE_C5, Tone::NOTE_E5, Tone::NOTE_NULL,
    );

    let periods_1v: Vec<i32> = vec!(
      250,500,350,125,500,375,125,650,500,
    );
  
    let tones_2v: Vec<Tone> = vec!(
      Tone::NOTE_F4, Tone::NOTE_GS4, Tone::NOTE_F4, Tone::NOTE_C5, Tone::NOTE_A4, Tone::NOTE_F4, 
      Tone::NOTE_C5, Tone::NOTE_A4, Tone::NOTE_NULL,
    );

    let periods_2v: Vec<i32> = vec!(
      250,500,375,125,500,375,125,650,650,  
    );

    let mut tones: Vec<Tone> = vec!();
    tones.extend(&tones_1s);
    tones.extend(&tones_2s);
    tones.extend(&tones_1v);
    tones.extend(&tones_2s);
    tones.extend(&tones_2v);

    let mut periods: Vec<i32> = vec!();
    periods.extend(&periods_1s);
    periods.extend(&periods_2s);
    periods.extend(&periods_1v);
    periods.extend(&periods_2s);
    periods.extend(&periods_2v);

    tones.truncate(10);
    periods.truncate(10);

    Sounds::play_sound(buzzer, tones, periods)
  }

  pub fn play_doremifa(buzzer: &mut Buzzer) -> Result<(), String> {

    let mut tones: Vec<Tone> = vec!(
      Tone::NOTE_C4, Tone::NOTE_D4, Tone::NOTE_E4, Tone::NOTE_F4, Tone::NOTE_NULL, Tone::NOTE_F4, Tone::NOTE_NULL, Tone::NOTE_F4, 
      Tone::NOTE_C4, Tone::NOTE_D4, Tone::NOTE_C4, Tone::NOTE_D4, Tone::NOTE_NULL, Tone::NOTE_D4, Tone::NOTE_NULL, Tone::NOTE_D4, 
      Tone::NOTE_C4, Tone::NOTE_G4, Tone::NOTE_F4, Tone::NOTE_E4, Tone::NOTE_NULL, Tone::NOTE_E4, Tone::NOTE_NULL, Tone::NOTE_E4, 
      Tone::NOTE_C4, Tone::NOTE_D4, Tone::NOTE_E4, Tone::NOTE_F4, Tone::NOTE_NULL, Tone::NOTE_F4,  Tone::NOTE_NULL, Tone::NOTE_F4,  
    );

    let mut periods: Vec<i32> = vec!(
      600,600,600,600,100,400,100,400,600,600,600,600,100,400,100,400,600,600,600,600,100,400,100,400,600,600,600,600,100,400,100,400,
    );

    tones.truncate(8);
    periods.truncate(8);

    Sounds::play_sound(buzzer, tones, periods)
  }

  pub fn play_bip(buzzer: &mut Buzzer) -> Result<(), String> {
    let tones: Vec<Tone> = vec!(
      Tone::NOTE_C4, Tone::NOTE_NULL, Tone::NOTE_D4, Tone::NOTE_NULL, Tone::NOTE_E4, Tone::NOTE_NULL, Tone::NOTE_F4, Tone::NOTE_NULL, 
    );

    let periods: Vec<i32> = vec!(
      150,150,150,150,150,150,150,150
    );

    Sounds::play_sound(buzzer, tones, periods)
  }
}