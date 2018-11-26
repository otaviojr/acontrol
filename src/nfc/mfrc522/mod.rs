use nfc::{NfcReader};

use std::sync::Arc;
use std::sync::Mutex;

use std::thread;
use std::time::Duration;

use std::io::prelude::*;
use spidev::{Spidev, SpidevOptions, SpidevTransfer, SPI_MODE_0};
use sysfs_gpio::{Direction, Pin};

#[derive(Clone, Copy)]
enum Register {
    BitFraming = 0x0d,
    Coll = 0x0e,
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

struct Mfrc522ThreadSafe {
  spidev: Option<Spidev>,
  ss: Option<Pin>
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

  fn write(&mut self, reg: Register, val: u8) -> Result<(), std::io::Error>{
    self.with_ss(|ref mut mfrc| {
      if let Some(ref mut spidev) = mfrc.spidev {
        try!(spidev.write(&[reg.write(), val]));
      }
      return Ok(())
    })
  }
}

unsafe impl Send for Mfrc522ThreadSafe {}
unsafe impl Sync for Mfrc522ThreadSafe {}

pub struct Mfrc522 {
  mfrc522: Arc<Mutex<Mfrc522ThreadSafe>>
}

impl Mfrc522 {
  pub fn new() -> Self {
    return Mfrc522 {mfrc522: Arc::new(Mutex::new(Mfrc522ThreadSafe {spidev: None, ss: None}))};
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

    if let Ok(version) = mfrc522.lock().unwrap().read(Register::Version) {
      println!("NFC hardware version: 0x{:X}",version);
      if version != 0x91 && version != 0x92 {
        return Err(format!("{}(=>0x{:X})", "NFC Hardware with an invalid version",version));
      }
    } else {
      return Err(format!("{}", "NFC error. Could not retrieve hardware version"));
    }

    Ok(())
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
