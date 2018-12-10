pub mod webserver;

#[derive(Serialize, Deserialize)]
struct WebServerDefaultResponse {
  ret: bool,
  msg: String
}

#[derive(Serialize, Deserialize)]
struct WebCard {
  id: i32,
  uuid: Vec<u8>,
  name: String,
}

#[derive(Serialize, Deserialize)]
struct WebServerNfcListResponse {
  ret: bool,
  msg: String,
  cards: Vec<WebCard>,
}

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
