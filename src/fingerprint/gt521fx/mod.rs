use fingerprint::{Fingerprint};

use std::io;
use std::time::Duration;
use std::thread;
use std::sync::Arc;
use std::sync::Mutex;

use std::io::prelude::*;
use serial::prelude::*;

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

struct ResponsePacket {
  response: u16,
  parameter: u32
}

pub struct Gt521fxThreadSafe {
  port: Option<Box<SerialPort>>,
}

impl Gt521fxThreadSafe {
  pub fn open(&mut self, device: &str) -> Result<(),serial::Error> {
    match serial::open(device) {
      Ok(mut port) => {

        port.reconfigure(&|settings| {
          try!(settings.set_baud_rate(serial::Baud9600));
          settings.set_char_size(serial::Bits8);
          settings.set_parity(serial::ParityNone);
          settings.set_stop_bits(serial::Stop1);
          settings.set_flow_control(serial::FlowNone);
          Ok(())
        });

        self.port = Some(Box::new(port));
      },
      Err(err) => return Err(err)
    }

    Ok(())
  }

  fn calc_crc(&self, data: &Vec<u8>) -> u16 {
    let mut ret: u16 = 0;

    for i in data {
      ret += ((*i) as u16);
    }

    return ret;
  }

  fn send_command(&mut self, command: Command, parameter: u32) -> Result<(), serial::Error> {
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

    let crc:u16 = self.calc_crc(&data);

    data.push( (crc & 0xFF) as u8 );
    data.push( ((crc >> 8) & 0xFF) as u8 );


    if let Some(ref mut port) = self.port {
      (**port).set_timeout(Duration::from_millis(1000));

      let mut buf: Vec<u8> = vec!(0;255);

      (**port).write(data.as_slice());
      thread::sleep(Duration::from_millis(100));
      (**port).read(&mut buf[..]);

      println!("Send: {:X?}", data);    
      println!("Received: {:X?}", buf);    
    }

    Ok(())
  }

  pub fn led(&mut self, state: bool)-> Result<(), serial::Error> {
    Ok(())
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

    if let Err(err) = gt521fx_locked.open("/dev/serial0") {
      return Err(format!("Error openning serial port at {}","/dev/serial0"));
    } else {
      gt521fx_locked.send_command(Command::Open, 0x1);
      println!("Fingerprint device initialized successfully");
    }

    Ok(())
  }

  fn unload(&mut self) -> Result<(), String> {
    Ok(())
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
