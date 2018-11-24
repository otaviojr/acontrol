use nfc::{NfcReader};

use std::io::prelude::*;
use spidev::{Spidev, SpidevOptions, SpidevTransfer, SPI_MODE_0};

pub struct Mfrc522 {
  spidev: Option<Spidev>,
}

impl Mfrc522 {
  pub fn new() -> Mfrc522 {
    return Mfrc522 {spidev: None};
  }
}

impl NfcReader for Mfrc522 {
  fn init(&mut self) -> Result<String, String> {
    self.spidev = match Spidev::open("/dev/spidev0.0") {
      Ok(spidev) => Some(spidev),
      Err(err) => return Err(format!("{} - {}", String::from("Error initializing spi port"), err)),
    };

    Ok(String::from("Spi port initialized successfully"))
  }

  fn signature(&self) -> String {
    return String::from("MFRC522 Mifare Reader Module");
  }
}

unsafe impl Send for Mfrc522 {}

unsafe impl Sync for Mfrc522 {}
