pub mod neopixel;

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum DisplayState {
  Idle,
  WaitInput,
  WaitProcessing,
  Success,
  Error,
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum ErrorType {
  Authorization,
  Network,
  Hardware,
}

pub trait Display : Sync + Send {
  fn init(&mut self) -> Result<(), String>;
  fn show_success(&mut self, message: &str, dismiss: u64) -> Result<(), String>;
  fn show_error(&mut self, message: &str, error_type: ErrorType, dismiss: u64) -> Result<(), String>;
  fn show_waiting(&mut self, message: &str, dismiss: u64) -> Result<(), String>;
  fn unload(&mut self) -> Result<(), String>;
  fn signature(&self) -> String;
}

pub fn display_by_name(name: &str) -> Option<Box<Display+Sync+Send>> {
    match name {
      "neopixel" => return Some(Box::new(neopixel::NeoPixel::new())),
      _ => return None
    }
}
