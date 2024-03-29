/**
 * @file   webserver/mod.rs
 * @author Otavio Ribeiro
 * @date   24 Dec 2017
 * @brief  WebAPI/Iron driver
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
use iron::prelude::*;
use router::Router;

use serde_json;

use crate::acontrol_system_log;
use crate::log::LogType;

use super::super::system;
use super::{Server,WebServerDefaultResponse,WebCard,WebServerNfcListResponse};

use std::collections::HashMap;

pub struct WebServer {
  host: String,
  port: u32,
}

impl WebServer {
  pub fn new() -> Self {
    return WebServer { host: "".to_string(), port: 0};
  }

  //fn hello_world(req: &mut Request) -> IronResult<Response> {
  //  let ref query = req.extensions.get::<Router>().unwrap().find("query").unwrap_or("Unknow");
  //  Ok(Response::with((iron::status::Ok, format!("Hello {}", query))))
  //}

  fn nfc_authorize(req: &mut Request) -> IronResult<Response> {

    let mut params: HashMap<String,String> = HashMap::new();
    let mut resp: Option<Response> = None;
    let json_body = req.get::<bodyparser::Json>();

    match json_body {
        Ok(Some(json_body)) => {
          if json_body.get("name").is_some() {
            params.insert(String::from("name"), String::from(json_body["name"].as_str().unwrap()));
          }
        },
        Ok(None) => {
          resp = Some(Response::with((iron::status::BadRequest,
             serde_json::to_string(&WebServerDefaultResponse {ret: false, msg: String::from("No body. Or body is not a valid json")} ).unwrap())
          ));
        }
        Err(_err) => {
          resp = Some(Response::with((iron::status::BadRequest,
             serde_json::to_string(&WebServerDefaultResponse {ret: false, msg: String::from("No body. Or body is not a valid json")} ).unwrap())
          ));
        }
    }

    if params.contains_key(&String::from("name")) == false && resp.is_none() {
      resp = Some(Response::with((iron::status::BadRequest,
         serde_json::to_string(&WebServerDefaultResponse {ret: false, msg: String::from("Name field is required")} ).unwrap())
      ));
    }

    if resp.is_none() {
      system::acontrol_system_set_nfc_state(system::NFCSystemState::AUTHORIZE,Some(params));
      resp = Some(Response::with((iron::status::Ok,
         serde_json::to_string(&WebServerDefaultResponse {ret: true, msg: String::from("Ok")} ).unwrap())
      ));
    }

    let mut final_resp = resp.unwrap();

    final_resp.headers.set(iron::headers::ContentType(
      iron::mime::Mime(iron::mime::TopLevel::Application, iron::mime::SubLevel::Json, vec![])
    ));

    Ok(final_resp)
  }

  fn nfc_restore(_req: &mut Request) -> IronResult<Response> {
    system::acontrol_system_set_nfc_state(system::NFCSystemState::RESTORE,None);

    let mut resp = Response::with((iron::status::Ok,
       serde_json::to_string(&WebServerDefaultResponse {ret: true, msg: String::from("Ok")} ).unwrap())
    );

    resp.headers.set(iron::headers::ContentType(
      iron::mime::Mime(iron::mime::TopLevel::Application, iron::mime::SubLevel::Json, vec![])
    ));

    Ok(resp)
  }

  fn nfc_list(_req: &mut Request) -> IronResult<Response> {
    let mut cards: Vec<WebCard> = Vec::new();
    let mut resp: Option<Response> = None;

    if let Err(err) = system::acontrol_system_get_persist_drv(|drv| {
      if let Ok(ret) =  drv.nfc_list() {
        for card in ret {
          cards.push(WebCard {id: card.id, uuid: card.uuid, name: String::from_utf8(card.name).unwrap()});
        }
      } else {
        resp = Some(Response::with((iron::status::InternalServerError,
          serde_json::to_string(&WebServerDefaultResponse {ret: false, msg: String::from("Error searching cards")} ).unwrap())
        ));
      }
    }) {
      resp = Some(Response::with((iron::status::InternalServerError,
        serde_json::to_string(&WebServerDefaultResponse {ret: false, msg: format!("Persistence driver not found: {}", err)} ).unwrap())
      ));
    }

    if resp.is_none() {
      resp = Some(Response::with((iron::status::Ok,
         serde_json::to_string(&WebServerNfcListResponse {ret: true, msg: String::from("Ok"), cards: cards} ).unwrap())
      ));
    }

    let mut resp_final = resp.unwrap();

    resp_final.headers.set(iron::headers::ContentType(
      iron::mime::Mime(iron::mime::TopLevel::Application, iron::mime::SubLevel::Json, vec![])
    ));

    Ok(resp_final)
  }

  fn fingerprint_delete_all(_req: &mut Request) -> IronResult<Response> {
      let params: HashMap<String,String> = HashMap::new();

      acontrol_system_log!(LogType::Info, "Calling system fingerprint delete all");

      let mut resp: Response = match system::acontrol_system_fingerprint_delete_all(params) {
        Ok(_) => Response::with((iron::status::Ok,
             serde_json::to_string(&WebServerDefaultResponse {ret: true, msg: String::from("Ok")} ).unwrap())
          ),
        Err(err) => Response::with((iron::status::Ok,
             serde_json::to_string(&WebServerDefaultResponse {ret: false, msg: err} ).unwrap())
          )
      };

      resp.headers.set(iron::headers::ContentType(
        iron::mime::Mime(iron::mime::TopLevel::Application, iron::mime::SubLevel::Json, vec![])
      ));

      Ok(resp)
  }

  fn fingerprint_start_enroll(req: &mut Request) -> IronResult<Response> {

    let mut params: HashMap<String,String> = HashMap::new();
    let mut resp: Option<Response> = None;
    let json_body = req.get::<bodyparser::Json>();

    acontrol_system_log!(LogType::Info, "Server Start Enroll");

    match json_body {
        Ok(Some(json_body)) => {

          if json_body.get("name").is_some() {
            params.insert(String::from("name"), String::from(json_body["name"].as_str().unwrap()));
          }

          if json_body.get("pos").is_some() {
            params.insert(String::from("pos"), String::from(json_body["pos"].as_str().unwrap()));
          }
        },
        Ok(None) => {
          resp = Some(Response::with((iron::status::BadRequest,
             serde_json::to_string(&WebServerDefaultResponse {ret: false, msg: String::from("No body. Or body is not a valid json")} ).unwrap())
          ));
        }
        Err(_err) => {
          resp = Some(Response::with((iron::status::BadRequest,
             serde_json::to_string(&WebServerDefaultResponse {ret: false, msg: String::from("No body. Or body is not a valid json")} ).unwrap())
          ));
        }
    }

    if params.contains_key(&String::from("name")) == false && resp.is_none() {
      resp = Some(Response::with((iron::status::BadRequest,
         serde_json::to_string(&WebServerDefaultResponse {ret: false, msg: String::from("Name field is required")} ).unwrap())
      ));
    }

    if params.contains_key(&String::from("pos")) == false && resp.is_none() {
      resp = Some(Response::with((iron::status::BadRequest,
         serde_json::to_string(&WebServerDefaultResponse {ret: false, msg: String::from("Pos field is required")} ).unwrap())
      ));
    }

    if resp.is_none() {
      acontrol_system_log!(LogType::Info,"Calling system start enroll");
      match system::acontrol_system_fingerprint_start_enroll(params) {
        Ok(()) => {
          resp = Some(Response::with((iron::status::Ok,
             serde_json::to_string(&WebServerDefaultResponse {ret: true, msg: String::from("Ok")} ).unwrap())
          ));
        },
        Err(err) => {
          resp = Some(Response::with((iron::status::Ok,
             serde_json::to_string(&WebServerDefaultResponse {ret: false, msg: err} ).unwrap())
          ));
        }
      }
    }

    let mut final_resp = resp.unwrap();

    final_resp.headers.set(iron::headers::ContentType(
      iron::mime::Mime(iron::mime::TopLevel::Application, iron::mime::SubLevel::Json, vec![])
    ));

    Ok(final_resp)
  }
}

impl Server for WebServer {
  fn port(&mut self, port: u32) -> Box<&mut dyn Server> {
    self.port = port;
    return Box::new(self);
  }

  fn host(&mut self, host: &str) -> Box<&mut dyn Server> {
    self.host = host.to_string();
    return Box::new(self);
  }

  fn init(&self) -> Result<(), String> {
    acontrol_system_log!(LogType::Info,"{}",self.signature());

    let mut router = Router::new();

    router.get("/nfc/card", WebServer::nfc_list, "nfc_list");
    router.post("/nfc/card/authorize", WebServer::nfc_authorize,"nfc_authorize");
    router.get("/nfc/card/restore", WebServer::nfc_restore, "nfc_restore");

    router.post("/fingerprint/enroll",WebServer::fingerprint_start_enroll, "fingerprint_start_enroll");
    router.get("/fingerprint/delete_all",WebServer::fingerprint_delete_all, "fingerprint_delete_all");

    let chain = Chain::new(router);

    if let Err(err) = Iron::new(chain).http(format!("{}:{}",self.host,self.port.to_string())) {
      return Err(format!("{}(=> {})", "Error initializing webserver",err));
    }
    Ok( () )
  }

  fn signature(&self) -> String {
    return format!("{}{}:{}",String::from("WebServer running: "), self.host, self.port);
  }
}
