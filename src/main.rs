extern crate iron;
extern crate router;
extern crate rustc_serialize;
extern crate clap;

pub mod fingerprint;
pub mod nfc;
pub mod service;

use std::process;

use clap::{Arg,App};

const DEFAULT_LOGS_PATH:&str = "/var/log/acontrol";
const HTTP_DEFAULT_PORT:i32 = 8088;

fn main(){
  let fingerprint_drv;
  let nfcreader_drv;  
  let mut http_port:i32 = HTTP_DEFAULT_PORT;

  let matches = App::new("Access Control")
	.version("0.0.1")
	.author("Otavio Ribeiro <otavio.ribeiro@gmail.com")
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
        .arg(Arg::with_name("http-service-port")
                .required(false)
                .takes_value(true)
                .short("p")
                .long("http-service-port")
                .help("Rest API http port to listen"))
	.get_matches();

  let http_service_port = matches.value_of("http-service-port");
  if http_service_port.is_some() {
    http_port = http_service_port.unwrap().parse::<i32>().unwrap();
  }

  println!("Http Service Port: {}", http_port);

  let fingerprint = matches.value_of("fingerprint-module").unwrap();
  let nfc = matches.value_of("nfc-module").unwrap();

  let fingerprint_b = fingerprint::fingerprint_by_name(fingerprint);
  let nfcreader_b = nfc::nfcreader_by_name(nfc);

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

  println!("Fingerprint driver: {}",fingerprint_drv.signature());
  println!("Nfc driver: {}",nfcreader_drv.signature());

  let service_b = service::create_service_by_name("iron");
  let mut service;

  if service_b.is_none() {
    eprintln!("service module not found!");
    process::exit(-1);
  } else {
    service = service_b.unwrap();
  }
  service.host("localhost".to_string()).port(http_port).init();
}
