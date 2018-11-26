pub mod mfrc522;

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
