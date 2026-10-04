use crate::bluetooth::aacp::BatteryStatus;
use crate::bluetooth::case_battery;
use crate::devices::enums::{DeviceData, DeviceInformation, DeviceType};
use crate::ui::tray::MyTray;
use crate::utils::{ah, get_devices_path, get_preferences_path};
use aes::Aes128;
use aes::cipher::Array;
use aes::cipher::{BlockCipherDecrypt, KeyInit};
use bluer::monitor::{Monitor, MonitorEvent, Pattern};
use bluer::{Address, Session};
use futures::StreamExt;
use hex;
use log::{debug, info};
use serde_json;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;

fn decrypt(key: &[u8; 16], data: &[u8; 16]) -> [u8; 16] {
    let cipher = Aes128::new(&Array::from(*key));
    let mut block = Array::from(*data);
    cipher.decrypt_block(&mut block);
    block.into()
}

fn verify_rpa(addr: &str, irk: &[u8; 16]) -> bool {
    let rpa: Vec<u8> = addr
        .split(':')
        .map(|s| u8::from_str_radix(s, 16).unwrap())
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    if rpa.len() != 6 {
        return false;
    }
    let prand_slice = &rpa[3..6];
    let prand: [u8; 3] = prand_slice.try_into().unwrap();
    let hash_slice = &rpa[0..3];
    let hash: [u8; 3] = hash_slice.try_into().unwrap();
    let computed_hash = ah(irk, &prand);
    debug!(
        "Verifying RPA: addr={}, hash={:?}, computed_hash={:?}",
        addr, hash, computed_hash
    );
    hash == computed_hash
}

pub async fn start_le_monitor(tray_handle: Option<ksni::Handle<MyTray>>) -> bluer::Result<()> {
    let session = Session::new().await?;
    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;

    let mut all_devices: HashMap<String, DeviceData> = std::fs::read_to_string(get_devices_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    let mut verified_macs: HashMap<Address, String> = HashMap::new();
    let mut watchers: HashMap<Address, tokio::task::JoinHandle<()>> = HashMap::new();
    let connecting_macs = Arc::new(Mutex::new(HashSet::<Address>::new()));

    let pattern = Pattern {
        data_type: 0xFF, // Manufacturer specific data
        start_position: 0,
        content: vec![0x4C, 0x00], // Apple manufacturer ID (76) in LE
    };

    let mm = adapter.monitor().await?;
    let mut monitor_handle = mm
        .register(Monitor {
            monitor_type: bluer::monitor::Type::OrPatterns,
            rssi_low_threshold: None,
            rssi_high_threshold: None,
            rssi_low_timeout: None,
            rssi_high_timeout: None,
            rssi_sampling_period: None,
            patterns: Some(vec![pattern]),
            ..Default::default()
        })
        .await?;

    debug!("Started LE monitor");

    // Trigger a tray change on expiry so consumers can use their last-known cache.
    if let Some(handle) = tray_handle.clone() {
        tokio::spawn(async move {
            let mut timer = tokio::time::interval(std::time::Duration::from_secs(5));
            loop {
                timer.tick().await;
                handle
                    .update(|tray: &mut MyTray| {
                        if let Err(e) = tray
                            .telemetry
                            .publish(crate::bluetooth::battery_telemetry::now_ms())
                        {
                            log::warn!("Battery telemetry write failed: {}", e);
                        }
                    })
                    .await;
            }
        });
    }
    // BlueZ can retain advertisers across a LibrePods restart without sending
    // DeviceFound again. Subscribe to future changes on these objects as well,
    // but never count their cached ManufacturerData as a fresh observation.
    let mut existing_advertisers = Vec::new();
    for addr in adapter.device_addresses().await? {
        if let Ok(device) = adapter.device(addr) {
            if let Ok(Some(data)) = device.manufacturer_data().await {
                if data.contains_key(&76) {
                    existing_advertisers.push(addr);
                }
            }
        }
    }
    let bootstrap_scan = all_devices.values().any(|device| {
        matches!(&device.information, Some(DeviceInformation::AirPods(info))
            if hex::decode(&info.le_keys.enc_key).is_ok_and(|key| key.len() == 16))
    });
    // BlueZ normally suppresses repeated identical ManufacturerData, so a
    // full bud in a closed case otherwise ages out despite continued adverts.
    // Request duplicates briefly only for a recently seen independent case.
    if let Some(handle) = tray_handle.clone() {
        let refresh_adapter = adapter.clone();
        tokio::spawn(async move {
            let filter = bluer::DiscoveryFilter {
                transport: bluer::DiscoveryTransport::Le,
                duplicate_data: true,
                ..Default::default()
            };
            if let Err(e) = refresh_adapter.set_discovery_filter(filter).await {
                log::warn!("Case refresh filter unavailable: {}", e);
                return;
            }
            let mut timer = tokio::time::interval(std::time::Duration::from_secs(30));
            timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            let mut bootstrap = bootstrap_scan;
            loop {
                timer.tick().await;
                let Some(needed) = handle
                    .update(|tray: &mut MyTray| {
                        tray.telemetry
                            .needs_case_refresh(crate::bluetooth::battery_telemetry::now_ms())
                    })
                    .await
                else {
                    break;
                };
                if !needed && !bootstrap {
                    continue;
                }
                let scan_seconds = if bootstrap { 20 } else { 8 };
                bootstrap = false;
                match refresh_adapter.discover_devices().await {
                    Ok(events) => {
                        info!(
                            "Refreshing case advertisements ({}-second LE scan)",
                            scan_seconds
                        );
                        futures::pin_mut!(events);
                        let _ = tokio::time::timeout(
                            std::time::Duration::from_secs(scan_seconds),
                            async { while events.next().await.is_some() {} },
                        )
                        .await;
                        // Dropping our discovery stream releases only our scan request.
                    }
                    Err(e) => log::warn!("Case refresh scan unavailable: {}", e),
                }
            }
        });
    }
    loop {
        let (device_address, newly_found) = if let Some(addr) = existing_advertisers.pop() {
            (addr, false)
        } else {
            match monitor_handle.next().await {
                Some(MonitorEvent::DeviceFound(devid)) => (devid.device, true),
                Some(MonitorEvent::DeviceLost(_)) => {
                    // Monitor loss is not D-Bus object removal. Discovery can
                    // still deliver fresh properties on this existing object.
                    watchers.retain(|_, task| !task.is_finished());
                    continue;
                }
                Some(_) => continue,
                None => break,
            }
        };
        {
            // Keys may arrive after the monitor starts or after a reconnect.
            if let Ok(contents) = std::fs::read_to_string(get_devices_path()) {
                if let Ok(devices) = serde_json::from_str(&contents) {
                    all_devices = devices;
                }
            }
            let adapter_monitor_clone = adapter.clone();
            let dev = adapter_monitor_clone.device(device_address)?;
            let addr = dev.address();
            let addr_str = addr.to_string();
            if watchers.get(&addr).is_some_and(|task| !task.is_finished()) {
                continue;
            }
            let case_keys: Vec<(String, [u8; 16])> = all_devices
                .iter()
                .filter_map(|(mac, device)| {
                    if device.type_ != DeviceType::AirPods {
                        return None;
                    }
                    let Some(DeviceInformation::AirPods(info)) = &device.information else {
                        return None;
                    };
                    let key = hex::decode(&info.le_keys.enc_key).ok()?;
                    Some((mac.clone(), key.as_slice().try_into().ok()?))
                })
                .collect();

            let matched_airpods_mac: Option<String>;
            let mut matched_enc_key: Option<[u8; 16]> = None;

            if let Some(airpods_mac) = verified_macs.get(&addr) {
                matched_airpods_mac = Some(airpods_mac.clone());
            } else {
                debug!("Checking RPA for device: {}", addr_str);
                let mut found_mac = None;
                for (airpods_mac, device_data) in &all_devices {
                    if device_data.type_ == DeviceType::AirPods
                        && let Some(DeviceInformation::AirPods(info)) = &device_data.information
                        && let Ok(irk_bytes) = hex::decode(&info.le_keys.irk)
                        && irk_bytes.len() == 16
                    {
                        let irk: [u8; 16] = irk_bytes.as_slice().try_into().unwrap();
                        debug!("Checking stored AirPods IRK");
                        if verify_rpa(&addr_str, &irk) {
                            info!(
                                "Matched our device ({}) with the irk for {}",
                                addr, airpods_mac
                            );
                            verified_macs.insert(addr, airpods_mac.clone());
                            found_mac = Some(airpods_mac.clone());
                            break;
                        }
                    }
                }

                if let Some(mac) = found_mac {
                    matched_airpods_mac = Some(mac);
                } else {
                    debug!("Device {} did not match any of our irks", addr);
                    // The case has its own RPA; it need not match the earbud IRK.
                    matched_airpods_mac = None;
                }
            }

            if let Some(ref mac) = matched_airpods_mac
                && let Some(device_data) = all_devices.get(mac)
                && let Some(DeviceInformation::AirPods(info)) = &device_data.information
                && let Ok(enc_key_bytes) = hex::decode(&info.le_keys.enc_key)
                && enc_key_bytes.len() == 16
            {
                matched_enc_key = Some(enc_key_bytes.as_slice().try_into().unwrap());
            }

            if matched_airpods_mac.is_some() || !case_keys.is_empty() {
                let live_events = dev.events().await?;
                // Only new sightings may use the initial ManufacturerData as a fresh sample.
                let initial_data = if newly_found {
                    dev.manufacturer_data().await?
                } else {
                    None
                };
                let initial_events = initial_data.into_iter().map(|data| {
                    bluer::DeviceEvent::PropertyChanged(bluer::DeviceProperty::ManufacturerData(
                        data,
                    ))
                });
                let mut events = futures::stream::iter(initial_events).chain(live_events);
                let tray_handle_clone = tray_handle.clone();
                let connecting_macs_clone = Arc::clone(&connecting_macs);
                let task = tokio::spawn(async move {
                    let mut last_battery = None;
                    let mut last_case = None;
                    while let Some(ev) = events.next().await {
                        match ev {
                            bluer::DeviceEvent::PropertyChanged(prop) => {
                                if let bluer::DeviceProperty::ManufacturerData(data) = prop {
                                    if let Some(apple_data) = data.get(&76)
                                        && let Some((pair, components)) =
                                            case_battery::match_components(apple_data, &case_keys)
                                    {
                                        let value = components[2];
                                        if last_case != Some(value) {
                                            info!("Independent case battery byte: {value:#04x}");
                                            last_case = Some(value);
                                        }
                                        if let Some(handle) = &tray_handle_clone {
                                            handle
                                                .update(|tray: &mut MyTray| {
                                                    let now =
                                                        crate::bluetooth::battery_telemetry::now_ms(
                                                        );
                                                    for (name, byte) in ["left", "right", "case"]
                                                        .into_iter()
                                                        .zip(components)
                                                    {
                                                        tray.telemetry
                                                            .update(pair, "case", name, byte, now);
                                                    }
                                                    let _ = tray.telemetry.publish(now);
                                                })
                                                .await;
                                        }
                                        continue;
                                    }
                                    if let Some(enc_key) = &matched_enc_key
                                        && let Some(apple_data) = data.get(&76)
                                        && apple_data.len() == 27
                                        && apple_data[..3] == [0x07, 0x19, 0x01]
                                    {
                                        let last_16: [u8; 16] =
                                            apple_data[apple_data.len() - 16..].try_into().unwrap();
                                        let decrypted = decrypt(enc_key, &last_16);
                                        debug!(
                                            "Decrypted data from airpods_mac {}: {}",
                                            matched_airpods_mac
                                                .as_ref()
                                                .unwrap_or(&"unknown".to_string()),
                                            hex::encode(decrypted)
                                        );

                                        let connection_state = apple_data[10] as usize;
                                        debug!("Connection state: {}", connection_state);
                                        if connection_state == 0x00 {
                                            let pref_path = get_preferences_path();
                                            let preferences: HashMap<
                                                String,
                                                HashMap<String, bool>,
                                            > = std::fs::read_to_string(&pref_path)
                                                .ok()
                                                .and_then(|s| serde_json::from_str(&s).ok())
                                                .unwrap_or_default();
                                            let auto_connect = preferences
                                                .get(matched_airpods_mac.as_ref().unwrap())
                                                .and_then(|prefs| prefs.get("autoConnect"))
                                                .copied()
                                                .unwrap_or(true);
                                            debug!(
                                                "Auto-connect preference for {}: {}",
                                                matched_airpods_mac.as_ref().unwrap(),
                                                auto_connect
                                            );
                                            if auto_connect {
                                                let real_address =
                                                    Address::from_str(&addr_str).unwrap();
                                                let mut cm = connecting_macs_clone.lock().await;
                                                if cm.contains(&real_address) {
                                                    info!(
                                                        "Already connecting to {}, skipping duplicate attempt.",
                                                        matched_airpods_mac.as_ref().unwrap()
                                                    );
                                                    return;
                                                }
                                                cm.insert(real_address);
                                                info!(
                                                    "AirPods are disconnected, attempting to connect to {}",
                                                    matched_airpods_mac.as_ref().unwrap()
                                                );
                                                let output =
                                                    tokio::process::Command::new("bluetoothctl")
                                                        .arg("connect")
                                                        .arg(matched_airpods_mac.as_ref().unwrap())
                                                        .output()
                                                        .await;
                                                match output {
                                                    Ok(output) => {
                                                        if output.status.success() {
                                                            info!(
                                                                "Successfully connected to AirPods {}",
                                                                matched_airpods_mac
                                                                    .as_ref()
                                                                    .unwrap()
                                                            );
                                                            cm.remove(&real_address);
                                                        } else {
                                                            let stderr = String::from_utf8_lossy(
                                                                &output.stderr,
                                                            );
                                                            info!(
                                                                "Failed to connect to AirPods {}: {}",
                                                                matched_airpods_mac
                                                                    .as_ref()
                                                                    .unwrap(),
                                                                stderr
                                                            );
                                                        }
                                                    }
                                                    Err(e) => {
                                                        info!(
                                                            "Failed to execute bluetoothctl to connect to AirPods {}: {}",
                                                            matched_airpods_mac.as_ref().unwrap(),
                                                            e
                                                        );
                                                    }
                                                }
                                                info!(
                                                    "Auto-connect is disabled for {}, not attempting to connect.",
                                                    matched_airpods_mac.as_ref().unwrap()
                                                );
                                            }
                                        }

                                        let status = apple_data[5] as usize;
                                        let primary_left = (status >> 5) & 0x01 == 1;
                                        let this_in_case = (status >> 6) & 0x01 == 1;
                                        let xor_factor = primary_left ^ this_in_case;
                                        let is_left_in_ear = if xor_factor {
                                            (status & 0x02) != 0
                                        } else {
                                            (status & 0x08) != 0
                                        };
                                        let is_right_in_ear = if xor_factor {
                                            (status & 0x08) != 0
                                        } else {
                                            (status & 0x02) != 0
                                        };
                                        let is_flipped = !primary_left;

                                        let left_byte_index = if is_flipped { 2 } else { 1 };
                                        let right_byte_index = if is_flipped { 1 } else { 2 };

                                        let left_byte = decrypted[left_byte_index] as i32;
                                        let right_byte = decrypted[right_byte_index] as i32;
                                        let case_byte = decrypted[3] as i32;

                                        let (left_battery, left_charging) = if left_byte == 0xff {
                                            (0, false)
                                        } else {
                                            (left_byte & 0x7F, (left_byte & 0x80) != 0)
                                        };
                                        let (right_battery, right_charging) = if right_byte == 0xff
                                        {
                                            (0, false)
                                        } else {
                                            (right_byte & 0x7F, (right_byte & 0x80) != 0)
                                        };
                                        let (case_battery, case_charging) = if case_byte == 0xff {
                                            (0, false)
                                        } else {
                                            (case_byte & 0x7F, (case_byte & 0x80) != 0)
                                        };

                                        let battery_bytes = (left_byte, right_byte, case_byte);
                                        if last_battery != Some(battery_bytes) {
                                            info!(
                                                "BLE battery update: case={}",
                                                if case_byte == 0xff {
                                                    "disconnected".to_string()
                                                } else {
                                                    format!(
                                                        "{}% (charging: {})",
                                                        case_battery, case_charging
                                                    )
                                                }
                                            );
                                            last_battery = Some(battery_bytes);
                                        }

                                        if let Some(handle) = &tray_handle_clone {
                                            handle
                                                .update(|tray: &mut MyTray| {
                                                    let now =
                                                        crate::bluetooth::battery_telemetry::now_ms(
                                                        );
                                                    if let Some(pair) =
                                                        matched_airpods_mac.as_deref()
                                                    {
                                                        for (name, byte) in
                                                            ["left", "right", "case"]
                                                                .into_iter()
                                                                .zip([
                                                                    left_byte, right_byte,
                                                                    case_byte,
                                                                ])
                                                        {
                                                            tray.telemetry.update(
                                                                pair, "earbuds", name, byte as u8,
                                                                now,
                                                            );
                                                        }
                                                        let _ = tray.telemetry.publish(now);
                                                    }
                                                    if tray.active_airpods.is_none() {
                                                        tray.active_airpods =
                                                            matched_airpods_mac.clone();
                                                    }
                                                    if tray.active_airpods != matched_airpods_mac {
                                                        return;
                                                    }
                                                    tray.battery_l = if left_byte == 0xff {
                                                        None
                                                    } else {
                                                        Some(left_battery as u8)
                                                    };
                                                    tray.battery_l_status = if left_byte == 0xff {
                                                        Some(BatteryStatus::Disconnected)
                                                    } else if left_charging {
                                                        Some(BatteryStatus::Charging)
                                                    } else {
                                                        Some(BatteryStatus::NotCharging)
                                                    };
                                                    tray.battery_r = if right_byte == 0xff {
                                                        None
                                                    } else {
                                                        Some(right_battery as u8)
                                                    };
                                                    tray.battery_r_status = if right_byte == 0xff {
                                                        Some(BatteryStatus::Disconnected)
                                                    } else if right_charging {
                                                        Some(BatteryStatus::Charging)
                                                    } else {
                                                        Some(BatteryStatus::NotCharging)
                                                    };
                                                    tray.battery_c = if case_byte == 0xff {
                                                        None
                                                    } else {
                                                        Some(case_battery as u8)
                                                    };
                                                    tray.battery_c_status = if case_byte == 0xff {
                                                        Some(BatteryStatus::Disconnected)
                                                    } else if case_charging {
                                                        Some(BatteryStatus::Charging)
                                                    } else {
                                                        Some(BatteryStatus::NotCharging)
                                                    };
                                                })
                                                .await;
                                        }

                                        debug!(
                                            "Battery status: Left: {}, Right: {}, Case: {}, InEar: L:{} R:{}",
                                            if left_byte == 0xff {
                                                "disconnected".to_string()
                                            } else {
                                                format!(
                                                    "{}% (charging: {})",
                                                    left_battery, left_charging
                                                )
                                            },
                                            if right_byte == 0xff {
                                                "disconnected".to_string()
                                            } else {
                                                format!(
                                                    "{}% (charging: {})",
                                                    right_battery, right_charging
                                                )
                                            },
                                            if case_byte == 0xff {
                                                "disconnected".to_string()
                                            } else {
                                                format!(
                                                    "{}% (charging: {})",
                                                    case_battery, case_charging
                                                )
                                            },
                                            is_left_in_ear,
                                            is_right_in_ear
                                        );
                                    }
                                }
                            }
                        }
                    }
                });
                watchers.insert(addr, task);
            }
        }
    }
    for (_, task) in watchers {
        task.abort();
    }

    Ok(())
}
