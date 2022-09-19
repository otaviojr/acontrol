/**
 * @file   bt/mod.rs
 * @author Otavio Ribeiro
 * @date   16 Set 2022
 * @brief  Bluetooth global interface
 *
 * Copyright (c) 2022 Otávio Ribeiro <otavio.ribeiro@gmail.com>
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

mod bluez;

use std::collections::HashMap;
use async_trait::async_trait;
use std::time::Instant;
use std::sync::{Mutex,Arc};

#[derive(Eq, Hash, PartialEq)]
#[derive(Clone)]
pub struct BluetoothDevice {
  pub name: String,
  pub addr: String,
  pub rssi: i16,
  pub created: Instant,
  pub access: Option<Instant>,
}

impl BluetoothDevice {
    pub fn new(addr: String) -> Self {
      return BluetoothDevice { name: String::from(""), addr: addr, rssi: 0, created: Instant::now(), access: Option::None };
    }
}

#[derive(Eq, Hash, PartialEq)]
pub enum BluetoothProps {
  Address,
  Name,
  Icon,
  Class,
  Uuids,
  Paired,
  Connected,
  Trusted,
  Rssi,
  TxPower,
  Manufacturer,
  Service,
}

impl BluetoothProps {
    pub fn name(&self) -> &'static str {
      match *self {
        BluetoothProps::Address => "ADDRESS",
        BluetoothProps::Name => "NAME",
        BluetoothProps::Icon => "ICON",
        BluetoothProps::Class => "CLASS",
        BluetoothProps::Uuids => "UUIDS",
        BluetoothProps::Paired => "PAIRED",
        BluetoothProps::Connected => "CONNECTED",
        BluetoothProps::Trusted => "TRUSTED",
        BluetoothProps::Rssi => "RSSI",
        BluetoothProps::TxPower => "TX_POWER",
        BluetoothProps::Manufacturer => "MANUFACTURER",
        BluetoothProps::Service => "SERVICE",
      }
    }
  
    pub fn set(&mut self, new_state:BluetoothProps) {
      *self = new_state;
    }
}

pub struct BluetoothData {
    pub address: Option<String>,
    pub name: Option<String>
}

impl BluetoothData {
    pub fn new(address: &str, name: &str) -> Self {
        BluetoothData { address: Some(String::from(address)), name: Some(String::from(name)) }
    }

    pub fn empty() -> Self {
        BluetoothData { address: None, name: None }
    }
}

#[async_trait]
pub trait Bluetooth {
    async fn init(&mut self) -> Result<(), String>;
    async fn find_devices(&mut self, func: fn(device: BluetoothDevice) -> bool) -> Result<(),String>;
    fn unload(&mut self) -> Result<(), String>;
    fn delete_all(&mut self) -> bool;
    fn start_enroll(&mut self, data: &BluetoothData) -> bool;
    fn signature(&self) -> String;
  }
  
pub fn bluetooth_by_name(name: &str) -> Option<Box<dyn Bluetooth+Sync+Send>> {
    match name {
        "bluez" => return Some(Box::new(bluez::BlueZ::new())),
        _ => return None
    }
}
