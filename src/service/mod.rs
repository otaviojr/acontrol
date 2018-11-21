pub mod iron;

pub trait Service {
  fn port(&mut self, port: i32) -> Box<&mut Service>;
  fn host(&mut self, host: String) -> Box<&mut Service>;
  fn init(&self) -> bool;
  fn signature(&self) -> String;
}

pub fn create_service_by_name(name:&str) -> Option<Box<Service+'static>> {
  match name {
    "iron" => return Some(Box::new(iron::IronWebService::new())),
    _ => return None
  }
}
