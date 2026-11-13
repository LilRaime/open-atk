/* Application state container, UI event handlers, and hardware command dispatchers. */

#![allow(dead_code)]

pub use crate::models::*;
pub use crate::preferences::*;
pub use crate::worker::APP_FOCUSED;

use crate::worker::spawn_device_worker;
use atk_device::ConnectionMode;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, Instant};

pub struct AtkControlApp {
    pub active_nav: NavItem,
    pub mouse: MouseData,
    pub connected: bool,
    pub status_message: Option<(String, Instant, bool)>,
    pub last_poll: Instant,
    pub selected_button_idx: Option<usize>,
    pub mouse_color: MouseColor,
    pub preferences: AppPreferences,
    pub mouse_texture: Option<eframe::egui::TextureHandle>,
    pub mouse_white_texture: Option<eframe::egui::TextureHandle>,
    pub mouse_black_texture: Option<eframe::egui::TextureHandle>,
    pub vxe_r1_white_texture: Option<eframe::egui::TextureHandle>,
    pub vxe_r1_black_texture: Option<eframe::egui::TextureHandle>,
    pub vgn_f1_white_texture: Option<eframe::egui::TextureHandle>,
    pub vgn_f1_black_texture: Option<eframe::egui::TextureHandle>,
    pub atk_madr_white_texture: Option<eframe::egui::TextureHandle>,
    pub atk_madr_black_texture: Option<eframe::egui::TextureHandle>,
    pub atk_logo_texture: Option<eframe::egui::TextureHandle>,
    pub device_list: Vec<DeviceEntry>,
    pub selected_device_idx: usize,
    pub selected_device_id: String,
    pub device_selector_open: bool,
    pub action_tx: Option<Sender<DeviceAction>>,
    pub update_rx: Option<Receiver<DeviceUpdate>>,
    pub macros: Vec<MacroItem>,
    pub selected_macro_idx: Option<usize>,
    pub recording_key_mode: bool,
    pub new_macro_key_input: String,
    pub new_macro_delay_input: u32,
    pub last_low_battery_warn: Option<Instant>,
    pub color_picker_open_stage: Option<u8>,
    pub is_minimized_to_tray: bool,
    pub startup_time: Instant,
    pub initial_trim_done: bool,
}

impl AtkControlApp {
    pub fn mouse_texture_for(&self, model_name: &str, color: MouseColor) -> Option<&eframe::egui::TextureHandle> {
        let name_upper = model_name.to_uppercase();
        if name_upper.contains("MAD") {
            match color {
                MouseColor::White => self.atk_madr_white_texture.as_ref().or(self.atk_madr_black_texture.as_ref()),
                MouseColor::Black => self.atk_madr_black_texture.as_ref().or(self.atk_madr_white_texture.as_ref()),
            }
        } else if name_upper.contains("F1") || name_upper.contains("DRAGONFLY") {
            match color {
                MouseColor::White => self.vgn_f1_white_texture.as_ref().or(self.vgn_f1_black_texture.as_ref()),
                MouseColor::Black => self.vgn_f1_black_texture.as_ref().or(self.vgn_f1_white_texture.as_ref()),
            }
        } else {
            match color {
                MouseColor::White => self.vxe_r1_white_texture.as_ref().or(self.mouse_white_texture.as_ref()),
                MouseColor::Black => self.vxe_r1_black_texture.as_ref().or(self.mouse_black_texture.as_ref()),
            }
        }
    }

    pub fn current_mouse_texture(&self) -> Option<&eframe::egui::TextureHandle> {
        let model = self.active_device_name();
        self.mouse_texture_for(model, self.mouse_color)
    }

    pub fn set_active_mouse_color(&mut self, color: MouseColor) {
        self.mouse_color = color;
        let mut dev_id = String::new();
        let mut dev_model = self.mouse.model_name.clone();

        if let Some(entry) = self.device_list.get_mut(self.selected_device_idx) {
            entry.color = color;
            dev_id = entry.id.clone();
            dev_model = entry.model_name.clone();
        }

        self.preferences.set_device_color(&dev_id, &dev_model, color);
        let color_name = match color {
            MouseColor::White => "White",
            MouseColor::Black => "Black",
        };
        self.set_status(format!("Mouse color set to {}", color_name), false);
    }

    pub fn select_device(&mut self, idx: usize) {
        if idx < self.device_list.len() {
            self.selected_device_idx = idx;
            let dev = self.device_list[idx].clone();
            self.selected_device_id = dev.id.clone();
            self.mouse.model_name = dev.model_name;
            self.mouse.mode = dev.mode;
            self.mouse.battery_pct = dev.battery_pct;
            self.mouse.is_charging = dev.is_charging;
            self.mouse_color = dev.color;
            if let Some(tx) = &self.action_tx {
                let _ = tx.send(DeviceAction::SwitchDevice(dev.id));
            }
        }
    }

    pub fn active_device_name(&self) -> &str {
        if let Some(entry) = self.device_list.get(self.selected_device_idx) {
            &entry.model_name
        } else if !self.mouse.model_name.is_empty() && self.mouse.model_name != "No Device Connected" {
            &self.mouse.model_name
        } else {
            "ATK Mouse"
        }
    }

    pub fn new() -> Self {
        let (action_tx, action_rx) = channel::<DeviceAction>();
        let (update_tx, update_rx) = channel::<DeviceUpdate>();

        spawn_device_worker(action_rx, update_tx);

        /* Built-in starter macros */
        let default_macros = vec![
            MacroItem {
                id: "rapid_fire".to_string(),
                name: "Rapid Fire (LMB Spam)".to_string(),
                repeat_count: 5,
                trigger_mode: MacroTriggerMode::Once,
                steps: vec![
                    MacroStep::MouseDown("Left".to_string()),
                    MacroStep::DelayMs(20),
                    MacroStep::MouseUp("Left".to_string()),
                    MacroStep::DelayMs(30),
                ],
            },
            MacroItem {
                id: "triple_click".to_string(),
                name: "Triple Click Burst".to_string(),
                repeat_count: 3,
                trigger_mode: MacroTriggerMode::Once,
                steps: vec![
                    MacroStep::MouseDown("Left".to_string()),
                    MacroStep::DelayMs(15),
                    MacroStep::MouseUp("Left".to_string()),
                    MacroStep::DelayMs(25),
                ],
            },
            MacroItem {
                id: "jump_throw".to_string(),
                name: "CS2 Jump-Throw Combo".to_string(),
                repeat_count: 1,
                trigger_mode: MacroTriggerMode::Once,
                steps: vec![
                    MacroStep::KeyDown("Space".to_string()),
                    MacroStep::DelayMs(10),
                    MacroStep::MouseUp("Left".to_string()),
                    MacroStep::DelayMs(20),
                    MacroStep::KeyUp("Space".to_string()),
                ],
            },
        ];

        let preferences = AppPreferences::load();
        let initial_color = preferences.get_color("", "VXE R1 PRO MAX");

        Self {
            active_nav: NavItem::KeyChange,
            mouse: MouseData::default(),
            connected: false,
            status_message: None,
            last_poll: Instant::now(),
            selected_button_idx: None,
            mouse_color: initial_color,
            preferences,
            mouse_texture: None,
            mouse_white_texture: None,
            mouse_black_texture: None,
            vxe_r1_white_texture: None,
            vxe_r1_black_texture: None,
            vgn_f1_white_texture: None,
            vgn_f1_black_texture: None,
            atk_madr_white_texture: None,
            atk_madr_black_texture: None,
            atk_logo_texture: None,
            device_list: Vec::new(),
            selected_device_idx: 0,
            selected_device_id: String::new(),
            device_selector_open: false,
            action_tx: Some(action_tx),
            update_rx: Some(update_rx),
            macros: default_macros,
            selected_macro_idx: Some(0),
            recording_key_mode: false,
            new_macro_key_input: String::new(),
            new_macro_delay_input: 50,
            last_low_battery_warn: None,
            color_picker_open_stage: None,
            is_minimized_to_tray: false,
            startup_time: Instant::now(),
            initial_trim_done: false,
        }
    }

    pub fn drain_device_updates(&mut self) {
        let mut updates = Vec::new();
        if let Some(rx) = &self.update_rx {
            while let Ok(update) = rx.try_recv() {
                updates.push(update);
            }
        }
        for update in updates {
            match update {
                DeviceUpdate::Disconnected => {
                    self.connected = false;
                    self.device_list.clear();
                    self.mouse.model_name = "No Device Connected".to_string();
                    self.mouse.battery_pct = 0;
                    self.mouse.is_charging = false;
                    self.selected_device_idx = 0;
                    self.selected_device_id.clear();
                }
                DeviceUpdate::DeviceList(mut list) => {
                    /* Apply persistent user color preferences for each detected device */
                    for entry in &mut list {
                        entry.color = self.preferences.get_color(&entry.id, &entry.model_name);
                    }
                    self.device_list = list;
                    self.connected = !self.device_list.is_empty();
                    /* If we previously selected a device by ID, preserve that exact device selection */
                    if !self.selected_device_id.is_empty() {
                        if let Some(pos) = self.device_list.iter().position(|e| e.id == self.selected_device_id) {
                            self.selected_device_idx = pos;
                        }
                    }
                    if self.selected_device_idx >= self.device_list.len() {
                        self.selected_device_idx = 0;
                    }
                    if let Some(selected) = self.device_list.get(self.selected_device_idx) {
                        self.selected_device_id = selected.id.clone();
                        self.mouse.model_name = selected.model_name.clone();
                        self.mouse.mode = selected.mode;
                        if selected.battery_pct > 0 {
                            self.mouse.battery_pct = selected.battery_pct;
                            self.mouse.is_charging = selected.is_charging || selected.mode == ConnectionMode::Wired;
                        }
                        self.mouse_color = selected.color;
                    }
                }
                DeviceUpdate::Status {
                    device_id,
                    battery_pct,
                    is_charging,
                    voltage_mv,
                    active_stage,
                    stage_count,
                } => {
                    self.connected = true;
                    if let Some(entry) = self.device_list.iter_mut().find(|e| e.id == device_id) {
                        if battery_pct > 0 {
                            entry.battery_pct = battery_pct;
                            entry.is_charging = is_charging || entry.mode == ConnectionMode::Wired;
                        }
                    }
                    if self.selected_device_id.is_empty() || self.selected_device_id == device_id {
                        if battery_pct > 0 {
                            self.mouse.battery_pct = battery_pct;
                            self.mouse.is_charging = is_charging || self.mouse.mode == ConnectionMode::Wired;
                            self.mouse.voltage_mv = voltage_mv;
                        }
                        self.mouse.active_stage = active_stage;
                        if stage_count > 0 {
                            self.mouse.stage_count = stage_count;
                        }
                    }
                }
                DeviceUpdate::FullRefresh {
                    device_id,
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
                } => {
                    self.connected = true;
                    let is_target_selected = self.selected_device_id.is_empty() || self.selected_device_id == device_id;
                    if let Some(entry) = self.device_list.iter_mut().find(|e| e.id == device_id) {
                        let name_upper = model_name.to_uppercase();
                        let is_generic = name_upper.contains("GAMING MOUSE")
                            || name_upper.contains("DONGLE")
                            || name_upper.contains("RECEIVER");
                        let cur_is_specific = !entry.model_name.is_empty()
                            && !entry.model_name.to_uppercase().contains("GAMING MOUSE");
                        if !model_name.is_empty() && (!is_generic || !cur_is_specific) {
                            entry.model_name = model_name.clone();
                            entry.color = self.preferences.get_color(&entry.id, &entry.model_name);
                        }
                        if battery_pct > 0 {
                            entry.battery_pct = battery_pct;
                            entry.is_charging = is_charging || entry.mode == ConnectionMode::Wired;
                        }
                        if is_target_selected {
                            self.mouse.model_name = entry.model_name.clone();
                            self.mouse_color = entry.color;
                        }
                    }

                    if is_target_selected {
                        if !model_name.is_empty() && (!model_name.to_uppercase().contains("GAMING MOUSE") || self.mouse.model_name.is_empty()) {
                            self.mouse.model_name = model_name;
                        }
                        self.mouse.vendor_id = vendor_id;
                    self.mouse.product_id = product_id;
                    self.mouse.cid = cid;
                    self.mouse.mid = mid;
                    self.mouse.firmware = firmware;
                    if battery_pct > 0 {
                        self.mouse.battery_pct = battery_pct;
                        self.mouse.is_charging = is_charging || self.mouse.mode == ConnectionMode::Wired;
                        self.mouse.voltage_mv = voltage_mv;
                    }
                    self.mouse.rf_id = rf_id;
                    self.mouse.polling_rate = polling_rate;
                    self.mouse.active_stage = active_stage;
                    self.mouse.stage_count = stage_count;
                    if !stages_dpi.is_empty() {
                        self.mouse.stages_dpi = stages_dpi;
                    }
                    if !stage_colors.is_empty() {
                        self.mouse.stage_colors = stage_colors;
                    }
                    while self.mouse.stage_colors.len() < self.mouse.stage_count as usize {
                        let palette = [
                            (155, 93, 229),
                            (255, 255, 255),
                            (0, 180, 255),
                            (255, 0, 128),
                            (0, 230, 118),
                            (255, 214, 0),
                        ];
                        let idx = self.mouse.stage_colors.len();
                        self.mouse.stage_colors.push(palette.get(idx).copied().unwrap_or((255, 255, 255)));
                    }
                    while self.mouse.independent_xy.len() < self.mouse.stage_count as usize {
                        self.mouse.independent_xy.push(false);
                    }
                    for (idx, &(x, y)) in self.mouse.stages_dpi.iter().enumerate() {
                        if x != y && idx < self.mouse.independent_xy.len() {
                            self.mouse.independent_xy[idx] = true;
                        }
                    }
                    self.mouse.lod_code = lod_code;
                    self.mouse.debounce_ms = debounce_ms;
                    self.mouse.motion_sync = motion_sync;
                    self.mouse.straight_line_correction = straight_line_correction;
                    self.mouse.ripple_control = ripple_control;
                    self.mouse.ultra_long_range = ultra_long_range;
                    self.mouse.sleep_time_idx = sleep_time_idx;
                    self.mouse.dpi_light_mode = dpi_light_mode;
                    self.mouse.dpi_light_brightness = dpi_light_brightness;
                    self.mouse.dpi_light_speed = dpi_light_speed;
                    for (idx, (kc, v1, v2)) in button_mappings.into_iter().enumerate() {
                        if let Some(btn) = self.mouse.buttons.get_mut(idx) {
                            if !btn.action.starts_with("Macro:") {
                                btn.action = decode_button_action(kc, v1, v2);
                            }
                        }
                    }
                    }
                }
                DeviceUpdate::Toast(msg, is_err) => {
                    self.set_status(msg, is_err);
                }
            }
        }
    }

    pub fn poll_status_fast(&mut self) {
        self.drain_device_updates();
    }

    pub fn poll_battery_fast(&mut self) {
        self.drain_device_updates();
    }

    pub fn set_status(&mut self, msg: impl Into<String>, is_error: bool) {
        let text = msg.into();
        let dev_name = self.active_device_name();

        /* Strip any existing bracketed prefix if present so we always apply the true active device name */
        let clean_text = if text.starts_with('[') {
            if let Some(pos) = text.find(']') {
                text[pos + 1..].trim_start()
            } else {
                &text
            }
        } else {
            &text
        };

        let formatted = if self.device_list.len() >= 2
            && !dev_name.is_empty()
            && dev_name != "No Device Connected"
        {
            format!("[{}] {}", dev_name, clean_text)
        } else {
            clean_text.to_string()
        };
        self.status_message = Some((formatted, Instant::now(), is_error));
    }

    pub fn refresh_hardware(&mut self) {
        if let Some(tx) = &self.action_tx {
            let _ = tx.send(DeviceAction::RefreshHardware);
        }
        self.set_status("Refreshing hardware from mouse...", false);
    }

    /* Check if battery is low and deliver a desktop toast notification */
    pub fn check_low_battery_warning(&mut self) {
        if self.connected
            && self.preferences.low_battery_warning
            && !self.mouse.is_charging
            && self.mouse.battery_pct > 0
            && self.mouse.battery_pct <= 15
        {
            let should_warn = self
                .last_low_battery_warn
                .map(|t| t.elapsed() > Duration::from_secs(900))
                .unwrap_or(true);
            if should_warn {
                self.last_low_battery_warn = Some(Instant::now());
                show_desktop_notification(
                    "OpenATK - Low Battery",
                    &format!(
                        "{} battery is at {}%. Please connect USB-C cable.",
                        self.mouse.model_name, self.mouse.battery_pct
                    ),
                );
            }
        }
    }

    /* Export user macros to an external JSON file */
    pub fn export_macros_to_disk(&mut self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.macros) {
            let path = std::path::PathBuf::from("open_atk_macros.json");
            match std::fs::write(&path, json) {
                Ok(_) => self.set_status(format!("Exported {} macros to {}", self.macros.len(), path.display()), false),
                Err(e) => self.set_status(format!("Failed to export macros: {e}"), true),
            }
        }
    }

    /* Import user macros from an external JSON file */
    pub fn import_macros_from_disk(&mut self) {
        let path = std::path::PathBuf::from("open_atk_macros.json");
        match std::fs::read_to_string(&path) {
            Ok(json) => match serde_json::from_str::<Vec<MacroItem>>(&json) {
                Ok(imported) => {
                    let mut count = 0;
                    for item in imported {
                        if !self.macros.iter().any(|m| m.id == item.id) {
                            self.macros.push(item);
                            count += 1;
                        }
                    }
                    self.set_status(format!("Imported {} new macro(s) from {}", count, path.display()), false);
                }
                Err(e) => self.set_status(format!("Failed to parse macros JSON: {e}"), true),
            },
            Err(_) => self.set_status(format!("File '{}' not found. Export first!", path.display()), true),
        }
    }

    pub fn apply_active_stage(&mut self, stage_idx: u8) {
        self.mouse.active_stage = stage_idx;
        if let Some(tx) = &self.action_tx {
            let _ = tx.send(DeviceAction::SetActiveStage(stage_idx));
        }
        self.set_status(format!("Switched to Stage {}", stage_idx + 1), false);
    }

    pub fn apply_dpi(&mut self, stage_idx: u8, x: u32, y: u32) {
        if let Some(st) = self.mouse.stages_dpi.get_mut(stage_idx as usize) {
            *st = (x, y);
        }
        if let Some(tx) = &self.action_tx {
            let _ = tx.send(DeviceAction::SetStageDpi { stage: stage_idx, x, y });
        }
        self.set_status(format!("Stage {} DPI saved: {} x {}", stage_idx + 1, x, y), false);
    }

    pub fn apply_stage_color(&mut self, stage: u8, r: u8, g: u8, b: u8) {
        let s_idx = stage as usize;
        if s_idx < self.mouse.stage_colors.len() {
            self.mouse.stage_colors[s_idx] = (r, g, b);
        } else {
            while self.mouse.stage_colors.len() <= s_idx {
                self.mouse.stage_colors.push((255, 255, 255));
            }
            self.mouse.stage_colors[s_idx] = (r, g, b);
        }
        if let Some(tx) = &self.action_tx {
            let _ = tx.send(DeviceAction::SetStageColor { stage, r, g, b });
            /* If DPI indicator LED is enabled and this is the active stage, refresh LED state so hardware reflects the new color immediately */
            if self.mouse.dpi_light_mode != 0 && stage == self.mouse.active_stage {
                let _ = tx.send(DeviceAction::SetDpiLight {
                    mode: self.mouse.dpi_light_mode,
                    brightness: self.mouse.dpi_light_brightness,
                    speed: self.mouse.dpi_light_speed,
                });
            }
        }
        self.set_status(format!("DPI {} Color updated", stage + 1), false);
    }

    pub fn apply_add_stage(&mut self) {
        if self.mouse.stage_count >= 6 {
            self.set_status("Maximum 6 DPI stages reached", true);
            return;
        }
        let new_count = self.mouse.stage_count + 1;
        let new_idx = self.mouse.stage_count;
        self.mouse.stage_count = new_count;

        let default_palette = [
            (155, 93, 229),
            (255, 255, 255),
            (0, 180, 255),
            (255, 0, 128),
            (0, 230, 118),
            (255, 214, 0),
        ];
        let new_color = default_palette.get(new_idx as usize).copied().unwrap_or((255, 255, 255));
        let last_dpi = self.mouse.stages_dpi.last().copied().unwrap_or((1600, 1600));

        if self.mouse.stages_dpi.len() < new_count as usize {
            self.mouse.stages_dpi.push(last_dpi);
        }
        if self.mouse.stage_colors.len() < new_count as usize {
            self.mouse.stage_colors.push(new_color);
        }
        if self.mouse.independent_xy.len() < new_count as usize {
            self.mouse.independent_xy.push(false);
        }

        if let Some(tx) = &self.action_tx {
            let _ = tx.send(DeviceAction::SetStageCount(new_count));
            let _ = tx.send(DeviceAction::SetStageDpi {
                stage: new_idx,
                x: last_dpi.0,
                y: last_dpi.1,
            });
            let _ = tx.send(DeviceAction::SetStageColor {
                stage: new_idx,
                r: new_color.0,
                g: new_color.1,
                b: new_color.2,
            });
        }
        self.set_status(format!("Added DPI Stage {}", new_count), false);
    }

    pub fn apply_delete_stage(&mut self, stage_idx: u8) {
        if self.mouse.stage_count <= 1 {
            self.set_status("At least 1 DPI stage is required", true);
            return;
        }
        let idx = stage_idx as usize;
        if idx < self.mouse.stages_dpi.len() {
            self.mouse.stages_dpi.remove(idx);
        }
        if idx < self.mouse.stage_colors.len() {
            self.mouse.stage_colors.remove(idx);
        }
        if idx < self.mouse.independent_xy.len() {
            self.mouse.independent_xy.remove(idx);
        }
        let new_count = self.mouse.stage_count - 1;
        self.mouse.stage_count = new_count;

        if self.mouse.active_stage >= new_count {
            self.mouse.active_stage = new_count.saturating_sub(1);
            if let Some(tx) = &self.action_tx {
                let _ = tx.send(DeviceAction::SetActiveStage(self.mouse.active_stage));
            }
        }

        if let Some(tx) = &self.action_tx {
            let _ = tx.send(DeviceAction::SetStageCount(new_count));
            for (i, &(x, y)) in self.mouse.stages_dpi.iter().enumerate().take(new_count as usize) {
                let _ = tx.send(DeviceAction::SetStageDpi { stage: i as u8, x, y });
                if let Some(&(r, g, b)) = self.mouse.stage_colors.get(i) {
                    let _ = tx.send(DeviceAction::SetStageColor { stage: i as u8, r, g, b });
                }
            }
        }
        self.set_status(format!("Removed DPI Stage {}", stage_idx + 1), false);
    }

    pub fn apply_polling(&mut self, hz: u16) {
        self.mouse.polling_rate = hz;
        if let Some(tx) = &self.action_tx {
            let _ = tx.send(DeviceAction::SetPolling(hz));
        }
        self.set_status(format!("Polling Rate: {} Hz", hz), false);
    }

    pub fn apply_debounce(&mut self, ms: u8) {
        self.mouse.debounce_ms = ms;
        if let Some(tx) = &self.action_tx {
            let _ = tx.send(DeviceAction::SetDebounce(ms));
        }
        self.set_status(format!("Debounce: {} ms", ms), false);
    }

    pub fn apply_motion_sync(&mut self, enabled: bool) {
        self.mouse.motion_sync = enabled;
        if let Some(tx) = &self.action_tx {
            let _ = tx.send(DeviceAction::SetMotionSync(enabled));
        }
        self.set_status(format!("Motion Sync: {}", if enabled { "ON" } else { "OFF" }), false);
    }

    pub fn apply_straight_line(&mut self, enabled: bool) {
        self.mouse.straight_line_correction = enabled;
        if let Some(tx) = &self.action_tx {
            let _ = tx.send(DeviceAction::SetStraightLine(enabled));
        }
        self.set_status(
            format!("Straight Line Correction: {}", if enabled { "ON" } else { "OFF" }),
            false,
        );
    }

    pub fn apply_ripple_control(&mut self, enabled: bool) {
        self.mouse.ripple_control = enabled;
        if let Some(tx) = &self.action_tx {
            let _ = tx.send(DeviceAction::SetRipple(enabled));
        }
        self.set_status(
            format!("Ripple Correction: {}", if enabled { "ON" } else { "OFF" }),
            false,
        );
    }

    pub fn apply_ultra_long_range(&mut self, enabled: bool) {
        self.mouse.ultra_long_range = enabled;
        if let Some(tx) = &self.action_tx {
            let _ = tx.send(DeviceAction::SetUltraLongRange(enabled));
        }
        self.set_status(
            format!("Ultra Long Range Mode: {}", if enabled { "ON" } else { "OFF" }),
            false,
        );
    }

    pub fn apply_sleep_time(&mut self, idx: usize) {
        self.mouse.sleep_time_idx = idx;
        if let Some(tx) = &self.action_tx {
            let _ = tx.send(DeviceAction::SetSleepTime(idx));
        }
        const LABELS: [&str; 8] = ["30s", "1min", "2min", "3min", "5min", "20min", "25min", "30min"];
        let label = LABELS.get(idx).copied().unwrap_or("1min");
        self.set_status(format!("Sleep Time: {}", label), false);
    }

    pub fn apply_dpi_light(&mut self, mode: u8, brightness: u8, speed: u8) {
        self.mouse.dpi_light_mode = mode;
        self.mouse.dpi_light_brightness = brightness;
        self.mouse.dpi_light_speed = speed;
        if let Some(tx) = &self.action_tx {
            /* When turning on the DPI indicator LED, also sync the active stage RGB color to 0x002C so the LED matches the active DPI stage color */
            if mode != 0 {
                let active = self.mouse.active_stage;
                if let Some(&(r, g, b)) = self.mouse.stage_colors.get(active as usize) {
                    let _ = tx.send(DeviceAction::SetStageColor {
                        stage: active,
                        r,
                        g,
                        b,
                    });
                }
            }
            let _ = tx.send(DeviceAction::SetDpiLight { mode, brightness, speed });
        }
        let desc = match mode {
            1 => "Always On",
            2 => "Breathing",
            _ => "Off",
        };
        self.set_status(format!("DPI Light: {}", desc), false);
    }

    pub fn apply_lod(&mut self, code: u8) {
        self.mouse.lod_code = code;
        if let Some(tx) = &self.action_tx {
            let _ = tx.send(DeviceAction::SetLod(code));
        }
        let desc = match code {
            1 => "1.0 mm (Low)",
            2 => "2.0 mm (High)",
            _ => "Default",
        };
        self.set_status(format!("LOD: {}", desc), false);
    }

    pub fn apply_button_mapping(&mut self, btn_idx: usize, action: &str) {
        if action.starts_with("Macro:") {
            let macro_name = action.strip_prefix("Macro:").unwrap_or("").trim();
            if let Some(m) = self.macros.iter().find(|m| m.name == macro_name).cloned() {
                let bindings = crate::macro_player::get_global_macro_bindings();
                if let Ok(mut guard) = bindings.write() {
                    guard.insert(btn_idx, m);
                };
            }

            /* On hardware, keep native hardware button class so mouse physical switch sends
             * clean events to Windows that our global hook captures! */
            let (key_class, val1, val2) = match btn_idx {
                0 => (0x01, 0x01, 0x00),
                1 => (0x01, 0x02, 0x00),
                2 => (0x01, 0x04, 0x00),
                3 => (0x01, 0x08, 0x00),
                4 => (0x01, 0x10, 0x00),
                5 => (0x02, 0x01, 0x00),
                _ => (0x01, 0x01, 0x00),
            };

            if let Some(btn) = self.mouse.buttons.get_mut(btn_idx) {
                btn.action = action.to_string();
            }

            if let Some(tx) = &self.action_tx {
                let _ = tx.send(DeviceAction::SetButton {
                    btn_idx: btn_idx as u8,
                    key_class,
                    val1,
                    val2,
                });
            }

            let title = self.mouse.buttons.get(btn_idx).map(|b| b.title).unwrap_or("Button");
            self.set_status(format!("{} bound to {} (Active in all games/apps)", title, action), false);
            return;
        }

        /* Standard non-macro action: remove from global macro bindings */
        {
            let bindings = crate::macro_player::get_global_macro_bindings();
            if let Ok(mut guard) = bindings.write() {
                guard.remove(&btn_idx);
            };
        }

        let (key_class, val1, val2) = encode_button_action(action);
        if let Some(btn) = self.mouse.buttons.get_mut(btn_idx) {
            btn.action = action.to_string();
        }
        if let Some(tx) = &self.action_tx {
            let _ = tx.send(DeviceAction::SetButton {
                btn_idx: btn_idx as u8,
                key_class,
                val1,
                val2,
            });
        }
        let title = self.mouse.buttons.get(btn_idx).map(|b| b.title).unwrap_or("Button");
        self.set_status(format!("{} remapped to {}", title, action), false);
    }

    pub fn restore_defaults(&mut self) {
        {
            let bindings = crate::macro_player::get_global_macro_bindings();
            if let Ok(mut guard) = bindings.write() {
                guard.clear();
            };
        }

        for btn in &mut self.mouse.buttons {
            btn.action = btn.title.to_string();
        }
        if let Some(tx) = &self.action_tx {
            let _ = tx.send(DeviceAction::RestoreDefaults);
        }
        self.set_status("All button mappings restored to factory defaults", false);
    }
}
