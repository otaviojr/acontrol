extern crate clap;

pub mod fingerprint;
pub mod nfc;

use std::process;

use clap::{Arg,App};
use fingerprint::{Fingerprint};
use fingerprint::gt521fx::{Gt521fx};
use nfc::{NfcReader};
use nfc::mfrc522::{Mfrc522};

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

  match fingerprint {
    "gt521fx" => fingerprint_drv = Some(Gt521fx::new()),
    _ => fingerprint_drv = None
  }

  match nfc {
    "mfrc522" => nfcreader_drv = Some(Mfrc522::new()),
    _ => nfcreader_drv = None
  }

  if fingerprint_drv.is_none() {
    eprintln!("fingerprint module \"{}\" not found!", fingerprint);
    process::exit(-1);
  }

  if nfcreader_drv.is_none() {
    eprintln!("nfc module \"{}\" not found!", nfc);
    process::exit(-1); 
  }

  println!("Fingerprint driver: {}",fingerprint_drv.unwrap().signature());
  println!("Nfc driver: {}",nfcreader_drv.unwrap().signature());
}
