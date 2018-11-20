pub mod mfrc522;

pub trait NfcReader {
  fn new() -> Self;
  fn signature(&self) -> String;
}
