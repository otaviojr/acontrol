use persist::{Persist};

use std::collections::HashMap;

pub struct SQLitePersist {
}

impl SQLitePersist {
  pub fn new() -> Self {
    return SQLitePersist {};
  }
}

impl Persist for SQLitePersist {
  fn initialize(&mut self, params: &HashMap<String,String>) -> Result<(), String> {
    Ok(())
  }

  fn nfc_save(&mut self, uuid: &Vec<u8>)-> Result<(), String> {
    Ok(())
  }

  fn nfc_find(&mut self, uuid: &Vec<u8>) -> Result<(), String> {
    Ok(())
  }

  fn nfc_delete(&mut self, uuid: &Vec<u8>) -> Result<(), String> {
    Ok(())
  }
}

unsafe impl Send for SQLitePersist {}
unsafe impl Sync for SQLitePersist {}
