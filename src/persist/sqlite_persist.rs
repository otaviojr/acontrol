/**
 * @file   sqlite/mod.rs
 * @author Otavio Ribeiro
 * @date   24 Dec 2017
 * @brief  SQLite persistence driver
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
use super::{Persist, Card, Fingerprint, Bluetooth};

use std::path::Path;
use std::collections::HashMap;
use rusqlite::{Connection,NO_PARAMS};
use rusqlite::types::ToSql;

pub struct SQLitePersist {
  conn: Option<Connection>,
}

impl SQLitePersist {
  pub fn new() -> Self {
    return SQLitePersist {conn: None};
  }
}

impl Persist for SQLitePersist {
  fn init(&mut self, params: &HashMap<String,String>) -> Result<(), String> {
    let path = Path::new(&params["DATA_PATH"]).join("acontrol.db");
    self.conn = match Connection::open(path) {
      Ok(conn) => Some(conn),
      Err(err) => return Err(format!("Error openning database file: {}",err)),
    };

    if let Some(ref conn) = self.conn {
      if let Err(err) = conn.execute(
          "create table if not exists cards (
               id integer primary key,
               uuid varchar(10) not null,
               name varchar(255) not null
           )",
          NO_PARAMS,
      ) {
        return Err(format!("Error creating table cards: {}",err));
      }

      if let Err(err) = conn.execute(
          "create table if not exists fingerprint (
               id integer primary key,
               pos integer not null,
               name varchar(255) not null
           )",
          NO_PARAMS,
      ) {
        return Err(format!("Error creating table fingerprint: {}",err));
      }

      if let Err(err) = conn.execute(
        "create table if not exists bluetooth (
             id integer primary key,
             addr varchar(255) not null,
             name varchar(255) not null
         )",
        NO_PARAMS,
    ) {
      return Err(format!("Error creating table bluetooth: {}",err));
    }
    }

    Ok(())
  }

  fn unload(&mut self) -> Result<(), String> {
    println!("Persistence driver unloading");
    Ok(())
  }

  fn nfc_add(&mut self, uuid: &Vec<u8>, name: &Vec<u8>)-> Result<(), String> {
    if let Some(ref conn) = self.conn {
      if let Err(err) = conn.execute("INSERT INTO cards (uuid, name) VALUES (?1,?2)",
          &[uuid as &dyn ToSql, name as &dyn ToSql],
      ) {
        return Err(format!("Error inserting card to the database: {}", err));
      }
    }

    Ok(())
  }

  fn nfc_list(&mut self) -> Result<Vec<Card>, String> {

    let mut ret: Vec<Card> = Vec::new();

    if let Some(ref conn) = self.conn {
      let mut stmt = conn
        .prepare("SELECT id,uuid,name FROM cards")
        .unwrap();

      let card_iter = stmt
        .query_map(NO_PARAMS, |row| Ok(Card {
            id: row.get(0).unwrap_or(0),
            uuid: row.get(1).unwrap_or(Vec::new()),
            name: row.get(2).unwrap_or(Vec::new()),
        })).unwrap();

      for card in card_iter {
        ret.push(card.unwrap());
      }
      return Ok(ret);
    } else {
      return Err(format!("{}","Database not connected"));
    }
  }

  fn nfc_find(&mut self, uuid: &Vec<u8>) -> Result<Card, String> {

    if let Some(ref conn) = self.conn {
      let mut stmt = conn
        .prepare("SELECT id,uuid,name FROM cards where uuid=?1")
        .unwrap();

      let card_iter = stmt
        .query_map(&[uuid as &dyn ToSql], |row| Ok(Card {
            id: row.get(0).unwrap_or(0),
            uuid: row.get(1).unwrap_or(Vec::new()),
            name: row.get(2).unwrap_or(Vec::new()),
        })).unwrap();

      for card in card_iter {
        return Ok(card.unwrap());
      }
    } else {
      return Err(format!("{}","Database not connected"));
    }

    Err(format!("{}","Card Not Found"))
  }

  fn nfc_delete(&mut self, _uuid: &Vec<u8>) -> Result<(), String> {
    Ok(())
  }

  fn fingerprint_add(&mut self, pos: i32, name: &Vec<u8>) -> Result<(), String> {
    if let Some(ref conn) = self.conn {
      if let Err(err) = conn.execute("INSERT INTO fingerprint (pos, name) VALUES (?1,?2)",
          &[&pos, name as &dyn ToSql],
      ) {
        return Err(format!("Error inserting fingerprint to the database: {}", err));
      }
    }

    Ok(())
  }

  fn fingerprint_find(&mut self, pos: i32) -> Result<Fingerprint, String> {

    if let Some(ref conn) = self.conn {
      let mut stmt = conn
        .prepare("SELECT id,pos,name FROM fingerprint where pos=?1")
        .unwrap();

      let fingerprint_iter = stmt
        .query_map(&[pos], |row| Ok(Fingerprint {
            id: row.get(0).unwrap_or(0),
            pos: row.get(1).unwrap_or(0),
            name: row.get(2).unwrap_or(Vec::new()),
        })).unwrap();

      for fingerprint in fingerprint_iter {
        return Ok(fingerprint.unwrap());
      }
    } else {
      return Err(format!("{}","Database not connected"));
    }

    Err(format!("{}","Fingerprint Not Found"))
  }

  fn fingerprint_list(&mut self) -> Result<Vec<Fingerprint>, String> {

    let mut ret: Vec<Fingerprint> = Vec::new();

    if let Some(ref conn) = self.conn {
      let mut stmt = conn
        .prepare("SELECT id,pos,name FROM fingerprint")
        .unwrap();

      let fingerprint_iter = stmt
        .query_map(NO_PARAMS, |row| Ok(Fingerprint {
            id: row.get(0).unwrap_or(0),
            pos: row.get(1).unwrap_or(0),
            name: row.get(2).unwrap_or(Vec::new())
        })).unwrap();

      for fingerprint in fingerprint_iter {
        ret.push(fingerprint.unwrap());
      }
      return Ok(ret);
    } else {
      return Err(format!("{}","Database not connected"));
    }
  }

  fn fingerprint_delete(&mut self, _pos: i32) -> Result<(), String> {
    Ok(())
  }




  fn bluetooth_add(&mut self, addr: &Vec<u8>, name: &Vec<u8>) -> Result<(), String> {
    if let Some(ref conn) = self.conn {
      if let Err(err) = conn.execute("INSERT INTO bluetooth (addr, name) VALUES (?1,?2)",
          &[addr as &dyn ToSql, name as &dyn ToSql],
      ) {
        return Err(format!("Error inserting bluetooth device to the database: {}", err));
      }
    }

    Ok(())
  }

  fn bluetooth_find(&mut self, addr: &Vec<u8>) -> Result<Bluetooth, String> {

    if let Some(ref conn) = self.conn {
      let mut stmt = conn
        .prepare("SELECT id,addr,name FROM bluetooth where addr=?1")
        .unwrap();

      let bluetooth_iter = stmt
        .query_map(&[addr as &dyn ToSql], |row| Ok(Bluetooth {
            id: row.get(0).unwrap_or(0),
            addr: row.get(1).unwrap_or(Vec::new()),
            name: row.get(2).unwrap_or(Vec::new()),
        })).unwrap();

      for bluetooth in bluetooth_iter {
        return Ok(bluetooth.unwrap());
      }
    } else {
      return Err(format!("{}","Database not connected"));
    }

    Err(format!("{}","Bluetooth Device Not Found"))
  }

  fn bluetooth_list(&mut self) -> Result<Vec<Bluetooth>, String> {

    let mut ret: Vec<Bluetooth> = Vec::new();

    if let Some(ref conn) = self.conn {
      let mut stmt = conn
        .prepare("SELECT id,addr,name FROM bluetooth")
        .unwrap();

      let bluetooth_iter = stmt
        .query_map(NO_PARAMS, |row| Ok(Bluetooth {
            id: row.get(0).unwrap_or(0),
            addr: row.get(1).unwrap_or(Vec::new()),
            name: row.get(2).unwrap_or(Vec::new())
        })).unwrap();

      for bluetooth in bluetooth_iter {
        ret.push(bluetooth.unwrap());
      }
      return Ok(ret);
    } else {
      return Err(format!("{}","Database not connected"));
    }
  }

  fn bluetooth_delete(&mut self, _addr: &Vec<u8>) -> Result<(), String> {
    Ok(())
  }
}

unsafe impl Send for SQLitePersist {}
unsafe impl Sync for SQLitePersist {}
