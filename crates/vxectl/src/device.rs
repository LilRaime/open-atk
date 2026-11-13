/*
 * Device discovery, identification, short aliases resolution, and selection.
 */

use atk_device::{AtkDevice, ConnectionMode, DeviceScanner, DiscoveredDevice};
use atk_hid::HidManager;
use crate::theme::{Card, Theme};

pub struct CliDeviceInfo {
    pub index: usize,
    pub model_name: String,
    pub aliases: Vec<String>,
    pub mode: ConnectionMode,
    pub battery_info: Option<(u8, bool, u16)>,
    pub channel: DiscoveredDevice,
}

/* Returns all recognized short aliases and abbreviations for a mouse model */
pub fn get_device_aliases(model_name: &str) -> Vec<String> {
    let mut aliases = Vec::new();
    let upper = model_name.to_uppercase();
    let norm = upper.replace([' ', '-', '_'], "").to_lowercase();
    aliases.push(norm);

    /* Strip known brand prefixes */
    for brand in ["VXE", "VGN", "ATK"] {
        if upper.starts_with(brand) {
            let without_brand = upper.trim_start_matches(brand).trim();
            let wb_norm = without_brand.replace([' ', '-', '_'], "").to_lowercase();
            if !wb_norm.is_empty() {
                aliases.push(wb_norm);
            }
        }
    }

    /* VXE R1 Series short abbreviations */
    if upper.contains("R1") {
        aliases.push("r1".to_string());
        if upper.contains("PRO MAX") || upper.contains("PROMAX") {
            aliases.extend(["r1pm", "r1-pm", "pm", "promax", "r1promax"].iter().map(|s| s.to_string()));
        } else if upper.contains("PRO") {
            aliases.extend(["r1p", "r1-p", "pro", "r1pro"].iter().map(|s| s.to_string()));
        }

        if upper.contains("SE+") || upper.contains("SE PLUS") {
            aliases.extend(["r1se+", "r1sep", "r1seplus", "se+", "sep", "seplus"].iter().map(|s| s.to_string()));
        } else if upper.contains("SE") {
            aliases.extend(["r1se", "se"].iter().map(|s| s.to_string()));
        }
    }

    /* VGN Dragonfly F1 Series short abbreviations */
    if upper.contains("F1") || upper.contains("DRAGONFLY") {
        aliases.push("f1".to_string());
        aliases.push("dragonfly".to_string());
        if upper.contains("PRO MAX") || upper.contains("PROMAX") {
            aliases.extend(["f1pm", "f1-pm", "f1promax"].iter().map(|s| s.to_string()));
        } else if upper.contains("PRO") {
            aliases.extend(["f1p", "f1-p", "f1pro"].iter().map(|s| s.to_string()));
        }
        if upper.contains("MOBA") {
            aliases.extend(["f1m", "f1moba", "moba"].iter().map(|s| s.to_string()));
        }
    }

    /* ATK Mad R Series short abbreviations */
    if upper.contains("MAD") {
        aliases.extend(["madr", "mad-r", "mad"].iter().map(|s| s.to_string()));
        if upper.contains("MAJOR") {
            aliases.extend(["madrm", "madrmajor", "madr-m", "major"].iter().map(|s| s.to_string()));
        }
        if upper.contains("PLUS") || upper.contains('+') {
            aliases.extend(["madr+", "madrp", "madrplus"].iter().map(|s| s.to_string()));
        }
    }

    /* ATK X1 Series short abbreviations */
    if upper.contains("X1") {
        aliases.push("x1".to_string());
        if upper.contains("PRO MAX") || upper.contains("PROMAX") {
            aliases.extend(["x1pm", "x1promax"].iter().map(|s| s.to_string()));
        } else if upper.contains("PRO") {
            aliases.extend(["x1p", "x1pro"].iter().map(|s| s.to_string()));
        } else if upper.contains("ULTRA") {
            aliases.extend(["x1u", "x1ultra"].iter().map(|s| s.to_string()));
        }
    }

    /* Deduplicate maintaining order */
    let mut unique = Vec::new();
    for a in aliases {
        if !unique.contains(&a) {
            unique.push(a);
        }
    }
    unique
}

/* Scans for all configuration channels and queries model name and battery */
pub fn scan_all_devices(hid_mgr: &HidManager) -> Vec<CliDeviceInfo> {
    let mut channels = DeviceScanner::scan_config_channels(hid_mgr);
    /* Prioritize Wired connections over WirelessDongles */
    channels.sort_by_key(|d| match d.mode {
        ConnectionMode::Wired => 0,
        ConnectionMode::WirelessDongle => 1,
        ConnectionMode::Bluetooth => 2,
        ConnectionMode::Unknown => 3,
    });
    let has_wired = channels.iter().any(|c| c.mode == ConnectionMode::Wired);
    let mut devices = Vec::new();

    let channel_list = channels.clone();
    for target in channel_list {
        let (resolved_name, resolved_mode) = DeviceScanner::resolve_device_name_and_mode(
            target.info.vendor_id,
            target.info.product_id,
            target.info.product_string.as_deref(),
        );

        /* If a wired mouse is plugged in, check whether wireless dongles of the SAME model family are currently idle */
        if has_wired && target.mode == ConnectionMode::WirelessDongle {
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
                if let Ok(dev) = AtkDevice::open_with_info(hid_mgr, target.info.clone(), target.mode) {
                    if let Ok(rcv) = dev.query_receiver() {
                        is_active_dongle = rcv.online;
                    }
                }
                if !is_active_dongle {
                    /* Suppress duplicate idle wireless dongle when this exact model is plugged in via USB cable to charge */
                    continue;
                }
            }
        }

        let idx = devices.len();
        let mut model_name = resolved_name.clone();
        let mut battery_info = None;

        /* Try querying live identity & battery to refine the model name */
        if let Ok(dev) = AtkDevice::open_with_info(hid_mgr, target.info.clone(), target.mode) {
            let mut fw_ver = String::new();
            if let Ok(fw) = dev.query_firmware() {
                fw_ver = fw;
            }
            if let Ok(id) = dev.query_identity() {
                let q_name = id.model_name.trim();
                if !q_name.is_empty() && !q_name.starts_with("Unknown") && q_name != "ATK Gaming Mouse" {
                    model_name = q_name.to_uppercase();
                }
            }
            /* Disambiguation: Compx MCU mice (R1 SE / R1 SE+) report firmware v3.xx on 1k dongle (0xF58A) */
            let name_upper = model_name.to_uppercase();
            let is_generic_or_wrong = name_upper.contains("GAMING MOUSE")
                || name_upper.contains("PRO MAX")
                || name_upper.contains("DONGLE")
                || name_upper.contains("RECEIVER");
            if (fw_ver.starts_with("3.") || fw_ver.starts_with("v3.")) && is_generic_or_wrong {
                model_name = "VXE R1SE+".to_string();
            }
            if let Ok((bat, mv)) = dev.query_battery() {
                battery_info = Some((bat.percentage, bat.is_charging || target.mode == ConnectionMode::Wired, mv));
            }
        }

        let aliases = get_device_aliases(&model_name);

        devices.push(CliDeviceInfo {
            index: idx,
            model_name,
            aliases,
            mode: resolved_mode,
            battery_info,
            channel: target,
        });
    }

    devices
}

/* Select device based on index, alias, or substring selector */
pub fn select_device<'a>(devices: &'a [CliDeviceInfo], selector: Option<&str>) -> &'a CliDeviceInfo {
    if devices.is_empty() {
        eprintln!("[-] No compatible ATK / VXE mouse detected.");
        eprintln!("    Make sure your 2.4 GHz wireless dongle or USB-C cable is connected.");
        std::process::exit(1);
    }

    if let Some(query) = selector {
        let q_trimmed = query.trim();
        /* 1. Try exact index match */
        if let Ok(idx) = q_trimmed.parse::<usize>() {
            if let Some(dev) = devices.iter().find(|d| d.index == idx) {
                return dev;
            } else {
                eprintln!("[!] Device index [{}] not found.", idx);
                eprintln!("    Available device indices: 0 to {}", devices.len() - 1);
                std::process::exit(1);
            }
        }

        let q_norm = q_trimmed.replace([' ', '-', '_'], "").to_lowercase();
        let q_lower = q_trimmed.to_lowercase();

        /* 2. Priority 1: Exact alias or exact name match (e.g. 'r1pm' or 'r1p') */
        let exact_matches: Vec<_> = devices
            .iter()
            .filter(|d| d.aliases.iter().any(|a| a == &q_norm || a == &q_lower))
            .collect();

        if exact_matches.len() == 1 {
            return exact_matches[0];
        } else if exact_matches.len() > 1 {
            eprintln!("[!] Ambiguous device selector '{}' matches multiple devices:", query);
            for m in &exact_matches {
                eprintln!("    [{}] {}", m.index, m.model_name);
            }
            eprintln!("    Please specify device by index (e.g. -d {})", exact_matches[0].index);
            std::process::exit(1);
        }

        /* 3. Priority 2: Substring match on model name, aliases, or HID path */
        let sub_matches: Vec<_> = devices
            .iter()
            .filter(|d| {
                d.model_name.to_lowercase().contains(&q_lower)
                    || d.aliases.iter().any(|a| a.contains(&q_norm))
                    || d.channel.info.path.to_lowercase().contains(&q_lower)
            })
            .collect();

        if sub_matches.len() == 1 {
            return sub_matches[0];
        } else if sub_matches.len() > 1 {
            eprintln!("[!] Ambiguous device selector '{}' matches multiple devices:", query);
            for m in &sub_matches {
                eprintln!("    [{}] {}", m.index, m.model_name);
            }
            eprintln!("    Please specify device by index (e.g. -d {})", sub_matches[0].index);
            std::process::exit(1);
        } else {
            eprintln!("[!] No device matching '{}' found.", query);
            eprintln!("    Run `vxectl list` to view all connected devices and their aliases.");
            std::process::exit(1);
        }
    }

    &devices[0]
}

/* Require explicit device selection for write/mutation commands if more than 1 device is detected */
pub fn select_device_for_mutation<'a>(devices: &'a [CliDeviceInfo], selector: Option<&str>) -> &'a CliDeviceInfo {
    if devices.is_empty() {
        eprintln!("[-] No compatible ATK / VXE mouse detected.");
        std::process::exit(1);
    }

    if selector.is_none() && devices.len() > 1 {
        eprintln!("[!] Multiple devices detected ({}).", devices.len());
        eprintln!("    Please specify which device to configure using `-d <index|alias>`:");
        for d in devices {
            let key_aliases: Vec<_> = d
                .aliases
                .iter()
                .filter(|a| a.len() <= 9 && !a.is_empty())
                .take(3)
                .cloned()
                .collect();
            let hint = if !key_aliases.is_empty() {
                format!(" (-d {} or -d {})", d.index, key_aliases[0])
            } else {
                format!(" (-d {})", d.index)
            };
            eprintln!("      [{}] {}{}", d.index, d.model_name, hint);
        }
        std::process::exit(1);
    }

    select_device(devices, selector)
}

/* Display connected devices table as a styled Unicode card or JSON array */
pub fn show_device_list(devices: &[CliDeviceInfo], theme: Theme, json: bool) {
    if json {
        let entries: Vec<_> = devices
            .iter()
            .map(|d| {
                let mode_str = match d.mode {
                    ConnectionMode::Wired => "Wired USB-C",
                    ConnectionMode::WirelessDongle => "2.4 GHz Wireless",
                    ConnectionMode::Bluetooth => "Bluetooth",
                    ConnectionMode::Unknown => "Unknown",
                };
                serde_json::json!({
                    "index": d.index,
                    "model": d.model_name,
                    "connection": mode_str,
                    "aliases": d.aliases.iter().filter(|a| a.len() <= 7 && !a.is_empty()).take(5).collect::<Vec<_>>(),
                    "battery": d.battery_info.map(|(pct, chg, mv)| {
                        serde_json::json!({
                            "percentage": pct,
                            "charging": chg,
                            "voltage_mv": mv,
                            "voltage_v": mv as f32 / 1000.0
                        })
                    }),
                    "path": d.channel.info.path
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&entries).unwrap());
        return;
    }

    if devices.is_empty() {
        let card = Card::new(theme, 70, "Connected ATK / VXE Devices")
            .badge(theme.red("[-] Offline"))
            .blank()
            .raw(theme.dim("No compatible ATK / VXE devices detected."))
            .raw(theme.dim("Please connect a 2.4 GHz wireless receiver or USB-C cable."))
            .blank()
            .footer("vxectl v0.1.0");
        print!("{}", card.render());
        return;
    }

    let badge = format!("{} {}", theme.green("●"), theme.bold(&format!("{} Detected", devices.len())));
    let mut card = Card::new(theme, 70, "Connected ATK / VXE Devices")
        .badge(badge);

    for (i, dev) in devices.iter().enumerate() {
        card = card.blank();
        let mode_str = match dev.mode {
            ConnectionMode::Wired => "Wired USB-C",
            ConnectionMode::WirelessDongle => "2.4 GHz Wireless",
            ConnectionMode::Bluetooth => "Bluetooth",
            ConnectionMode::Unknown => "Unknown",
        };
        let bat_text = if let Some((pct, charging, mv)) = dev.battery_info {
            format!("{}  ({:.2} V)", theme.battery_bar(pct, charging), mv as f32 / 1000.0)
        } else {
            theme.dim("N/A")
        };

        let key_aliases: Vec<_> = dev
            .aliases
            .iter()
            .filter(|a| a.len() <= 7 && !a.is_empty())
            .take(5)
            .cloned()
            .collect();

        let target_alias = key_aliases.first().map(|s| s.as_str()).unwrap_or("0");

        let title_line = format!(
            "[{}] {}  {}",
            dev.index,
            theme.bold_white(&dev.model_name),
            theme.mode_badge(mode_str, true)
        );
        card = card.raw(title_line);

        if !key_aliases.is_empty() {
            card = card.key_value("Aliases", theme.yellow(&key_aliases.join(", ")));
        }
        card = card.key_value("Battery", bat_text);
        card = card.key_value("Quick Target", theme.cyan(&format!("vxectl -d {} status", target_alias)));

        if i + 1 < devices.len() {
            card = card.blank();
        }
    }

    card = card.blank().footer("vxectl v0.1.0  •  Target with: -d <alias>");
    print!("{}", card.render());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_aliases() {
        let r1pm = get_device_aliases("VXE R1 PRO MAX");
        assert!(r1pm.contains(&"r1pm".to_string()));
        assert!(r1pm.contains(&"r1promax".to_string()));
        assert!(r1pm.contains(&"pm".to_string()));

        let r1p = get_device_aliases("VXE R1 PRO");
        assert!(r1p.contains(&"r1p".to_string()));
        assert!(r1p.contains(&"r1pro".to_string()));
        assert!(!r1p.contains(&"r1pm".to_string()));

        let r1sep = get_device_aliases("VXE R1 SE+");
        assert!(r1sep.contains(&"r1sep".to_string()));
        assert!(r1sep.contains(&"r1se+".to_string()));

        let f1pm = get_device_aliases("VGN DRAGONFLY F1 PRO MAX");
        assert!(f1pm.contains(&"f1pm".to_string()));

        let f1p = get_device_aliases("VGN DRAGONFLY F1 PRO");
        assert!(f1p.contains(&"f1p".to_string()));

        let f1m = get_device_aliases("VGN DRAGONFLY F1 MOBA");
        assert!(f1m.contains(&"f1m".to_string()));
        assert!(f1m.contains(&"moba".to_string()));

        let madr = get_device_aliases("ATK MAD R");
        assert!(madr.contains(&"madr".to_string()));

        let madrm = get_device_aliases("ATK MAD R MAJOR");
        assert!(madrm.contains(&"madr".to_string()));
        assert!(madrm.contains(&"madrm".to_string()));

        let x1pm = get_device_aliases("ATK X1 PRO MAX");
        assert!(x1pm.contains(&"x1pm".to_string()));
    }
}

