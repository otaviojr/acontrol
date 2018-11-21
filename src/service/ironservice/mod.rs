use iron::prelude::*;
use router::Router;
use rustc_serialize::json;

use service::{Service};

pub struct IronWebService {
  host: String,
  port: i32,
}

impl IronWebService {
  pub fn new() -> Self {
    return IronWebService { host: "".to_string(), port: 0};
  }

  fn hello_world(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((iron::status::Ok, "Hello World")))
  }
}

impl Service for IronWebService {
  fn port(&mut self, port: i32) -> Box<&mut Service> {
    self.port = port;
    return Box::new(self);
  }

  fn host(&mut self, host: String) -> Box<&mut Service> {
    self.host = host;
    return Box::new(self);
  }

  fn init(&self) -> bool {
    println!("{}",self.signature());
    let mut chain = Chain::new(IronWebService::hello_world);
    let _server = Iron::new(chain).http(format!("{}:{}",self.host,self.port.to_string())).unwrap();
    return true;
  }

  fn signature(&self) -> String {
    return format!("{}{}",String::from("Iron WebService running at port "), self.port);
  }
}
