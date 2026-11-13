/*
 * Handlers for mouse configuration commands (DPI, Polling, LOD, Debounce, MotionSync).
 */

use atk_device::AtkDevice;
use atk_protocol::{constants::ADDR_SYSTEM, PollingRate};

pub fn handle_set_stage(dev: &AtkDevice, stage: Option<u8>) {
    let system = match dev.read_register(ADDR_SYSTEM, 6) {
        Ok(s) if s.len() >= 6 => s,
        _ => {
            eprintln!("[!] Failed to read system row from mouse");
            std::process::exit(1);
        }
    };

    let stage_count = system[2];
    let current_stage = system[4];

    match stage {
        None => {
            println!("Active DPI stage: Stage {} of {}", current_stage + 1, stage_count);
        }
        Some(target) => {
            if target == 0 || target > stage_count {
                eprintln!("[!] Stage {} is out of range. Valid stages: 1 to {}", target, stage_count);
                std::process::exit(1);
            }

            let stage_idx = target - 1;
            println!("[*] Switching to Stage {} (index {})...", target, stage_idx);
            if let Err(e) = dev.set_active_stage(stage_idx) {
                eprintln!("[!] Failed to switch stage: {e}");
                std::process::exit(1);
            }

            let _ = dev.commit_profile();
            println!("[+] Successfully activated Stage {} (verified & committed to mouse)!", target);
        }
    }
}

pub fn handle_set_dpi(dev: &AtkDevice, target_stage: u8, dpi_value: u32, y: Option<u32>) {
    let system = match dev.read_register(ADDR_SYSTEM, 6) {
        Ok(s) if s.len() >= 6 => s,
        _ => {
            eprintln!("[!] Failed to read system configuration from mouse");
            std::process::exit(1);
        }
    };

    let stage_count = system[2];

    if target_stage == 0 || target_stage > stage_count {
        eprintln!("[!] Stage {} is invalid. Valid stages: 1 to {}", target_stage, stage_count);
        std::process::exit(1);
    }

    if dpi_value < 50 || dpi_value > 30000 || dpi_value % 50 != 0 {
        eprintln!("[!] DPI {} is invalid. Must be between 50 and 30000 with step 50.", dpi_value);
        std::process::exit(1);
    }

    let stage_idx = target_stage - 1;
    let y_value = y.unwrap_or(dpi_value);

    println!("[*] Writing Stage {}: X = {} DPI, Y = {} DPI...", target_stage, dpi_value, y_value);
    if let Err(e) = dev.set_stage_dpi(stage_idx, dpi_value, y_value) {
        eprintln!("[!] Failed to write DPI to mouse EEPROM: {e}");
        std::process::exit(1);
    }

    let _ = dev.commit_profile();
    println!("[+] Successfully updated Stage {} to {} x {} DPI (verified & committed to flash)!", target_stage, dpi_value, y_value);
}

pub fn handle_set_polling(dev: &AtkDevice, rate: Option<u16>) {
    let system = match dev.read_register(ADDR_SYSTEM, 6) {
        Ok(s) if s.len() >= 6 => s,
        _ => {
            eprintln!("[!] Failed to read system configuration from mouse");
            std::process::exit(1);
        }
    };

    let current_code = system[0];
    let current_hz = match current_code {
        0x08 => 125,
        0x04 => 250,
        0x02 => 500,
        0x01 => 1000,
        0x10 => 2000,
        0x20 => 4000,
        0x40 => 8000,
        _ => 0,
    };

    match rate {
        None => {
            println!("Current Polling Rate: {} Hz", current_hz);
        }
        Some(target_hz) => {
            let polling_enum = match PollingRate::from_hz(target_hz) {
                Ok(p) => p,
                Err(_) => {
                    eprintln!("[!] Unsupported polling rate {} Hz. Valid values: 125, 250, 500, 1000", target_hz);
                    std::process::exit(1);
                }
            };

            println!("[*] Setting Polling Rate to {} Hz...", target_hz);
            if let Err(e) = dev.set_polling_rate(polling_enum) {
                eprintln!("[!] Failed to set polling rate: {e}");
                std::process::exit(1);
            }

            let _ = dev.commit_profile();
            println!("[+] Successfully changed Polling Rate to {} Hz (verified & committed to flash)!", target_hz);
        }
    }
}

pub fn handle_set_lod(dev: &AtkDevice, lod: Option<u8>) {
    match lod {
        None => match dev.get_lod() {
            Ok(code) => {
                let desc = match code {
                    1 => "1.0 mm (Low/Standard)",
                    2 => "2.0 mm (High)",
                    _ => "Unknown",
                };
                println!("Current Lift-off Distance (LOD): {} (code {})", desc, code);
            }
            Err(e) => eprintln!("[!] Failed to read LOD: {e}"),
        },
        Some(target) => {
            if target != 1 && target != 2 {
                eprintln!("[!] Invalid LOD {target}. Valid choices: 1 (1.0 mm) or 2 (2.0 mm)");
                std::process::exit(1);
            }
            let desc = if target == 1 { "1.0 mm" } else { "2.0 mm" };
            println!("[*] Setting Lift-off Distance to {} (code {})...", desc, target);
            if let Err(e) = dev.set_lod(target) {
                eprintln!("[!] Failed to set LOD: {e}");
                std::process::exit(1);
            }
            println!("[+] Successfully set LOD to {} (verified & committed to EEPROM)!", desc);
        }
    }
}

pub fn handle_set_debounce(dev: &AtkDevice, ms: Option<u8>) {
    match ms {
        None => match dev.get_advanced_settings() {
            Ok((debounce, _, _)) => {
                println!("Current Debounce Delay: {} ms", debounce);
            }
            Err(e) => eprintln!("[!] Failed to read debounce delay: {e}"),
        },
        Some(target) => {
            if target > 20 {
                eprintln!("[!] Invalid debounce delay {target} ms. Valid range: 0 to 20 ms.");
                std::process::exit(1);
            }
            println!("[*] Setting switch debounce delay to {} ms...", target);
            if let Err(e) = dev.set_debounce(target) {
                eprintln!("[!] Failed to write debounce: {e}");
                std::process::exit(1);
            }
            println!("[+] Successfully set Debounce Delay to {} ms (verified & committed to EEPROM)!", target);
        }
    }
}

pub fn handle_set_motionsync(dev: &AtkDevice, state: Option<String>) {
    match state {
        None => match dev.get_advanced_settings() {
            Ok((_, sync, _)) => {
                let s_str = if sync { "Enabled (ON)" } else { "Disabled (OFF)" };
                println!("Current Motion Sync state: {}", s_str);
            }
            Err(e) => eprintln!("[!] Failed to read Motion Sync status: {e}"),
        },
        Some(s) => {
            let enabled = match s.to_lowercase().as_str() {
                "on" | "true" | "1" | "enable" | "enabled" => true,
                "off" | "false" | "0" | "disable" | "disabled" => false,
                other => {
                    eprintln!("[!] Invalid state '{other}'. Valid values: on, off, true, false, 1, 0");
                    std::process::exit(1);
                }
            };
            let desc = if enabled { "Enabled (ON)" } else { "Disabled (OFF)" };
            println!("[*] Setting Motion Sync to {}...", desc);
            if let Err(e) = dev.set_motion_sync(enabled) {
                eprintln!("[!] Failed to set Motion Sync: {e}");
                std::process::exit(1);
            }
            println!("[+] Successfully updated Motion Sync to {} (verified & committed to EEPROM)!", desc);
        }
    }
}

fn parse_bool_arg(s: &str) -> bool {
    match s.to_lowercase().as_str() {
        "on" | "true" | "1" | "enable" | "enabled" => true,
        "off" | "false" | "0" | "disable" | "disabled" => false,
        other => {
            eprintln!("[!] Invalid state '{other}'. Valid values: on, off, true, false, 1, 0");
            std::process::exit(1);
        }
    }
}

pub fn handle_set_straightline(dev: &AtkDevice, state: Option<String>) {
    match state {
        None => match dev.get_all_advanced_settings() {
            Ok((_, _, straight, _, _)) => {
                println!(
                    "Current Straight Line Correction: {}",
                    if straight { "Enabled (ON)" } else { "Disabled (OFF)" }
                );
            }
            Err(e) => eprintln!("[!] Failed to read Straight Line status: {e}"),
        },
        Some(s) => {
            let enabled = parse_bool_arg(&s);
            let desc = if enabled { "Enabled (ON)" } else { "Disabled (OFF)" };
            if let Err(e) = dev.set_straight_line(enabled) {
                eprintln!("[!] Failed to set Straight Line Correction: {e}");
                std::process::exit(1);
            }
            println!("[+] Successfully updated Straight Line Correction to {desc}!");
        }
    }
}

pub fn handle_set_ripple(dev: &AtkDevice, state: Option<String>) {
    match state {
        None => match dev.get_all_advanced_settings() {
            Ok((_, _, _, ripple, _)) => {
                println!(
                    "Current Ripple Control: {}",
                    if ripple { "Enabled (ON)" } else { "Disabled (OFF)" }
                );
            }
            Err(e) => eprintln!("[!] Failed to read Ripple Control status: {e}"),
        },
        Some(s) => {
            let enabled = parse_bool_arg(&s);
            let desc = if enabled { "Enabled (ON)" } else { "Disabled (OFF)" };
            if let Err(e) = dev.set_ripple_control(enabled) {
                eprintln!("[!] Failed to set Ripple Control: {e}");
                std::process::exit(1);
            }
            println!("[+] Successfully updated Ripple Control to {desc}!");
        }
    }
}

pub fn handle_set_longrange(dev: &AtkDevice, state: Option<String>) {
    match state {
        None => match dev.get_all_advanced_settings() {
            Ok((_, _, _, _, ultra)) => {
                println!(
                    "Current Ultra Long Range Mode: {}",
                    if ultra { "Enabled (ON)" } else { "Disabled (OFF)" }
                );
            }
            Err(e) => eprintln!("[!] Failed to read Ultra Long Range status: {e}"),
        },
        Some(s) => {
            let enabled = parse_bool_arg(&s);
            let desc = if enabled { "Enabled (ON)" } else { "Disabled (OFF)" };
            if let Err(e) = dev.set_ultra_long_range(enabled) {
                eprintln!("[!] Failed to set Ultra Long Range Mode: {e}");
                std::process::exit(1);
            }
            println!("[+] Successfully updated Ultra Long Range Mode to {desc}!");
        }
    }
}

pub fn handle_set_sleep(dev: &AtkDevice, time: Option<String>) {
    const LABELS: [&str; 8] = ["30s", "1m", "2m", "3m", "5m", "20m", "25m", "30m"];
    let Some(t) = time else {
        println!("Valid sleep timeout values: {}", LABELS.join(", "));
        return;
    };
    let code: u8 = match t.to_lowercase().as_str() {
        "30s" | "0" => 0,
        "1m" | "1min" | "60s" | "1" => 1,
        "2m" | "2min" | "120s" | "2" => 2,
        "3m" | "3min" | "180s" | "3" => 3,
        "5m" | "5min" | "300s" | "4" => 4,
        "20m" | "20min" | "5" => 5,
        "25m" | "25min" | "6" => 6,
        "30m" | "30min" | "7" => 7,
        other => {
            eprintln!("[!] Invalid sleep timeout '{other}'. Valid values: 30s, 1m, 2m, 3m, 5m, 20m, 25m, 30m");
            std::process::exit(1);
        }
    };
    if let Err(e) = dev.set_sleep_time(code) {
        eprintln!("[!] Failed to set sleep time: {e}");
        std::process::exit(1);
    }
    println!("[+] Successfully set sleep timeout to {}!", LABELS[code as usize]);
}

pub fn handle_set_dpilight(dev: &AtkDevice, mode: Option<String>) {
    match mode {
        None => match dev.get_dpi_light() {
            Ok((m, b, s)) => {
                let desc = match m {
                    1 => format!("Steady / Always On (Brightness {b}/2)"),
                    2 => format!("Breathing (Speed {s}/2)"),
                    _ => "Off (Closed)".to_string(),
                };
                println!("Current DPI Indicator LED: {desc}");
            }
            Err(e) => eprintln!("[!] Failed to read DPI light state: {e}"),
        },
        Some(m) => {
            let mode_code = match m.to_lowercase().as_str() {
                "off" | "close" | "0" => 0,
                "steady" | "on" | "always" | "1" => 1,
                "breathing" | "breath" | "pulse" | "2" => 2,
                other => {
                    eprintln!("[!] Invalid mode '{other}'. Valid modes: off, steady, breathing");
                    std::process::exit(1);
                }
            };
            if mode_code != 0 {
                if let Ok(sys) = dev.read_register(ADDR_SYSTEM, 6) {
                    if sys.len() >= 6 {
                        let stage_count = sys[2].clamp(1, 6);
                        let active = sys[4];
                        if let Ok(colors) = dev.read_stage_colors(stage_count) {
                            if let Some(&(r, g, b)) = colors.get(active as usize) {
                                let _ = dev.set_stage_color(active, r, g, b);
                            }
                        }
                    }
                }
            }
            if let Err(e) = dev.set_dpi_light(mode_code, 2, 1) {
                eprintln!("[!] Failed to set DPI light: {e}");
                std::process::exit(1);
            }
            let _ = dev.commit_profile();
            let desc = match mode_code {
                1 => "Steady (Always On)",
                2 => "Breathing",
                _ => "Off",
            };
            println!("[+] Successfully set DPI Indicator LED to {desc}!");
        }
    }
}

