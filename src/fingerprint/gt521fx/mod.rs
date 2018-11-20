use fingerprint::{Fingerprint};

pub struct Gt521fx {}

impl Gt521fx {
}

impl Fingerprint for Gt521fx {

  fn new() -> Gt521fx {
    return Gt521fx {};
  }

  fn signature(&self) -> String {
    return String::from("gt521fx Fingerprint Module");
  }

  fn start_enroll(&self) -> bool {
    return true;
  }
}
