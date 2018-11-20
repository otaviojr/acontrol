pub mod gt521fx;

pub trait Fingerprint {
  fn new() -> Self;
  fn signature(&self) -> String;
  fn start_enroll(&self) -> bool;
}
