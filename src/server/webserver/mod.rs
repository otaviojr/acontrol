use iron::prelude::*;
use router::Router;
use rustc_serialize::json;

use server::{Server};

pub struct WebServer {
  host: String,
  port: u32,
}

impl WebServer {
  pub fn new() -> Self {
    return WebServer { host: "".to_string(), port: 0};
  }

  fn hello_world(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((iron::status::Ok, "Hello World")))
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

  fn init(&self) -> bool {
    println!("{}",self.signature());
    let mut chain = Chain::new(WebServer::hello_world);
    let _server = Iron::new(chain).http(format!("{}:{}",self.host,self.port.to_string())).unwrap();
    return true;
  }

  fn signature(&self) -> String {
    return format!("{}{}:{}",String::from("WebServer running: "), self.host, self.port);
  }
}
