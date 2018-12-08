use iron::prelude::*;
use router::Router;
use system;

use server::{Server};

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
    system::acontrol_system_set_nfc_state(system::NFCSystemState::AUTHORIZE);
    Ok(Response::with((iron::status::Ok, format!("Ok"))))
  }

  fn nfc_restore(req: &mut Request) -> IronResult<Response> {
    system::acontrol_system_set_nfc_state(system::NFCSystemState::RESTORE);
    Ok(Response::with((iron::status::Ok, format!("Ok"))))
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

    router.get("/nfc/authorize", WebServer::nfc_authorize,"nfc_authorize");
    router.get("/nfc/restore", WebServer::nfc_restore, "nfc_restore");
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
