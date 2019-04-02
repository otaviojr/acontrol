use nfc::{NfcReader, MiFare, PICC, MifareAuthKey, WriteSecMode};

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
    BitFraming = 0x0d,
    Coll = 0x0e,
    ComIEn = 0x02,
    ComIrq = 0x04,
    Command = 0x01,
    CrcResultH = 0x21,
    CrcResultL = 0x22,
    Demod = 0x19,
    DivIrq = 0x05,
    Error = 0x06,
    FifoData = 0x09,
    FifoLevel = 0x0a,
    ModWidth = 0x24,
    Mode = 0x11,
    ReloadH = 0x2c,
    ReloadL = 0x2d,
    RxMode = 0x13,
    Status1 = 0x07,
    Status2 = 0x08,
    TCountValH = 0x2e,
    TCountValL = 0x2f,
    TMode = 0x2a,
    TPrescaler = 0x2b,
    TxAsk = 0x15,
    TxControl = 0x14,
    TxMode = 0x12,
    Version = 0x37,
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
enum Command {
  Idle			        = 0b0000,
  Mem			          = 0b0001,
  GenerateRandomID	= 0b0010,
  CalcCRC		        = 0b0011,
  Transmit		      = 0b0100,
  NoCmdChange		    = 0b0111,
  Receive		        = 0b1000,
  Transceive		    = 0b1100,
  MFAuthent		      = 0b1110,
  SoftReset		      = 0b1111
}

#[allow(dead_code)]
impl Command {
  fn name(&self) -> &'static str {
    match *self {
      Command::Idle =>              "Idle",
      Command::Mem =>               "Mem",
      Command::GenerateRandomID =>  "GenerateRandomID",
      Command::CalcCRC =>           "CalcCRC",
      Command::Transmit =>          "Transmit",
      Command::NoCmdChange =>       "NoCmdChange",
      Command::Receive =>           "Receive",
      Command::Transceive =>        "Transceive",
      Command::MFAuthent =>         "MFAuthent",
      Command::SoftReset =>         "SoftReset"
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
  /// FIFO buffer overflow
  BufferOverflow	= 0x01,
  /// Collision
  Collision		= 0x02,
  /// Wrong CRC
  Crc			= 0x03,
  /// Incomplete RX frame
  IncompleteFrame	= 0x04,
  /// Internal temperature sensor detects overheating
  Overheating		= 0x05,
  /// Parity check failed
  Parity		= 0x06,
  /// Error during MFAuthent operation
  Protocol		= 0x07,
  /// Write Error
  Wr			= 0x08,
  /// Timeout Error
  Timeout		= 0x09,
  //// No Memory
  NoMem			= 0x10,
  /// SPI Error
  SPI			= 0x11,
}

#[allow(dead_code)]
impl Error {
  fn value(&mut self) -> u8 {
    let value = *self as u8;
    value
  }

  fn name(&mut self) -> &str {
    match *self {
      Error::BufferOverflow => "BufferOverflow",
      Error::Collision => "Collision",
      Error::Crc => "CRC",
      Error::IncompleteFrame => "IncompleteFrame",
      Error::Overheating => "Overheating",
      Error::Parity => "Parity",
      Error::Protocol => "Protocol",
      Error::Wr => "WR",
      Error::Timeout => "Timeout",
      Error::NoMem => "NoMem",
      Error::SPI => "SPI",
    }
  }
}

#[allow(dead_code)]
struct Mfrc522ThreadSafe {
  spidev: Option<Spidev>,
  ss: Option<Pin>,
  mifare_key_a: Vec<u8>,
  mifare_key_b: Vec<u8>,
  mifare_access_bits: Vec<u8>
}

impl Mfrc522ThreadSafe {
  fn with_ss<F, T>(&mut self, f: F) -> T
  where
    F: FnOnce(&mut Self) -> T,
  {
    self.ss.unwrap().set_value(0).unwrap();
    let result = f(self);
    self.ss.unwrap().set_value(1).unwrap();

    result
  }

  fn read(&mut self, reg: Register) -> Result<u8, std::io::Error> {
    let tx_buf = [reg.read(),0];
    let mut rx_buf = [0 ; 2];
    self.with_ss(|ref mut mfrc| {
      if let Some(ref mut spidev) = mfrc.spidev {
        {
      	  let mut transfer = SpidevTransfer::read_write(&tx_buf, &mut rx_buf);
          try!(spidev.transfer(&mut transfer));
        }
        return Ok(rx_buf[1]);
      }
      Err(std::io::Error::new(std::io::ErrorKind::Other, "oh no!"))
    })
  }

  fn read_many<'a>(&mut self, reg: Register, buffer: &'a mut [u8]) -> Result<&'a mut [u8], std::io::Error> {
    let mut rx_buf = vec![0 ; buffer.len()+1];
    let tx_buf = vec![ reg.read() ; buffer.len()+1];

    self.with_ss(|ref mut mfrc| {
      if  let Some(ref mut spidev) = mfrc.spidev {
        {
	  let mut transfer = SpidevTransfer::read_write(&tx_buf, &mut rx_buf);
          try!(spidev.transfer(&mut transfer));
        }

        for i in 1..rx_buf.len() {
          buffer[i-1] = rx_buf[i];
        }

        return Ok(buffer);
      }
      Err(std::io::Error::new(std::io::ErrorKind::Other, "oh no!"))
    })
  }

  fn write(&mut self, reg: Register, val: u8) -> Result<(), std::io::Error>{
    self.with_ss(|ref mut mfrc| {
      if let Some(ref mut spidev) = mfrc.spidev {
        try!(spidev.write(&[reg.write(), val]));
      }
      return Ok(())
    })
  }

  fn write_many(&mut self, reg: Register, buffer: &[u8]) -> Result<(), std::io::Error>{
    self.with_ss(|ref mut mfrc| {
      if let Some(ref mut spidev) = mfrc.spidev {
        try!(spidev.write(&[reg.write()]));
        try!(spidev.write(buffer));
        return Ok(());
      }
      Err(std::io::Error::new(std::io::ErrorKind::Other, "oh no!"))
    })
  }

  fn command(&mut self, command: Command) -> Result<(), std::io::Error> {
    self.write(Register::Command, command.value())
  }

  fn set_bit_mask(&mut self, reg: Register, mask: u8) -> Result<(),std::io::Error> {
    match self.read(reg) {
      Ok(val) => {
        let new_val = val | mask;
        return self.write(reg, new_val);
      },
      Err(err) => return Err(err)
    }
  }

  fn clear_bit_mask(&mut self, reg: Register, mask: u8) -> Result<(), std::io::Error> {
    match self.read(reg) {
      Ok(val) => {
        let new_val = val & !mask;
        return self.write(reg, new_val);
      },
      Err(err) => return Err(err)
    }
  }

  fn version(&mut self) -> Result<u8, std::io::Error>{
    self.read(Register::Version)
  }

  fn flush_fifo(&mut self) -> Result<(),std::io::Error>{
    try!(self.set_bit_mask(Register::FifoLevel, 0x80));
    Ok(())
  }

  fn calc_crc(&mut self, data: &[u8]) -> Result<[u8;2], std::io::Error> {
    //stop the ongoing command
    try!(self.command(Command::Idle));
    //clear CRC_IRQ flag
    try!(self.clear_bit_mask(Register::DivIrq, 1<<2));
    //clear fifo buffer
    try!(self.flush_fifo());
    //write our data to fifo buffer
    try!(self.write_many(Register::FifoData, data));
    //calc crc command
    try!(self.command(Command::CalcCRC));
    
    let now = Instant::now();
    loop {
      let sec = (now.elapsed().as_secs() as f64) + (now.elapsed().subsec_nanos() as f64 / 1000_000_000.0);

      if sec > 5.0 {
        break Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "CRC Timeout"));
      }

      if let Ok(irq) = self.read(Register::DivIrq) {
        //check if crc calculation cames to an end
        if irq & (1<<2) != 0 {
          //stop execution
          try!(self.command(Command::Idle));
          //read crc
          let crc = [try!(self.read(Register::CrcResultL)),
                     try!(self.read(Register::CrcResultH))];
          break Ok(crc);
        }
      }
    }
  }

  fn check_error(&mut self) -> Result<(), Error> {

    let err = match self.read(Register::Error) {
      Err(err) => return Err(Error::SPI),
      Ok(err) => err
    };

    if err & (1 << 0) != 0 {
      Err(Error::Protocol)
    } else if err & (1 << 1) != 0 {
      Err(Error::Parity)
    } else if err & (1 << 2) != 0 {
      Err(Error::Crc)
    } else if err & (1 << 3) != 0 {
      Err(Error::Collision)
    } else if err & (1 << 4) != 0 {
      Err(Error::BufferOverflow)
    } else if err & (1 << 6) != 0 {
      Err(Error::Overheating)
    } else if err & (1 << 7) != 0 {
      Err(Error::Wr)
    } else {
      Ok(())
    }
  }

  fn transceive<'a>(&mut self, tx_buffer: &[u8], bits: u8) -> Result<Vec<u8>,Error>{
    self.send(Command::Transceive, tx_buffer, bits)
  }

  fn authent<'a>(&mut self, tx_buffer: &[u8], bits: u8) -> Result<Vec<u8>,Error>{
    self.send(Command::MFAuthent, tx_buffer, bits)
  }

  fn send<'a>(&mut self, command: Command, tx_buffer: &[u8], bits: u8) -> Result<Vec<u8>,Error>{

    let irq_wait = if command.value() == Command::Transceive.value() { 0x30 } else { 0x10 };

    // stop any ongoing command
    if let Err(_err) = self.command(Command::Idle) { return Err(Error::SPI); }
    // set interruption
    if let Err(_err) = self.write(Register::ComIEn, 0x77 | 0x80) { return Err(Error::SPI); }
    // clear all interrupt flags
    if let Err(_err) = self.write(Register::ComIrq, 0x7f) { return Err(Error::SPI); }
    // cler the fifo buffer
    if let Err(_err) = self.flush_fifo()  { return Err(Error::SPI); }
    // write data to transmit to the fifo buffer
    if let Err(_err) = self.write_many(Register::FifoData, tx_buffer) { return Err(Error::SPI); }
    // send tranceive or MFauth command
    if let Err(_err) = self.command(command) { return Err(Error::SPI); }

    if command.value() == Command::Transceive.value() {
      // configure bit framing
      if let Err(_err) = self.write(Register::BitFraming, (1 << 7) | bits) { return Err(Error::SPI); }
    }

    let mut irq;
    let now = Instant::now();
    loop {
      let sec = (now.elapsed().as_secs() as f64) + (now.elapsed().subsec_nanos() as f64 / 1000_000_000.0);

      if sec > 1.0 {
        return  Err(Error::Timeout);
      }

      irq = 0;
      if let Err(_err) = self.read(Register::ComIrq).map(|val| irq = val) {
        return Err(Error::SPI);
      }

      if irq & irq_wait != 0 {
        break;
      } else if irq & 0x01 != 0{
        return Err(Error::Timeout);
      }
    }

    //check for errors
    self.check_error()?;

    let received:usize;
    match self.read(Register::FifoLevel) {
      Ok(val) => received = val as usize,
      Err(_err) => return Err(Error::SPI)
    }

    //println!("Tranceive received {} bytes", received);

    let mut rx_buffer:Vec<u8> = vec![0 ; received];
    if let Err(_err) = self.read_many(Register::FifoData, &mut rx_buffer) {
      return Err(Error::SPI);
    }

    Ok(rx_buffer)
  }

  fn initialize(&mut self) -> Result<(), std::io::Error> {
    // soft reset
    try!(self.command(Command::SoftReset));
    
    // check the power down flag and wait until reset finish
    let now = Instant::now();
    loop {
      let sec = (now.elapsed().as_secs() as f64) + (now.elapsed().subsec_nanos() as f64 / 1000_000_000.0);
      if sec > 1.0 {
        return  Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "Reset Timeout"));
      }
      if try!(self.read(Register::Command)) & (1 << 4) == 0 {
        break;
      }
    }

    // timer at 10Khz
    try!(self.write(Register::Demod, 0x4d | (1 << 4) ));
    try!(self.write(Register::TMode, 0x0 | (1 << 7) | 0b10 ));
    try!(self.write(Register::TPrescaler, 165));
    
    // 5ms timeout
    try!(self.write(Register::ReloadL, 50));

    // 100% ASK modulation
    try!(self.write(Register::TxAsk, 1 << 6 ));

    // CRC preset value to 0x6363
    try!(self.write(Register::Mode, 0x3F & (!0b11) | 0b01));

    // enable antenna
    try!(self.write(Register::TxControl, 0xB0 | 0b11));

    Ok(())
  }

  fn reset(&mut self) -> Result<(), String> {
    if let Err(_err) = self.initialize() {
      return Err(format!("{}", "NFC error. Error reseting device"));
    }
    Ok(())
  }

}

unsafe impl Send for Mfrc522ThreadSafe {}
unsafe impl Sync for Mfrc522ThreadSafe {}

static MIFARE_DEFAULT_KEY_A:       &'static [u8] = &[0xff,0xff,0xff,0xff,0xff,0xff];
static MIFARE_DEFAULT_KEY_B:       &'static [u8] = &[0x00,0x00,0x00,0x00,0x00,0x00];
static MIFARE_DEFAULT_ACCESS_BITS: &'static [u8] = &[0xff,0x07,0x80,0x00];

impl MiFare for Mfrc522ThreadSafe {
  fn send_req_a(&mut self) -> Result<Vec<u8>, String> {
    match self.transceive(&[PICC::REQIDL.value()],7) {
      Ok(val) => Ok(val),
      Err(ref mut err) => Err(format!("{} => {}","NFC MiFare REQA error", err.name()))
    }
  }

  fn select(&mut self, cascade: u8, uuid: &Vec<u8>) -> Result<Vec<u8>, String> {
    let mut tx_buf = vec![cascade,0x70];
    let mut serial: u8 = 0;
    
    for i in uuid {
      tx_buf.push(*i);
      serial ^= *i;
    }

    tx_buf.push(serial);

    if let Err(_err) = self.calc_crc(&tx_buf).map(|val| {
      tx_buf.push(val[0]);
      tx_buf.push(val[1]);
    }) {
      return Err(format!("NFC MiFare crc calc error when selecting card"));
    }


    if let Err(err) = self.clear_bit_mask(Register::Status2, 0x08) {
      return Err(format!("select error: {}", err));
    }

    match self.transceive(&tx_buf, 0) {
      Ok(val) => {
        Ok(val)
      },
      Err(ref mut err) => Err(format!("{} => {}","NFC MiFare SELECT error", err.name()))
    }
  }

  fn anticoll(&mut self, cascade: u8, uuid: &Vec<u8>) -> Result<Vec<u8>, String> {
    let mut tx_buf = vec![cascade,0x20];
    let mut serial: u8 = 0;

    for i in uuid {
      tx_buf.push(*i);
      serial ^= *i;
    }
    if uuid.len() > 0 {
      tx_buf.push(serial);
    }

    match self.transceive(&tx_buf, 0) {
      Ok(val) => {
        Ok(val)
      },
      Err(ref mut err) => Err(format!("{} => {}","NFC MiFare anti collision loop error", err.name()))
    }
  }

  fn auth(&mut self, auth_mode: u8, addr: u8, uuid: &Vec<u8>, key: MifareAuthKey) -> Result<(), String> {
    let mut tx_buf = vec![auth_mode, addr];
    match key {
      MifareAuthKey::DefaultKeyA => tx_buf.extend(MIFARE_DEFAULT_KEY_A),
      MifareAuthKey::DefaultKeyB => tx_buf.extend(MIFARE_DEFAULT_KEY_B),
      MifareAuthKey::CustomKeyA => tx_buf.extend(&self.mifare_key_a),
      MifareAuthKey::CustomKeyB => tx_buf.extend(&self.mifare_key_b)
    }
    tx_buf.extend(uuid);

    match self.authent(&tx_buf, 0) {
      Ok(val) => Ok(()),
      Err(ref mut err) => Err(format!("Error authenticating ({})",err.name()))
    }
  }

  fn write_data(&mut self, addr: u8, data: &Vec<u8>) -> Result<(), String> {
    let mut tx_buf = vec![PICC::WRITE.value(), addr];

    if data.len() < 16 {
      return Err(format!("{}","write_data error: Invalid stream size"));
    }

    if let Err(_err) = self.calc_crc(&tx_buf).map(|crc| {
      tx_buf.push(crc[0]);
      tx_buf.push(crc[1]);
    }) {
      return Err(format!("NFC MiFare crc calc error  at address {}", addr));
    }

    match self.transceive(&tx_buf, 0) {
      Ok(val) => {
        let mut buf: Vec<u8> = data[..16].to_vec();

        if let Err(_err) = self.calc_crc(&buf).map(|crc| {
          buf.push(crc[0]);
          buf.push(crc[1]);
        }) {
          return Err(format!("NFC MiFare crc calc error  at address {}", addr));
        }


        match self.transceive(&buf, 0) {
          Ok(val) => {
            Ok(())
          },
          Err(ref mut err) => Err(format!("{} {} => {}","NFC MiFare error reading address {}", addr, err.name()))
        }
      },
      Err(ref mut err) => Err(format!("{} {} => {}","NFC MiFare error reading address {}", addr, err.name()))
    }
  }

  fn read_data(&mut self, addr: u8) -> Result<(Vec<u8>), String> {
    let mut tx_buf = vec![PICC::READ.value(), addr];

    if let Err(_err) = self.calc_crc(&tx_buf).map(|crc| {
      tx_buf.push(crc[0]);
      tx_buf.push(crc[1]);
    }) {
      return Err(format!("NFC MiFare crc calc error  at address {}", addr));
    }
    
    match self.transceive(&tx_buf, 0) {
      Ok(val) => {
        let buf: Vec<u8> = val[..16].to_vec();

        if let Err(_err) = self.calc_crc(&buf).map(|crc| {
          if crc[0] != val[16] || crc[1] != val[17] {
            return Err(format!("NFC MiFare read crc error reading address {}", addr));
          }
          Ok(())
        }) {
          return Err(format!("NFC MiFare crc calc error  at address {}", addr));
        }

        Ok(buf)
      },
      Err(ref mut err) => Err(format!("{} {} => {}","NFC MiFare error reading address {}", addr, err.name()))    
    }
  }

  fn write_sec(&mut self, uuid: &Vec<u8>, mode: WriteSecMode) -> Result<(), String> {
    println!("{}","write_sec: reached");

    let mut addr:u8 = 3;
    let mut packet:Vec<u8> = Vec::new();
    let key:MifareAuthKey;

    match mode {
      WriteSecMode::Format => {
        packet.extend(&self.mifare_key_a);
        packet.extend(MIFARE_DEFAULT_ACCESS_BITS);
        packet.extend(&self.mifare_key_b);
        key = MifareAuthKey::DefaultKeyA;
      },
      WriteSecMode::Restore => {
        packet.extend(MIFARE_DEFAULT_KEY_A);
        packet.extend(MIFARE_DEFAULT_ACCESS_BITS);
        packet.extend(MIFARE_DEFAULT_KEY_B);
        key = MifareAuthKey::CustomKeyA;
      }
    }

    loop {
      match self.auth(PICC::AUTH1A.value(), addr, uuid, key) {
        Ok(val) => {
          if let Err(err) = self.write_data(addr, &packet) {
            return Err(err);
          }

          if addr < 62 { addr+=4; } else { break; }
        },
        Err(err) => return Err(err)
      }
    }

    Ok(())
  }


}

pub struct Mfrc522 {
  mfrc522: Arc<Mutex<Mfrc522ThreadSafe>>
}

impl Mfrc522 {
  pub fn new() -> Self {
    return Mfrc522 {mfrc522: Arc::new(Mutex::new(Mfrc522ThreadSafe 
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

impl NfcReader for Mfrc522 {
  fn init(&mut self) -> Result<(), String> {
    let mfrc522 = self.mfrc522.clone();
    mfrc522.lock().unwrap().spidev = match Spidev::open("/dev/spidev0.0") {
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

    mfrc522.lock().unwrap().ss = Some(pin);

    let mut mfrc522_init = false;

    for i in 0..10 {
      thread::sleep(Duration::from_millis(50));
      if let Ok(version) = mfrc522.lock().unwrap().version() {
        println!("NFC hardware version: 0x{:X}", version);
        if version == 0x91 || version == 0x92 {
          mfrc522_init = true;
          break;          
        } else {
          println!("{}(=>0x{:X})", "NFC Hardware with an invalid version", version);
        }
      }
    }

    if !mfrc522_init{
      return Err(format!("{}", "NFC error. Could not retrieve hardware version"));
    }

    if let Err(_err) = mfrc522.lock().unwrap().initialize() {
      return Err(format!("{}", "NFC error. Error initializing device"));
    } else {
      println!("NFC device initialized successfully");
    }

    Ok(())
  }

  fn find_tag(&mut self, func: fn(Vec<u8>, Vec<u8>) -> bool) -> Result<(),String> {
    let mfrc522 = self.mfrc522.clone();
    
    let _handler = thread::spawn(move || {
      loop {
        let ret: Result<(), String>;
        let mut uuid:Vec<u8> = Vec::new();
        let mut sak:Vec<u8> = Vec::new();
        let mut complete:bool;

        {
          let mut mfrc522_inner = mfrc522.lock().unwrap();

          if let Err(err) = mfrc522_inner.reset() {
            println!("Error reseting reader");
            break;
          }

          println!("Searching tag...");
          ret = match mfrc522_inner.send_req_a() {
            Ok(val) => {
              println!("Card Answer {:?}", val);
              let ret: Result<(), String> = match mfrc522_inner.anticoll(PICC::ANTICOLL1.value(), &Vec::new()){
                Ok(val) => {
                  println!("ANTICOLL CASCADE 1 value: {:?}", val);
                  if val[0] == 0x88 { complete = false; uuid.extend_from_slice(&val[1..val.len()-1]); } else { complete = true; uuid.extend_from_slice(&val[..val.len()-1]); }
                  let ret:Result<(), String> = match mfrc522_inner.select(PICC::ANTICOLL1.value(),&uuid) {
                    Ok(val) => {
                      let ret1: Result<(), String>;
                      println!("SELECT CASCADE 1 answer: {:?}",val);
                      if complete == false {
                        ret1 = match mfrc522_inner.anticoll(PICC::ANTICOLL2.value(), &uuid){
                          Ok(val) => {
                            println!("ANTICOLL CASCADE 2 value: {:?}", val);
                            if val[0] == 0x88 { complete = false; uuid.extend_from_slice(&val[1..val.len()-1]); } else { complete = true; uuid.extend_from_slice(&val[..val.len()-1]); }                          
                            let ret: Result<(), String> = match mfrc522_inner.select(PICC::ANTICOLL2.value(),&uuid) {
                              Ok(val) => {
                                let ret2:Result<(),String>;
                                println!("SELECT CASCADE 2 answer: {:?}",val);
                                if complete == false {
                                  ret2 = Err(format!("{}","Tripple byte card not implemented yet!"));
                                } else {
                                  sak = val;
                                  ret2 = Ok(());
                                }
                                ret2
                              },
                              Err(ref mut err) => Err(format!("SELECT CASCADE 2 => {}", err))
                            };
                            ret
                          },
                          Err(ref mut err) => Err(format!("ANTICOLL 2 => {}", err))
                        };
                      } else {
                        sak = val;
                        ret1 = Ok(());
                      }
                      ret1
                    },
                    Err(ref mut err) => Err(format!("SELECT CASCADE 1 => {}", err))
                  };
                  ret
                },
                Err(ref mut err) => Err(format!("ANTICOLL 1 => {}", err))
              };
              ret
            },
            Err(ref mut err) => Err(format!("REQA => {}", err))
          }
        };

        if let Ok(val) = ret {
          func(uuid, sak);
        }

        if let Err(val) = ret {
          eprintln!("{}", val);
        }

        //buzzer.lock().unwrap().set_buzz(true);
        thread::sleep(Duration::from_millis(500));
        //buzzer.lock().unwrap().set_buzz(false);
      }
    });
    Ok(())
  }

  fn set_auth_keys(&mut self, key_a: &Vec<u8>, key_b: &Vec<u8>) -> Result<(), String> {
    let mfrc522 = self.mfrc522.clone();
    let mut mfrc522_inner = mfrc522.lock().unwrap();

    mfrc522_inner.mifare_key_a = key_a.to_vec();
    mfrc522_inner.mifare_key_b = key_b.to_vec();

    Ok(())
  }

  fn set_auth_bits(&mut self, _access_bits: Vec<u8>) -> Result<(), String> {
    Ok(())
  }

  fn format(&mut self, uuid: &Vec<u8>) -> Result<(), String> {
    let mfrc522 = self.mfrc522.clone();
    let mut mfrc522_inner = mfrc522.lock().unwrap();

    println!("{}","format: reached");

    mfrc522_inner.write_sec(uuid, WriteSecMode::Format)
  }

  fn restore(&mut self, uuid: &Vec<u8>) -> Result<(), String> {
    let mfrc522 = self.mfrc522.clone();
    let mut mfrc522_inner = mfrc522.lock().unwrap();

    println!("{}","format: reached");

    mfrc522_inner.write_sec(uuid, WriteSecMode::Restore)
  }

  fn read_data(&mut self, uuid: &Vec<u8>, addr: u8, blocks: u8) -> Result<(Vec<u8>), String> {
    let mfrc522 = self.mfrc522.clone();
    let mut mfrc522_inner = mfrc522.lock().unwrap();
    
    println!("{}","read_data: reached");

    let mut cur_addr:u8 = addr;
    let mut buffer: Vec<u8> = Vec::new();

    loop {

      if cur_addr+1 % 4 == 0 { cur_addr += 1; }

      match mfrc522_inner.auth(PICC::AUTH1A.value(), cur_addr, uuid, MifareAuthKey::CustomKeyA) {
        Ok(val) => {
          //println!("Successfuly authenticated on block {}", addr);

          match mfrc522_inner.read_data(cur_addr) {
            Ok(val) => {
              //println!("Addr {} => {:?}", addr, val);
              buffer.extend(val);
            },
            Err(err) => return Err(err),
          }

          if cur_addr < addr+blocks { cur_addr+=1; } else { break; }
        },
        Err(err) => return Err(err)
      }
    }

    Ok(buffer)
  }

  fn write_data(&mut self, uuid: &Vec<u8>, addr: u8, data: &Vec<u8>) -> Result<(u8), String> {
    let mfrc522 = self.mfrc522.clone();
    let mut mfrc522_inner = mfrc522.lock().unwrap();

    println!("{}","write_data: reached");

    let mut cur_addr:u8 = addr;
    let mut buffer:VecDeque<u8> = VecDeque::new();
    let mut packet:Vec<u8> = Vec::new();

    buffer.extend(data);

    loop {

      if cur_addr+1 % 4 == 0 { cur_addr += 1; }

      match mfrc522_inner.auth(PICC::AUTH1A.value(), cur_addr, uuid, MifareAuthKey::CustomKeyA) {
        Ok(val) => {
          
          packet.clear();

          if buffer.len() == 0 { break; }

          loop {
              match buffer.pop_front(){
                Some(val) => packet.push(val),
                None => packet.push(0),
              }
              if packet.len() >= 16 { break  };
          }

          if let Err(err) = mfrc522_inner.write_data(cur_addr, &packet) {
            return Err(err);
          }

          if cur_addr < 62 { cur_addr+=1; } else { break; }
        },
        Err(err) => return Err(err)
      }
    }

    Ok(cur_addr - addr)
  }

  fn unload(&mut self) -> Result<(), String>{
    println!("NFC driver unloading");
    let mfrc522 = self.mfrc522.clone();
    let pin = mfrc522.lock().unwrap().ss.unwrap();
    if let Err(err) = pin.unexport() {
      return Err(format!("{}(=>{})", "NFC driver error",err));
    }
    Ok(())
  }

  fn signature(&self) -> String {
    return String::from("MFRC522 Mifare Reader Module");
  }
}

unsafe impl Send for Mfrc522 {}
unsafe impl Sync for Mfrc522 {}
