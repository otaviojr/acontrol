/**
 * @file   persist/mod.rs
 * @author Otavio Ribeiro
 * @date   24 Dec 2017
 * @brief  Persistence global interface
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
pub mod sqlite_persist;

use std::collections::HashMap;

pub enum PersistError {
}

pub struct Card  {
  pub id: i32,
  pub uuid: Vec<u8>,
  pub name: Vec<u8>
}

pub struct Fingerprint {
  pub id: i32,
  pub pos: i32,
  pub name: Vec<u8>
}

pub trait Persist {
  fn init(&mut self, params: &HashMap<String,String>) -> Result<(), String>;
  fn unload(&mut self) -> Result<(), String>;

  fn nfc_add(&mut self, uuid: &Vec<u8>, name: &Vec<u8>) -> Result<(), String>;
  fn nfc_find(&mut self, uuid: &Vec<u8>) -> Result<Card, String>;
  fn nfc_list(&mut self) -> Result<Vec<Card>, String>;
  fn nfc_delete(&mut self, uuid: &Vec<u8>) -> Result<(), String>;

  fn fingerprint_add(&mut self, pos: i32, name: &Vec<u8>) -> Result<(), String>;
  fn fingerprint_find(&mut self, pos: i32) -> Result<Fingerprint, String>;
  fn fingerprint_list(&mut self) -> Result<Vec<Fingerprint>, String>;
  fn fingerprint_delete(&mut self, pos: i32) -> Result<(), String>;
}

pub fn persist_by_name(name: &str) -> Option<Box<dyn Persist+Sync+Send>> {
    match name {
      "sqlite" => return Some(Box::new(sqlite_persist::SQLitePersist::new())),
      _ => return None
    }
}
