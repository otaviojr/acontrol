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

#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
#[allow(non_camel_case_types)]

pub enum BluetoothAction {
    Add,
    Remove,
    Change,
}

impl BluetoothAction {
    pub fn name(&self) -> &'static str {
      match *self {
        BluetoothAction::Add => "ADD",
        BluetoothAction::Remove => "REMOVE",
        BluetoothAction::Change => "CHANGE",
      }
    }
  
    pub fn set(&mut self, new_state:BluetoothAction) {
      *self = new_state;
    }
}

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

pub trait Bluetooth {
    fn init(&mut self) -> Result<(), String>;
    fn find_devices(&mut self, func: fn(device: &HashMap<String, String>, action: Option<&str>) -> bool) -> Result<(),String>;
    fn unload(&mut self) -> Result<(), String>;
    fn delete_all(&mut self) -> bool;
    fn start_enroll(&mut self, data: &BluetoothData) -> bool;
  }
  
pub fn bluetooth_by_name(name: &str) -> Option<Box<dyn Bluetooth+Sync+Send>> {
    match name {
        "bluez" => return Some(Box::new(bluez::BlueZ::new())),
        _ => return None
    }
}