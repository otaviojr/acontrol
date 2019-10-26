/**
 * @file   audio/mod.rs
 * @author Otavio Ribeiro
 * @date   24 Dec 2017
 * @brief  Audio global interface
 *
 * Copyright (c) 2019 Otávio Ribeiro <otavio.ribeiro@gmail.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
 * THE SOFTWARE.
 *
 */
pub mod buzzer;

pub trait Audio {
  fn init(&mut self) -> Result<(),String>;
  fn play_new(&mut self) -> Result<(), String>;
  fn play_granted(&mut self) -> Result<(), String>;
  fn play_denied(&mut self) -> Result<(), String>;
  fn play_success(&mut self) -> Result<(), String>;
  fn play_error(&mut self) -> Result<(), String>;
  fn play_alert(&mut self) -> Result<(), String>;
  fn unload(&mut self) -> Result<(),String>;
  fn signature(&self) -> String;
}

pub fn audio_by_name(name: &str) -> Option<Box<dyn Audio+Sync+Send>> {
    match name {
      "buzzer" => return Some(Box::new(buzzer::Buzzer::new())),
      _ => return None
    }
}
