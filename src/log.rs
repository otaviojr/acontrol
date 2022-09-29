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
mod console;

use std::collections::HashMap;
use chrono::{DateTime,Utc};

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

struct LogUtils {
}

impl LogUtils {
  fn current_date_time() -> String {
    let now:DateTime<Utc> = Utc::now();
    now.to_rfc3339()
  }

  fn formatted_message(log_type: LogType, message: String) -> String {
    format!("{} - [{}] - {}",LogUtils::current_date_time(), log_type.name(),message)
  }
}

pub trait Log {
  fn init(&mut self) -> Result<(),String>;
  fn log(&mut self, log_type: LogType, message: String) -> Result<(), String>;
  fn unload(&mut self) -> Result<(),String>;
  fn signature(&self) -> String;
}

pub struct MainLog {
  inner_log: Box<dyn Log>,
  log_level: LogType
}

impl MainLog {
  pub fn new(log_level: LogType, inner_log: Box<dyn Log>) -> Self {
    return MainLog { log_level: log_level, inner_log: inner_log};
  }
}

unsafe impl Sync for MainLog {}
unsafe impl Send for MainLog {}

impl Log for MainLog {
  fn init(&mut self) -> Result<(),String>{
    self.inner_log.init()
  }

  fn log(&mut self, log_type: LogType, message: String) -> Result<(), String>{
    if log_type.value() >= self.log_level.value(){
      self.inner_log.log(log_type, message)
    } else {
      Ok(())
    }
  }

  fn unload(&mut self) -> Result<(),String>{
    self.inner_log.unload()
  }

  fn signature(&self) -> String {
    self.inner_log.signature()
  }
}

pub fn log_by_name(name: &str, log_level: LogType, params: HashMap<String, String>) -> Option<Box<dyn Log+Sync+Send>> {
    match name {
      "console" => return Some(Box::new(MainLog::new(log_level, Box::new(console::ConsoleLog::new(params))))),
      "file" => return Some(Box::new(MainLog::new(log_level, Box::new(file::FileLog::new(params))))),
      _ => return None
    }
}
