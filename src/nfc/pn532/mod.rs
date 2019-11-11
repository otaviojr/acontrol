use nfc::{NfcReader, MifareAuthKey, WriteSecMode};

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;

use std::thread;
use std::time::{Duration,Instant};

use std::io::prelude::*;
use spidev::{Spidev, SpidevOptions, SpidevTransfer, SPI_MODE_0};
use sysfs_gpio::{Direction, Pin};

#[derive(Clone, Copy)]
#[allow(dead_code)]
enum Register {
    BitFraming =  0x0d,
    Coll =        0x0e,
    ComIEn =      0x02,
    ComIrq =      0x04,
    Command =     0x01,
    CrcResultH =  0x21,
    CrcResultL =  0x22,
    Demod =       0x19,
    DivIrq =      0x05,
    Error =       0x06,
    FifoData =    0x09,
    FifoLevel =   0x0a,
    ModWidth =    0x24,
    Mode =        0x11,
    ReloadH =     0x2c,
    ReloadL =     0x2d,
    RxMode =      0x13,
    Status1 =     0x07,
    Status2 =     0x08,
    TCountValH =  0x2e,
    TCountValL =  0x2f,
    TMode =       0x2a,
    TPrescaler =  0x2b,
    TxAsk =       0x15,
    TxControl =   0x14,
    TxMode =      0x12,
    Version =     0x37,
}

const R: u8 = 1 << 7;
const W: u8 = 0 << 7;

impl Register {
    fn read(&self) -> u8 {
        ((*self as u8) << 1) | R
    }

    fn write(&self) -> u8 {
        ((*self as u8) << 1) | W
    }
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
enum SpiCommand {
    ReadStatus              = 0x02,
    ReadData                = 0x03,
    WriteData               = 0x01,
}

#[allow(dead_code)]
impl SpiCommand {
  fn name(&self) -> &'static str {
    match *self {
      SpiCommand::ReadStatus =>              "ReadStatus",
      SpiCommand::ReadData =>                "ReadData",
      SpiCommand::WriteData =>               "WriteData",
    }
  }

  fn value(&self) -> u8 {
    let value = *self as u8;
    value
  }
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
enum FrameDirection {
    FromHost              = 0xd4,
    FromRemote            = 0xd5,
}

#[allow(dead_code)]
impl FrameDirection {
  fn name(&self) -> &'static str {
    match *self {
      FrameDirection::FromHost =>              "FromHost",
      FrameDirection::FromRemote =>            "FromRemote",
    }
  }

  fn value(&self) -> u8 {
    let value = *self as u8;
    value
  }
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
enum FrameType {
    Normal              = 0x01,
    Extended            = 0x02,
    Ack                 = 0x03,
    NAck                = 0x04,
    Error               = 0x05,
    Unknown             = 0xff,
}

#[allow(dead_code)]
impl FrameType {
  fn name(&self) -> &'static str {
    match *self {
      FrameType::Normal =>              "Normal",
      FrameType::Extended =>            "Extended",
      FrameType::Ack =>                 "Ack",
      FrameType::NAck =>                "NAck",
      FrameType::Error =>               "Error",
      FrameType::Unknown =>             "Unknown",
    }
  }

  fn value(&self) -> u8 {
    let value = *self as u8;
    value
  }

  fn is_ack(&self) -> bool {
      match *self {
          FrameType::Ack => true,
          _ => false,
      }
  }

  fn is_error(&self) -> bool {
      match *self {
          FrameType::Error => true,
          _ => false,
      }
  }
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
enum Command {
    Diagnose	             = 0x00,
    GetFirmwareVersion	     = 0x02,
    GetGeneralStatus	     = 0x04,
    ReadRegister	         = 0x06,
    WriteRegister	         = 0x08,
    ReadGPIO	             = 0x0C,
    WriteGPIO	             = 0x0E,
    SetSerialBaudRate	     = 0x10,
    SetParameters	         = 0x12,
    SAMConfiguration	     = 0x14,
    PowerDown	             = 0x16,
    RFConfiguration	         = 0x32,
    RFRegulationTest	     = 0x58,
    InJumpForDEP	         = 0x56,
    InJumpForPSL	         = 0x46,
    InListPassiveTarget	     = 0x4A,
    InATR	                 = 0x50,
    InPSL	                 = 0x4E,
    InDataExchange	         = 0x40,
    InCommunicateThru	     = 0x42,
    InDeselect	             = 0x44,
    InRelease	             = 0x52,
    InSelect	             = 0x54,
    InAutoPoll	             = 0x60,
    TgInitAsTarget	         = 0x8C,
    TgSetGeneralBytes	     = 0x92,
    TgGetData	             = 0x86,
    TgSetData	             = 0x8E,
    TgSetMetaData	         = 0x94,
    TgGetInitiatorCommand	 = 0x88,
    TgResponseToInitiator	 = 0x90,
    TgGetTargetStatus	     = 0x8A,
}

#[allow(dead_code)]
impl Command {
  fn name(&self) -> &'static str {
    match *self {
      Command::Diagnose =>              "Diagnose",
      Command::GetFirmwareVersion =>    "GetFirmwareVersion",
      Command::GetGeneralStatus =>      "GetGeneralStatus",
      Command::ReadRegister =>          "ReadRegister",
      Command::WriteRegister =>         "WriteRegister",
      Command::ReadGPIO =>              "ReadGPIO",
      Command::WriteGPIO =>             "WriteGPIO",
      Command::SetSerialBaudRate =>     "SetSerialBaudRate",
      Command::SetParameters =>         "SetParameters",
      Command::SAMConfiguration =>      "SAMConfiguration",
      Command::PowerDown =>             "PowerDown",
      Command::RFConfiguration =>       "RFConfiguration",
      Command::RFRegulationTest =>      "RFRegulationTest",
      Command::InJumpForDEP =>          "InJumpForDEP",
      Command::InJumpForPSL =>          "InJumpForPSL",
      Command::InListPassiveTarget =>   "InListPassiveTarget",
      Command::InATR =>                 "InATR",
      Command::InPSL =>                 "InPSL",
      Command::InDataExchange =>        "InDataExchange",
      Command::InCommunicateThru =>     "InCommunicateThru",
      Command::InDeselect =>            "InDeselect",
      Command::InRelease =>             "InRelease",
      Command::InSelect =>              "InSelect",
      Command::InAutoPoll =>            "InAutoPoll",
      Command::TgInitAsTarget =>        "TgInitAsTarget",
      Command::TgSetGeneralBytes =>     "TgSetGeneralBytes",
      Command::TgGetData =>             "TgGetData",
      Command::TgSetData =>             "TgSetData",
      Command::TgSetMetaData =>         "TgSetMetaData",
      Command::TgGetInitiatorCommand => "TgGetInitiatorCommand",
      Command::TgResponseToInitiator => "TgResponseToInitiator",
      Command::TgGetTargetStatus =>     "TgGetTargetStatus",
    }
  }

  fn value(&self) -> u8 {
    let value = *self as u8;
    value
  }
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
enum Error {
  /// SPI Error
  SPI			        = 0x11,
}

#[allow(dead_code)]
impl Error {
  fn value(&mut self) -> u8 {
    let value = *self as u8;
    value
  }

  fn name(&mut self) -> &str {
    match *self {
      Error::SPI => "SPI",
    }
  }
}

struct Frame {
    buffer: Vec<u8>
}

impl Frame {
    fn from_vec(data: &Vec<u8>) -> Result<Frame,std::io::Error>{
        let len = data.len() as u8 + 1;

        if len > 0xfe {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid buffer length"));
        }

        let lcs = (0x100 - len as u16) as u8;

        let mut dcs = FrameDirection::FromHost.value();
        for b in data {
            dcs += b;
        }
        dcs = (0x100 - (dcs & 0xff) as u16) as u8;

        let mut b = vec![
            0x00, // preamble
            0x00, 0xff,  // start
        ];
        b.push(len);
        b.push(lcs);
        b.push(FrameDirection::FromHost.value()); // direction
        b.extend(data);
        b.push(dcs);
        b.push(0x00); // postamble

        Ok(Frame {buffer: b})
    }

    fn from_buffer(data: &[u8]) -> Result<Frame,std::io::Error>{
        Frame::from_vec(&data.to_vec())
    }

    fn frame_type(&self) -> FrameType {
        if self.buffer.len() < 5 {
            return FrameType::Unknown;
        }

        if self.buffer[3] == 0x00 && self.buffer[4] == 0xFF {
            return FrameType::Ack;
        } else if self.buffer[3] == 0xFF && self.buffer[4] == 0x00 {
            return FrameType::NAck;
        } else if self.buffer[3] == 0xFF && self.buffer[4] == 0xFF {
            return FrameType::Extended;
        } else if self.buffer[3] == 0x01 && self.buffer[4] == 0xFF {
            return FrameType::Error;
        }

        return FrameType::Normal;
    }

    fn data(&self) -> Result<Vec<u8>, std::io::Error> {
        match self.frame_type() {
            FrameType::Normal => Ok(self.buffer[6..self.buffer.len()-2].to_vec()),
            _ => Err(std::io::Error::new(std::io::ErrorKind::Other, "Frame has no data"))
        }
    }
}

struct Pn532ThreadSafe {
  spidev: Option<Spidev>,
  ss: Option<Pin>,
  mifare_key_a: Vec<u8>,
  mifare_key_b: Vec<u8>,
  mifare_access_bits: Vec<u8>
}

impl Pn532ThreadSafe {

  fn with_ss<F, T>(&mut self, f: F) -> T
  where
    F: FnOnce(&mut Self) -> T,
  {
    self.ss.unwrap().set_value(0).unwrap();
    let result = f(self);
    self.ss.unwrap().set_value(1).unwrap();

    result
  }

  fn read_frame(&mut self) -> Result<Frame, std::io::Error> {
    let mut rx_buf = [0u8; 256];
    let mut tx_buf = vec![ SpiCommand::ReadStatus.value() ];

    let status = self.with_ss(|ref mut pn| {
      if  let Some(ref mut spidev) = pn.spidev {
        {
          let mut transfer = SpidevTransfer::read_write(&tx_buf, &mut rx_buf);
          try!(spidev.transfer(&mut transfer));

          if rx_buf[0] > 0 {
              return Ok(());
          }
          return Err(std::io::Error::new(std::io::ErrorKind::Other, "No data"))
        }
      }
      Err(std::io::Error::new(std::io::ErrorKind::Other, "SPI dev not found"))
    });

    if let Err(error) = status {
        return Err(error);
    }

    self.with_ss(|ref mut pn| {
      if  let Some(ref mut spidev) = pn.spidev {
        {
            try!(spidev.write(&[SpiCommand::ReadData.value()]));
            try!(spidev.read(&mut rx_buf));
            return Ok(Frame { buffer: rx_buf.to_vec() });
        }
      }
      Err(std::io::Error::new(std::io::ErrorKind::Other, "SPI dev not found"))
    })
  }

  fn read_frame_timeout(&mut self, timeout: Duration) -> Result<Frame,std::io::Error> {
      let now = Instant::now();
      loop {
          if now.elapsed() > timeout {
              return Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout"));
          }

          if let Ok(ret) = self.read_frame() {
              return Ok(ret);
          }

          thread::sleep(Duration::from_millis(1));
      }
  }

  fn write_frame(&mut self, frame: Frame) -> Result<(), std::io::Error>{
    self.with_ss(|ref mut pn| {
      if let Some(ref mut spidev) = pn.spidev {
        try!(spidev.write(&[SpiCommand::WriteData.value()]));
        try!(spidev.write(&frame.buffer));
        return Ok(());
      }
      Err(std::io::Error::new(std::io::ErrorKind::Other, "SPI device not found."))
    })
  }

  fn command(&mut self, command: Command, data: Option<&[u8]>) -> Result<Frame, std::io::Error> {
    let mut buffer = vec![command as u8];
    if let Some(data) = data {
        buffer.extend_from_slice(data);
    }

    match Frame::from_vec(&buffer) {
        Ok(frame) => {
            try!(self.write_frame(frame));
            match self.read_frame_timeout(Duration::from_millis(1000)) {
                Ok(frame) => {
                    if frame.frame_type().is_ack() {
                        if let Ok(frame) = self.read_frame_timeout(Duration::from_millis(1000)) {
                            return Ok(frame);
                        } else {
                            return Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "TimedOut"));
                        }
                    } else {
                        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Ack not received"));
                    }
                },
                Err(err) => Err(err)
            }
        },
        Err(err) => Err(err)
    }
  }

  fn version(&mut self) -> Result<Vec<u8>, std::io::Error>{
    match self.command(Command::GetFirmwareVersion, Option::None) {
        Ok(frame) => {
            Ok(try!(frame.data()))
        },
        Err(err) => Err(err)
    }
  }

  fn flush_fifo(&mut self) -> Result<(),std::io::Error>{
    Err(std::io::Error::new(std::io::ErrorKind::Other, "oh no!"))
  }

  fn calc_crc(&mut self, data: &[u8]) -> Result<[u8;2], std::io::Error> {
    Err(std::io::Error::new(std::io::ErrorKind::Other, "oh no!"))
  }

  fn transceive<'a>(&mut self, tx_buffer: &[u8], bits: u8) -> Result<Vec<u8>, Error>{
    //self.send(Command::Transceive, tx_buffer, bits)
    Err(Error::SPI)
  }

  fn authent<'a>(&mut self, tx_buffer: &[u8], bits: u8) -> Result<Vec<u8>, Error>{
    //self.send(Command::MFAuthent, tx_buffer, bits)
    Err(Error::SPI)
  }

  fn send<'a>(&mut self, command: Command, tx_buffer: &[u8], bits: u8) -> Result<Vec<u8>, Error>{
    Err(Error::SPI)
  }

  fn initialize(&mut self) -> Result<(), std::io::Error> {
    Err(std::io::Error::new(std::io::ErrorKind::Other, "oh no!"))
  }

  fn reset(&mut self) -> Result<(), String> {
    Err(String::from("Not Implement"))
  }
}

unsafe impl Send for Pn532ThreadSafe {}
unsafe impl Sync for Pn532ThreadSafe {}

static MIFARE_DEFAULT_KEY_A:       &'static [u8] = &[0xff,0xff,0xff,0xff,0xff,0xff];
static MIFARE_DEFAULT_KEY_B:       &'static [u8] = &[0x00,0x00,0x00,0x00,0x00,0x00];
static MIFARE_DEFAULT_ACCESS_BITS: &'static [u8] = &[0xff,0x07,0x80,0x00];

pub struct Pn532 {
  pn532: Arc<Mutex<Pn532ThreadSafe>>
}

impl Pn532 {
  pub fn new() -> Self {
    return Pn532 {pn532: Arc::new(Mutex::new(Pn532ThreadSafe
      {
        spidev: None,
        ss: None,
        mifare_key_a: vec![0xff,0xff,0xff,0xff,0xff,0xff],
        mifare_key_b: vec![0x00,0x00,0x00,0x00,0x00,0x00],
        mifare_access_bits: vec![0xff,0x07,0x80,0x69]
      }
    ))};
  }
}

impl NfcReader for Pn532 {
  fn init(&mut self) -> Result<(), String> {
    let pn532 = self.pn532.clone();
    pn532.lock().unwrap().spidev = match Spidev::open("/dev/spidev0.0") {
      Ok(mut spidev) => {
        let options = SpidevOptions::new()
          .bits_per_word(8)
          .max_speed_hz(20_000)
          .mode(SPI_MODE_0)
          .build();

        if let Err(err) = spidev.configure(&options) {
          return Err(format!("{}: {}","Error  spi port",err));
        }

        Some(spidev)
      },
      Err(err) => return Err(format!("{} - {}", String::from("Error initializing spi port"), err)),
    };

    let pin = Pin::new(17);
    if let Err(err) = pin.export() {
      return Err(format!("{}: {}","Error initializing gpio port",err));
    }

    //for non root users, exporting a pin could have a delay to show up at sysfs
    thread::sleep(Duration::from_millis(100));
    pin.set_direction(Direction::Out).unwrap();

    pn532.lock().unwrap().ss = Some(pin);

    let mut pn532_init = false;

    for _i in 0..10 {
      thread::sleep(Duration::from_millis(50));
      if let Ok(version) = pn532.lock().unwrap().version() {
        println!("NFC hardware version: 0x{:?}", version);
        //if version == 0x91 || version == 0x92 {
          pn532_init = true;
        //  break;
        //} else {
        //  println!("{}(=>0x{:X})", "NFC Hardware with an invalid version", version);
        //}
      }
    }

    if !pn532_init{
      return Err(format!("{}", "NFC error. Could not retrieve hardware version"));
    }

    if let Err(_err) = pn532.lock().unwrap().initialize() {
      return Err(format!("{}", "NFC error. Error initializing device"));
    } else {
      println!("NFC device initialized successfully");
    }

    Ok(())
  }

  fn find_tag(&mut self, func: fn(Vec<u8>, Vec<u8>) -> bool) -> Result<(),String> {
    Err(String::from("Not Implement"))
  }

  fn set_auth_keys(&mut self, key_a: &Vec<u8>, key_b: &Vec<u8>) -> Result<(), String> {
    let pn532 = self.pn532.clone();
    let mut pn532_inner = pn532.lock().unwrap();

    pn532_inner.mifare_key_a = key_a.to_vec();
    pn532_inner.mifare_key_b = key_b.to_vec();

    Ok(())
  }

  fn set_auth_bits(&mut self, _access_bits: Vec<u8>) -> Result<(), String> {
    Err(String::from("Not Implement"))
  }

  fn format(&mut self, uuid: &Vec<u8>) -> Result<(), String> {
    Err(String::from("Not Implement"))
  }

  fn restore(&mut self, uuid: &Vec<u8>) -> Result<(), String> {
    Err(String::from("Not Implement"))
  }

  fn read_data(&mut self, uuid: &Vec<u8>, addr: u8, blocks: u8) -> Result<(Vec<u8>), String> {
    Err(String::from("Not Implement"))
  }

  fn write_data(&mut self, uuid: &Vec<u8>, addr: u8, data: &Vec<u8>) -> Result<(u8), String> {
    Err(String::from("Not Implement"))
  }

  fn unload(&mut self) -> Result<(), String>{
    Err(String::from("Not Implement"))
  }

  fn signature(&self) -> String {
    return String::from("PN532 NFC Reader Module");
  }
}

unsafe impl Send for Pn532 {}
unsafe impl Sync for Pn532 {}
