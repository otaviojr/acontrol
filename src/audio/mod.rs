pub mod buzzer;

pub trait Audio {
  fn init(&mut self) -> Result<String,String>;
  fn signature(&self) -> String;
}

pub fn audio_by_name(name: &str) -> Option<Box<Audio+Sync+Send>> {
    match name {
      "buzzer" => return Some(Box::new(buzzer::Buzzer::new())),
      _ => return None
    }
}
