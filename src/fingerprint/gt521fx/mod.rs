use fingerprint::{Fingerprint};

use std::time::{Duration,Instant};
use std::thread;
use std::sync::Arc;
use std::sync::Mutex;

use std::io::prelude::*;
use std::io::{ErrorKind};

use serialport::prelude::*;

#[derive(Clone, Copy)]
#[allow(dead_code)]
enum Command {
    Open = 0x01,
    Close = 0x02,
    UsbInternalCheck = 0x03,
    ChangeBaundRate = 0x04,
    CmosLed = 0x12,
    GetEnrollCount = 0x20,
    CheckEnrolled = 0x21,
    EnrollStart = 0x22,
    Enroll1 = 0x23,
    Enroll2 = 0x24,
    Enroll3 = 0x25,
    IsPressFinger = 0x26,
    DeleteID = 0x40,
    DeleteAll = 0x41,
    Verify = 0x50,
    Identify = 0x51,
    VerifyTemplate = 0x52,
    IdentifyTemplate = 0x53,
    CaptureFinger = 0x60,
    MakeTemplate = 0x61,
    GetImage = 0x62,
    GetRawImage = 0x63,
    GetTemplate = 0x70,
    SetTemplate = 0x71,
    GetDatabaseStart = 0x72,
    GetDatabaseEnd = 0x73,
    SetSecurityLevel = 0xF0,
    GetSecurityLevel = 0xF1,
    IdentifyTemplate2 = 0xF9,
    Ack = 0x30,
    Nack = 0x31
}

impl Command {
  fn value(&self) -> u16 {
    return (*self) as u16;
  }
}


#[derive(Clone, Copy)]
#[allow(dead_code)]
enum Error {
    NackTimeout = 0x1001,
    NackInvalidBaudRate = 0x1002,
    NackInvalidPos = 0x1003,
    NackIsNotUsed = 0x1004,
    NackIsAlreadyUsed = 0x1005,
    NackCommErr = 0x1006,
    NackVerifyFailed = 0x1007,
    NackIdentifyFailed = 0x1008,
    NackDbIsFull = 0x1009,
    NackDbIsEmpty = 0x100A,
    NackTurnErr = 0x100B,
    NackBadFinger = 0x100C,
    NackEnrollFailed = 0x100D,
    NackIsNotSupported = 0x100E,
    NackDevErr = 0x100F,
    NackCaptureCanceled = 0x1010,
    NackInvalidParam = 0x1011,
    NackFingerIsNotPresent = 0x1012
}

impl Error {
  fn value(&self) -> u16 {
    return (*self) as u16;
  }
}

trait Parser {
  fn size(&self) -> u32;
  fn parser(&mut self, data: &mut Vec<u8>) -> Result<bool, std::io::Error>;
}

/* Response Parser */

struct Response {
  device_id: u16,
  parameter: u32,
  response: u16,
}

impl Response {
  fn new() -> Self {
    Response {device_id: 0, parameter: 0, response: 0}
  }
}

impl Parser for Response {
  fn size(&self) -> u32 {
    return 12;
  }

  fn parser(&mut self, data: &mut Vec<u8>) -> Result <bool, std::io::Error> {

    let response_data: Vec<u8> = data.drain(0..12).collect();

    if response_data[0] != 0x55 && response_data[1] != 0xAA {
      return Err(std::io::Error::new(ErrorKind::InvalidData, "Invalid response signature"));
    }

    let calc_checksum = Gt521fxThreadSafe::calc_crc(&response_data[0..9]);
    let mut checksum: u16 = (response_data[11] as u16) << 8;
    checksum |= response_data[10] as u16;

    if checksum != calc_checksum {
      return Err(std::io::Error::new(ErrorKind::InvalidData, format!("Invalid checksum 0x{:X} - 0x{:X}", checksum, calc_checksum)));
    }

    self.device_id = (response_data[3] as u16) << 8;
    self.device_id |= response_data[2] as u16;

    self.parameter = (response_data[7] as u32) << 24;
    self.parameter |= (response_data[6] as u32) << 16;
    self.parameter |= (response_data[5] as u32) << 8;
    self.parameter |= (response_data[4] as u32);

    self.response = (response_data[9] as u16) << 8;
    self.response |= (response_data[8] as u16);

    if self.response != Command::Ack.value() {
      return Ok(false)
    }

    Ok(true)
  }
}

/* Data Packet Parsers */

struct OpenDataPacket {
  firmware_version: u32,
  iso_area_max_size: u32,
  device_serial_num: [u8;16],
}

impl OpenDataPacket {
  fn new() -> Self {
    OpenDataPacket {firmware_version: 0, iso_area_max_size: 0, device_serial_num: [0;16]}
  }
}

impl Parser for OpenDataPacket {
  fn size(&self) -> u32 {
    return 30;
  }

  fn parser(&mut self, data: &mut Vec<u8>) -> Result<bool,std::io::Error> {

    let response_data: Vec<u8> = data.drain(0..30).collect();

    if response_data[0] != 0x5A && response_data[1] != 0xA5 {
      return Err(std::io::Error::new(ErrorKind::InvalidData, "Invalid response signature"));
    }

    let calc_checksum = Gt521fxThreadSafe::calc_crc(&response_data[0..28]);
    let mut checksum: u16 = (response_data[29] as u16) << 8;
    checksum |= response_data[28] as u16;

    if checksum != calc_checksum {
      return Err(std::io::Error::new(ErrorKind::InvalidData, format!("Invalid checksum 0x{:X} - 0x{:X}", checksum, calc_checksum)));
    }

    self.firmware_version = (response_data[7] as u32) << 24;
    self.firmware_version |= (response_data[6] as u32) << 16;
    self.firmware_version |= (response_data[5] as u32) << 8;
    self.firmware_version |= (response_data[4] as u32);

    self.iso_area_max_size = (response_data[11] as u32) << 24;
    self.iso_area_max_size |= (response_data[10] as u32) << 16;
    self.iso_area_max_size |= (response_data[9] as u32) << 8;
    self.iso_area_max_size |= (response_data[8] as u32);

    for i in 0..15{
      self.device_serial_num[i] = response_data[i+12];
    }
    
    Ok(true)
  }
}

pub struct Gt521fxThreadSafe {
  port: Option<Box<SerialPort>>
}

impl Gt521fxThreadSafe {
  pub fn open(&mut self, device: &str) -> Result<(),serialport::Error> {

    let s = SerialPortSettings {
        baud_rate: 9600,
        data_bits: DataBits::Eight,
        flow_control: FlowControl::None,
        parity: Parity::None,
        stop_bits: StopBits::One,
        timeout: Duration::from_millis(10000),
    };

    match serialport::open_with_settings("/dev/serial0", &s) {
      Ok(mut port) => {
        self.port = Some(port);
        Ok(())
      },
      Err(err) => Err(err)
    }
  }

  fn calc_crc(data: &[u8]) -> u16 {
    let mut ret: u16 = 0;

    for i in data {
      ret += ((*i) as u16);
    }

    return ret;
  }

  fn send_command(&mut self, command: Command, parameter: u32, parser: Option<&mut Parser>) -> Result<Response, std::io::Error> {
    let mut data: Vec<u8>= Vec::new();

    data.push(0x55);
    data.push(0xaa);

    //Fixed device id = 0x0001
    data.push(0x01);
    data.push(0x00);

    data.push( (parameter & 0xFF) as u8);
    data.push( ((parameter >> 8) & 0xFF) as u8);
    data.push( ((parameter >> 16) & 0xFF) as u8);
    data.push( ((parameter >> 24) & 0xFF) as u8);

    data.push ( ( (command as u16) & 0xFF) as u8);
    data.push ( (( (command as u16) >> 8) & 0xFF) as u8);

    let crc:u16 = Gt521fxThreadSafe::calc_crc(&data[..]);

    data.push( (crc & 0xFF) as u8 );
    data.push( ((crc >> 8) & 0xFF) as u8 );

    let mut response = Response::new();

    if let Some(ref mut port) = self.port {
      (*port).clear(ClearBuffer::All);

      //println!("Sending: {:X?}", data);

      if let Err(err) = (*port).write(&data[..]) {
        println!("Error sending data: {}", err);
      }
      let now = Instant::now();

      if let Err(err) = loop {
        let sec = (now.elapsed().as_secs() as f64) + (now.elapsed().subsec_nanos() as f64 / 1000_000_000.0);

        if sec > 10.0 {
          break Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "Timeout"));
        }

        match (*port).bytes_to_read() {
          Ok(bytes) => {
            if bytes >= response.size() + if let Some(ref parser) = parser { (*parser).size() } else { 0 } {
              break Ok(());
            }
          },
          Err(err) => {
            println!("Error reading from serial port: {}", err);
          }
        }
      }{
        return Err(err);
      }

      let mut buf: Vec<u8> = vec!(0;255);
      if let Err(err) = (*port).read(&mut buf[..]) {
        println!("Error reading from serial: {}", err);
      }

      //println!("Received: {:X?}", buf);

      if let Err(err) = response.parser(&mut buf) {
        return Err(err);
      }

      //println!("Received: {:X?}", buf);    

      if let Some(parser) = parser {
        if let Err(err) = parser.parser(&mut buf) {
          return Err(err);
        }
      }
    } else {
      return Err(std::io::Error::new(ErrorKind::InvalidData, format!("{}","Error opening serial")));
    }

    Ok(response)
  }
}

unsafe impl Send for Gt521fxThreadSafe {}
unsafe impl Sync for Gt521fxThreadSafe {}

pub struct Gt521fx {
  gt521fx: Arc<Mutex<Gt521fxThreadSafe>>,
}

impl Gt521fx {
  pub fn new() -> Self {
    return Gt521fx { gt521fx: Arc::new(Mutex::new(Gt521fxThreadSafe { port: None } ))};
  }
}

impl Fingerprint for Gt521fx {

  fn init(&mut self) -> Result<(), String> {
    let gt521fx = self.gt521fx.clone();
    let mut gt521fx_locked = gt521fx.lock().unwrap();

    let mut open_data = OpenDataPacket::new();

    if let Err(err) = gt521fx_locked.open("/dev/serial0") {
      return Err(format!("{}","Error openning serial port."));
    }

    match gt521fx_locked.send_command(Command::Open, 0x1, Some(&mut open_data)) {
      Ok(response) => {
        println!("Fingerprint firmware version = {:X}", open_data.firmware_version);
        println!("Fingerprint serial: {:X?}",open_data.device_serial_num);
        println!("Fingerprint device initialized successfully");
      },
      Err(err) => {
        println!("Error initializing fingerprint device: {}",err);
      }
    }

    gt521fx_locked.send_command(Command::CmosLed, 0x1, None);

    Ok(())
  }

  fn wait_for_finger(&mut self, func: fn() -> bool) -> Result<(),String> {
    let gt521fx = self.gt521fx.clone();

    let _handler = thread::spawn( move || {
      loop {
        let mut result = None;

        if let Ok(ref mut gt521fx_locked) = gt521fx.lock() {
          println!("Checking finger");
          result = Some(gt521fx_locked.send_command(Command::CaptureFinger, 0x00, None));
        }

        match result {
          Some(Err(err)) => {
            println!("Erro checking fingerprint");
          },
          Some(Ok(ref response)) => {
            if response.response == Command::Ack.value() {
              println!("=========>Ok, I can see your finger<==========");
              let mut result = None;
              if let Ok(ref mut gt521fx_locked) = gt521fx.lock() {
                println!("Identifying finger");
                result = Some(gt521fx_locked.send_command(Command::Identify, 0x00, None));
              }
              match result {
                Some(Err(err)) => {
                  println!("Error identifying fingerprint!");
                }
                Some(Ok(ref response)) => {
                  if response.response == Command::Ack.value() {
                    println!("============>Fingerprint is Registered<=============");
                  } else {
                    println!("============>Fingerprint is NOT Registered<=============");
                  }
                },
                None => {}
              }
            } else {
              println!("No finger!");
            }
          },
          None => {}
        }

        thread::sleep(Duration::from_millis(500));
      }
    });

    Ok(())
  }

  fn unload(&mut self) -> Result<(), String> {
    let gt521fx = self.gt521fx.clone();
    let mut gt521fx_locked = gt521fx.lock().unwrap();

    if let Err(err) = gt521fx_locked.send_command(Command::CmosLed, 0x0, None) {
      println!("Error turning off fingerprint led: {}", err);
    }

    match gt521fx_locked.send_command(Command::Close, 0x0, None) {
      Ok(response) => {
        println!("Fingerprint device closed successfully");
        Ok(())
      },
      Err(err) => {
        Err(format!("Error closing fingerprint device: {}",err))
      }
    }
  }

  fn signature(&self) -> String {
    return String::from("gt521fx Fingerprint Module");
  }

  fn start_enroll(&self) -> bool {
    return true;
  }
}

unsafe impl Send for Gt521fx {}
unsafe impl Sync for Gt521fx {}
