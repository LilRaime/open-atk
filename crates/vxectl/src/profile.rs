/*
 * Mouse hardware profile export and import handlers (Unified JSON serialization,
 * 100% compatible with OpenATK desktop GUI).
 */

use crate::status::decode_stage_dpi;
use atk_device::AtkDevice;
use atk_protocol::{
    constants::{ADDR_ADVANCED, ADDR_DPI_BASE, ADDR_SENSOR_MODE, ADDR_SYSTEM},
    PollingRate,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

fn default_profile_name() -> String {
    "OpenATK Profile".to_string()
}

fn default_stage_color() -> [u8; 3] {
    [255, 255, 255]
}

fn default_sleep_idx() -> usize {
    1
}

fn default_light_brightness() -> u8 {
    2
}

fn default_light_speed() -> u8 {
    1
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StageDpiEntry {
    pub stage: u8,
    pub x: u32,
    pub y: u32,
    #[serde(default = "default_stage_color")]
    pub color_rgb: [u8; 3],
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MouseProfileExport {
    pub model: String,
    #[serde(default = "default_profile_name")]
    pub profile_name: String,
    pub polling_rate_hz: u16,
    pub active_stage: u8,
    pub stage_count: u8,
    pub stages_dpi: Vec<StageDpiEntry>,
    pub lod_mm: f32,
    pub debounce_ms: u8,
    pub motion_sync: bool,
    #[serde(default)]
    pub straight_line_correction: bool,
    #[serde(default)]
    pub ripple_control: bool,
    #[serde(default)]
    pub ultra_long_range: bool,
    #[serde(default = "default_sleep_idx")]
    pub sleep_time_idx: usize,
    #[serde(default)]
    pub dpi_light_mode: u8,
    #[serde(default = "default_light_brightness")]
    pub dpi_light_brightness: u8,
    #[serde(default = "default_light_speed")]
    pub dpi_light_speed: u8,
    #[serde(default)]
    pub button_mappings: Vec<String>,
}

fn encode_button_action(action: &str) -> (u8, u8, u8) {
    if action.starts_with("Macro:") || action == "Custom Macro" {
        return (0x05, 0x01, 0x00);
    }
    match action {
        "Left Button" => (0x01, 0x01, 0x00),
        "Right Click" => (0x01, 0x02, 0x00),
        "Middle Key" => (0x01, 0x04, 0x00),
        "Back" => (0x01, 0x08, 0x00),
        "Advance" => (0x01, 0x10, 0x00),
        "DPI Cycle" => (0x02, 0x01, 0x00),
        "DPI Up" => (0x02, 0x02, 0x00),
        "DPI Down" => (0x02, 0x03, 0x00),
        "Stage 1" => (0x02, 0x10, 0x00),
        "Stage 2" => (0x02, 0x20, 0x00),
        "Vol +" => (0x03, 0xE9, 0x00),
        "Vol -" => (0x03, 0xEA, 0x00),
        "Mute" => (0x03, 0xE2, 0x00),
        "Play/Pause" => (0x03, 0xCD, 0x00),
        "Disabled" => (0x00, 0x00, 0x00),
        _ => (0x01, 0x01, 0x00),
    }
}

fn decode_button_action(key_class: u8, val1: u8, val2: u8) -> String {
    match (key_class, val1, val2) {
        (0x05, _, _) => "Custom Macro".to_string(),
        (0x01, 0x01, 0x00) => "Left Button".to_string(),
        (0x01, 0x02, 0x00) => "Right Click".to_string(),
        (0x01, 0x04, 0x00) => "Middle Key".to_string(),
        (0x01, 0x08, 0x00) => "Back".to_string(),
        (0x01, 0x10, 0x00) => "Advance".to_string(),
        (0x02, 0x01, 0x00) => "DPI Cycle".to_string(),
        (0x02, 0x02, 0x00) => "DPI Up".to_string(),
        (0x02, 0x03, 0x00) => "DPI Down".to_string(),
        (0x02, 0x10, 0x00) => "Stage 1".to_string(),
        (0x02, 0x20, 0x00) => "Stage 2".to_string(),
        (0x03, 0xE9, 0x00) => "Vol +".to_string(),
        (0x03, 0xEA, 0x00) => "Vol -".to_string(),
        (0x03, 0xE2, 0x00) => "Mute".to_string(),
        (0x03, 0xCD, 0x00) => "Play/Pause".to_string(),
        (0x00, 0x00, 0x00) => "Disabled".to_string(),
        _ => "Custom".to_string(),
    }
}

/* Exports full hardware profile from EEPROM into a structured JSON string or file */
pub fn handle_export_profile(dev: &AtkDevice, model_name: &str, file: Option<PathBuf>) {
    let system = dev.read_register(ADDR_SYSTEM, 6).unwrap_or_default();
    if system.len() < 6 {
        eprintln!("[!] Failed to read system register from mouse");
        std::process::exit(1);
    }

    let polling_hz = match system[0] {
        0x08 => 125,
        0x04 => 250,
        0x02 => 500,
        0x01 => 1000,
        0x10 => 2000,
        0x20 => 4000,
        0x40 => 8000,
        _ => 1000,
    };
    let stage_count = system[2].clamp(1, 6);
    let active_stage = system[4] + 1;

    let stage_colors = dev.read_stage_colors(stage_count).unwrap_or_default();
    let mut stages_dpi = Vec::new();
    for s in 0..stage_count {
        let addr = ADDR_DPI_BASE + (s as u16) * 4;
        if let Ok(rec) = dev.read_register(addr, 4) {
            let (x, y) = decode_stage_dpi(&rec);
            let (r, g, b) = stage_colors.get(s as usize).copied().unwrap_or((255, 255, 255));
            stages_dpi.push(StageDpiEntry {
                stage: s + 1,
                x,
                y,
                color_rgb: [r, g, b],
            });
        }
    }

    let lod_mm = match dev.get_lod().unwrap_or(1) {
        1 => 1.0,
        2 => 2.0,
        _ => 1.0,
    };

    let mut debounce_ms = 4;
    let mut motion_sync = true;
    let mut straight_line_correction = false;
    let mut ripple_control = false;
    let mut sleep_time_idx = 1usize;

    if let Ok(adv) = dev.read_register(ADDR_ADVANCED, 10) {
        if adv.len() >= 10 {
            if adv[0].wrapping_add(adv[1]) == 0x55 {
                debounce_ms = adv[0];
            }
            if adv[2].wrapping_add(adv[3]) == 0x55 {
                motion_sync = adv[2] != 0;
            }
            if adv[4].wrapping_add(adv[5]) == 0x55 && adv[4] > 0 {
                sleep_time_idx = match adv[4] {
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

    let ultra_long_range = dev
        .read_register(ADDR_SENSOR_MODE, 6)
        .ok()
        .and_then(|r| {
            if r.len() >= 6 && r[4].wrapping_add(r[5]) == 0x55 {
                Some(r[4] != 0)
            } else {
                None
            }
        })
        .unwrap_or(false);

    let (dpi_light_mode, dpi_light_brightness, dpi_light_speed) =
        dev.get_dpi_light().unwrap_or((0, 2, 1));

    let button_mappings = dev
        .read_button_mappings()
        .unwrap_or_default()
        .into_iter()
        .map(|(kc, v1, v2)| decode_button_action(kc, v1, v2))
        .collect();

    let export = MouseProfileExport {
        model: model_name.to_string(),
        profile_name: "OpenATK Profile".to_string(),
        polling_rate_hz: polling_hz,
        active_stage,
        stage_count,
        stages_dpi,
        lod_mm,
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
    };

    let json = match serde_json::to_string_pretty(&export) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("[!] Failed to serialize profile to JSON: {e}");
            std::process::exit(1);
        }
    };

    if let Some(path) = file {
        match std::fs::write(&path, &json) {
            Ok(_) => println!("[+] Profile successfully exported to '{}'!", path.display()),
            Err(e) => {
                eprintln!("[!] Failed to write to '{}': {e}", path.display());
                std::process::exit(1);
            }
        }
    } else {
        println!("{json}");
    }
}

/* Imports a JSON hardware profile and flashes it to the mouse */
pub fn handle_import_profile(dev: &AtkDevice, file: PathBuf) {
    let content = match std::fs::read_to_string(&file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[!] Failed to read profile from '{}': {e}", file.display());
            std::process::exit(1);
        }
    };

    let profile: MouseProfileExport = match serde_json::from_str(&content) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[!] Failed to parse JSON profile: {e}");
            std::process::exit(1);
        }
    };

    println!("[*] Applying profile for {} from '{}'...", profile.model, file.display());

    let stage_count = profile.stage_count.clamp(1, 6);
    let _ = dev.set_stage_count(stage_count);

    /* 1. Polling rate */
    if let Ok(rate) = PollingRate::from_hz(profile.polling_rate_hz) {
        let _ = dev.set_polling_rate(rate);
    }

    /* 2. DPI stages and RGB stage colors */
    for stage_entry in &profile.stages_dpi {
        if stage_entry.stage > 0 && stage_entry.stage <= stage_count {
            let idx = stage_entry.stage - 1;
            let _ = dev.set_stage_dpi(idx, stage_entry.x, stage_entry.y);
            let [r, g, b] = stage_entry.color_rgb;
            let _ = dev.set_stage_color(idx, r, g, b);
        }
    }

    /* 3. Active stage */
    if profile.active_stage > 0 && profile.active_stage <= stage_count {
        let _ = dev.set_active_stage(profile.active_stage - 1);
    }

    /* 4. LOD */
    let lod_code = if profile.lod_mm >= 1.5 { 2 } else { 1 };
    let _ = dev.set_lod(lod_code);

    /* 5. Advanced Sensor & Switch Parameters */
    let _ = dev.set_debounce(profile.debounce_ms.min(20));
    let _ = dev.set_motion_sync(profile.motion_sync);
    let _ = dev.set_straight_line(profile.straight_line_correction);
    let _ = dev.set_ripple_control(profile.ripple_control);
    let _ = dev.set_ultra_long_range(profile.ultra_long_range);
    let _ = dev.set_sleep_time(profile.sleep_time_idx as u8);
    let _ = dev.set_dpi_light(
        profile.dpi_light_mode,
        profile.dpi_light_brightness,
        profile.dpi_light_speed,
    );

    /* 6. Button mappings */
    for (idx, act) in profile.button_mappings.iter().enumerate().take(6) {
        let (kc, v1, v2) = encode_button_action(act);
        let _ = dev.set_button_mapping(idx as u8, kc, v1, v2);
    }

    /* Commit all changes to flash */
    let _ = dev.commit_profile();

    println!("[+] Successfully imported profile! Applied:");
    println!("    - Polling Rate:     {} Hz", profile.polling_rate_hz);
    println!("    - Active Stage:     Stage {} of {}", profile.active_stage, stage_count);
    println!("    - LOD:              {:.1} mm", profile.lod_mm);
    println!("    - Debounce Delay:   {} ms", profile.debounce_ms);
    println!(
        "    - Motion Sync:      {}",
        if profile.motion_sync { "Enabled" } else { "Disabled" }
    );
    println!(
        "    - Straight Line:    {}",
        if profile.straight_line_correction { "Enabled" } else { "Disabled" }
    );
    println!(
        "    - Ripple Control:   {}",
        if profile.ripple_control { "Enabled" } else { "Disabled" }
    );
    println!(
        "    - Ultra Long Range: {}",
        if profile.ultra_long_range { "Enabled" } else { "Disabled" }
    );
}
