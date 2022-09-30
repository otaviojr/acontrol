
/**
 * @file   bt/bluez.rs
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
use crate::bt::{Bluetooth, BluetoothData, BluetoothDevice};
use crate::{acontrol_system_log, log::LogType};
use async_trait::async_trait;

use tokio::sync::{mpsc,Mutex};
use bluer::{Session, Adapter, monitor::{Monitor, RegisteredMonitorHandle, Pattern}};
use std::sync::{Arc};

pub struct BlueZ {
    session: Arc<Mutex<Option<Session>>>,
    adapter: Arc<Mutex<Option<Adapter>>>,
    monitor_handle: Box<Option<RegisteredMonitorHandle>>,
    event_tx: mpsc::Sender<bluer::Address>,
    event_rx: Arc<Mutex<mpsc::Receiver<bluer::Address>>>,
}

impl BlueZ {
    pub fn new() -> Self {
        let (e_tx, e_rx) = mpsc::channel(1);

        return BlueZ { session: Arc::new(Mutex::new(Option::None)), 
                        adapter: Arc::new(Mutex::new(Option::None)), 
                        monitor_handle: Box::new(Option::None),
                        event_tx: e_tx, event_rx: Arc::new(tokio::sync::Mutex::new(e_rx)),
                     };
    }
}

#[async_trait]
impl Bluetooth for BlueZ {
    async fn init(&mut self) -> Result<(), String> {
        if let Ok(session) = Session::new().await {
            if let Ok(adapter) = session.default_adapter().await {
                if adapter.set_powered(true).await.is_ok() {
                    acontrol_system_log!(LogType:: Info, "Bluetooth: registering monitor");
                    if let Ok(mut monitor_handle) = adapter.register_monitor().await {
                        let tx = self.event_tx.clone();
                        let _ = monitor_handle.add_monitor(Monitor {
                            activate: Some(Box::new(move || {
                                Box::pin(async {
                                    acontrol_system_log!(LogType::Debug, "Monitor 1: Activate funcion called");
                                    Ok(())
                                })
                            })),
                            release: Some(Box::new(move || {
                                Box::pin(async {
                                    acontrol_system_log!(LogType::Debug, "Monitor 1: Release funcion called");
                                    Ok(())
                                })
                            })),
                            device_found: Some(Box::new(move |device| {
                                let tx1 = tx.clone();
                                Box::pin(async move {
                                    acontrol_system_log!(LogType::Debug,"Monitor 1: DeviceFound funcion called: {}",device.addr);
                                    let _ = tx1.send(device.addr).await;
                                    Ok(())
                                })
                            })),
                            patterns: Some(vec!(Pattern {
                                start_position: 4,
                                ad_data_type: 0xff,
                                content_of_pattern: vec!(0x9b,0xfb,0xef,0x3a,0x21,0x0a,0x4b, 0x3a,0x9e,0x58,0x24,0xe7,0xcd,0x83,0x54,0xed)
                            })),
                            rssi_low_threshold: Some(127),
                            rssi_high_threshold: Some(127),
                            rssi_low_timeout: Some(0),
                            rssi_high_timeout: Some(0),
                            ..Default::default()
                        }).await;
                        self.monitor_handle = Box::new(Some(monitor_handle));
                        self.session = Arc::new(Mutex::new(Some(session)));
                        self.adapter = Arc::new(Mutex::new(Some(adapter)));
                    } else {
                        acontrol_system_log!(LogType::Error, "Monitor: Somethings gets wrong with the monitor");
                    }
                    return Ok(());
                }
            } else {
                acontrol_system_log!(LogType::Error, "BlueZ: no default adapter");
            }
        } else {
            acontrol_system_log!(LogType::Error, "BlueZ: Session error");
        }

        Err(String::from("BlueZ: Error initializing bluetooth module"))
    }

    async fn find_devices(&mut self, func: fn(device: BluetoothDevice) -> bool) -> Result<(),String>{
        let adapter_mutex = self.adapter.clone();
        let rx = self.event_rx.clone();

        let _ = tokio::spawn(async move {
            let ref adapter_lock = adapter_mutex.lock().await;
            if let Some(ref adapter) = **adapter_lock {
                let mut rx1 = rx.lock().await;
                while let Some(addr) = rx1.recv().await {
                    let device = adapter.device(addr).unwrap();

                    let address_type = device.address_type().await.unwrap();
                    let name = device.name().await.unwrap_or_default();
                    let icon = device.icon().await.unwrap_or_default();
                    let class = device.class().await.unwrap_or_default();
                    let uuids = device.uuids().await.unwrap_or_default();
                    let paired = device.is_paired().await.unwrap_or_default();
                    let connected = device.is_connected().await.unwrap_or_default();
                    let trusted = device.is_trusted().await.unwrap_or_default();
                    let modalias = device.modalias().await.unwrap_or_default();
                    let rssi = device.rssi().await.unwrap_or_default();
                    let tx_power = device.tx_power().await.unwrap_or_default();
                    let manufacturer_data = device.manufacturer_data().await.unwrap_or_default();
                    let service_data = device.service_data().await.unwrap_or_default();


                    acontrol_system_log!(LogType::Debug, "Bluetooth monitor thread: DeviceFound funcion called: {}",addr);
                    acontrol_system_log!(LogType::Debug, "-------------------------");
                    acontrol_system_log!(LogType::Debug, "    Address:            {}", addr);
                    acontrol_system_log!(LogType::Debug, "    Address type:       {}", address_type);
                    acontrol_system_log!(LogType::Debug, "    Name:               {:?}", name);
                    acontrol_system_log!(LogType::Debug, "    Icon:               {:?}", icon);
                    acontrol_system_log!(LogType::Debug, "    Class:              {:?}", class);
                    acontrol_system_log!(LogType::Debug, "    UUIDs:              {:?}", uuids);
                    acontrol_system_log!(LogType::Debug, "    Paired:             {:?}", paired);
                    acontrol_system_log!(LogType::Debug, "    Connected:          {:?}", connected);
                    acontrol_system_log!(LogType::Debug, "    Trusted:            {:?}", trusted);
                    acontrol_system_log!(LogType::Debug, "    Modalias:           {:?}", modalias);
                    acontrol_system_log!(LogType::Debug, "    RSSI:               {:?}", rssi);
                    acontrol_system_log!(LogType::Debug, "    TX power:           {:?}", tx_power);
                    acontrol_system_log!(LogType::Debug, "    Manufacturer data:  {:?}", manufacturer_data);
                    acontrol_system_log!(LogType::Debug, "    Service data:       {:?}", service_data);
                    acontrol_system_log!(LogType::Debug, "-------------------------");

                    let bd = BluetoothDevice::new(addr.to_string());
                    func(bd);
                }

                acontrol_system_log!(LogType::Warning, "Bluetooth thread ended");    
            }
            ()
        });

        Ok(())
    }

    fn unload(&mut self) -> Result<(), String>{
        if let Some(monitor_handle) = &*self.monitor_handle {
            acontrol_system_log!(LogType::Info, "Stopping bluetooth le monitoring.");
            drop(monitor_handle);
        }
        Ok(())
    }

    fn delete_all(&mut self) -> bool{
        return false;
    }

    fn start_enroll(&mut self, _data: &BluetoothData) -> bool{
        return false;
    }

    fn signature(&self) -> String {
        return String::from("BlueZ bluetooth module");
    }
}
