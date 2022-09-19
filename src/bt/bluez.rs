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

use async_trait::async_trait;
use tokio::runtime::{Handle,Runtime};
use bluer::{Session, Adapter, AdapterEvent, Address, DeviceEvent,DeviceProperty, adv::Advertisement, adv::AdvertisementHandle};
use futures::{pin_mut, stream::SelectAll, StreamExt};
use std::{collections::HashSet, env, collections::HashMap};
use std::time::{Duration};
use std::thread;
use std::sync::Arc;
use std::sync::Mutex;

pub struct BlueZ {
    session: Arc<Mutex<Option<Session>>>,
    adapter: Arc<Mutex<Option<Adapter>>>,
    adv_handle: Box<Option<AdvertisementHandle>>
}

impl BlueZ {
    pub fn new() -> Self {
        return BlueZ { session: Arc::new(Mutex::new(Option::None)), adapter: Arc::new(Mutex::new(Option::None)), adv_handle: Box::new(Option::None) };
    }
}

#[async_trait]
impl Bluetooth for BlueZ {
    async fn init(&mut self) -> Result<(), String> {
        if let Ok(session) = Session::new().await {
            if let Ok(adapter) = session.default_adapter().await {
                if adapter.set_powered(true).await.is_ok() {
                    let le_advertisement = Advertisement {
                        advertisement_type: bluer::adv::Type::Peripheral,
                        service_uuids: vec!["123e4567-e89b-12d3-a456-426614174000".parse().unwrap()].into_iter().collect(),
                        discoverable: Some(true),
                        discoverable_timeout: Some(Duration::from_millis(0)),
                        local_name: Some("acontrol".to_string()),
                        min_interval: Some(Duration::from_millis(500)),
                        max_interval: Some(Duration::from_millis(1000)),
                        tx_power: Some(20),
                        ..Default::default()
                    };
                    println!("Bluetooth Advertising:");
                    println!("{:?}", &le_advertisement);
                    if let Ok(adv_handle) = adapter.advertise(le_advertisement).await{
                        self.adv_handle = Box::new(Some(adv_handle));
                        self.session = Arc::new(Mutex::new(Some(session)));
                        self.adapter = Arc::new(Mutex::new(Some(adapter)));    
                    }
                    return Ok(());
                }
            }
        }

        Err(String::from("BlueZ: Error initializing bluetooth module"))
    }

    async fn find_devices(&mut self, func: fn(device: &HashMap<String, String>, action: &BluetoothAction) -> bool) -> Result<(),String>{
        let adapter_mutex = self.adapter.clone();
        thread::spawn( move || {
            println!("BlueZ module searching for devices");
            let rt = Runtime::new().unwrap();
            let handle_async = rt.handle().clone();
            handle_async.block_on(async {
                if let Ok(ref mut adapter_locked) = adapter_mutex.lock() {
                    if let Some(ref adapter) = **adapter_locked {
                        println!("Discovering devices using Bluetooth adapater {}", adapter.name());
                        match adapter.discover_devices().await {
                            Ok(device_events) => {
                                pin_mut!(device_events);
        
                                let mut all_change_events = SelectAll::new();
        
                                loop {
                                    tokio::select! {    
                                        Some(device_event) = device_events.next() => {
                                            match device_event {
                                                AdapterEvent::DeviceAdded(addr) => {
                                                    //println!("Bluetooth device added: {}", addr);
                                                    if let Ok(device) = adapter.device(addr) {
                                                        if let Ok(change_events) = device.events().await {
                                                            let event = change_events.map(move |evt| (addr, evt));
                                                            all_change_events.push(event);                                   
                                                        } else {
                                                            println!("Error getting bluetooth device events: {}", addr);
                                                        }
                                                    } else {
                                                        println!("Error getting bluetooth device: {}", addr);
                                                    }
                                                }
                                                AdapterEvent::DeviceRemoved(addr) => {
                                                    //println!("Bluetooth device removed: {}", addr);
                                                }
                                                _ => (),
                                            }
                                        }
                                        Some((addr,DeviceEvent::PropertyChanged(property))) = all_change_events.next() => {
                                            let device = adapter.device(addr).unwrap();
        
                                            match property {
                                                DeviceProperty::Rssi(rssi) => {
                                                    if rssi < -50 && device.is_paired().await.unwrap_or_default() == true {
                                                        println!("-------------------------");
                                                        println!("    Property:           {:?}", property);
                                                        println!("    Address:            {}", addr);
                                                        println!("    Address type:       {}", device.address_type().await.unwrap());
                                                        println!("    Name:               {:?}", device.name().await.unwrap_or_default());
                                                        println!("    Icon:               {:?}", device.icon().await.unwrap_or_default());
                                                        println!("    Class:              {:?}", device.class().await.unwrap_or_default());
                                                        println!("    UUIDs:              {:?}", device.uuids().await.unwrap_or_default());
                                                        println!("    Paired:             {:?}", device.is_paired().await.unwrap_or_default());
                                                        println!("    Connected:          {:?}", device.is_connected().await.unwrap_or_default());
                                                        println!("    Trusted:            {:?}", device.is_trusted().await.unwrap_or_default());
                                                        println!("    Modalias:           {:?}", device.modalias().await.unwrap_or_default());
                                                        println!("    RSSI:               {:?}", device.rssi().await.unwrap_or_default());
                                                        println!("    TX power:           {:?}", device.tx_power().await.unwrap_or_default());
                                                        println!("    Manufacturer data:  {:?}", device.manufacturer_data().await.unwrap_or_default());
                                                        println!("    Service data:       {:?}", device.service_data().await.unwrap_or_default());
                                                        println!("-------------------------");
                                                    }
                                                }
                                                _ => {
                                                }
                                            }
                                        }
                                    }
                                    thread::sleep(Duration::from_millis(100));
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
                Err("Bluetooth thread ended")
            });
        });
        Ok(())
    }

    fn unload(&mut self) -> Result<(), String>{
        if let Some(adv_handle) = &*self.adv_handle {
            println!("Stopping bluetooth advertising.");
            drop(adv_handle);
        }
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
