pub mod buzzer;

pub trait Audio {
  fn init(&mut self) -> Result<(),String>;
  fn play_new(&mut self) -> Result<(), String>;
  fn play_granted(&mut self) -> Result<(), String>;
  fn play_denieded(&mut self) -> Result<(), String>;
  fn play_success(&mut self) -> Result<(), String>;
  fn play_error(&mut self) -> Result<(), String>;
  fn play_alert(&mut self) -> Result<(), String>;
  fn unload(&mut self) -> Result<(),String>;
  fn signature(&self) -> String;
}

pub fn audio_by_name(name: &str) -> Option<Box<Audio+Sync+Send>> {
    match name {
      "buzzer" => return Some(Box::new(buzzer::Buzzer::new())),
      _ => return None
    }
}
