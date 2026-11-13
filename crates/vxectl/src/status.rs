/*
 * Hardware status, telemetry rendering, and battery monitoring.
 */

use atk_device::{AtkDevice, ConnectionMode};
use atk_hid::HidManager;
use atk_protocol::{
    constants::{ADDR_DPI_BASE, ADDR_SYSTEM},
    decode_dpi_axis,
};
use crate::device::CliDeviceInfo;
use crate::theme::{Card, Theme};
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct HardwareIdJson {
    pub cid: u8,
    pub mid: u8,
}

#[derive(Serialize, Debug, Clone)]
pub struct BatteryJson {
    pub percentage: u8,
    pub charging: bool,
    pub voltage_mv: u16,
    pub voltage_v: f32,
}

#[derive(Serialize, Debug, Clone)]
pub struct StageStatusJson {
    pub stage: u8,
    pub dpi_x: u32,
    pub dpi_y: u32,
    pub rgb: String,
    pub active: bool,
}

#[derive(Serialize, Debug, Clone)]
pub struct TuningStatusJson {
    pub lod_mm: f32,
    pub debounce_ms: u8,
    pub motion_sync: bool,
    pub straight_line: bool,
    pub ripple_control: bool,
    pub ultra_long_range: bool,
    pub dpi_light_effect: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sleep_timeout_sec: Option<u16>,
}

#[derive(Serialize, Debug, Clone)]
pub struct DeviceStatusJson {
    pub index: usize,
    pub model: String,
    pub connection_mode: String,
    pub firmware_version: String,
    pub usb_vid: String,
    pub usb_pid: String,
    pub usb_device: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver_rf_id: Option<String>,
    pub is_sleeping: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hardware_identity: Option<HardwareIdJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub battery: Option<BatteryJson>,
    pub aliases: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub polling_rate_hz: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_stage: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage_count: Option<u8>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dpi_stages: Vec<StageStatusJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tuning: Option<TuningStatusJson>,
}

fn format_usb_vid_pid(vid: u16, pid: u16) -> String {
    match pid {
        0xF58A => format!("0x{vid:04X} / 0x{pid:04X} (Compx Nordic 1K)"),
        0xF58C => format!("0x{vid:04X} / 0x{pid:04X} (Nordic 4K/8K)"),
        0xF58E => format!("0x{vid:04X} / 0x{pid:04X} (VXE R1SE+ Wireless)"),
        0xF58F => format!("0x{vid:04X} / 0x{pid:04X} (VXE R1SE+ Wired)"),
        0xF588 => format!("0x{vid:04X} / 0x{pid:04X} (VXE R1 Pro Wireless)"),
        0xF589 => format!("0x{vid:04X} / 0x{pid:04X} (VXE R1 Pro Wired)"),
        0xF580 => format!("0x{vid:04X} / 0x{pid:04X} (VXE R1 SE Wireless)"),
        0xF581 => format!("0x{vid:04X} / 0x{pid:04X} (VXE R1 SE Wired)"),
        0x1085 => format!("0x{vid:04X} / 0x{pid:04X} (VXE Wireless Dongle)"),
        _ => format!("0x{vid:04X} / 0x{pid:04X}"),
    }
}

fn format_sleep_timeout(sec: u16) -> String {
    if sec == 0 {
        "Disabled".to_string()
    } else if sec < 60 {
        format!("{sec} sec")
    } else if sec % 60 == 0 {
        format!("{} min", sec / 60)
    } else {
        format!("{}m {}s", sec / 60, sec % 60)
    }
}

/* Builds a structured JSON representation of the device hardware state */
pub fn get_status_json(dev: &AtkDevice, target: &CliDeviceInfo) -> DeviceStatusJson {
    let mut identity = dev.query_identity().ok();
    let mut fw = dev.query_firmware().ok();

    if identity.is_none() && fw.is_none() && dev.mode() == ConnectionMode::WirelessDongle {
        std::thread::sleep(std::time::Duration::from_millis(60));
        identity = dev.query_identity().ok();
        fw = dev.query_firmware().ok();
    }

    let fw_str = fw.map(|f| format!("v{f}")).unwrap_or_else(|| "Unknown".to_string());
    let mut model_name = match identity.as_ref().map(|i| i.model_name.trim()) {
        Some(name) if !name.is_empty() && name != "ATK Gaming Mouse" && !name.starts_with("Unknown") => {
            name.to_string()
        }
        _ => target.model_name.clone(),
    };

    let name_upper = model_name.to_uppercase();
    let is_generic_or_wrong = name_upper.contains("GAMING MOUSE")
        || name_upper.contains("PRO MAX")
        || name_upper.contains("DONGLE")
        || name_upper.contains("RECEIVER");
    if (fw_str.starts_with("v3.") || fw_str.starts_with("3.")) && is_generic_or_wrong {
        model_name = "VXE R1SE+".to_string();
    }

    let is_sleeping = identity.is_none() && dev.mode() == ConnectionMode::WirelessDongle;

    let mode_str = match dev.mode() {
        ConnectionMode::Wired => "Wired USB-C",
        ConnectionMode::WirelessDongle => "2.4 GHz Wireless",
        ConnectionMode::Bluetooth => "Bluetooth",
        ConnectionMode::Unknown => "Unknown",
    };

    let hw_id = identity.as_ref().and_then(|id| {
        if id.cid > 0 || id.mid > 0 {
            Some(HardwareIdJson { cid: id.cid, mid: id.mid })
        } else {
            None
        }
    });

    let battery = dev.query_battery().ok().map(|(bat, mv)| {
        let is_charging = bat.is_charging || dev.mode() == ConnectionMode::Wired;
        BatteryJson {
            percentage: bat.percentage,
            charging: is_charging,
            voltage_mv: mv,
            voltage_v: mv as f32 / 1000.0,
        }
    });

    let aliases: Vec<String> = target
        .aliases
        .iter()
        .filter(|a| a.len() <= 7 && !a.is_empty())
        .take(5)
        .cloned()
        .collect();

    let mut polling_rate_hz = None;
    let mut active_stage = None;
    let mut stage_count_val = None;
    let mut dpi_stages = Vec::new();
    let mut tuning = None;

    if let Ok(system) = dev.read_register(ADDR_SYSTEM, 6) {
        if system.len() >= 6 {
            let polling_code = system[0];
            let hz = match polling_code {
                0x08 => 125,
                0x04 => 250,
                0x02 => 500,
                0x01 => 1000,
                0x10 => 2000,
                0x20 => 4000,
                0x40 => 8000,
                _ => 1000,
            };
            polling_rate_hz = Some(hz);
            let count = system[2];
            let active = system[4];
            active_stage = Some(active + 1);
            stage_count_val = Some(count);

            let colors = dev.read_stage_colors(count).unwrap_or_default();
            for s in 0..count {
                let addr = ADDR_DPI_BASE + (s as u16) * 4;
                if let Ok(data) = dev.read_register(addr, 4) {
                    if data.len() >= 4 {
                        let (x, y) = decode_stage_dpi(&data);
                        let (r, g, b) = colors.get(s as usize).copied().unwrap_or((255, 255, 255));
                        dpi_stages.push(StageStatusJson {
                            stage: s + 1,
                            dpi_x: x,
                            dpi_y: y,
                            rgb: format!("#{:02X}{:02X}{:02X}", r, g, b),
                            active: s == active,
                        });
                    }
                }
            }

            let lod = match dev.get_lod().unwrap_or(1) {
                1 => 1.0,
                2 => 2.0,
                _ => 1.0,
            };

            let (deb, msync, straight, ripple, ultra) = dev.get_all_advanced_settings().unwrap_or((8, true, false, false, false));
            let sleep_timeout_sec = dev.get_sleep_time_seconds().ok();
            let light_str = match dev.get_dpi_light().unwrap_or((0, 2, 1)).0 {
                1 => "Always On",
                2 => "Breathing",
                _ => "Off (Closed)",
            };

            tuning = Some(TuningStatusJson {
                lod_mm: lod,
                debounce_ms: deb,
                motion_sync: msync,
                straight_line: straight,
                ripple_control: ripple,
                ultra_long_range: ultra,
                dpi_light_effect: light_str.to_string(),
                sleep_timeout_sec,
            });
        }
    }

    let vid = target.channel.info.vendor_id;
    let pid = target.channel.info.product_id;
    let receiver_rf_id = if dev.mode() == ConnectionMode::WirelessDongle {
        dev.query_receiver()
            .ok()
            .map(|r| r.rf_id)
            .filter(|id| !id.is_empty())
    } else {
        None
    };

    DeviceStatusJson {
        index: target.index,
        model: model_name,
        connection_mode: mode_str.to_string(),
        firmware_version: fw_str,
        usb_vid: format!("0x{vid:04X}"),
        usb_pid: format!("0x{pid:04X}"),
        usb_device: format_usb_vid_pid(vid, pid),
        receiver_rf_id,
        is_sleeping,
        hardware_identity: hw_id,
        battery,
        aliases,
        polling_rate_hz,
        active_stage,
        stage_count: stage_count_val,
        dpi_stages,
        tuning,
    }
}

pub fn decode_stage_dpi(record: &[u8]) -> (u32, u32) {
    if record.len() < 4 {
        return (0, 0);
    }
    let xb = record[0];
    let yb = record[1];
    let mode = record[2];

    let x = decode_dpi_axis(xb, mode & 0x0F);
    let y = decode_dpi_axis(yb, (mode >> 4) & 0x0F);
    (x, y)
}

/* Displays complete telemetry and hardware status for a device formatted as a Unicode card */
pub fn show_status(dev: &AtkDevice, target: &CliDeviceInfo, total_devices: usize, theme: Theme) {
    /* Identity & FW with retry for wireless mice in light standby */
    let mut identity = dev.query_identity().ok();
    let mut fw = dev.query_firmware().ok();

    if identity.is_none() && fw.is_none() && dev.mode() == ConnectionMode::WirelessDongle {
        std::thread::sleep(std::time::Duration::from_millis(60));
        identity = dev.query_identity().ok();
        fw = dev.query_firmware().ok();
    }

    let fw_str = fw.map(|f| format!("v{f}")).unwrap_or_else(|| "Unknown".to_string());
    let mut model_name = match identity.as_ref().map(|i| i.model_name.trim()) {
        Some(name) if !name.is_empty() && name != "ATK Gaming Mouse" && !name.starts_with("Unknown") => {
            name.to_string()
        }
        _ => target.model_name.clone(),
    };

    /* Disambiguation: Compx MCU mice (R1 SE / R1 SE+) report firmware v3.xx on 1k dongle (0xF58A) */
    let name_upper = model_name.to_uppercase();
    let is_generic_or_wrong = name_upper.contains("GAMING MOUSE")
        || name_upper.contains("PRO MAX")
        || name_upper.contains("DONGLE")
        || name_upper.contains("RECEIVER");
    if (fw_str.starts_with("v3.") || fw_str.starts_with("3.")) && is_generic_or_wrong {
        model_name = "VXE R1SE+".to_string();
    }

    /* Check if wireless mouse is currently asleep/inactive */
    let is_sleeping = identity.is_none() && dev.mode() == ConnectionMode::WirelessDongle;
    let rcv_info = dev.query_receiver().ok();

    if is_sleeping {
        let title = if total_devices > 1 {
            format!("[{}] {} ({} of {})", target.index, theme.bold_white(&model_name), target.index + 1, total_devices)
        } else {
            format!("[{}] {}", target.index, theme.bold_white(&model_name))
        };

        let badge = theme.yellow("Standby (Mouse Asleep)");
        let mut card = Card::new(theme, 70, title)
            .badge(badge)
            .blank();

        card = card.key_value("Connection Mode", "2.4 GHz Wireless Receiver");
        card = card.key_value(
            "USB VID / PID",
            format_usb_vid_pid(target.channel.info.vendor_id, target.channel.info.product_id),
        );
        if let Some(ref rcv) = rcv_info {
            if !rcv.rf_id.is_empty() {
                card = card.key_value("Receiver RF ID", &rcv.rf_id);
            }
        }
        card = card.key_value("Device State", theme.yellow("Standby / Sleeping (Stationary)"));
        card = card.key_value("Wakeup Hint", theme.dim("Move mouse to wake up, then re-run vxectl"));

        let key_aliases: Vec<_> = target
            .aliases
            .iter()
            .filter(|a| a.len() <= 7 && !a.is_empty())
            .take(5)
            .cloned()
            .collect();
        if !key_aliases.is_empty() {
            card = card.key_value("Target Aliases", theme.yellow(&key_aliases.join(", ")));
        }
        card = card.key_value("Quick Target", theme.cyan(&format!("vxectl -d {} status", target.index)));

        card = card.blank().footer("vxectl v0.1.0");
        print!("{}", card.render());
        return;
    }

    let mode_str = match dev.mode() {
        ConnectionMode::Wired => "Wired USB-C",
        ConnectionMode::WirelessDongle => "2.4 GHz Wireless",
        ConnectionMode::Bluetooth => "Bluetooth",
        ConnectionMode::Unknown => "Unknown",
    };

    let title = if total_devices > 1 {
        format!("[{}] {} ({} of {})", target.index, theme.bold_white(&model_name), target.index + 1, total_devices)
    } else {
        format!("[{}] {}", target.index, theme.bold_white(&model_name))
    };

    let badge = theme.mode_badge(mode_str, true);
    let mut card = Card::new(theme, 70, title)
        .badge(badge)
        .blank();

    if let Some(ref id) = identity {
        if id.cid > 0 || id.mid > 0 {
            card = card.key_value("Hardware Identity", format!("CID {}, MID {}", id.cid, id.mid));
        }
    }

    card = card.key_value(
        "USB VID / PID",
        format_usb_vid_pid(target.channel.info.vendor_id, target.channel.info.product_id),
    );
    if dev.mode() == ConnectionMode::WirelessDongle {
        if let Some(ref rcv) = rcv_info {
            if !rcv.rf_id.is_empty() {
                card = card.key_value("Receiver RF ID", &rcv.rf_id);
            }
        }
    }

    card = card.key_value("Firmware Version", &fw_str);

    /* Battery */
    if let Ok((bat, mv)) = dev.query_battery() {
        let is_charging = bat.is_charging || dev.mode() == ConnectionMode::Wired;
        let bat_text = format!("{}  ({:.2} V)", theme.battery_bar(bat.percentage, is_charging), mv as f32 / 1000.0);
        card = card.key_value("Battery Telemetry", bat_text);
    }

    let key_aliases: Vec<_> = target
        .aliases
        .iter()
        .filter(|a| a.len() <= 7 && !a.is_empty())
        .take(5)
        .cloned()
        .collect();
    if !key_aliases.is_empty() {
        card = card.key_value("Target Aliases", theme.yellow(&key_aliases.join(", ")));
    }

    /* System row */
    if let Ok(system) = dev.read_register(ADDR_SYSTEM, 6) {
        if system.len() >= 6 {
            let polling_code = system[0];
            let polling_hz = match polling_code {
                0x08 => 125,
                0x04 => 250,
                0x02 => 500,
                0x01 => 1000,
                0x10 => 2000,
                0x20 => 4000,
                0x40 => 8000,
                _ => 0,
            };
            let stage_count = system[2];
            let active_stage = system[4];

            card = card.blank().section("Performance & Sensor");

            let ms_str = if polling_hz > 0 {
                format!("{:.1} ms interval", 1000.0 / polling_hz as f32)
            } else {
                "Unknown interval".to_string()
            };
            card = card.key_value("Polling Frequency", format!("{} Hz  {}", theme.bold(&polling_hz.to_string()), theme.dim(&format!("({ms_str})"))));
            card = card.key_value("Active DPI Stage", format!("Stage {} of {}", theme.bold(&(active_stage + 1).to_string()), stage_count));

            let mut stages_str = String::new();
            for s in 0..stage_count {
                let addr = ADDR_DPI_BASE + (s as u16) * 4;
                if let Ok(data) = dev.read_register(addr, 4) {
                    if data.len() >= 4 {
                        let (x, y) = decode_stage_dpi(&data);
                        let label = if x == y {
                            format!("{x}")
                        } else {
                            format!("{x}x{y}")
                        };
                        if s == active_stage {
                            stages_str.push_str(&format!("  {}", theme.bold_yellow(&format!("[● {} ★]", label))));
                        } else {
                            stages_str.push_str(&format!("  {}", theme.dim(&format!("[● {}]", label))));
                        }
                    }
                }
            }
            if !stages_str.is_empty() {
                card = card.key_value("DPI Stage Table", stages_str.trim_start());
            }

            if let Ok(lod) = dev.get_lod() {
                let desc = match lod {
                    1 => "1.0 mm (Low / Standard)",
                    2 => "2.0 mm (High)",
                    _ => "Default",
                };
                card = card.key_value("Lift-off Distance", desc);
            }

            card = card.blank().section("Advanced Switch & Radio Tuning");

            if let Ok((deb, msync, straight, ripple, ultra)) = dev.get_all_advanced_settings() {
                card = card.key_value("Debounce Delay", format!("{} ms", deb));
                let fmt_bool = |b: bool| if b { theme.green("Enabled") } else { theme.dim("Disabled") };
                card = card.key_value("Movement Sync", fmt_bool(msync));
                card = card.key_value("Straight Line", fmt_bool(straight));
                card = card.key_value("Ripple Control", fmt_bool(ripple));
                card = card.key_value("Ultra Long Range", fmt_bool(ultra));
            }

            if let Ok(sleep_sec) = dev.get_sleep_time_seconds() {
                card = card.key_value("Sleep Timeout", format_sleep_timeout(sleep_sec));
            }

            if let Ok((mode, bright, speed)) = dev.get_dpi_light() {
                let mode_str = match mode {
                    1 => format!("Always On (Brightness {}/2)", bright),
                    2 => format!("Breathing (Speed {}/2)", speed),
                    _ => "Off (Closed)".to_string(),
                };
                card = card.key_value("DPI Indicator LED", mode_str);
            }
        }
    }

    card = card.blank().footer("vxectl v0.1.0");
    print!("{}", card.render());
}

/* Outputs battery information for a single device */
pub fn show_single_battery(dev: &AtkDevice, json: bool, model_name: &str, theme: Theme) {
    match dev.query_battery() {
        Ok((status, mv)) => {
            if json {
                println!(
                    r#"{{"model": "{}", "percentage": {}, "charging": {}, "voltage_mv": {}}}"#,
                    model_name, status.percentage, status.is_charging, mv
                );
            } else {
                let is_charging = status.is_charging || dev.mode() == ConnectionMode::Wired;
                println!(
                    "{}  ({:.2} V)",
                    theme.battery_bar(status.percentage, is_charging),
                    mv as f32 / 1000.0
                );
            }
        }
        Err(e) => {
            eprintln!("[!] Could not read battery: {e}");
            std::process::exit(1);
        }
    }
}

/* Outputs battery status for all connected devices */
pub fn show_all_batteries(devices: &[CliDeviceInfo], hid_mgr: &HidManager, json: bool, theme: Theme) {
    if json {
        let mut entries = Vec::new();
        for dev_info in devices {
            if let Ok(dev) = AtkDevice::open_with_info(hid_mgr, dev_info.channel.info.clone(), dev_info.mode) {
                if let Ok((bat, mv)) = dev.query_battery() {
                    let charging = bat.is_charging || dev.mode() == ConnectionMode::Wired;
                    entries.push(format!(
                        r#"{{"index": {}, "model": "{}", "percentage": {}, "charging": {}, "voltage_mv": {}}}"#,
                        dev_info.index, dev_info.model_name, bat.percentage, charging, mv
                    ));
                }
            }
        }
        println!("[\n  {}\n]", entries.join(",\n  "));
    } else {
        let badge = format!("{} Devices", devices.len());
        let mut card = Card::new(theme, 68, "Battery Telemetry")
            .badge(badge)
            .blank();

        for dev_info in devices {
            let bat_str = if let Ok(dev) = AtkDevice::open_with_info(hid_mgr, dev_info.channel.info.clone(), dev_info.mode) {
                if let Ok((bat, mv)) = dev.query_battery() {
                    let is_charging = bat.is_charging || dev_info.mode == ConnectionMode::Wired;
                    format!("{}  ({:.2} V)", theme.battery_bar(bat.percentage, is_charging), mv as f32 / 1000.0)
                } else {
                    theme.dim("N/A")
                }
            } else {
                theme.dim("N/A")
            };
            let label = format!("[{}] {}", dev_info.index, dev_info.model_name);
            card = card.key_value(label, bat_str);
        }
        card = card.blank().footer("vxectl v0.1.0");
        print!("{}", card.render());
    }
}
