use fingerprint::{Fingerprint};

pub struct Gt521fx {
}

impl Gt521fx {
  pub fn new() -> Self {
    return Gt521fx {};
  }
}

impl Fingerprint for Gt521fx {
  fn signature(&self) -> String {
    return String::from("gt521fx Fingerprint Module");
  }

  fn start_enroll(&self) -> bool {
    return true;
  }
}

unsafe impl Send for Gt521fx {}

unsafe impl Sync for Gt521fx {}
