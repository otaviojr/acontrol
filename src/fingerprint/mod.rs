/**
 * @file   fingerprint/mod.rs
 * @author Otavio Ribeiro
 * @date   24 Dec 2017
 * @brief  Fingerprint global interface
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
pub mod gt521fx;

#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub enum FingerprintState {
  IDLE,
  WAITING,
  READING,
  NOT_AUTHORIZED,
  AUTHORIZED,
  ENROLL,
  ERROR,
  SUCCESS,
}

impl FingerprintState {
  pub fn name(&self) -> &'static str {
    match *self {
      FingerprintState::IDLE => "IDLE",
      FingerprintState::WAITING => "WAITING",
      FingerprintState::READING => "READING",
      FingerprintState::NOT_AUTHORIZED => "NOT_AUTHORIZED",
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
