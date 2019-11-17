use nfc::{NfcReader, MifareAuthKey, WriteSecMode, CardType};

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;

use std::thread;
use std::time::{Duration,Instant};

use std::io::prelude::*;
use spidev::{Spidev, SpidevOptions, SpidevTransfer, SPI_MODE_0};
use sysfs_gpio::{Direction, Pin};

const BITREVERSETABLE256:[u8;256] = [0x00, 0x80, 0x40, 0xC0, 0x20, 0xA0, 0x60, 0xE0, 0x10, 0x90, 0x50, 0xD0, 0x30, 0xB0, 0x70, 0xF0,
                                     0x08, 0x88, 0x48, 0xC8, 0x28, 0xA8, 0x68, 0xE8, 0x18, 0x98, 0x58, 0xD8, 0x38, 0xB8, 0x78, 0xF8,
                                     0x04, 0x84, 0x44, 0xC4, 0x24, 0xA4, 0x64, 0xE4, 0x14, 0x94, 0x54, 0xD4, 0x34, 0xB4, 0x74, 0xF4,
                                     0x0C, 0x8C, 0x4C, 0xCC, 0x2C, 0xAC, 0x6C, 0xEC, 0x1C, 0x9C, 0x5C, 0xDC, 0x3C, 0xBC, 0x7C, 0xFC,
                                     0x02, 0x82, 0x42, 0xC2, 0x22, 0xA2, 0x62, 0xE2, 0x12, 0x92, 0x52, 0xD2, 0x32, 0xB2, 0x72, 0xF2,
                                     0x0A, 0x8A, 0x4A, 0xCA, 0x2A, 0xAA, 0x6A, 0xEA, 0x1A, 0x9A, 0x5A, 0xDA, 0x3A, 0xBA, 0x7A, 0xFA,
                                     0x06, 0x86, 0x46, 0xC6, 0x26, 0xA6, 0x66, 0xE6, 0x16, 0x96, 0x56, 0xD6, 0x36, 0xB6, 0x76, 0xF6,
                                     0x0E, 0x8E, 0x4E, 0xCE, 0x2E, 0xAE, 0x6E, 0xEE, 0x1E, 0x9E, 0x5E, 0xDE, 0x3E, 0xBE, 0x7E, 0xFE,
                                     0x01, 0x81, 0x41, 0xC1, 0x21, 0xA1, 0x61, 0xE1, 0x11, 0x91, 0x51, 0xD1, 0x31, 0xB1, 0x71, 0xF1,
                                     0x09, 0x89, 0x49, 0xC9, 0x29, 0xA9, 0x69, 0xE9, 0x19, 0x99, 0x59, 0xD9, 0x39, 0xB9, 0x79, 0xF9,
                                     0x05, 0x85, 0x45, 0xC5, 0x25, 0xA5, 0x65, 0xE5, 0x15, 0x95, 0x55, 0xD5, 0x35, 0xB5, 0x75, 0xF5,
                                     0x0D, 0x8D, 0x4D, 0xCD, 0x2D, 0xAD, 0x6D, 0xED, 0x1D, 0x9D, 0x5D, 0xDD, 0x3D, 0xBD, 0x7D, 0xFD,
                                     0x03, 0x83, 0x43, 0xC3, 0x23, 0xA3, 0x63, 0xE3, 0x13, 0x93, 0x53, 0xD3, 0x33, 0xB3, 0x73, 0xF3,
                                     0x0B, 0x8B, 0x4B, 0xCB, 0x2B, 0xAB, 0x6B, 0xEB, 0x1B, 0x9B, 0x5B, 0xDB, 0x3B, 0xBB, 0x7B, 0xFB,
                                     0x07, 0x87, 0x47, 0xC7, 0x27, 0xA7, 0x67, 0xE7, 0x17, 0x97, 0x57, 0xD7, 0x37, 0xB7, 0x77, 0xF7,
                                     0x0F, 0x8F, 0x4F, 0xCF, 0x2F, 0xAF, 0x6F, 0xEF, 0x1F, 0x9F, 0x5F, 0xDF, 0x3F, 0xBF, 0x7F, 0xFF];

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

#[derive(Clone, Copy)]
#[allow(dead_code)]
enum ResponseSize {
    Ack,
    Frame,
}

impl ResponseSize {

    fn name(&self) -> &'static str {
        match *self {
            ResponseSize::Ack =>      "Ack",
            ResponseSize::Frame =>    "Frame",
        }
    }

    fn size(&self, len: usize) -> usize {
        match *self {
            ResponseSize::Ack =>      0x07 + len,
            ResponseSize::Frame =>    0x08 + len,
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

        let lcs = !len + 1;

        let mut dcs = FrameDirection::FromHost as u8;
        for b in data {
            dcs = dcs.wrapping_add(*b);
        }

        dcs = !dcs + 1;

        let mut b = vec![
            0x00, // preamble
            0x00, 0xff,  // start
        ];
        b.push(len);
        b.push(lcs);
        b.push(FrameDirection::FromHost as u8); // direction
        b.extend(data);
        b.push(dcs);
        b.push(0x00); // postamble

        Ok(Frame {buffer: b})
    }

    fn from_buffer(data: &[u8]) -> Result<Frame,std::io::Error>{
        Frame::from_vec(&data.to_vec())
    }

    fn frame_type(&self) -> FrameType {

        //println!("Check type: {:X?}", &self.buffer);

        if self.buffer.len() < 5 {
            return FrameType::Unknown;
        }

        //SPI send the first byte as 0x01(DataWrite) before write
        //This is a SPI only feature
        if self.buffer[0] != 0x01 {
            return FrameType::Unknown;
        }

        //SPI - 4 and 5 because SPI adds a byte on start
        if self.buffer[4] == 0x00 && self.buffer[5] == 0xFF {
            return FrameType::Ack;
        } else if self.buffer[4] == 0xFF && self.buffer[5] == 0x00 {
            return FrameType::NAck;
        } else if self.buffer[4] == 0xFF && self.buffer[5] == 0xFF {
            return FrameType::Extended;
        } else if self.buffer[4] == 0x01 && self.buffer[5] == 0xFF {
            return FrameType::Error;
        }

        return FrameType::Normal;
    }

    fn data(&self) -> Result<Vec<u8>, std::io::Error> {
        //println!("Check for data: {:X?}", &self.buffer);
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
    thread::sleep(Duration::from_millis(10));
    let result = f(self);
    self.ss.unwrap().set_value(1).unwrap();

    result
  }

  fn wake_up(&mut self) -> Result<(),std::io::Error> {
      self.with_ss(|ref mut pn| {
          thread::sleep(Duration::from_millis(1000));
          Ok(())
      })
  }

  fn reverse_bits(&self, buffer: &mut[u8]) -> Result<bool,std::io::Error> {
      if buffer.len() == 0 {
          return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid buffer length"));
      }

      for x in 0..buffer.len() {
          buffer[x] = BITREVERSETABLE256[buffer[x] as usize] as u8;
      }

      Ok(true)
  }

  fn read_frame(&mut self, len: Option<usize>) -> Result<Frame, std::io::Error> {
    let mut tx_buf = [SpiCommand::ReadStatus as u8, 0];
    let mut rx_buf = [0 ; 2];

    try!(self.reverse_bits(&mut tx_buf));

    try!(self.with_ss(|ref mut pn| {

        if  let Some(ref mut spidev) = pn.spidev {
            {
                let mut transfer = SpidevTransfer::read_write(&tx_buf, &mut rx_buf);
                try!(spidev.transfer(&mut transfer));
            }
        } else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "SPI dev not found"));
        }

        try!(pn.reverse_bits(&mut rx_buf));

        if rx_buf[1] == 0 {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("No data: 0x{:X} 0x{:X}", rx_buf[0], rx_buf[1])));
        }

        Ok(())
    }));

    let mut l = 256;
    if let Some(len) = len {
        l = len;
    }

    let mut tx_buf = vec![SpiCommand::ReadData as u8; l];
    let mut rx_buf = vec![0 ; l];

    try!(self.reverse_bits(&mut tx_buf));

    try!(self.with_ss(|ref mut pn| {
        if  let Some(ref mut spidev) = pn.spidev {
            {
                let mut transfer = SpidevTransfer::read_write(&tx_buf, &mut rx_buf);
                try!(spidev.transfer(&mut transfer));
            }
        } else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "SPI dev not found"));
        }

        Ok(())
    }));

    try!(self.reverse_bits(&mut rx_buf));

    Ok(Frame { buffer: rx_buf.to_vec() })
  }

  fn read_frame_timeout(&mut self, len: Option<usize>, timeout: Duration) -> Result<Frame,std::io::Error> {
      let now = Instant::now();
      loop {
          if now.elapsed() > timeout {
              return Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "TimedOut"));
          }

          if let Ok(ret) = self.read_frame(len) {
              return Ok(ret);
          }

          thread::sleep(Duration::from_millis(1));
      }
  }

  fn write_frame(&mut self, frame: Frame) -> Result<(), std::io::Error> {

    let mut tx_buf = vec![SpiCommand::WriteData as u8];
    tx_buf.extend(&frame.buffer);

    try!(self.reverse_bits(&mut tx_buf));

    self.with_ss(|ref mut pn| {
        if let Some(ref mut spidev) = pn.spidev {
            try!(spidev.write(&tx_buf));
        } else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "SPI device not found."));
        }

        Ok(())
    })
  }

  fn command(&mut self, command: Command, data: Option<&[u8]>) -> Result<(), std::io::Error> {
    let mut buffer = vec![command as u8];
    if let Some(data) = data {
        buffer.extend_from_slice(data);
    }

    match Frame::from_vec(&buffer) {
        Ok(frame) => {
            try!(self.write_frame(frame));
            match self.read_frame_timeout(Some(ResponseSize::Ack.size(0)),Duration::from_millis(1000)) {
                Ok(frame) => {
                    if frame.frame_type().is_ack() {
                        return Ok(());
                    } else {
                        return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Not an ack frame: {:?}", &frame.buffer)));
                    }
                },
                Err(err) => Err(std::io::Error::new(err.kind(), format!("Ack frame error: {}", err)))
            }
        },
        Err(err) => Err(err)
    }
  }

  fn setup(&mut self) -> Result<Vec<u8>, std::io::Error>{
      try!(self.wake_up());
      match self.command(Command::SAMConfiguration, Some(&[0x01])) {
          Ok(_) => {
              if let Ok(frame) = self.read_frame_timeout(Some(ResponseSize::Frame.size(2)), Duration::from_millis(1000)) {
                  return Ok(try!(frame.data()));
              }

              return Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "TimedOut"));
          },
          Err(err) => Err(err)
      }
  }

  fn version(&mut self) -> Result<Vec<u8>, std::io::Error> {
    match self.command(Command::GetFirmwareVersion, Option::None) {
        Ok(_) => {
            if let Ok(frame) = self.read_frame_timeout(Some(ResponseSize::Frame.size(6)), Duration::from_millis(1000)) {
                return Ok(try!(frame.data())[3..5].to_vec());
            }

            return Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "TimedOut"));
        },
        Err(err) => Err(err)
    }
  }

  fn read_passive_target(&mut self, card_type: CardType) -> Result<Vec<u8>, std::io::Error> {

      let freq:u8 = match card_type {
          CardType::Mifare => 0x00,
          CardType::FelicaA => 0x01,
          CardType::FelicaB => 0x02,
          CardType::Jewel => 0x04,
      };

      match self.command(Command::InListPassiveTarget, Some(&[0x01, freq])) {
          Ok(_) => {
              if let Ok(frame) = self.read_frame_timeout(Some(ResponseSize::Frame.size(17)), Duration::from_millis(1000)) {
                  return Ok(try!(frame.data()));
              }

              return Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "TimedOut"));
          },
          Err(err) => Err(std::io::Error::new(err.kind(), format!("Command Error: {}",err)))
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
    Ok(())
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

pub struct Pn532Spi {
  pn532: Arc<Mutex<Pn532ThreadSafe>>
}

impl Pn532Spi {
  pub fn new() -> Self {
    return Pn532Spi {pn532: Arc::new(Mutex::new(Pn532ThreadSafe
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

impl NfcReader for Pn532Spi {
  fn init(&mut self) -> Result<(), String> {
    let pn532 = self.pn532.clone();
    let spidev = match Spidev::open("/dev/spidev0.0") {
      Ok(mut spidev) => {
        let options = SpidevOptions::new()
          .bits_per_word(8)
          .max_speed_hz(500_000)
          .mode(SPI_MODE_0)
          .build();

        if let Err(err) = spidev.configure(&options) {
          return Err(format!("{}: {}","Error spi port",err));
        }

        Ok(spidev)
      },
      Err(err) => Err(format!("{} - {}", String::from("Error initializing spi port"), err)),
    };

    match spidev {
        Ok(spidev) => {
            pn532.lock().unwrap().spidev = Some(spidev);
        },
        Err(err) => {
            pn532.lock().unwrap().spidev = None;
            return Err(err);
        }
    }

    let pin = Pin::new(17);
    if let Err(err) = pin.export() {
      return Err(format!("{}: {}","Error initializing gpio port",err));
    }

    //for non root users, exporting a pin could have a delay to show up at sysfs
    thread::sleep(Duration::from_millis(100));
    pin.set_direction(Direction::Out).unwrap();

    pn532.lock().unwrap().ss = Some(pin);

    let mut pn532_init = false;

    match pn532.lock().unwrap().setup() {
        Ok(_) => {
            println!("NFC hardware initialized");
        },
        Err(err) => println!("NFC hardware setup error: {}", err)
    };

    for _i in 0..10 {
      thread::sleep(Duration::from_millis(50));
      match pn532.lock().unwrap().version() {
          Ok(version) => {
              println!("NFC hardware version: {}.{}", version[0], version[1]);
              pn532_init = true;
              break;
          },
          Err(err) => println!("NFC hardware version error: {}", err)
      };
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

  fn find_tag(&mut self, func: fn(CardType, Vec<u8>) -> bool) -> Result<(),String> {
    let pn532 = self.pn532.clone();

    let _handler = thread::spawn(move || {
        loop {
            let ret: Result<(), String>;
            let mut uuid:Vec<u8> = Vec::new();

            {
                let mut pn532_inner = pn532.lock().unwrap();

                match pn532_inner.read_passive_target(CardType::Mifare) {
                    Ok(uuid) => {
                        func(CardType::Mifare, uuid);
                    },
                    Err(err) => println!("No Card Found: {}", err),
                };

                thread::sleep(Duration::from_millis(500));
            }
        }
    });
    Ok(())
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

unsafe impl Send for Pn532Spi {}
unsafe impl Sync for Pn532Spi {}
