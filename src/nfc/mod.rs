pub mod mfrc522;

#[derive(Clone, Copy)]
pub enum PICC {
  REQIDL	= 0x26,
  REQALL	= 0x52,
  ANTICOLL1	= 0x93,
  ANTICOOL2	= 0x95,
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

pub enum MiResult {
  OK		= 0,
  NOTAGERR	= 1,
  ERR		= 2
}

pub trait NfcReader {
  fn init(&mut self) -> Result<(), String>;
  fn unload(&mut self) -> Result<(), String>;
  fn signature(&self) -> String;
}

pub fn nfcreader_by_name(name: &str) -> Option<Box<NfcReader+Sync+Send>> {
    match name {
      "mfrc522" => return Some(Box::new(mfrc522::Mfrc522::new())),
      _ => return None
    }
}
