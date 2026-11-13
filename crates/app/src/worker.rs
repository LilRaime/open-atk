/* Background USB/HID hardware worker thread and full EEPROM reader. */

#![allow(dead_code)]

use crate::models::{DeviceAction, DeviceEntry, DeviceUpdate, MouseColor};
use atk_device::{AtkDevice, ConnectionMode, DeviceScanner};
use atk_hid::{HidDeviceInfo, HidManager};
use atk_protocol::{
    constants::{ADDR_ADVANCED, ADDR_DPI_BASE, ADDR_LIFT_OFF, ADDR_SENSOR_MODE, ADDR_SYSTEM},
    decode_dpi_axis, PollingRate,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, Instant};

/* Tracks whether the application window is currently focused in the foreground */
pub static APP_FOCUSED: AtomicBool = AtomicBool::new(true);

/* Optional reference to egui context to trigger reactive UI repaint on incoming events */
static EGUI_CTX: std::sync::Mutex<Option<eframe::egui::Context>> = std::sync::Mutex::new(None);

pub fn register_egui_ctx(ctx: eframe::egui::Context) {
    if let Ok(mut lock) = EGUI_CTX.lock() {
        *lock = Some(ctx);
    }
}

pub fn wake_ui() {
    if let Ok(lock) = EGUI_CTX.lock() {
        if let Some(ctx) = lock.as_ref() {
            ctx.request_repaint();
        }
    }
}

pub fn read_device_full(dev: &AtkDevice) -> DeviceUpdate {
    let vendor_id = dev.info().vendor_id;
    let product_id = dev.info().product_id;
    let (resolved_name, _) = DeviceScanner::resolve_device_name_and_mode(
        vendor_id,
        product_id,
        dev.info().product_string.as_deref(),
    );
    let mut model_name = resolved_name.clone();
    let mut cid = 0u8;
    let mut mid = 0u8;
    let mut firmware = "v1.0".to_string();
    let mut battery_pct = 0;
    let mut is_charging = dev.mode() == atk_device::ConnectionMode::Wired;
    let mut voltage_mv = 0;
    let mut rf_id = "00 00 00".to_string();
    let mut polling_rate = 1000;
    let mut stage_count = 4;
    let mut active_stage = 0;
    let mut stages_dpi = vec![(400, 400), (800, 800), (1600, 1600), (3200, 3200)];
    let mut lod_code = 1;
    let mut debounce_ms = 4;
    let mut motion_sync = true;
    let mut ripple_control = false;
    let mut button_mappings = Vec::new();

    if let Ok(id) = dev.query_identity() {
        cid = id.cid;
        mid = id.mid;
        let q_name = id.model_name.trim();
        if !q_name.is_empty() && !q_name.starts_with("Unknown") && q_name != "ATK Gaming Mouse" {
            model_name = q_name.to_uppercase();
        }
    }
    if let Ok(fw) = dev.query_firmware() {
        firmware = format!("v{fw}");
        /* Firmware v3.xx disambiguation: Nordic MCUs (R1 Pro/Max) run v1.xx, Compx MCUs (R1 SE/SE+) run v3.xx */
        let name_upper = model_name.to_uppercase();
        let is_generic_or_wrong = name_upper.contains("GAMING MOUSE")
            || name_upper.contains("PRO MAX")
            || name_upper.contains("DONGLE")
            || name_upper.contains("RECEIVER");
        if (fw.starts_with("3.") || firmware.starts_with("v3.")) && is_generic_or_wrong {
            model_name = "VXE R1SE+".to_string();
        }
    }
    if let Ok((bat, mv)) = dev.query_battery() {
        battery_pct = bat.percentage;
        is_charging = bat.is_charging || dev.mode() == atk_device::ConnectionMode::Wired;
        voltage_mv = mv;
    }
    if let Ok(rcv) = dev.query_receiver() {
        rf_id = rcv.rf_id;
    }
    if let Ok(system) = dev.read_register(ADDR_SYSTEM, 6) {
        if system.len() >= 6 {
            polling_rate = match system[0] {
                0x08 => 125,
                0x04 => 250,
                0x02 => 500,
                0x01 => 1000,
                0x10 => 2000,
                0x20 => 4000,
                0x40 => 8000,
                _ => 1000,
            };
            stage_count = system[2];
            active_stage = system[4];

            let mut stages = Vec::new();
            for s in 0..stage_count {
                let addr = ADDR_DPI_BASE + (s as u16) * 4;
                if let Ok(data) = dev.read_register(addr, 4) {
                    if data.len() >= 4 {
                        let xb = data[0];
                        let yb = data[1];
                        let mode = data[2];
                        let x = decode_dpi_axis(xb, mode & 0x0F);
                        let y = decode_dpi_axis(yb, (mode >> 4) & 0x0F);
                        stages.push((x, y));
                    }
                }
            }
            if !stages.is_empty() {
                stages_dpi = stages;
            }
        }
    }
    let mut straight_line_correction = false;
    let mut ultra_long_range = false;
    let mut sleep_time_idx = 1;
    let mut dpi_light_mode = 0;
    let mut dpi_light_brightness = 2;
    let mut dpi_light_speed = 1;

    if let Ok(lod) = dev.read_register(ADDR_LIFT_OFF, 2) {
        if lod.len() >= 2 && lod[0].wrapping_add(lod[1]) == 0x55 {
            lod_code = lod[0];
        }
    }
    if let Ok(adv) = dev.read_register(ADDR_ADVANCED, 10) {
        if adv.len() >= 10 {
            if adv[0].wrapping_add(adv[1]) == 0x55 {
                debounce_ms = adv[0];
            }
            if adv[2].wrapping_add(adv[3]) == 0x55 {
                motion_sync = adv[2] != 0;
            }
            if adv[4].wrapping_add(adv[5]) == 0x55 && adv[4] > 0 {
                let sleep_units = adv[4];
                sleep_time_idx = match sleep_units {
                    0..=4 => 0,
                    5..=9 => 1,
                    10..=15 => 2,
                    16..=24 => 3,
                    25..=90 => 4,
                    91..=135 => 5,
                    136..=165 => 6,
                    _ => 7,
                };
            }
            if adv[6].wrapping_add(adv[7]) == 0x55 {
                straight_line_correction = adv[6] != 0;
            }
            if adv[8].wrapping_add(adv[9]) == 0x55 {
                ripple_control = adv[8] != 0;
            }
        }
    }
    if let Ok(sensor_row) = dev.read_register(ADDR_SENSOR_MODE, 6) {
        if sensor_row.len() >= 6 && sensor_row[4].wrapping_add(sensor_row[5]) == 0x55 {
            ultra_long_range = sensor_row[4] != 0;
        }
    }
    if let Ok((mode, bright, speed)) = dev.get_dpi_light() {
        dpi_light_mode = mode;
        dpi_light_brightness = bright;
        dpi_light_speed = speed;
    }
    if let Ok(mappings) = dev.read_button_mappings() {
        button_mappings = mappings;
    }
    let stage_colors = dev.read_stage_colors(stage_count).unwrap_or_else(|_| {
        vec![
            (155, 93, 229),
            (255, 255, 255),
            (0, 180, 255),
            (255, 0, 128),
        ]
    });

    DeviceUpdate::FullRefresh {
        device_id: dev.info().path.clone(),
        model_name,
        vendor_id,
        product_id,
        cid,
        mid,
        firmware,
        battery_pct,
        is_charging,
        voltage_mv,
        rf_id,
        polling_rate,
        active_stage,
        stage_count,
        stages_dpi,
        stage_colors,
        lod_code,
        debounce_ms,
        motion_sync,
        straight_line_correction,
        ripple_control,
        ultra_long_range,
        sleep_time_idx,
        dpi_light_mode,
        dpi_light_brightness,
        dpi_light_speed,
        button_mappings,
    }
}

pub fn spawn_device_worker(action_rx: Receiver<DeviceAction>, update_tx: Sender<DeviceUpdate>) {
    /* Background worker thread for real hardware EEPROM I/O with persistent device handle */
    std::thread::spawn(move || {
        struct WakingSender(Sender<DeviceUpdate>);
        impl WakingSender {
            fn send(&self, update: DeviceUpdate) -> Result<(), std::sync::mpsc::SendError<DeviceUpdate>> {
                let res = self.0.send(update);
                wake_ui();
                res
            }
        }
        let update_tx = WakingSender(update_tx);

        let mut hid_mgr = HidManager::new().ok();
        let mut active_dev: Option<AtkDevice> = None;
        let mut active_path = String::new();
        let mut preferred_device_id: Option<String> = None;
        let mut had_wired = false;
        let mut was_connected = false;
        let mut last_scan_time = Instant::now() - Duration::from_secs(10);
        let mut last_bat_poll = Instant::now() - Duration::from_secs(10);
        let mut last_stage_poll = Instant::now() - Duration::from_secs(10);
        let mut last_scanned_paths: Vec<String> = Vec::new();
        let mut periodic_device_list_timer = Instant::now();

        let mut cached_stage = 0u8;
        let mut cached_count = 4u8;
        let mut cached_bat = 0u8;
        let mut cached_chg = false;
        let mut cached_volt = 0u16;
        let mut consecutive_failures = 0u32;
        let mut was_focused = true;

        /* Helper function to scan for connected ATK / VXE devices */
        let scan_connected_devices = |mgr: &mut HidManager, preferred: Option<&str>, current_active: &str, active_d: Option<&AtkDevice>| -> (Vec<DeviceEntry>, Option<(HidDeviceInfo, ConnectionMode)>) {
            let _ = mgr.refresh();
            let mut channels = DeviceScanner::scan_config_channels(mgr);
            /* Prioritize Wired connections over WirelessDongles unless user explicitly preferred a device */
            channels.sort_by_key(|d| match d.mode {
                ConnectionMode::Wired => 0,
                ConnectionMode::WirelessDongle => 1,
                ConnectionMode::Bluetooth => 2,
                ConnectionMode::Unknown => 3,
            });

            let has_wired = channels.iter().any(|c| c.mode == ConnectionMode::Wired);
            let mut entries = Vec::new();
            let mut primary = None;

            for target in &channels {
                if entries.iter().any(|e: &DeviceEntry| e.id == target.info.path) {
                    continue;
                }

                let is_active_or_preferred = target.info.path == current_active
                    || preferred.map(|p| p.eq_ignore_ascii_case(&target.info.path) || target.info.path.contains(p) || p.contains(&target.info.path)).unwrap_or(false);

                let (resolved_name, resolved_mode) = DeviceScanner::resolve_device_name_and_mode(
                    target.info.vendor_id,
                    target.info.product_id,
                    target.info.product_string.as_deref(),
                );

                /* If a wired mouse is plugged in, check whether wireless dongles of the SAME model family are currently idle */
                if has_wired && target.mode == ConnectionMode::WirelessDongle && !is_active_or_preferred {
                    /* Only check duplicate suppression if a wired connection of the exact same model exists */
                    let same_model_wired_exists = channels.iter().any(|c| {
                        if c.mode != ConnectionMode::Wired {
                            return false;
                        }
                        let (w_name, _) = DeviceScanner::resolve_device_name_and_mode(
                            c.info.vendor_id,
                            c.info.product_id,
                            c.info.product_string.as_deref(),
                        );
                        w_name.eq_ignore_ascii_case(&resolved_name)
                    });

                    if same_model_wired_exists {
                        let mut is_active_dongle = false;
                        if let Ok(dev) = AtkDevice::open_with_info(mgr, target.info.clone(), target.mode) {
                            if let Ok(rcv) = dev.query_receiver() {
                                is_active_dongle = rcv.online;
                            }
                        }
                        if !is_active_dongle {
                            /* The dongle is idle because this exact mouse model is connected via USB cable to charge. Suppress duplicate idle dongle entry. */
                            continue;
                        }
                    }
                }

                let mut name = resolved_name.clone();
                let mut bat_pct = 0u8;
                let mut is_chg = target.mode == ConnectionMode::Wired;

                /* If device is already open as active_dev, query it directly to avoid lock contention */
                if target.info.path == current_active && active_d.is_some() {
                    let dev = active_d.unwrap();
                    let mut fw_ver = String::new();
                    if let Ok(fw) = dev.query_firmware() {
                        fw_ver = fw;
                    }
                    if let Ok(id) = dev.query_identity() {
                        let q_name = id.model_name.trim();
                        if !q_name.is_empty() && !q_name.starts_with("Unknown") && q_name != "ATK Gaming Mouse" {
                            name = q_name.to_uppercase();
                        }
                    }
                    let name_upper = name.to_uppercase();
                    let is_generic_or_wrong = name_upper.contains("GAMING MOUSE")
                        || name_upper.contains("PRO MAX")
                        || name_upper.contains("DONGLE")
                        || name_upper.contains("RECEIVER");
                    if (fw_ver.starts_with("3.") || fw_ver.starts_with("v3.")) && is_generic_or_wrong {
                        name = "VXE R1SE+".to_string();
                    }
                    if let Ok((bat, _mv)) = dev.query_battery() {
                        bat_pct = bat.percentage;
                        is_chg = bat.is_charging || target.mode == ConnectionMode::Wired;
                    }
                } else if let Ok(dev) = AtkDevice::open_with_info(mgr, target.info.clone(), target.mode) {
                    let mut fw_ver = String::new();
                    if let Ok(fw) = dev.query_firmware() {
                        fw_ver = fw;
                    }
                    if let Ok(id) = dev.query_identity() {
                        let q_name = id.model_name.trim();
                        if !q_name.is_empty() && !q_name.starts_with("Unknown") && q_name != "ATK Gaming Mouse" {
                            name = q_name.to_uppercase();
                        }
                    }
                    let name_upper = name.to_uppercase();
                    let is_generic_or_wrong = name_upper.contains("GAMING MOUSE")
                        || name_upper.contains("PRO MAX")
                        || name_upper.contains("DONGLE")
                        || name_upper.contains("RECEIVER");
                    if (fw_ver.starts_with("3.") || fw_ver.starts_with("v3.")) && is_generic_or_wrong {
                        name = "VXE R1SE+".to_string();
                    }
                    if let Ok((bat, _mv)) = dev.query_battery() {
                        bat_pct = bat.percentage;
                        is_chg = bat.is_charging || target.mode == ConnectionMode::Wired;
                    }
                }

                if is_active_or_preferred && primary.is_none() {
                    primary = Some((target.info.clone(), target.mode));
                }

                let color = MouseColor::White;

                entries.push(DeviceEntry {
                    id: target.info.path.clone(),
                    model_name: name,
                    mode: resolved_mode,
                    battery_pct: bat_pct,
                    is_charging: is_chg,
                    color,
                });
            }

            if primary.is_none() {
                if let Some(target) = channels.first() {
                    primary = Some((target.info.clone(), target.mode));
                }
            }

            (entries, primary)
        };

        /* Initial hardware scan on startup */
        if let Some(mgr) = &mut hid_mgr {
            had_wired = DeviceScanner::scan_config_channels(mgr).iter().any(|d| d.mode == ConnectionMode::Wired);
            let mut scan_res = scan_connected_devices(mgr, None, &active_path, active_dev.as_ref());
            /* If initial scan only found 1 device or none, give Windows HID subsystem a brief moment (80ms) and re-check in case a second device is still enumerating */
            if scan_res.0.len() <= 1 {
                std::thread::sleep(Duration::from_millis(80));
                let _ = mgr.refresh();
                let retry_res = scan_connected_devices(mgr, None, &active_path, active_dev.as_ref());
                if retry_res.0.len() > scan_res.0.len() {
                    scan_res = retry_res;
                }
            }
            let (entries, primary) = scan_res;
            last_scanned_paths = entries.iter().map(|e| e.id.clone()).collect();
            if !entries.is_empty() {
                if let Some((info, mode)) = primary {
                    if let Ok(dev) = AtkDevice::open_with_info(mgr, info.clone(), mode) {
                        active_path = info.path;
                        /* Immediately query battery & active stage first and push to UI so battery % appears in <50ms */
                        if let Ok((bat, mv)) = dev.query_battery() {
                            cached_bat = bat.percentage;
                            cached_chg = bat.is_charging || mode == ConnectionMode::Wired;
                            cached_volt = mv;
                        }
                        let _ = update_tx.send(DeviceUpdate::DeviceList(entries));
                        if cached_bat > 0 {
                            let _ = update_tx.send(DeviceUpdate::Status {
                                device_id: active_path.clone(),
                                battery_pct: cached_bat,
                                is_charging: cached_chg,
                                voltage_mv: cached_volt,
                                active_stage: cached_stage,
                                stage_count: cached_count,
                            });
                        }
                        let full = read_device_full(&dev);
                        if let DeviceUpdate::FullRefresh {
                            active_stage,
                            stage_count,
                            battery_pct,
                            is_charging,
                            voltage_mv,
                            ..
                        } = &full
                        {
                            cached_stage = *active_stage;
                            cached_count = *stage_count;
                            if *battery_pct > 0 {
                                cached_bat = *battery_pct;
                                cached_chg = *is_charging;
                                cached_volt = *voltage_mv;
                            }
                        }
                        let _ = update_tx.send(full);
                        active_dev = Some(dev);
                        was_connected = true;
                    } else {
                        let _ = update_tx.send(DeviceUpdate::DeviceList(entries));
                    }
                } else {
                    let _ = update_tx.send(DeviceUpdate::DeviceList(entries));
                }
            } else {
                let _ = update_tx.send(DeviceUpdate::Disconnected);
                was_connected = false;
            }
        } else {
            let _ = update_tx.send(DeviceUpdate::Disconnected);
            was_connected = false;
        }

        loop {
            /* Adaptive timeout: 30ms for low latency when focused, 500ms when minimized/background to save CPU */
            let timeout = if APP_FOCUSED.load(Ordering::Relaxed) {
                Duration::from_millis(30)
            } else {
                Duration::from_millis(500)
            };
            match action_rx.recv_timeout(timeout) {
                Ok(action) => {
                    if active_dev.is_none() {
                        if let Some(mgr) = &mut hid_mgr {
                            let (entries, primary) = scan_connected_devices(mgr, preferred_device_id.as_deref(), &active_path, active_dev.as_ref());
                            if !entries.is_empty() {
                                let _ = update_tx.send(DeviceUpdate::DeviceList(entries));
                                if let Some((info, mode)) = primary {
                                    if let Ok(dev) = AtkDevice::open_with_info(mgr, info.clone(), mode) {
                                        active_path = info.path;
                                        active_dev = Some(dev);
                                        was_connected = true;
                                    }
                                }
                            }
                        }
                    }

                    let dev = match &active_dev {
                        Some(d) => d,
                        None => {
                            let _ = update_tx.send(DeviceUpdate::Toast("No device connected".into(), true));
                            continue;
                        }
                    };

                    let result = match action {
                        DeviceAction::SetActiveStage(s) => {
                            cached_stage = s;
                            let res = dev.set_active_stage(s);
                            let _ = dev.commit_profile();
                            if let Ok(sys) = dev.read_register(ADDR_SYSTEM, 6) {
                                if sys.len() >= 6 {
                                    cached_stage = sys[4];
                                    cached_count = sys[2];
                                    let _ = update_tx.send(DeviceUpdate::Status {
                                        device_id: active_path.clone(),
                                        battery_pct: cached_bat,
                                        is_charging: cached_chg,
                                        voltage_mv: cached_volt,
                                        active_stage: cached_stage,
                                        stage_count: cached_count,
                                    });
                                }
                            }
                            res
                        }
                        DeviceAction::SetStageDpi { stage, x, y } => dev.set_stage_dpi(stage, x, y),
                        DeviceAction::SetStageColor { stage, r, g, b } => dev.set_stage_color(stage, r, g, b),
                        DeviceAction::SetStageCount(count) => dev.set_stage_count(count),
                        DeviceAction::SetPolling(hz) => {
                            if let Ok(p) = PollingRate::from_hz(hz) {
                                dev.set_polling_rate(p)
                            } else {
                                Ok(())
                            }
                        }
                        DeviceAction::SetDebounce(ms) => dev.set_debounce(ms),
                        DeviceAction::SetMotionSync(enabled) => dev.set_motion_sync(enabled),
                        DeviceAction::SetStraightLine(enabled) => dev.set_straight_line(enabled),
                        DeviceAction::SetRipple(enabled) => dev.set_ripple_control(enabled),
                        DeviceAction::SetUltraLongRange(enabled) => dev.set_ultra_long_range(enabled),
                        DeviceAction::SetSleepTime(idx) => dev.set_sleep_time(idx as u8),
                        DeviceAction::SetDpiLight { mode, brightness, speed } => dev.set_dpi_light(mode, brightness, speed),
                        DeviceAction::SetLod(code) => dev.set_lod(code),
                        DeviceAction::SetButton { btn_idx, key_class, val1, val2 } => {
                            dev.set_button_mapping(btn_idx, key_class, val1, val2)
                        }
                        DeviceAction::CommitProfile => dev.commit_profile(),
                        DeviceAction::RestoreDefaults => {
                            let defaults = [
                                (0, 0x01, 0x01, 0x00),
                                (1, 0x01, 0x02, 0x00),
                                (2, 0x01, 0x04, 0x00),
                                (3, 0x01, 0x08, 0x00),
                                (4, 0x01, 0x10, 0x00),
                                (5, 0x02, 0x01, 0x00),
                            ];
                            for (idx, kc, v1, v2) in defaults {
                                let _ = dev.set_button_mapping(idx, kc, v1, v2);
                            }
                            let full = read_device_full(dev);
                            let _ = update_tx.send(full);
                            let _ = update_tx.send(DeviceUpdate::Toast("Restored factory button mappings".into(), false));
                            Ok(())
                        }
                        DeviceAction::RefreshHardware => {
                            if let Some(mgr) = &mut hid_mgr {
                                let (entries, _) = scan_connected_devices(mgr, preferred_device_id.as_deref(), &active_path, active_dev.as_ref());
                                if entries.is_empty() {
                                    active_dev = None;
                                    active_path.clear();
                                    was_connected = false;
                                    let _ = update_tx.send(DeviceUpdate::Disconnected);
                                    let _ = update_tx.send(DeviceUpdate::Toast("No ATK devices found".into(), true));
                                } else {
                                    let _ = update_tx.send(DeviceUpdate::DeviceList(entries));
                                    if let Some(dev) = &active_dev {
                                        let full = read_device_full(dev);
                                        let _ = update_tx.send(full);
                                        let _ = update_tx.send(DeviceUpdate::Toast("Hardware refreshed from EEPROM".into(), false));
                                    }
                                }
                            }
                            Ok(())
                        }
                        DeviceAction::SwitchDevice(target_id) => {
                            preferred_device_id = if target_id.is_empty() { None } else { Some(target_id.clone()) };
                            if let Some(mgr) = &mut hid_mgr {
                                /* Explicitly release previous device handle */
                                active_dev = None;
                                active_path.clear();

                                let _ = mgr.refresh();
                                let channels = DeviceScanner::scan_config_channels(mgr);
                                for t in channels {
                                    let matches = t.info.path.eq_ignore_ascii_case(&target_id)
                                        || target_id.is_empty()
                                        || t.info.path.contains(&target_id)
                                        || target_id.contains(&t.info.path);
                                    if matches {
                                        let mut opened = AtkDevice::open_with_info(mgr, t.info.clone(), t.mode);
                                        if opened.is_err() {
                                            std::thread::sleep(Duration::from_millis(60));
                                            opened = AtkDevice::open_with_info(mgr, t.info.clone(), t.mode);
                                        }
                                        if let Ok(d) = opened {
                                            active_path = t.info.path.clone();
                                            let full = read_device_full(&d);
                                            if let DeviceUpdate::FullRefresh {
                                                active_stage,
                                                stage_count,
                                                battery_pct,
                                                is_charging,
                                                voltage_mv,
                                                ..
                                            } = &full
                                            {
                                                cached_stage = *active_stage;
                                                cached_count = *stage_count;
                                                if *battery_pct > 0 {
                                                    cached_bat = *battery_pct;
                                                    cached_chg = *is_charging;
                                                    cached_volt = *voltage_mv;
                                                }
                                            }
                                            let _ = update_tx.send(full);
                                            active_dev = Some(d);
                                            was_connected = true;
                                            break;
                                        }
                                    }
                                }
                            }
                            Ok(())
                        }
                    };

                    if let Err(e) = result {
                        tracing::warn!("Device action error: {e}");
                    }
                }
                Err(RecvTimeoutError::Timeout) => {
                    let is_focused = APP_FOCUSED.load(Ordering::Relaxed);
                    if let Some(dev) = &active_dev {
                        /* When window regains focus after being in background, sync full hardware state immediately */
                        if is_focused && !was_focused {
                            was_focused = true;
                            last_stage_poll = Instant::now();
                            let full = read_device_full(dev);
                            if let DeviceUpdate::FullRefresh {
                                active_stage,
                                stage_count,
                                battery_pct,
                                is_charging,
                                voltage_mv,
                                ..
                            } = &full
                            {
                                cached_stage = *active_stage;
                                cached_count = *stage_count;
                                cached_bat = *battery_pct;
                                cached_chg = *is_charging;
                                cached_volt = *voltage_mv;
                            }
                            let _ = update_tx.send(full);
                        } else if !is_focused {
                            was_focused = false;
                        }

                        /* Fast 80ms stage check ONLY when window is focused so physical DPI switches are instant
                         * while background operation sends 0 packets and never interferes with Chrome ATK Web Hub */
                        if is_focused && last_stage_poll.elapsed() >= Duration::from_millis(80) {
                            last_stage_poll = Instant::now();
                            match dev.read_register(ADDR_SYSTEM, 6) {
                                Ok(sys) => {
                                    consecutive_failures = 0;
                                    if sys.len() >= 6 {
                                        let current_stage = sys[4];
                                        let current_count = sys[2];
                                        if current_stage != cached_stage || current_count != cached_count {
                                            cached_stage = current_stage;
                                            cached_count = current_count;
                                            let _ = update_tx.send(DeviceUpdate::Status {
                                                device_id: active_path.clone(),
                                                battery_pct: cached_bat,
                                                is_charging: cached_chg,
                                                voltage_mv: cached_volt,
                                                active_stage: current_stage,
                                                stage_count: current_count,
                                            });
                                        }
                                    }
                                }
                                Err(_) => {
                                    consecutive_failures += 1;
                                    if consecutive_failures >= 2 {
                                        consecutive_failures = 0;
                                        active_dev = None;
                                        active_path.clear();
                                        if let Some(mgr) = &mut hid_mgr {
                                            let (entries, primary) = scan_connected_devices(mgr, preferred_device_id.as_deref(), &active_path, None);
                                            if entries.is_empty() {
                                                let _ = update_tx.send(DeviceUpdate::Disconnected);
                                                was_connected = false;
                                            } else {
                                                let _ = update_tx.send(DeviceUpdate::DeviceList(entries));
                                                if let Some((info, mode)) = primary {
                                                    if let Ok(dev) = AtkDevice::open_with_info(mgr, info.clone(), mode) {
                                                        active_path = info.path;
                                                        let full = read_device_full(&dev);
                                                        if let DeviceUpdate::FullRefresh {
                                                            active_stage,
                                                            stage_count,
                                                            battery_pct,
                                                            is_charging,
                                                            voltage_mv,
                                                            ..
                                                        } = &full
                                                        {
                                                            cached_stage = *active_stage;
                                                            cached_count = *stage_count;
                                                            if *battery_pct > 0 {
                                                                cached_bat = *battery_pct;
                                                                cached_chg = *is_charging;
                                                                cached_volt = *voltage_mv;
                                                            }
                                                        }
                                                        let _ = update_tx.send(full);
                                                        active_dev = Some(dev);
                                                        was_connected = true;
                                                    }
                                                }
                                            }
                                        } else {
                                            let _ = update_tx.send(DeviceUpdate::Disconnected);
                                            was_connected = false;
                                        }
                                    }
                                }
                            }
                        }

                        /* Fast 500ms battery poll if battery is still 0% (mouse waking up), else every 15s while focused */
                        let bat_poll_interval = if cached_bat == 0 {
                            Duration::from_millis(500)
                        } else {
                            Duration::from_secs(15)
                        };
                        if is_focused && active_dev.is_some() && last_bat_poll.elapsed() > bat_poll_interval {
                            last_bat_poll = Instant::now();
                            if let Some(d) = &active_dev {
                                if let Ok((bat, mv)) = d.query_battery() {
                                    cached_bat = bat.percentage;
                                    cached_chg = bat.is_charging;
                                    cached_volt = mv;
                                    let _ = update_tx.send(DeviceUpdate::Status {
                                        device_id: active_path.clone(),
                                        battery_pct: cached_bat,
                                        is_charging: cached_chg,
                                        voltage_mv: cached_volt,
                                        active_stage: cached_stage,
                                        stage_count: cached_count,
                                    });
                                }
                            }
                        }

                        /* Check periodically (every 1s) for hardware topology changes */
                        if last_scan_time.elapsed() >= Duration::from_millis(1000) {
                            last_scan_time = Instant::now();
                            if let Some(mgr) = &mut hid_mgr {
                                let _ = mgr.refresh();
                                let current_channels = DeviceScanner::scan_config_channels(mgr);
                                let current_wired = current_channels
                                    .iter()
                                    .any(|d| d.mode == ConnectionMode::Wired);
                                let cable_just_plugged = current_wired && !had_wired;
                                had_wired = current_wired;

                                let current_paths: Vec<String> = current_channels.iter().map(|c| c.info.path.clone()).collect();
                                let topology_changed = current_paths != last_scanned_paths;

                                /* Only auto-switch to cable if cable was JUST plugged in and user didn't explicitly lock onto a different device */
                                if cable_just_plugged && preferred_device_id.is_none() {
                                    active_dev = None;
                                    active_path.clear();
                                    last_scanned_paths = current_paths;
                                    let (entries, primary) = scan_connected_devices(mgr, None, &active_path, None);
                                    let _ = update_tx.send(DeviceUpdate::DeviceList(entries));
                                    if let Some((info, mode)) = primary {
                                        if let Ok(dev) = AtkDevice::open_with_info(mgr, info.clone(), mode) {
                                            active_path = info.path;
                                            let full = read_device_full(&dev);
                                            if let DeviceUpdate::FullRefresh {
                                                active_stage,
                                                stage_count,
                                                battery_pct,
                                                is_charging,
                                                voltage_mv,
                                                ..
                                            } = &full
                                            {
                                                cached_stage = *active_stage;
                                                cached_count = *stage_count;
                                                if *battery_pct > 0 {
                                                    cached_bat = *battery_pct;
                                                    cached_chg = *is_charging;
                                                    cached_volt = *voltage_mv;
                                                }
                                            }
                                            let _ = update_tx.send(full);
                                            active_dev = Some(dev);
                                            was_connected = true;
                                        }
                                    }
                                } else if topology_changed || periodic_device_list_timer.elapsed() >= Duration::from_secs(3) {
                                    /* Periodic background refresh of device list so newly connected / awakened mice appear automatically without clicking Refresh */
                                    last_scanned_paths = current_paths;
                                    periodic_device_list_timer = Instant::now();
                                    let (entries, _) = scan_connected_devices(mgr, preferred_device_id.as_deref(), &active_path, active_dev.as_ref());
                                    if !entries.is_empty() {
                                        let _ = update_tx.send(DeviceUpdate::DeviceList(entries));
                                    }
                                }
                            }
                        }
                    } else {
                        /* When no device is active: re-scan periodically every 800ms to detect newly plugged devices */
                        if last_scan_time.elapsed() >= Duration::from_millis(800) {
                            last_scan_time = Instant::now();
                            if let Some(mgr) = &mut hid_mgr {
                                let (entries, primary) = scan_connected_devices(mgr, preferred_device_id.as_deref(), &active_path, None);
                                if !entries.is_empty() {
                                    let _ = update_tx.send(DeviceUpdate::DeviceList(entries));
                                    if let Some((info, mode)) = primary {
                                        if let Ok(dev) = AtkDevice::open_with_info(mgr, info.clone(), mode) {
                                            active_path = info.path;
                                            let full = read_device_full(&dev);
                                            if let DeviceUpdate::FullRefresh {
                                                active_stage,
                                                stage_count,
                                                battery_pct,
                                                is_charging,
                                                voltage_mv,
                                                ..
                                            } = &full
                                            {
                                                cached_stage = *active_stage;
                                                cached_count = *stage_count;
                                                if *battery_pct > 0 {
                                                    cached_bat = *battery_pct;
                                                    cached_chg = *is_charging;
                                                    cached_volt = *voltage_mv;
                                                }
                                            }
                                            let _ = update_tx.send(full);
                                            active_dev = Some(dev);
                                            was_connected = true;
                                        }
                                    }
                                } else if was_connected {
                                    let _ = update_tx.send(DeviceUpdate::Disconnected);
                                    was_connected = false;
                                }
                            } else {
                                hid_mgr = HidManager::new().ok();
                            }
                        }
                    }
                }
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
    });
}

