
extern crate nix;
extern crate iron;
extern crate router;
extern crate rustc_serialize;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate clap;

extern crate spidev;
extern crate sysfs_gpio;

pub mod fingerprint;
pub mod nfc;
pub mod audio;
pub mod server;
pub mod system;

use nix::sys::signal;
use std::process;
use clap::{Arg,App};

const DEFAULT_LOGS_PATH:&str = "/var/log/acontrol";

const HTTP_DEFAULT_HOST:&str = "localhost";
const HTTP_DEFAULT_PORT:u32 = 8088;

extern "C" fn handle_sigint(_:i32) {
  println!("Exiting...");
  system::end_acontrol_system();
  process::exit(0);
}

fn main(){
  let fingerprint_drv;
  let nfcreader_drv;
  let audio_drv;

  let sig_action = signal::SigAction::new(signal::SigHandler::Handler(handle_sigint),
                                          signal::SaFlags::empty(),
                                          signal::SigSet::empty());

  unsafe{
    signal::sigaction(signal::SIGINT, &sig_action);
    signal::sigaction(signal::SIGKILL, &sig_action);
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
	.get_matches();

  let http_port:u32 = value_t!(matches, "http-server-port",u32).unwrap_or(HTTP_DEFAULT_PORT);
  let http_host:&str = matches.value_of("http-server-host").unwrap_or(HTTP_DEFAULT_HOST);

  let fingerprint = matches.value_of("fingerprint-module").unwrap();
  let nfc = matches.value_of("nfc-module").unwrap();
  let audio = matches.value_of("audio-module").unwrap();

  let fingerprint_b = fingerprint::fingerprint_by_name(fingerprint);
  let nfcreader_b = nfc::nfcreader_by_name(nfc);
  let audio_b = audio::audio_by_name(audio);

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

  println!("Fingerprint driver: {}",fingerprint_drv.signature());
  println!("Nfc driver: {}",nfcreader_drv.signature());
  println!("Audio driver: {}", audio_drv.signature());

  if !system::init_acontrol_system(fingerprint_drv, nfcreader_drv, audio_drv) {
    process::exit(1);
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
  system::end_acontrol_system();
}
