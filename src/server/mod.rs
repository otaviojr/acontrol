/**
 * @file   server/mod.rs
 * @author Otavio Ribeiro
 * @date   24 Dec 2017
 * @brief  WebAPI global interface
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
pub mod webserver;

#[derive(Serialize, Deserialize)]
struct WebServerDefaultResponse {
  ret: bool,
  msg: String
}

#[derive(Serialize, Deserialize)]
struct WebCard {
  id: i32,
  uuid: Vec<u8>,
  name: String,
}

#[derive(Serialize, Deserialize)]
struct WebServerNfcListResponse {
  ret: bool,
  msg: String,
  cards: Vec<WebCard>,
}

pub trait Server {
  fn port(&mut self, port: u32) -> Box<&mut Server>;
  fn host(&mut self, host: &str) -> Box<&mut Server>;
  fn init(&self) -> Result<(),String>;
  fn signature(&self) -> String;
}

pub fn create_server_by_name(name:&str) -> Option<Box<Server+'static>> {
  match name {
    "generic" => return Some(Box::new(webserver::WebServer::new())),
    _ => return None
  }
}
