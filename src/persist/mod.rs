pub mod sqlite_persist;

use std::collections::HashMap;

#[derive(Debug)]
pub struct Card  {
  id: i32,
  uuid: String,
}

pub trait Persist {
  fn init(&mut self, params: &HashMap<String,String>) -> Result<(), String>;
  fn unload(&mut self) -> Result<(), String>;
  fn nfc_save(&mut self, uuid: &Vec<u8>) -> Result<(), String>;
  fn nfc_find(&mut self, uuid: &Vec<u8>) -> Result<(Card), String>;
  fn nfc_delete(&mut self, uuid: &Vec<u8>) -> Result<(), String>;
}

pub fn persist_by_name(name: &str) -> Option<Box<Persist+Sync+Send>> {
    match name {
      "sqlite" => return Some(Box::new(sqlite_persist::SQLitePersist::new())),
      _ => return None
    }
}
