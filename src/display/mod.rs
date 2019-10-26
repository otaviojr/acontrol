/**
 * @file   display/mod.rs
 * @author Otavio Ribeiro
 * @date   24 Dec 2017
 * @brief  Display global interface
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
pub mod neopixel;

//#[derive(Clone, Copy)]
//#[allow(dead_code)]
//pub enum DisplayState {
//  Idle,
//  WaitInput,
//  WaitProcessing,
//  Success,
//  Error,
//}

//#[derive(Clone, Copy)]
//#[allow(dead_code)]
//pub enum ErrorType {
//  Authorization,
//  Network,
//  Hardware,
//}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum AnimationType {
  Waiting,
  Error,
  Success
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum Animation {
  NoAnimation,
  MaterialSpinner,
  BlinkLoop,
  Blink,
  Wipe
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum AnimationColor {
  Orange = 0xFF8000,
  Red = 0xFF0000,
  Green = 0x00FF00,
  Blue = 0x0000FF
}

impl AnimationColor {
  fn value(&self) -> u32 {
    return (*self) as u32;
  }
}

pub trait Display : Sync + Send {
  fn init(&mut self) -> Result<(), String>;
  fn show_animation(&mut self, animation: Animation, color: AnimationColor, animation_type: AnimationType, message: &str, dismiss: u64) -> Result<(), String>;
  fn wait_animation_ends(&mut self) -> Result<(), String>;
  fn clear_and_stop_animations(&mut self) -> Result<(), String>;
  fn unload(&mut self) -> Result<(), String>;
  fn signature(&self) -> String;
}

pub fn display_by_name(name: &str) -> Option<Box<dyn Display+Sync+Send>> {
    match name {
      "neopixel" => return Some(Box::new(neopixel::NeoPixel::new())),
      _ => return None
    }
}
