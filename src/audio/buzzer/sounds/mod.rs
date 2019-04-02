use std::thread;
use std::time::Duration;

use audio::buzzer::{Buzzer, BuzzerThreadSafe};

#[derive(Clone, Copy)]
#[allow(dead_code)]
#[allow(non_camel_case_types)]
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
  pub fn value(&self) -> i32 {
    return (*self) as i32;
  }
}

pub struct Sounds {
}

impl Sounds {

  pub fn play_piratescaribean(buzzer: &Buzzer) -> Result<(), String> {
    let buzzer_cloned = buzzer.buzzer.clone();

    let _handler = thread::spawn( move || {
      if let Ok(ref mut buzzer_locked) = buzzer_cloned.lock() {
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E4, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_G4, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 250);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 125);
        thread::sleep(Duration::from_millis(50));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B4, 125); 
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5, 250); 
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5, 125);
        thread::sleep(Duration::from_millis(50));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_D5, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B4, 250);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B4, 125);
        thread::sleep(Duration::from_millis(50));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_G4, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 375);
        thread::sleep(Duration::from_millis(50));  
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E4, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_G4, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 250);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 125);
        thread::sleep(Duration::from_millis(50));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B4, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5, 250);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5, 125);
        thread::sleep(Duration::from_millis(50));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5,125); 
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_D5, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B4, 250);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B4, 125);
        thread::sleep(Duration::from_millis(50));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 125); 
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_G4, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 375);
        thread::sleep(Duration::from_millis(50));    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E4, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_G4, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 250);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 125);
        thread::sleep(Duration::from_millis(50));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_D5, 250);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_D5, 125);
        thread::sleep(Duration::from_millis(50));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_D5, 125); 
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E5, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_F5,250);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_F5, 125);
        thread::sleep(Duration::from_millis(50));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E5, 125); 
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_D5, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E5, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 250);
        thread::sleep(Duration::from_millis(50));  
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4,125); 
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B4, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5, 250);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5, 125);
        thread::sleep(Duration::from_millis(50));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_D5, 250);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E5, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 250);
        thread::sleep(Duration::from_millis(50));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B4, 250);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B4, 125);
        thread::sleep(Duration::from_millis(50));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B4, 375);
        thread::sleep(Duration::from_millis(200));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 250); 
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 125);     
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E4, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_G4, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 250);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 125);
        thread::sleep(Duration::from_millis(50)); 
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B4, 125); 
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5, 250); 
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5, 125);
        thread::sleep(Duration::from_millis(50));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_D5, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B4, 250);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B4, 125);
        thread::sleep(Duration::from_millis(50));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_G4, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 375);
        thread::sleep(Duration::from_millis(50));  
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E4, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_G4, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 250);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 125);
        thread::sleep(Duration::from_millis(50));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B4, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5, 250);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5, 125);
        thread::sleep(Duration::from_millis(50));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5,125); 
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_D5, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B4, 250);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B4, 125);
        thread::sleep(Duration::from_millis(50));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 125); 
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_G4, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 375);
        thread::sleep(Duration::from_millis(50));    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E4, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_G4, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 250);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 125);
        thread::sleep(Duration::from_millis(50));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_D5, 250);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_D5, 125);
        thread::sleep(Duration::from_millis(50));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_D5, 125); 
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E5, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_F5, 250);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_F5, 125);
        thread::sleep(Duration::from_millis(50));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E5, 125); 
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_D5, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E5, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 250);
        thread::sleep(Duration::from_millis(50));  
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4,125); 
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B4, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5, 250);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5, 125);
        thread::sleep(Duration::from_millis(50));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_D5, 250);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E5, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 250);
        thread::sleep(Duration::from_millis(50));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B4, 250);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B4, 125);
        thread::sleep(Duration::from_millis(50));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B4, 375);
        thread::sleep(Duration::from_millis(200));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E5, 250);
        thread::sleep(Duration::from_millis(400));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_F5, 250);
        thread::sleep(Duration::from_millis(400)); 
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E5, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E5, 125);
        thread::sleep(Duration::from_millis(50));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_G5, 125);
        thread::sleep(Duration::from_millis(50));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E5, 125);  
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_D5, 125);
        thread::sleep(Duration::from_millis(400));    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_D5, 250);
        thread::sleep(Duration::from_millis(400));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5, 250);
        thread::sleep(Duration::from_millis(400));  
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B4, 125);  
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5, 125);
        thread::sleep(Duration::from_millis(50));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B4, 125);
        thread::sleep(Duration::from_millis(50));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4,  500);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E5, 250);  
        thread::sleep(Duration::from_millis(400));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_F5, 250);
        thread::sleep(Duration::from_millis(400));    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E5, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E5, 125);
        thread::sleep(Duration::from_millis(50));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_G5, 125);
        thread::sleep(Duration::from_millis(50));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E5, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_D5, 125);
        thread::sleep(Duration::from_millis(400));  
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_D5, 250);
        thread::sleep(Duration::from_millis(400));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5, 250);
        thread::sleep(Duration::from_millis(400));  
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B4, 125);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5, 125);
        thread::sleep(Duration::from_millis(50));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B4, 125);
        thread::sleep(Duration::from_millis(50));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 500);
      };
    });
    Ok(())
  }

  pub fn play_harrypotter(buzzer: &Buzzer) -> Result<(), String> {
    let buzzer_cloned = buzzer.buzzer.clone();

    let _handler = thread::spawn( move || {
      if let Ok(ref mut buzzer_locked) = buzzer_cloned.lock() {
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B4, 333);  
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E5, 500);   
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_G5, 166);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_FS5, 333);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E5, 666);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B5, 333);   
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A5, 1000);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_FS5, 1000);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E5, 500);   
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_G5, 166);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_FS5, 333);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_DS5, 666);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_F5, 333);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B4, 1666);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B4, 333);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E5, 500);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_G5, 166);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_FS5, 333);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E5, 666);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B5, 333);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_D6, 666);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_CS6, 333);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C6, 666);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_GS5, 333);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C6, 500);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B5, 166);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_AS5, 333); 
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_AS4, 666);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_G5, 333);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E5, 1666);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_G5, 333);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B5, 666);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_G5, 333);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B5, 666);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_G5, 333);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C6, 666);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B5, 333);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_AS5, 666);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_FS5, 333);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_G5, 500);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B5, 166);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_AS5, 333);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_AS4, 666);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B4, 333);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B5, 1666);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_G5, 333);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B5, 666);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_G5, 333);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B5, 666);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_G5, 333);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_D6, 666);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_CS6, 333);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C6, 666);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_GS5, 333);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C6, 500);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B5, 166);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_AS5, 333);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_AS4, 666);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_G5, 333);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E5, 1666);    
      };
    });
    Ok(())
  }
  
  pub fn play_supermario(buzzer: &Buzzer) -> Result<(),String> {

    let buzzer_cloned = buzzer.buzzer.clone();

    let _handler = thread::spawn( move || {
      if let Ok(ref mut buzzer_locked) = buzzer_cloned.lock() {
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E5,200);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E5,200);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E5,200);    
        thread::sleep(Duration::from_millis(200));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5,200);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E5,200);
        thread::sleep(Duration::from_millis(200));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_G5,200);
        thread::sleep(Duration::from_millis(680));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_G4,200);
        thread::sleep(Duration::from_millis(680));

        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5,200);
        thread::sleep(Duration::from_millis(440));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_G4,200);
        thread::sleep(Duration::from_millis(440));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E4,200);
        thread::sleep(Duration::from_millis(440));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4,200);
        thread::sleep(Duration::from_millis(200));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B4,200);
        thread::sleep(Duration::from_millis(200));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_AS4,200);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4,200);
        thread::sleep(Duration::from_millis(200));

        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_G4,140);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E5,140);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_G5,140);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A5,200);
        thread::sleep(Duration::from_millis(200));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_F5,200);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_G5,200);    
        thread::sleep(Duration::from_millis(200));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E5,200);
        thread::sleep(Duration::from_millis(200));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5,200);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_D5,200);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B4,200);
        thread::sleep(Duration::from_millis(440));

        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5,200);
        thread::sleep(Duration::from_millis(440));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_G4,200);
        thread::sleep(Duration::from_millis(440));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E4,140);    
        thread::sleep(Duration::from_millis(440));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4,200);
        thread::sleep(Duration::from_millis(200));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B4,200);
        thread::sleep(Duration::from_millis(200));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_AS4,200);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4,200);
        thread::sleep(Duration::from_millis(200));

        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_G4,140);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E5,140);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_G5,140);    
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A5,200);
        thread::sleep(Duration::from_millis(200));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_F5,200);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_G5,200);    
        thread::sleep(Duration::from_millis(200));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E5,200);
        thread::sleep(Duration::from_millis(200));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5,200);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_D5,200);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B4,200);
        thread::sleep(Duration::from_millis(440));
      };
    });

    Ok(())
  }

  fn play_starwars_first_session(buzzer_locked: &BuzzerThreadSafe) -> Result<(), String> {
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 500);
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 500);    
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 500);
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_F4, 350);
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5, 150);
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 500);
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_F4, 350);
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5, 150);
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 650);

    thread::sleep(Duration::from_millis(500));

    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E5, 500);
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E5, 500);
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E5, 500);  
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_F5, 350);
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5, 150);
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_GS4, 500);
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_F4, 350);
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5, 150);
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 650);

    thread::sleep(Duration::from_millis(500));

    Ok(())
  }

  fn play_starwars_second_session(buzzer_locked: &BuzzerThreadSafe) -> Result<(), String> {
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A5, 500);
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 300);
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 150);
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A5, 500);
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_GS5, 325);
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_G5, 175);
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_FS5, 125);
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_F5, 125);    
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_FS5, 250);

    thread::sleep(Duration::from_millis(325));

    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_AS4, 250);
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_DS5, 500);
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_D5, 325);  
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_CS5, 175);  
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5, 125);  
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_B4, 125);  
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5, 250);  

    thread::sleep(Duration::from_millis(350));

    Ok(())
  }

  fn play_starwars_variant1(buzzer_locked: &BuzzerThreadSafe) -> Result<(), String> {
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_F4, 250);  
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_GS4, 500);  
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_F4, 350);  
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 125);
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5, 500);
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 375);  
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5, 125);
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E5, 650);
  
    thread::sleep(Duration::from_millis(500));

    Ok(())
  }

  fn play_starwars_variant2(buzzer_locked: &BuzzerThreadSafe) -> Result<(), String> {
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_F4, 250);  
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_GS4, 500);  
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_F4, 375);  
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5, 125);
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 500);  
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_F4, 375);  
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C5, 125);
    let _ret = (*buzzer_locked).play_tone(Tone::NOTE_A4, 650);  
  
    thread::sleep(Duration::from_millis(650));

    Ok(())
  }

  pub fn play_starwars(buzzer: &Buzzer) -> Result<(), String>  {
    let buzzer_cloned = buzzer.buzzer.clone();

    let _handler = thread::spawn( move || {
      if let Ok(ref mut buzzer_locked) = buzzer_cloned.lock() {

        let _ret = Sounds::play_starwars_first_session(buzzer_locked);
        let _ret = Sounds::play_starwars_second_session(buzzer_locked);

        let _ret = Sounds::play_starwars_variant1(buzzer_locked);
        let _ret = Sounds::play_starwars_second_session(buzzer_locked);
        let _ret = Sounds::play_starwars_variant2(buzzer_locked);      
      };
    });

    Ok(())
  }

  pub fn play_doremifa(buzzer: &Buzzer) -> Result<(), String> {
    let buzzer_cloned = buzzer.buzzer.clone();

    let _handler = thread::spawn( move || {
      if let Ok(ref mut buzzer_locked) = buzzer_cloned.lock() {
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C4, 600);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_D4, 600);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E4, 600);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_F4, 600);

        thread::sleep(Duration::from_millis(100));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_F4, 400);
        thread::sleep(Duration::from_millis(100));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_F4, 400);

        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C4, 600);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_D4, 600);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C4, 600);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_D4, 600);

        thread::sleep(Duration::from_millis(100));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_D4, 400);
        thread::sleep(Duration::from_millis(100));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_D4, 400);

        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C4, 600);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_G4, 600);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_F4, 600);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E4, 600);

        thread::sleep(Duration::from_millis(100));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E4, 400);
        thread::sleep(Duration::from_millis(100));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E4, 400);

        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_C4, 600);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_D4, 600);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_E4, 600);
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_F4, 600);

        thread::sleep(Duration::from_millis(100));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_F4, 400);
        thread::sleep(Duration::from_millis(100));
        let _ret = (*buzzer_locked).play_tone(Tone::NOTE_F4, 400);
      };
    });

    Ok(())
  }
}