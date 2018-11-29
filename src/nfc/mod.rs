pub mod mfrc522;

#[derive(Clone, Copy)]
pub enum PICC {
  REQIDL	= 0x26,
  REQALL	= 0x52,
  ANTICOLL1	= 0x93,
  ANTICOLL2	= 0x95,
  ANTICOLL3	= 0x97,
  AUTH1A	= 0x60,
  AUTH1B	= 0x61,
  READ		= 0x30,
  WRITE		= 0xA0,
  DECREMENT	= 0xC0,
  INCREMENT	= 0xC1,
  RESTORE	= 0xC2,
  TRANSFER	= 0xB0,
  HALT		= 0x50
}

impl PICC {
  fn value(&self) -> u8 {
    return (*self) as u8;
  }
}

pub trait NfcReader {
  fn init(&mut self) -> Result<(), String>;
  fn unload(&mut self) -> Result<(), String>;
  fn find_tag(&mut self) -> Result<(), String>;
  fn signature(&self) -> String;
}

pub trait MiFare {
  fn send_reqA(&mut self) -> Result<Vec<u8>, String>;
  fn select(&mut self, cascade: u8, uuid: &Vec<u8>) -> Result<Vec<u8>, String>;
  fn anticoll(&mut self, cascade: u8, uuid: &Vec<u8>) -> Result<Vec<u8>, String>;
}

pub fn nfcreader_by_name(name: &str) -> Option<Box<NfcReader+Sync+Send>> {
    match name {
      "mfrc522" => return Some(Box::new(mfrc522::Mfrc522::new())),
      _ => return None
    }
}
