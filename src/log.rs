/**
 * @file   log.rs
 * @author Otavio Ribeiro
 * @date   29 Sep 2022
 * @brief  Log global interface
 *
 * Copyright (c) 2022 Otávio Ribeiro <otavio.ribeiro@gmail.com>
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
mod file;

use std::collections::HashMap;

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum LogType {
    Debug,
    Info,
    Warning,
    Error,
    Fatal
}

#[allow(dead_code)]
impl LogType {
  fn name(&self) -> &'static str {
    match *self {
      LogType::Debug => "Debug",
      LogType::Info => "Info",
      LogType::Warning => "Warning",
      LogType::Error => "Error",
      LogType::Fatal => "Fatal",
    }
  }

  fn value(&self) -> u16 {
    return (*self) as u16;
  }
}

pub trait Log {
  fn init(&mut self) -> Result<(),String>;
  fn log(&mut self, log_type: LogType, message: String) -> Result<(), String>;
  fn unload(&mut self) -> Result<(),String>;
  fn signature(&self) -> String;
}

pub fn log_by_name(name: &str, params: HashMap<String, String>) -> Option<Box<dyn Log+Sync+Send>> {
    match name {
      "file" => return Some(Box::new(file::FileLog::new(params))),
      _ => return None
    }
}
