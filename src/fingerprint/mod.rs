pub mod gt521fx;

#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum FingerprintState {
  WAITING,
  READING,
  AUTHORIZED,
  ENROLL,
  ERROR,
  SUCCESS,
}

impl FingerprintState {
  pub fn name(&self) -> &'static str {
    match *self {
      FingerprintState::WAITING => "WAITING",
      FingerprintState::READING => "READING",
      FingerprintState::AUTHORIZED => "AUTHORIZED",
      FingerprintState::ENROLL => "ENROLL",
      FingerprintState::ERROR => "ERROR",
      FingerprintState::SUCCESS => "SUCCESS"
    }
  }

  pub fn set(&mut self, new_state:FingerprintState) {
    *self = new_state;
  }
}

pub struct FingerprintData {
  pub pos: Option<u16>,
  pub name: Option<String>
}

impl FingerprintData {
  pub fn new(pos: u16, name: &str) -> Self {
    FingerprintData { pos: Some(pos), name: Some(String::from(name)) }
  }

  pub fn empty() -> Self {
    FingerprintData { pos: None, name: None }
  }
}

pub trait Fingerprint {
  fn init(&mut self) -> Result<(), String>;
  fn wait_for_finger(&mut self, func: fn(state: &FingerprintState, value: Option<&str>) -> bool) -> Result<(),String>;
  fn unload(&mut self) -> Result<(), String>;
  fn signature(&self) -> String;
  fn start_enroll(&mut self, data: &FingerprintData) -> bool;
}

pub fn fingerprint_by_name(name: &str) -> Option<Box<Fingerprint+Sync+Send>> {
    match name {
      "gt521fx" => return Some(Box::new(gt521fx::Gt521fx::new())),
      _ => return None
    }
}
