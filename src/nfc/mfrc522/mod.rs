use nfc::{NfcReader};

pub struct Mfrc522 {}

impl Mfrc522 {
}

impl NfcReader for Mfrc522 {

  fn new() -> Mfrc522 {
    return Mfrc522 {};
  }

  fn signature(&self) -> String {
    return String::from("MFRC522 Mifare Reader Module");
  }
}
