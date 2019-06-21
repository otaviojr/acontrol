/**
 * @file   gt521fx/mod.rs
 * @author Otavio Ribeiro
 * @date   24 Dec 2017
 * @brief  gt521fx fingerprint sensor driver
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
use fingerprint::{Fingerprint,FingerprintState, FingerprintData};

use std::time::{Duration,Instant};
use std::thread;
use std::sync::Arc;
use std::sync::Mutex;

use std::io::prelude::*;
use std::io::{ErrorKind};

use serialport::prelude::*;

use sysfs_gpio::{Direction, Pin};

#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub enum FingerprintDriverState {
  IDLE,
  READ,
  ENROLL1,
  ENROLL2,
  ENROLL3,
  ENROLL1_WAIT,
  ENROLL2_WAIT,
  ENROLL_ERROR,
}

#[allow(dead_code)]
impl FingerprintDriverState {

  fn name(&self) -> &'static str {
    match *self {
      FingerprintDriverState::IDLE => "IDLE",
      FingerprintDriverState::READ => "READ",
      FingerprintDriverState::ENROLL1 => "ENROLL1",
      FingerprintDriverState::ENROLL2 => "ENROLL2",
      FingerprintDriverState::ENROLL3 => "ENROLL3",
      FingerprintDriverState::ENROLL1_WAIT => "ENROLL1_WAIT",
      FingerprintDriverState::ENROLL2_WAIT => "ENROLL2_WAIT",
      FingerprintDriverState::ENROLL_ERROR => "ENROLL_ERROR",
    }
  }

  fn value(&self) -> u16 {
    return (*self) as u16;
  }

  fn set(&mut self, next: FingerprintDriverState) -> bool {
    *self = next;
    println!("FingerprintDriverState changed to {}",(*self).name());
    return true;
  }
}

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
    NackUnknow = 0x0000,
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

#[allow(dead_code)]
impl Error {
  fn name(&self) -> &'static str {
    match *self {
      Error::NackUnknow => "NackUnknow",
      Error::NackTimeout => "NackTimeout",
      Error::NackInvalidBaudRate => "NackInvalidBaudRate",
      Error::NackInvalidPos => "NackInvalidPos",
      Error::NackIsNotUsed => "NackIsNotUsed",
      Error::NackIsAlreadyUsed => "NackIsAlreadyUsed",
      Error::NackCommErr => "NackCommErr",
      Error::NackVerifyFailed => "NackVerifyFailed",
      Error::NackIdentifyFailed => "NackIdentifyFailed",
      Error::NackDbIsFull => "NackDbIsFull",
      Error::NackDbIsEmpty => "NackDbIsEmpty",
      Error::NackTurnErr => "NackTurnErr",
      Error::NackBadFinger => "NackBadFinger",
      Error::NackEnrollFailed => "NackEnrollFailed",
      Error::NackIsNotSupported => "NackIsNotSupported",
      Error::NackDevErr => "NackDevErr",
      Error::NackCaptureCanceled => "NackCaptureCanceled",
      Error::NackInvalidParam => "NackInvalidParam",
      Error::NackFingerIsNotPresent => "NackFingerIsNotPresent",
    }
  }

  fn value(&self) -> u16 {
    return (*self) as u16;
  }
}

impl From<u32> for Error {
  fn from(value: u32) -> Self {
    match value {
      0x1001 => Error::NackTimeout,
      0x1002 => Error::NackInvalidBaudRate,
      0x1003 => Error::NackInvalidPos,
      0x1004 => Error::NackIsNotUsed,
      0x1005 => Error::NackIsAlreadyUsed,
      0x1006 => Error::NackCommErr,
      0x1007 => Error::NackVerifyFailed,
      0x1008 => Error::NackIdentifyFailed,
      0x1009 => Error::NackDbIsFull,
      0x100A => Error::NackDbIsEmpty,
      0x100B => Error::NackTurnErr,
      0x100C => Error::NackBadFinger,
      0x100D => Error::NackEnrollFailed,
      0x100E => Error::NackIsNotSupported,
      0x100F => Error::NackDevErr,
      0x1010 => Error::NackCaptureCanceled,
      0x1011 => Error::NackInvalidParam,
      0x1012 => Error::NackFingerIsNotPresent,
      _ => Error::NackUnknow,
    }
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

    self.parameter  = (response_data[7] as u32) << 24;
    self.parameter |= (response_data[6] as u32) << 16;
    self.parameter |= (response_data[5] as u32) << 8;
    self.parameter |= response_data[4] as u32;

    self.response  = (response_data[9] as u16) << 8;
    self.response |= response_data[8] as u16;

    if self.response != Command::Ack.value() {
      return Ok(false)
    }

    Ok(true)
  }
}

/* Data Packet Parsers */

struct FingerprintTemplatePacket {
  template: Option<Vec<u8>>
}

#[allow(dead_code)]
impl FingerprintTemplatePacket {
  fn new() -> Self {
    FingerprintTemplatePacket {template: None }
  }
}

impl Parser for FingerprintTemplatePacket {
  fn size(&self) -> u32 {
    return 498+6;
  }

  fn parser(&mut self, data: &mut Vec<u8>) -> Result<bool,std::io::Error> {

    let response_data: Vec<u8> = data.drain(0..self.size() as usize).collect();

    if response_data[0] != 0x5A && response_data[1] != 0xA5 {
      return Err(std::io::Error::new(ErrorKind::InvalidData, "Invalid response signature"));
    }

    let calc_checksum = Gt521fxThreadSafe::calc_crc(&response_data[0..502]);
    let mut checksum: u16 = (response_data[503] as u16) << 8;
    checksum |= response_data[502] as u16;

    if checksum != calc_checksum {
      return Err(std::io::Error::new(ErrorKind::InvalidData, format!("Invalid checksum 0x{:X} - 0x{:X}", checksum, calc_checksum)));
    }

    self.template = Some(response_data[4..498].to_vec());

    Ok(true)
  }
}

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
    return 24+6;
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

    self.firmware_version  = (response_data[7] as u32) << 24;
    self.firmware_version |= (response_data[6] as u32) << 16;
    self.firmware_version |= (response_data[5] as u32) << 8;
    self.firmware_version |= response_data[4] as u32;

    self.iso_area_max_size  = (response_data[11] as u32) << 24;
    self.iso_area_max_size |= (response_data[10] as u32) << 16;
    self.iso_area_max_size |= (response_data[9] as u32) << 8;
    self.iso_area_max_size |= response_data[8] as u32;

    for i in 0..15{
      self.device_serial_num[i] = response_data[i+12];
    }
    
    Ok(true)
  }
}

pub struct Gt521fxThreadSafe {
  port: Option<Box<SerialPort>>,
  pin: Option<Pin>
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

    match serialport::open_with_settings(device, &s) {
      Ok(port) => {
        self.port = Some(port);
        Ok(())
      },
      Err(err) => Err(err)
    }
  }

  pub fn pin_config(&mut self, pin_num: u64) -> Result<(), sysfs_gpio::Error> {
    let pin = Pin::new(pin_num);

    self.pin = Some(pin);

    Ok(())
  }

  fn calc_crc(data: &[u8]) -> u16 {
    let mut ret: u16 = 0;

    for i in data {
      ret += (*i) as u16;
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
      let _ret = (*port).clear(ClearBuffer::All);

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

        thread::sleep(Duration::from_millis(100));
      }{
        return Err(err);
      }

      let mut buf: Vec<u8> = vec!(0;1024);
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
  state: Arc<Mutex<FingerprintDriverState>>,
  expires: Arc<Mutex<Option<Instant>>>,
}

impl Gt521fx {
  pub fn new() -> Self {
    return Gt521fx { expires: Arc::new(Mutex::new(None)), state: Arc::new(Mutex::new(FingerprintDriverState::IDLE)), gt521fx: Arc::new(Mutex::new(Gt521fxThreadSafe { port: None, pin: None } ))};
  }
}

impl Fingerprint for Gt521fx {

  fn init(&mut self) -> Result<(), String> {
    let gt521fx = self.gt521fx.clone();
    let mut gt521fx_locked = gt521fx.lock().unwrap();

    let mut open_data = OpenDataPacket::new();

    if let Err(_err) = gt521fx_locked.open("/dev/serial0") {
      return Err(format!("{}","Error openning serial port."));
    }

    if let Err(_err) = gt521fx_locked.pin_config(16) {
      return Err(format!("{}","Error configuring fingerprint touch sensor pin."));
    }

    match gt521fx_locked.send_command(Command::Open, 0x1, Some(&mut open_data)) {
      Ok(_response) => {
        println!("Fingerprint firmware version = {:X}", open_data.firmware_version);
        println!("Fingerprint serial: {:X?}",open_data.device_serial_num);
        println!("Fingerprint device initialized successfully");
      },
      Err(err) => {
        println!("Error initializing fingerprint device: {}",err);
      }
    }

    //if let Err(err) = gt521fx_locked.send_command(Command::CmosLed, 0x1, None) {
    //  println!("Error initializing fingerprint device: {}",err);
    //} else {
    //  println!("Fingerprint LED on");
    //}

    Ok(())
  }

  fn wait_for_finger(&mut self, func: fn(state: &FingerprintState, value: Option<&str>) -> bool) -> Result<(),String> {
    let gt521fx = self.gt521fx.clone();
    let state = self.state.clone();
    let expires = self.expires.clone();

    let _handler = thread::spawn( move || {
      let mut fingerpress_counter = 0;
      loop {
        {
          if let Ok(ref mut state_locked) = state.lock() {
            if let Ok(ref mut expires_locked) = expires.lock() {

              let fingerprint_state = match **state_locked {
                FingerprintDriverState::IDLE => Some(FingerprintState::IDLE),
                FingerprintDriverState::READ => Some(FingerprintState::READING),
                FingerprintDriverState::ENROLL1 | FingerprintDriverState::ENROLL2 | FingerprintDriverState::ENROLL3 => Some(FingerprintState::WAITING),
                FingerprintDriverState::ENROLL1_WAIT | FingerprintDriverState::ENROLL2_WAIT => Some(FingerprintState::SUCCESS),
                FingerprintDriverState::ENROLL_ERROR => Some(FingerprintState::ERROR),
              };

              if let Some(ref state) = fingerprint_state {
                func(state, None);
              }

              let mut sec = 0.0;
              if let Some(expires) = **expires_locked {
                sec = (expires.elapsed().as_secs() as f64) + (expires.elapsed().subsec_nanos() as f64 / 1000_000_000.0);
              }

              println!("Current State Time: {}", sec);

              if sec > 120.0 {
                state_locked.set(FingerprintDriverState::IDLE);
                (**expires_locked) = None;
              }

              match **state_locked {
                FingerprintDriverState::ENROLL1 | FingerprintDriverState::ENROLL2 | FingerprintDriverState::ENROLL3 => {
                  let mut result = None;

                  if let Ok(ref mut gt521fx_locked) = gt521fx.lock() {
                    println!("Checking finger");
                    result = Some(gt521fx_locked.send_command(Command::CaptureFinger, 0x01, None));
                  }
              
                  match result {
                    Some(Err(_err)) => {
                      println!("Erro checking fingerprint");
                      func(&FingerprintState::ERROR, None);
                      state_locked.set(FingerprintDriverState::IDLE);
                      (**expires_locked) = None;
                    },
                    Some(Ok(ref response)) => {
                      if response.response == Command::Ack.value() {
                        let mut result = None;

                        if let Ok(ref mut gt521fx_locked) = gt521fx.lock() {
                          println!("Checking finger");
                          if **state_locked == FingerprintDriverState::ENROLL1 {
                            result = Some(gt521fx_locked.send_command(Command::Enroll1, 0x00, None));
                          } else if **state_locked == FingerprintDriverState::ENROLL2 {
                            result = Some(gt521fx_locked.send_command(Command::Enroll2, 0x00, None));
                          } else {
                            result = Some(gt521fx_locked.send_command(Command::Enroll3, 0x00, None));
                          }
                        }

                        match result {
                          Some(Err(err)) => {
                            println!("Enroll error: {}",err);
                            func(&FingerprintState::ERROR, None);
                            state_locked.set(FingerprintDriverState::IDLE);
                            (**expires_locked) = None;
                          },
                          Some(Ok(ref response)) => {
                            if response.response == Command::Ack.value() {
                              if **state_locked == FingerprintDriverState::ENROLL1 {
                                state_locked.set(FingerprintDriverState::ENROLL1_WAIT);
                                (**expires_locked) = Some(Instant::now());
                              } else if **state_locked == FingerprintDriverState::ENROLL2 {
                                state_locked.set(FingerprintDriverState::ENROLL2_WAIT);
                                (**expires_locked) = Some(Instant::now());
                              } else {
                                func(&FingerprintState::ENROLL, None);
                                state_locked.set(FingerprintDriverState::IDLE);
                                (**expires_locked) = None;
                              }
                            } else {
                              if response.parameter <= 2999 {
                                println!("Enrollment error: ID duplicated - {}", response.parameter);
                              } else {
                                println!("Enroll error: {}", (Error::from(response.parameter)).name());
                              }
                              func(&FingerprintState::ERROR, None);
                              state_locked.set(FingerprintDriverState::IDLE);
                              (**expires_locked) = None;
                            }
                          },
                          None => {},
                        }
                      }
                    },
                    None => {},
                  }
                },
                FingerprintDriverState::ENROLL1_WAIT | FingerprintDriverState::ENROLL2_WAIT => {

                  let mut result = None;

                  if let Ok(ref mut gt521fx_locked) = gt521fx.lock() {
                    println!("Checking finger");
                    result = Some(gt521fx_locked.send_command(Command::IsPressFinger, 0x00, None));
                  }

                  match result {
                    Some(Err(_err)) => {
                      println!("Erro checking fingerprint");
                      func(&FingerprintState::ERROR, None);
                      state_locked.set(FingerprintDriverState::IDLE);
                      (**expires_locked) = None;
                    },
                    Some(Ok(ref response)) => {
                      if response.response == Command::Ack.value(){
                        if response.parameter != 0x00 {
                          fingerpress_counter += 1;
                          //Sometimes the finger press sensor may fail.
                          //So, we need 3 consecutives finger leave reading to continue
                          if fingerpress_counter > 3 {
                            fingerpress_counter = 0;
                            if **state_locked == FingerprintDriverState::ENROLL1_WAIT {
                              state_locked.set(FingerprintDriverState::ENROLL2);
                              (**expires_locked) = Some(Instant::now());
                            } else if **state_locked == FingerprintDriverState::ENROLL2_WAIT {
                              state_locked.set(FingerprintDriverState::ENROLL3);
                              (**expires_locked) = Some(Instant::now());
                            }
                          }
                        } else {
                          fingerpress_counter = 0;
                          println!("========== RELEASE FINGER ==============");
                        }
                      }
                    },
                    None => {}
                  }
                },
                FingerprintDriverState::ENROLL_ERROR => {
                  state_locked.set(FingerprintDriverState::IDLE);
                  (**expires_locked) = Some(Instant::now());
                },
                FingerprintDriverState::IDLE => {
                  if let Ok(ref mut gt521fx_locked) = gt521fx.lock() {
                    println!("Checking finger touch pin");
                    if let Some(pin) = gt521fx_locked.pin {
                      let _ret = pin.with_exported(|| {
                        let _ret = pin.set_direction(Direction::In);
                        if let Ok(value) = pin.get_value() {
                          if value != 0 {
                            if let Err(err) = gt521fx_locked.send_command(Command::CmosLed, 0x1, None) {
                              println!("Error turning on fingerprint led: {}", err);
                            }
                            state_locked.set(FingerprintDriverState::READ);
                            (**expires_locked) = Some(Instant::now());
                          } else {
                            if let Err(err) = gt521fx_locked.send_command(Command::CmosLed, 0x0, None) {
                              println!("Error turning off fingerprint led: {}", err);
                            }
                          }
                        }
                        Ok(())
                      });
                    }
                  }
                },
                FingerprintDriverState::READ => {
                  if let Ok(ref mut gt521fx_locked) = gt521fx.lock() {
                    println!("Checking finger");
                    let result = gt521fx_locked.send_command(Command::IsPressFinger, 0x00, None);

                    match result {
                      Err(_err) => {
                        println!("Erro checking fingerprint");
                        func(&FingerprintState::ERROR, None);
                      },
                      Ok(ref response) => {
                        if response.response == Command::Ack.value(){
                          if response.parameter == 0x00 {

                            let result = gt521fx_locked.send_command(Command::CaptureFinger, 0x00, None);

                            match result {
                              Err(_err) => {
                                println!("Erro checking fingerprint");
                                func(&FingerprintState::ERROR, None);
                              },
                              Ok(ref response) => {
                                if response.response == Command::Ack.value() {
                                  println!("=========>Ok, I can see your finger<==========");
                                  let result = gt521fx_locked.send_command(Command::Identify, 0x00, None);

                                  match result {
                                    Err(_err) => {
                                      println!("Error identifying fingerprint!");
                                    },
                                    Ok(ref response) => {
                                      if response.response == Command::Ack.value() {
                                        println!("============>Fingerprint IS Registered<=============");
                                        func(&FingerprintState::AUTHORIZED, None);
                                        state_locked.set(FingerprintDriverState::IDLE);
                                        (**expires_locked) = None;
                                      } else {
                                        println!("============>Fingerprint is NOT Registered<=============");
                                        func(&FingerprintState::NOT_AUTHORIZED, None);
                                        state_locked.set(FingerprintDriverState::IDLE);
                                        (**expires_locked) = None;
                                      }
                                    },
                                  }
                                } else {
                                  println!("No finger!");
                                }
                              },
                            }
                          } else {
                            println!("No finger!");
                          }
                        }
                      },
                    }
                  }
                },
              }
            }
          }
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
      Ok(_response) => {
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

  fn start_enroll(&mut self, data: &FingerprintData) -> bool {
    println!("start enroll");
    let gt521fx = self.gt521fx.clone();
    let state_cloned = self.state.clone();
    let expires_cloned = self.expires.clone();

    if let Ok(mut gt521fx_locked) = gt521fx.lock() {

      if let Some(pos)  = data.pos {
        match gt521fx_locked.send_command(Command::EnrollStart, u32::from(pos), None){
          Ok(response) => {
            if response.response == Command::Ack.value() {

              if let Err(err) = gt521fx_locked.send_command(Command::CmosLed, 0x1, None) {
                println!("Error turning on fingerprint led: {}", err);
              } else {
                if let Ok(mut state_locked) = state_cloned.lock() {
                  (*state_locked).set(FingerprintDriverState::ENROLL1);
                }

                if let Ok(mut expires_locked) = expires_cloned.lock(){
                  (*expires_locked) = Some(Instant::now());
                }
              }

            } else {
              println!("EnrollStart error: {}", response.parameter);
              if let Ok(mut state_locked) = state_cloned.lock() {
               (*state_locked).set(FingerprintDriverState::ENROLL_ERROR);
              }
            }
          },
          Err(_err) => {
          }
        }
      } else {
        return false;
      }
    }

    return true;
  }
}

unsafe impl Send for Gt521fx {}
unsafe impl Sync for Gt521fx {}
