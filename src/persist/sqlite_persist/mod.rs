use persist::{Persist, Card, Fingerprint};

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
    let mut path = Path::new(&params["DATA_PATH"]).join("acontrol.db");
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
               name varchar(255) not null,
               template varchar(1000) not null
           )",
          NO_PARAMS,
      ) {
        return Err(format!("Error creating table fingerprint: {}",err));
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
          &[uuid as &ToSql, name as &ToSql],
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
        .query_map(NO_PARAMS, |row| Card {
            id: row.get(0),
            uuid: row.get(1),
            name: row.get(2),
        }).unwrap();

      for card in card_iter {
        ret.push(card.unwrap());
      }
      return Ok(ret);
    } else {
      return Err(format!("{}","Database not connected"));
    }

    Err(format!("{}","Card Not Found"))
  }
  
  fn nfc_find(&mut self, uuid: &Vec<u8>) -> Result<(Card), String> {

    if let Some(ref conn) = self.conn {
      let mut stmt = conn
        .prepare("SELECT id,uuid,name FROM cards where uuid=?1")
        .unwrap();

      let card_iter = stmt
        .query_map(&[uuid as &ToSql], |row| Card {
            id: row.get(0),
            uuid: row.get(1),
            name: row.get(2),
        }).unwrap();

      for card in card_iter {
        return Ok(card.unwrap());
      }
    } else {
      return Err(format!("{}","Database not connected"));
    }

    Err(format!("{}","Card Not Found"))
  }

  fn nfc_delete(&mut self, uuid: &Vec<u8>) -> Result<(), String> {
    Ok(())
  }

  fn fingerprint_add(&mut self, pos: i32, name: &Vec<u8>, template: &Vec<u8>) -> Result<(), String> {
    if let Some(ref conn) = self.conn {
      if let Err(err) = conn.execute("INSERT INTO fingerprint (pos, name, template) VALUES (?1,?2,?3)",
          &[&pos, name as &ToSql, template as &ToSql],
      ) {
        return Err(format!("Error inserting fingerprint to the database: {}", err));
      }
    }

    Ok(())
  }

  fn fingerprint_find(&mut self, pos: i32) -> Result<Fingerprint, String> {

    if let Some(ref conn) = self.conn {
      let mut stmt = conn
        .prepare("SELECT id,pos,name,template FROM fingerprint where pos=?1")
        .unwrap();

      let fingerprint_iter = stmt
        .query_map(&[pos], |row| Fingerprint {
            id: row.get(0),
            pos: row.get(1),
            name: row.get(2),
            template: row.get(3)
        }).unwrap();

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
        .prepare("SELECT id,pos,name,template FROM fingerprint")
        .unwrap();

      let fingerprint_iter = stmt
        .query_map(NO_PARAMS, |row| Fingerprint {
            id: row.get(0),
            pos: row.get(1),
            name: row.get(2),
            template: row.get(3)
        }).unwrap();

      for fingerprint in fingerprint_iter {
        ret.push(fingerprint.unwrap());
      }
      return Ok(ret);
    } else {
      return Err(format!("{}","Database not connected"));
    }

    Err(format!("{}","Fingerprint Not Found"))
  }

  fn fingerprint_delete(&mut self, pos: i32) -> Result<(), String> {
    Ok(())
  }
}

unsafe impl Send for SQLitePersist {}
unsafe impl Sync for SQLitePersist {}
