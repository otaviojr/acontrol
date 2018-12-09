use iron::prelude::*;
use router::Router;

use serde_derive;
use serde;
use serde_json;

use system;
use server::{Server};

use std::collections::HashMap;
use std::ptr::null;

#[derive(Serialize, Deserialize)]
struct WebServerDefaultResponse {
  ret: bool,
  msg: String
}

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
        Err(err) => {
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

  fn nfc_restore(req: &mut Request) -> IronResult<Response> {
    system::acontrol_system_set_nfc_state(system::NFCSystemState::RESTORE,None);

    let mut resp = Response::with((iron::status::Ok,
       serde_json::to_string(&WebServerDefaultResponse {ret: true, msg: String::from("Ok")} ).unwrap())
    );

    resp.headers.set(iron::headers::ContentType(
      iron::mime::Mime(iron::mime::TopLevel::Application, iron::mime::SubLevel::Json, vec![])
    ));

    Ok(resp)
  }
}

impl Server for WebServer {
  fn port(&mut self, port: u32) -> Box<&mut Server> {
    self.port = port;
    return Box::new(self);
  }

  fn host(&mut self, host: &str) -> Box<&mut Server> {
    self.host = host.to_string();
    return Box::new(self);
  }

  fn init(&self) -> Result<(), String> {
    println!("{}",self.signature());

    let mut router = Router::new();

    //router.get("/", WebServer::hello_world, "index");
    //router.get("/:query", WebServer::hello_world, "query");

    router.post("/nfc/card/authorize", WebServer::nfc_authorize,"nfc_authorize");
    router.get("/nfc/card/restore", WebServer::nfc_restore, "nfc_restore");
    let mut chain = Chain::new(router);

    if let Err(err) = Iron::new(chain).http(format!("{}:{}",self.host,self.port.to_string())) {
      return Err(format!("{}(=> {})", "Error initializing webserver",err));
    }
    Ok( () )
  }

  fn signature(&self) -> String {
    return format!("{}{}:{}",String::from("WebServer running: "), self.host, self.port);
  }
}
