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

pub trait Display {
  fn init(&mut self) -> Result<(), String>;
  fn unload(&mut self) -> Result<(), String>;
  fn signature(&self) -> String;
}

pub fn display_by_name(name: &str) -> Option<Box<Display+Sync+Send>> {
    match name {
      "neopixel" => return Some(Box::new(neopixel::NeoPixel::new())),
      _ => return None
    }
}
