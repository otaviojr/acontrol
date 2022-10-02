/**
 * @file   log/console.rs
 * @author Otavio Ribeiro
 * @date   24 Sep 2022
 * @brief  Console log module
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

use crate::{log::{Log, LogType, LogUtils}};
use std::collections::HashMap;

pub struct ConsoleLog {
}


impl ConsoleLog {
  pub fn new(_params: &HashMap<String, String>) -> Self {
      return ConsoleLog {};
  }
}

unsafe impl Sync for ConsoleLog {}
unsafe impl Send for ConsoleLog {}

impl Drop for ConsoleLog {
  fn drop(&mut self) {
    println!("Unloading log module driver");
    let _res = self.unload();
  }
}

impl Log for ConsoleLog {
  fn init(&mut self) -> Result<(), String> {
    Ok(())
  }

  fn log(&self, log_type: LogType, message: String) -> Result<(), String> {
    println!("{}",LogUtils::formatted_message(log_type, message));
    Ok(())
  }

  fn unload(&mut self) -> Result<(), String>{
    println!("Console driver unloading");
    Ok(())
  }

  fn signature(&self) -> String {
    return String::from("Console Log Module");
  }
}
