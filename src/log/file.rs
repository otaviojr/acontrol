/**
 * @file   log/file.rs
 * @author Otavio Ribeiro
 * @date   24 Sep 2022
 * @brief  File log module
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

use crate::{log::{Log, LogType}};
use std::{collections::HashMap, path::{PathBuf}, fs::{OpenOptions}, io::Write};

use super::LogUtils;

pub struct FileLog {
  log_file: PathBuf,
}


impl FileLog {
  pub fn new(_params: &HashMap<String, String>) -> Self {
      let mut path = PathBuf::from("/var/log/acontrol");
      path.push("acontrol.log");
      let _ = std::fs::remove_file(path.as_os_str());
      return FileLog {log_file: path };
  }
}

unsafe impl Sync for FileLog {}
unsafe impl Send for FileLog {}

impl Drop for FileLog {
  fn drop(&mut self) {
    println!("Unloading log module driver");
    let _res = self.unload();
  }
}

impl Log for FileLog {
  fn init(&mut self) -> Result<(), String> {
    Ok(())
  }

  fn log(&self, log_type: LogType, message: String) -> Result<(), String> {
    match OpenOptions::new()
    .create(true)
    .write(true)
    .append(true)
    .open(self.log_file.as_os_str()) {
      Ok(mut file) => {
        let mut final_message: String = message;
        final_message.push_str("\r\n");
        if let Err(err) = file.write_all(LogUtils::formatted_message(log_type,final_message).as_bytes()) {
          return Err(format!("Log Error: {}", err));
        }
        Ok(())
      },
      Err(err) => Err(format!("Log Error: {}", err))
    }
  }

  fn unload(&mut self) -> Result<(), String>{
    println!("Log driver unloading");
    Ok(())
  }

  fn signature(&self) -> String {
    return String::from("File Log Module");
  }
}
