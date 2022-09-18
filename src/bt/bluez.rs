/**
 * @file   bluez/mod.rs
 * @author Otavio Ribeiro
 * @date   16 Set 2022
 * @brief  BlueZ driver
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
use super::{Bluetooth,BluetoothAction, BluetoothData, BluetoothProps};

use tokio::runtime::{Handle,Runtime};
use bluer::{Adapter, AdapterEvent, Address, DeviceEvent};
use futures::{pin_mut, stream::SelectAll, StreamExt};
use std::{collections::HashSet, env, collections::HashMap};
use std::time::{Duration,Instant};
use std::thread;
use std::sync::Arc;
use std::sync::Mutex;

pub struct BlueZ {
}

impl BlueZ {
    pub fn new() -> Self {
        return BlueZ { };
    }
}

impl Bluetooth for BlueZ {
    fn init(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn find_devices(&mut self, func: fn(device: &HashMap<String, String>, action: &BluetoothAction) -> bool) -> Result<(),String>{
        let _handler = thread::spawn( move || {
            println!("BlueZ module searching for devices");
            let rt = Runtime::new().unwrap();
            let handle = rt.handle().clone();        
            handle.block_on(async {
                if let Ok(session) = bluer::Session::new().await {
                    if let Ok(adapter) = session.default_adapter().await {
                        println!("Discovering devices using Bluetooth adapater {}", adapter.name());
                        if let Err(err) = adapter.set_powered(true).await {
                            return Err("Error powering up bluetooth device");
                        } else {
                            match adapter.discover_devices().await {
                                Ok(device_events) => {
                                    pin_mut!(device_events);

                                    let mut all_change_events = SelectAll::new();
    
                                    loop {
                                        tokio::select! {    
                                            Some(device_event) = device_events.next() => {
                                                match device_event {
                                                    AdapterEvent::DeviceAdded(addr) => {
                                                        println!("Bluetooth device added: {}", addr);
                                                        if let Ok(device) = adapter.device(addr) {
                                                            if let Ok(change_events) = device.events().await {
                                                                //change_events.map(move |evt| (addr, evt));
                                                                all_change_events.push(change_events);                                   
                                                            } else {
                                                                println!("Error getting bluetooth device events: {}", addr);
                                                            }
                                                        } else {
                                                            println!("Error getting bluetooth device: {}", addr);
                                                        }
                                                    }
                                                    AdapterEvent::DeviceRemoved(addr) => {
                                                        println!("Bluetooth device removed: {}", addr);
                                                    }
                                                    _ => (),
                                                }
                                                println!();
                                            }
                                            Some(device_event) = all_change_events.next() => {
                                                match device_event {
                                                    DeviceEvent::PropertyChanged(property) => {
                                                        println!("Bluetooth device changed: {}", "?");
                                                        println!("    {:?}", property);
            
                                                    }
                                                    _ => (),
                                                }
                                            }
                                            else => break
                                        }
                                    }
                                    println!("Bluetooth module finished");
                                    return Ok(());
                                }
                                Err(err) => {
                                    println!("Bluetooth error discovering devices: {}",err);
                                }
                            }
                        }
                    }                
                }
                return Err("Error initializing bluetooth module");
            });
        });
        Ok(())
    }

    fn unload(&mut self) -> Result<(), String>{
        Ok(())
    }

    fn delete_all(&mut self) -> bool{
        return false;
    }

    fn start_enroll(&mut self, data: &BluetoothData) -> bool{
        return false;
    }

    fn signature(&self) -> String {
        return String::from("BlueZ bluetooth module");
    }
}
