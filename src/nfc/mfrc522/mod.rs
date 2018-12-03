use nfc::{NfcReader, MiFare, PICC};

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;

use std::thread;
use std::time::{Duration,Instant};

use std::io::prelude::*;
use spidev::{Spidev, SpidevOptions, SpidevTransfer, SPI_MODE_0};
use sysfs_gpio::{Direction, Pin};

#[derive(Clone, Copy)]
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
enum Command {
  Idle			= 0b0000,
  Mem			= 0b0001,
  GenerateRandomID	= 0b0010,
  CalcCRC		= 0b0011,
  Transmit		= 0b0100,
  NoCmdChange		= 0b0111,
  Receive		= 0b1000,
  Transceive		= 0b1100,
  MFAuthent		= 0b1110,
  SoftReset		= 0b1111
}

impl Command {
  fn name(&self) -> &'static str {
    match *self {
      Command::Idle => "Idle",
      Command::Mem => "Mem",
      Command::GenerateRandomID => "GenerateRandomID",
      Command::CalcCRC => "CalcCRC",
      Command::Transmit => "Transmit",
      Command::NoCmdChange => "NoCmdChange",
      Command::Receive => "Receive",
      Command::Transceive => "Transceive",
      Command::MFAuthent => "MFAuthent",
      Command::SoftReset => "SoftReset"
    }
  }

  fn value(&self) -> u8 {
    let value = *self as u8;
    value
  }
}

#[derive(Debug, Clone, Copy)]
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

impl Error {
  fn value(&mut self) -> u8 {
    let value = *self as u8;
    value
  }
}

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
    
    let mut irq: u8;
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
    self.command(Command::Idle).map_err(|_err| return Error::SPI);
    // set interruption
    self.write(Register::ComIEn, 0x77 | 0x80);
    // clear all interrupt flags
    self.write(Register::ComIrq, 0x7f).map_err( |_err| return Error::SPI);
    // cler the fifo buffer
    self.flush_fifo().map_err(|err| return Error::SPI);
    // write data to transmit to the fifo buffer
    self.write_many(Register::FifoData, tx_buffer).map_err(|_err| return Error::SPI);
    // send tranceive or MFauth command
    self.command(command).map_err(|_err| return Error::SPI);

    if command.value() == Command::Transceive.value() {
      // configure bit framing
      self.write(Register::BitFraming, (1 << 7) | bits).map_err(|_err| return Error::SPI);
    }

    let mut irq;
    let now = Instant::now();
    loop {
      let sec = (now.elapsed().as_secs() as f64) + (now.elapsed().subsec_nanos() as f64 / 1000_000_000.0);

      if sec > 1.0 {
        return  Err(Error::Timeout);
      }

      irq = 0;
      self.read(Register::ComIrq).map(|val| irq = val);

      if irq & irq_wait != 0 {
        break;
      } else if irq & 0x01 != 0{
        return Err(Error::Timeout);
      }
    }

    //check for errors
    self.check_error()?;

    let mut received:usize = 0;
    match self.read(Register::FifoLevel) {
      Ok(val) => received = val as usize,
      Err(_err) => return Err(Error::SPI)
    }

    //println!("Tranceive received {} bytes", received);
    
    let mut rx_buffer:Vec<u8> = vec![0 ; received];
    self.read_many(Register::FifoData, &mut rx_buffer);

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
    try!(self.write(Register::Mode, (0x3F & (!0b11) | 0b01) ));

    // enable antenna
    try!(self.write(Register::TxControl, 0xB0 | 0b11));

    Ok(())
  }
}

unsafe impl Send for Mfrc522ThreadSafe {}
unsafe impl Sync for Mfrc522ThreadSafe {}

static MIFARE_DEFAULT_KEY_A:       &'static [u8] = &[0xff,0xff,0xff,0xff,0xff,0xff];
static MIFARE_DEFAULT_KEY_B:       &'static [u8] = &[0x00,0x00,0x00,0x00,0x00,0x00];
static MIFARE_DEFAULT_ACCESS_BITS: &'static [u8] = &[0xff,0x07,0x80,0x00];

impl MiFare for Mfrc522ThreadSafe {
  fn send_reqA(&mut self) -> Result<Vec<u8>, String> {
    match self.transceive(&[PICC::REQIDL.value()],7) {
      Ok(val) => Ok(val),
      Err(ref mut err) => Err(format!("{} => 0x{:X}","NFC MiFare REQA error", err.value()))
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

    self.calc_crc(&tx_buf).map(|val| {
      tx_buf.push(val[0]);
      tx_buf.push(val[1]);
    });

    self.clear_bit_mask(Register::Status2, 0x08);

    match self.transceive(&tx_buf, 0) {
      Ok(val) => {
        Ok(val)
      },
      Err(ref mut err) => Err(format!("{} => 0x{:X}","NFC MiFare SELECT error", err.value()))
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
      Err(ref mut err) => Err(format!("{} => 0x{:X}","NFC MiFare anti collision loop error", err.value()))
    }
  }

  fn auth(&mut self, auth_mode: u8, addr: u8, uuid: &Vec<u8>) -> Result<(), String> {
    let mut tx_buf = vec![auth_mode, addr];
    tx_buf.extend(&self.mifare_key_a);
    tx_buf.extend(uuid);

    match self.authent(&tx_buf, 0) {
      Ok(val) => Ok(()),
      Err(err) => Err(format!("{}","Error authenticating"))
    }
  }

  fn write_data(&mut self, addr: u8, data: &Vec<u8>) -> Result<(), String> {
    let mut tx_buf = vec![PICC::WRITE.value(), addr];

    if data.len() < 16 {
      return Err(format!("{}","write_data error: Invalid stream size"));
    }

    self.calc_crc(&tx_buf).map(|crc| {
      tx_buf.push(crc[0]);
      tx_buf.push(crc[1]);
    });

    match self.transceive(&tx_buf, 0) {
      Ok(val) => {
        let mut buf: Vec<u8> = data[..16].to_vec();

        self.calc_crc(&buf).map(|crc| {
          buf.push(crc[0]);
          buf.push(crc[1]);
        });

        match self.transceive(&buf, 0) {
          Ok(val) => {
            Ok(())
          },
          Err(ref mut err) => Err(format!("{} {} => 0x{:X}","NFC MiFare error reading address {}", addr, err.value()))
        }
      },
      Err(ref mut err) => Err(format!("{} {} => 0x{:X}","NFC MiFare error reading address {}", addr, err.value()))
    }
  }

  fn read_data(&mut self, addr: u8) -> Result<(Vec<u8>), String> {
    let mut tx_buf = vec![PICC::READ.value(), addr];

    self.calc_crc(&tx_buf).map(|crc| {
      tx_buf.push(crc[0]);
      tx_buf.push(crc[1]);
    });
    
    match self.transceive(&tx_buf, 0) {
      Ok(val) => {
        let buf: Vec<u8> = val[..16].to_vec();

        self.calc_crc(&buf).map(|crc| {
          if crc[0] != val[16] || crc[1] != val[17] {
            return Err(format!("NFC MiFare read crc error reading address {}", addr));
          }
          Ok(())
        });

        Ok(buf)
      },
      Err(ref mut err) => Err(format!("{} {} => 0x{:X}","NFC MiFare error reading address {}", addr, err.value()))    
    }
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

    if let Ok(version) = mfrc522.lock().unwrap().version() {
      println!("NFC hardware version: 0x{:X}",version);
      if version != 0x91 && version != 0x92 {
        return Err(format!("{}(=>0x{:X})", "NFC Hardware with an invalid version",version));
      }
    } else {
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
    let mut mfrc522 = self.mfrc522.clone();
    
    let _handler = thread::spawn(move || {
      loop {
        let mut ret: Result<(), String>;
        let mut uuid:Vec<u8> = Vec::new();
        let mut sak:Vec<u8> = Vec::new();
        let mut complete:bool = false;
        {
          let mut mfrc522_inner = mfrc522.lock().unwrap();

          if let Err(err) = mfrc522_inner.initialize() {
            println!("Error initializing reader");
            break;
          }

          println!("Searching tag...");
          ret = match mfrc522_inner.send_reqA() {
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

  fn set_auth_keys(&mut self, key_a: Vec<u8>, key_b: Vec<u8>) -> Result<(), String> {
    Ok(())
  }

  fn set_auth_bits(&mut self, access_bits: Vec<u8>) -> Result<(), String> {
    Ok(())
  }

  fn format(&mut self) -> Result<(), String> {
    Ok(())
  }

  fn restore(&mut self) -> Result<(), String> {
    Ok(())
  }

  fn read_data(&mut self, uuid: &Vec<u8>, addr: u8, blocks: u8) -> Result<(Vec<u8>), String> {
    let mut mfrc522 = self.mfrc522.clone();
    let mut mfrc522_inner = mfrc522.lock().unwrap();
    
    println!("{}","read_data: reached");

    let mut cur_addr:u8 = addr;
    let mut buffer: Vec<u8> = Vec::new();

    loop {

      if cur_addr+1 % 4 == 0 { cur_addr += 1; }

      match mfrc522_inner.auth(PICC::AUTH1A.value(), cur_addr, uuid) {
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
    let mut mfrc522 = self.mfrc522.clone();
    let mut mfrc522_inner = mfrc522.lock().unwrap();

    println!("{}","write_data: reached");

    let mut cur_addr:u8 = addr;
    let mut buffer:VecDeque<u8> = VecDeque::new();
    let mut packet:Vec<u8> = Vec::new();

    buffer.extend(data);

    loop {

      if cur_addr+1 % 4 == 0 { cur_addr += 1; }

      match mfrc522_inner.auth(PICC::AUTH1A.value(), cur_addr, uuid) {
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
