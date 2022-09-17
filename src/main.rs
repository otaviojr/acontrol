/**
 * @file   main.rs
 * @author Otavio Ribeiro
 * @date   24 Dec 2017
 * @brief  System entry point
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

pub mod bt;
pub mod fingerprint;
pub mod nfc;
pub mod audio;
pub mod server;
pub mod persist;
pub mod system;
pub mod display;

#[macro_use]
extern crate nix;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate serde_derive;

use nix::sys::signal;
use std::process;
use clap::{Arg,App};
use std::collections::HashMap;

const DEFAULT_LOGS_PATH:&str = "/var/log/acontrol";
const DEFAULT_DATA_PATH:&str = "/var/lib/acontrol";

const HTTP_DEFAULT_HOST:&str = "localhost";
const HTTP_DEFAULT_PORT:u32 = 8088;
const MIFARE_DEFAULT_KEY:&str = "0xFF,0xFF,0xFF,0xFF,0xFF,0xFF";

extern "C" fn handle_sigint(_:i32) {
  println!("Exiting...");
  system::acontrol_system_end();
  process::exit(0);
}

fn main(){
  let bt_drv;
  let fingerprint_drv;
  let nfcreader_drv;
  let audio_drv;
  let persist_drv;
  let display_drv;

  let sig_action = signal::SigAction::new(signal::SigHandler::Handler(handle_sigint),
                                          signal::SaFlags::empty(),
                                          signal::SigSet::empty());

  unsafe{
    let _ = signal::sigaction(signal::SIGINT, &sig_action);
    let _ = signal::sigaction(signal::SIGKILL, &sig_action);
  }

  let matches = App::new("Access Control")
	.version("0.0.1")
	.author("Otávio Ribeiro <otavio.ribeiro@gmail.com>")
	.about("FingerPrint + NFC Card Access Control Software")
	.arg(Arg::with_name("fingerprint-module")
		.required(true)
		.takes_value(true)
		.short("f")
		.long("fingerprint-module")
		.help("Available modules: gt521fx"))
  .arg(Arg::with_name("nfc-module")
          .required(true)
          .takes_value(true)
          .short("n")
          .long("nfc-module")
          .help("Available modules: mfrc522"))
  .arg(Arg::with_name("mifare-key")
          .required(false)
          .takes_value(true)
          .short("k")
          .long("mifare-key")
          .help("Mifare key used to format/read/write card. Default (0xFF, 0xFF, 0XFF, 0xFF, 0xFF, 0xFF)"))
  .arg(Arg::with_name("audio-module")
          .required(true)
          .takes_value(true)
          .short("a")
          .long("audio-module")
          .help("Available modules: buzzer"))
  .arg(Arg::with_name("http-server-port")
          .required(false)
          .takes_value(true)
          .short("p")
          .long("http-server-port")
          .help("http server port to listen to"))
  .arg(Arg::with_name("http-server-host")
          .required(false)
          .takes_value(true)
          .short("h")
          .long("http-server-host")
          .help("http server host to bind to"))
  .arg(Arg::with_name("bluetooth-module")
          .required(true)
          .takes_value(true)
          .short("b")
          .long("bluetooth-module")
          .help("Available modules: bluez"))  
	.get_matches();

  let http_port:u32 = value_t!(matches, "http-server-port",u32).unwrap_or(HTTP_DEFAULT_PORT);
  let http_host:&str = matches.value_of("http-server-host").unwrap_or(HTTP_DEFAULT_HOST);

  let p:&[_] = &['0','x','X'];
  let mifare_key= matches.value_of("mifare-key").unwrap_or(MIFARE_DEFAULT_KEY);
  let mifare_vec=mifare_key.split(",");
  
  let mut mifare_key_bytes: Vec<u8> = Vec::new();
  for key in mifare_vec {
    if let Ok(byte) = u8::from_str_radix(key.trim_matches(p),16) {
      mifare_key_bytes.push(byte);
    } else {
      eprintln!("invalid mifare key");
      process::exit(-1);
    }
  }

  if mifare_key_bytes.len() != 6 {
    eprintln!("invalid mifare key");
    process::exit(-1);
  }

  let bluetooth = matches.value_of("bluetooth-module").unwrap();
  let fingerprint = matches.value_of("fingerprint-module").unwrap();
  let nfc = matches.value_of("nfc-module").unwrap();
  let audio = matches.value_of("audio-module").unwrap();

  let bluetooth_b = bt::bluetooth_by_name(bluetooth);
  let fingerprint_b = fingerprint::fingerprint_by_name(fingerprint);
  let nfcreader_b = nfc::nfcreader_by_name(nfc);
  let audio_b = audio::audio_by_name(audio);
  let display_drv_b = display::display_by_name("neopixel");

  if bluetooth_b.is_none() {
    eprintln!("bluetooth module \"{}\" not found!", fingerprint);
    process::exit(-1);
  } else {
    bt_drv = bluetooth_b.unwrap();
  }

  if fingerprint_b.is_none() {
    eprintln!("fingerprint module \"{}\" not found!", fingerprint);
    process::exit(-1);
  } else {
    fingerprint_drv = fingerprint_b.unwrap();
  }

  if nfcreader_b.is_none() {
    eprintln!("nfc module \"{}\" not found!", nfc);
    process::exit(-1); 
  } else {
    nfcreader_drv = nfcreader_b.unwrap();
  }

  if audio_b.is_none() {
    eprintln!("audio module \"{}\" not found!", audio);
    process::exit(-1);
  } else {
    audio_drv = audio_b.unwrap();
  }

  if display_drv_b.is_none() {
    eprintln!("Error creating display driver");
    process::exit(-1);
  } else {
    display_drv = display_drv_b.unwrap();
  }

  println!("Fingerprint driver: {}",fingerprint_drv.signature());
  println!("Nfc driver: {}",nfcreader_drv.signature());
  println!("Audio driver: {}", audio_drv.signature());
  println!("Display driver: {}", display_drv.signature());

  let persist_drv_b = persist::persist_by_name("sqlite");
  if persist_drv_b.is_none() {
    eprintln!("Error creating persistence driver");
    process::exit(-1);
  } else {
    persist_drv = persist_drv_b.unwrap();
  }

  let mut params: HashMap<String,String> = HashMap::new();
  params.insert("LOGS_PATH".to_string(), DEFAULT_LOGS_PATH.to_string());
  params.insert("DATA_PATH".to_string(), DEFAULT_DATA_PATH.to_string());

  if !system::acontrol_system_init(&params, bt_drv, fingerprint_drv, 
					nfcreader_drv, audio_drv, persist_drv, display_drv) {
    process::exit(-1);
  }

  if !system::acontrol_system_set_mifare_keys(&mifare_key_bytes, &mifare_key_bytes) {
    process::exit(-1);
  }

  let server_b = server::create_server_by_name("generic");
  let mut server;

  if server_b.is_none() {
    eprintln!("server module not found!");
    process::exit(-1);
  } else {
    server = server_b.unwrap();
  }

  if let Err(err) = server.host(http_host).port(http_port).init() {
    eprintln!("{}",err);
  }
  system::acontrol_system_end();
}
