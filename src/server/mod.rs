pub mod webserver;

pub trait Server {
  fn port(&mut self, port: u32) -> Box<&mut Server>;
  fn host(&mut self, host: &str) -> Box<&mut Server>;
  fn init(&self) -> Result<(),String>;
  fn signature(&self) -> String;
}

pub fn create_server_by_name(name:&str) -> Option<Box<Server+'static>> {
  match name {
    "generic" => return Some(Box::new(webserver::WebServer::new())),
    _ => return None
  }
}
