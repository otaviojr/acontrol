/**
 * @file   nfc/mod.rs
 * @author Otavio Ribeiro
 * @date   24 Dec 2017
 * @brief  NFC/Mifare global interface
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
pub mod mfrc522;
pub mod pn532_spi;

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum CardType{
  Mifare,
  FelicaA,
  FelicaB,
  Jewel
}

#[allow(dead_code)]
impl CardType {
  fn value(&mut self) -> u8 {
    let value = *self as u8;
    value
  }

  fn name(&mut self) -> &str {
    match *self {
      CardType::Mifare => "Mifare",
      CardType::FelicaA => "FelicaA",
      CardType::FelicaB => "FelicaB",
      CardType::Jewel => "Jewel",
    }
  }
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum WriteSecMode {
  Format,
  Restore
}

pub trait NfcReader {
  fn init(&mut self) -> Result<(), String>;
  fn unload(&mut self) -> Result<(), String>;
  fn find_tag(&mut self, func: fn(CardType, Vec<u8>) -> bool) -> Result<(), String>;
  fn set_auth_keys(&mut self, key_a: &Vec<u8>, key_b: &Vec<u8>) -> Result<(), String>;
  fn set_auth_bits(&mut self, access_bits: Vec<u8>) -> Result<(), String>;
  fn format(&mut self, uuid: &Vec<u8>) -> Result<(), String>;
  fn restore(&mut self, uuid: &Vec<u8>) -> Result<(), String>;
  fn read_data(&mut self, uuid: &Vec<u8>, addr: u8, blocks: u8) -> Result<(Vec<u8>), String>;
  fn write_data(&mut self, uuid: &Vec<u8>, addr: u8, data: &Vec<u8>) -> Result<(u8), String>;
  fn signature(&self) -> String;
}

pub fn nfcreader_by_name(name: &str) -> Option<Box<dyn NfcReader+Sync+Send>> {
    match name {
      "mfrc522" => return Some(Box::new(mfrc522::Mfrc522::new())),
      "pn532_spi" => return Some(Box::new(pn532_spi::Pn532Spi::new())),
      _ => return None
    }
}
