pub mod gt521fx;

#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum FingerprintState {
  WAITING,
  READING,
  AUTHENTICATING,
  ENROLL,
  ERROR,
  SUCCESS,
}

impl FingerprintState {
  pub fn name(&self) -> &'static str {
    match *self {
      FingerprintState::WAITING => "WAITING",
      FingerprintState::READING => "READING",
      FingerprintState::AUTHENTICATING => "AUTHENTICATING",
      FingerprintState::ENROLL => "ENROLL",
      FingerprintState::ERROR => "ERROR",
      FingerprintState::SUCCESS => "SUCCESS"
    }
  }
}

pub trait Fingerprint {
  fn init(&mut self) -> Result<(), String>;
  fn wait_for_finger(&mut self, func: fn(state: &FingerprintState, value: Option<&str>) -> bool) -> Result<(),String>;
  fn unload(&mut self) -> Result<(), String>;
  fn signature(&self) -> String;
  fn start_enroll(&mut self, pos: u16) -> bool;
}

pub fn fingerprint_by_name(name: &str) -> Option<Box<Fingerprint+Sync+Send>> {
    match name {
      "gt521fx" => return Some(Box::new(gt521fx::Gt521fx::new())),
      _ => return None
    }
}
