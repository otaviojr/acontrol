use service::{Service};

pub struct IronWebService {
  host: String,
  port: i32,
}

impl IronWebService {
  pub fn new() -> Self {
    return IronWebService { host: "".to_string(), port: 0};
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
    return true;
  }

  fn signature(&self) -> String {
    return format!("{}{}",String::from("Iron WebService running at port "), self.port);
  }
}
