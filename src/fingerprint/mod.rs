pub mod gt521fx;

pub trait Fingerprint {
  fn signature(&self) -> String;
  fn start_enroll(&self) -> bool;
}

pub fn fingerprint_by_name(name: &str) -> Option<Box<Fingerprint+'static>> {
    match name {
      "gt521fx" => return Some(Box::new(gt521fx::Gt521fx::new())),
      _ => return None
    }
}
