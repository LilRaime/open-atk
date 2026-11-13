/* Configuration Management View (2-Column Unified JSON Hardware Profile Backup & Parameters) */

use crate::state::AtkControlApp;
use crate::theme::{COLOR_ACCENT_BLUE, COLOR_ACCENT_ORANGE, COLOR_TEXT_MUTED, COLOR_TEXT_PRIMARY};
use eframe::egui::{self, Color32, Layout, RichText, Rounding, Stroke, Ui, Vec2};
use serde::{Deserialize, Serialize};

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
pub struct ExportedProfile {
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

pub fn render_config_management_view(ui: &mut Ui, app: &mut AtkControlApp) {
    ui.label(
        RichText::new("Configuration Management")
            .size(22.0)
            .strong()
            .color(COLOR_TEXT_PRIMARY),
    );
    ui.add(
        egui::Label::new(
            RichText::new("Full hardware profile backup, JSON export and import (100% compatible with vxectl)")
                .size(13.0)
                .color(COLOR_TEXT_MUTED),
        )
        .wrap(),
    );
    ui.add_space(16.0);

    let available_w = ui.available_width();
    let gap = 16.0_f32;
    let use_two_cols = available_w >= 720.0;

    if use_two_cols {
        let col_w = ((available_w - gap) / 2.0_f32).floor();

        /* Persistently sync and equalize the height of both cards across frames */
        let height_id = ui.make_persistent_id("config_top_cards_equal_height");
        let synced_h = ui.ctx().data(|d| d.get_temp::<f32>(height_id)).unwrap_or(400.0_f32);

        let mut backup_h = synced_h;
        let mut params_h = synced_h;

        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(gap, 0.0);

            /* Column 1 (Left Tile): Current Profile Parameters */
            ui.allocate_ui_with_layout(
                Vec2::new(col_w, 0.0),
                Layout::top_down(egui::Align::LEFT),
                |ui| {
                    let resp = render_params_card(ui, app, col_w, synced_h);
                    params_h = resp.rect.height();
                },
            );

            /* Column 2 (Right Tile): Profile Backup & Restore */
            ui.allocate_ui_with_layout(
                Vec2::new(col_w, 0.0),
                Layout::top_down(egui::Align::LEFT),
                |ui| {
                    let resp = render_backup_card(ui, app, col_w, synced_h);
                    backup_h = resp.rect.height();
                },
            );
        });

        let max_card_h = backup_h.max(params_h).max(400.0_f32);
        ui.ctx().data_mut(|d| d.insert_temp(height_id, max_card_h));
    } else {
        let col_w = available_w.floor();
        render_params_card(ui, app, col_w, 0.0);
        ui.add_space(16.0);
        render_backup_card(ui, app, col_w, 0.0);
    }
}

/* Column 2 (Right Tile): Profile File Operations & Factory Reset */
fn render_backup_card(ui: &mut Ui, app: &mut AtkControlApp, width: f32, min_height: f32) -> egui::Response {
    let inner_w = (width - 36.0_f32).max(180.0_f32);
    let inner_min_h = (min_height - 36.0_f32).max(0.0_f32);

    card_container(ui, |ui| {
        ui.set_min_width(inner_w);
        ui.set_max_width(inner_w);
        if inner_min_h > 0.0 {
            ui.set_min_height(inner_min_h);
        }

        ui.label(
            RichText::new("Onboard Profile Backup (JSON)")
                .size(16.5)
                .strong()
                .color(COLOR_TEXT_PRIMARY),
        );
        ui.add_space(4.0);
        ui.add(
            egui::Label::new(
                RichText::new(
                    "Save all mouse DPI stages, RGB colors, button mappings, sensor parameters, and LED effects into a portable JSON file.",
                )
                .size(12.5)
                .color(COLOR_TEXT_MUTED),
            )
            .wrap(),
        );
        ui.add_space(14.0);

        ui.horizontal(|ui| {
            if ui
                .button(RichText::new("Export Profile (JSON)").size(13.5).strong())
                .clicked()
            {
                let count = app.mouse.stage_count.clamp(1, 6) as usize;
                let mut stages_dpi = Vec::with_capacity(count);
                for idx in 0..count {
                    let (x, y) = app.mouse.stages_dpi.get(idx).copied().unwrap_or((800, 800));
                    let (r, g, b) = app
                        .mouse
                        .stage_colors
                        .get(idx)
                        .copied()
                        .unwrap_or((255, 255, 255));
                    stages_dpi.push(StageDpiEntry {
                        stage: (idx + 1) as u8,
                        x,
                        y,
                        color_rgb: [r, g, b],
                    });
                }

                let profile = ExportedProfile {
                    model: app.mouse.model_name.clone(),
                    profile_name: "OpenATK Profile".to_string(),
                    polling_rate_hz: app.mouse.polling_rate,
                    active_stage: app.mouse.active_stage + 1,
                    stage_count: app.mouse.stage_count,
                    stages_dpi,
                    lod_mm: if app.mouse.lod_code == 2 { 2.0 } else { 1.0 },
                    debounce_ms: app.mouse.debounce_ms,
                    motion_sync: app.mouse.motion_sync,
                    straight_line_correction: app.mouse.straight_line_correction,
                    ripple_control: app.mouse.ripple_control,
                    ultra_long_range: app.mouse.ultra_long_range,
                    sleep_time_idx: app.mouse.sleep_time_idx,
                    dpi_light_mode: app.mouse.dpi_light_mode,
                    dpi_light_brightness: app.mouse.dpi_light_brightness,
                    dpi_light_speed: app.mouse.dpi_light_speed,
                    button_mappings: app.mouse.buttons.iter().map(|b| b.action.clone()).collect(),
                };

                if let Ok(json_str) = serde_json::to_string_pretty(&profile) {
                    let export_path = "open_atk_profile.json";
                    if std::fs::write(export_path, json_str).is_ok() {
                        app.set_status(
                            format!("Profile successfully exported to {}", export_path),
                            false,
                        );
                    } else {
                        app.set_status("Failed to write profile JSON file to disk", true);
                    }
                }
            }

            ui.add_space(8.0);

            if ui
                .button(RichText::new("Import Profile (JSON)").size(13.5).strong())
                .clicked()
            {
                let import_path = "open_atk_profile.json";
                match std::fs::read_to_string(import_path) {
                    Ok(json_str) => match serde_json::from_str::<ExportedProfile>(&json_str) {
                        Ok(profile) => {
                            let target_count = profile.stage_count.clamp(1, 6);
                            app.mouse.stage_count = target_count;
                            if let Some(tx) = &app.action_tx {
                                let _ = tx.send(crate::state::DeviceAction::SetStageCount(target_count));
                            }
                            for entry in &profile.stages_dpi {
                                if entry.stage >= 1 && entry.stage <= target_count {
                                    let idx = entry.stage - 1;
                                    app.apply_dpi(idx, entry.x, entry.y);
                                    app.apply_stage_color(
                                        idx,
                                        entry.color_rgb[0],
                                        entry.color_rgb[1],
                                        entry.color_rgb[2],
                                    );
                                }
                            }

                            let active_zero_based = profile.active_stage.saturating_sub(1).min(target_count - 1);
                            app.apply_active_stage(active_zero_based);
                            app.apply_polling(profile.polling_rate_hz);

                            let lod_code = if profile.lod_mm >= 1.5 { 2 } else { 1 };
                            app.apply_lod(lod_code);
                            app.apply_debounce(profile.debounce_ms);
                            app.apply_motion_sync(profile.motion_sync);
                            app.apply_straight_line(profile.straight_line_correction);
                            app.apply_ripple_control(profile.ripple_control);
                            app.apply_ultra_long_range(profile.ultra_long_range);
                            app.apply_sleep_time(profile.sleep_time_idx);
                            app.apply_dpi_light(
                                profile.dpi_light_mode,
                                profile.dpi_light_brightness,
                                profile.dpi_light_speed,
                            );

                            for (idx, act) in profile.button_mappings.iter().enumerate() {
                                if idx < app.mouse.buttons.len() {
                                    app.apply_button_mapping(idx, act);
                                }
                            }

                            app.set_status(
                                format!("Profile imported from {} and applied to hardware!", import_path),
                                false,
                            );
                        }
                        Err(e) => app.set_status(format!("Invalid profile JSON format: {e}"), true),
                    },
                    Err(_) => app.set_status(
                        format!("File '{}' not found. Export one first!", import_path),
                        true,
                    ),
                }
            }
        });

        ui.add_space(10.0);
        ui.add(
            egui::Label::new(
                RichText::new("File: open_atk_profile.json (Compatible with vxectl CLI)")
                    .size(11.5)
                    .color(Color32::from_gray(120)),
            )
            .wrap(),
        );

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(12.0);

        ui.label(
            RichText::new("Factory Reset & Safety")
                .size(15.0)
                .strong()
                .color(COLOR_TEXT_PRIMARY),
        );
        ui.add_space(4.0);
        ui.add(
            egui::Label::new(
                RichText::new(
                    "Restore factory standard button bindings, clearing all custom hotkeys and active macros.",
                )
                .size(12.5)
                .color(COLOR_TEXT_MUTED),
            )
            .wrap(),
        );
        ui.add_space(12.0);

        if ui
            .button(
                RichText::new("Restore Factory Defaults")
                    .size(13.0)
                    .color(COLOR_ACCENT_ORANGE),
            )
            .clicked()
        {
            app.restore_defaults();
        }
    })
}

/* Column 1 (Left Tile): Live Hardware Parameters */
fn render_params_card(ui: &mut Ui, app: &mut AtkControlApp, width: f32, min_height: f32) -> egui::Response {
    let inner_w = (width - 36.0_f32).max(180.0_f32);
    let inner_min_h = (min_height - 36.0_f32).max(0.0_f32);

    card_container(ui, |ui| {
        ui.set_min_width(inner_w);
        ui.set_max_width(inner_w);
        if inner_min_h > 0.0 {
            ui.set_min_height(inner_min_h);
        }

        ui.label(
            RichText::new("Current Profile Parameters")
                .size(16.5)
                .strong()
                .color(COLOR_TEXT_PRIMARY),
        );
        ui.add_space(4.0);
        ui.add(
            egui::Label::new(
                RichText::new("Live parameters currently loaded on device hardware")
                    .size(12.5)
                    .color(COLOR_TEXT_MUTED),
            )
            .wrap(),
        );
        ui.add_space(14.0);

        param_row(ui, "Device Model:", &app.mouse.model_name, Some(COLOR_ACCENT_BLUE));
        param_row(ui, "Polling Frequency:", &format!("{} Hz", app.mouse.polling_rate), None);

        let active_dpi = app
            .mouse
            .stages_dpi
            .get(app.mouse.active_stage as usize)
            .map(|&(x, y)| if x == y { format!("{x} DPI") } else { format!("{x} x {y} DPI") })
            .unwrap_or_else(|| "800 DPI".to_string());
        param_row(
            ui,
            "Active DPI Stage:",
            &format!(
                "Stage {} of {} ({})",
                app.mouse.active_stage + 1,
                app.mouse.stage_count,
                active_dpi
            ),
            None,
        );

        param_row(
            ui,
            "LOD Height:",
            if app.mouse.lod_code == 2 { "2.0 mm (High)" } else { "1.0 mm (Low)" },
            None,
        );
        param_row(ui, "Switch Debounce:", &format!("{} ms", app.mouse.debounce_ms), None);

        const SLEEP_LABELS: [&str; 8] = ["30s", "1min", "2min", "3min", "5min", "20min", "25min", "30min"];
        let sleep_str = SLEEP_LABELS.get(app.mouse.sleep_time_idx).copied().unwrap_or("1min");
        param_row(ui, "Sleep Timeout:", sleep_str, None);

        let led_str = match app.mouse.dpi_light_mode {
            1 => "Always On",
            2 => "Breathing",
            _ => "Off (Closed)",
        };
        param_row(ui, "DPI Indicator LED:", led_str, None);

        bool_row(ui, "Motion Sync:", app.mouse.motion_sync);
        bool_row(ui, "Straight Line Correction:", app.mouse.straight_line_correction);
        bool_row(ui, "Ripple Correction:", app.mouse.ripple_control);
        bool_row(ui, "Ultra Long Range Mode:", app.mouse.ultra_long_range);
    })
}

fn param_row(ui: &mut Ui, label: &str, value: &str, custom_color: Option<Color32>) {
    ui.horizontal(|ui| {
        ui.allocate_ui_with_layout(
            Vec2::new(200.0, 18.0),
            Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.label(
                    RichText::new(label)
                        .size(13.0)
                        .color(Color32::from_gray(160)),
                );
            },
        );
        ui.label(
            RichText::new(value)
                .size(13.0)
                .strong()
                .color(custom_color.unwrap_or(COLOR_TEXT_PRIMARY)),
        );
    });
    ui.add_space(3.0);
}

fn bool_row(ui: &mut Ui, label: &str, enabled: bool) {
    let color = if enabled {
        Color32::from_rgb(0, 230, 118)
    } else {
        Color32::from_gray(140)
    };
    param_row(ui, label, if enabled { "Enabled" } else { "Disabled" }, Some(color));
}

fn card_container<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> egui::Response {
    let frame = eframe::egui::Frame::none()
        .fill(Color32::from_rgb(20, 20, 24))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(34, 34, 42)))
        .rounding(Rounding::same(8.0))
        .inner_margin(eframe::egui::Margin::same(18.0));
    frame.show(ui, add_contents).response
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{Pos2, Rect, Vec2};

    #[test]
    fn test_config_view_layout_at_1920() {
        let ctx = eframe::egui::Context::default();
        let mut raw_input = eframe::egui::RawInput::default();
        raw_input.screen_rect = Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1920.0, 1080.0)));

        let mut app = AtkControlApp::new();

        let _ = ctx.run(raw_input, |ctx| {
            eframe::egui::SidePanel::left("left_sidebar")
                .exact_width(250.0)
                .show(ctx, |_| {});

            eframe::egui::TopBottomPanel::bottom("bottom_disclaimer")
                .show(ctx, |_| {});

            eframe::egui::CentralPanel::default()
                .frame(
                    eframe::egui::Frame::none()
                        .inner_margin(eframe::egui::Margin {
                            left: 24.0,
                            right: 4.0,
                            top: 24.0,
                            bottom: 0.0,
                        }),
                )
                .show(ctx, |ui| {
                    eframe::egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            eframe::egui::Frame::none()
                                .inner_margin(eframe::egui::Margin {
                                    left: 0.0,
                                    right: 20.0,
                                    top: 0.0,
                                    bottom: 12.0,
                                })
                                .show(ui, |ui| {
                                    let available_w = ui.available_width();
                                    let gap = 16.0_f32;
                                    let col_w = ((available_w - gap) / 2.0_f32).floor();
                                    let top_card_min_h = 400.0_f32;

                                    let mut r1 = Rect::NOTHING;
                                    let mut r2 = Rect::NOTHING;
                                    ui.horizontal_top(|ui| {
                                        ui.spacing_mut().item_spacing = Vec2::new(gap, 0.0);
                                        ui.allocate_ui_with_layout(
                                            Vec2::new(col_w, 0.0),
                                            Layout::top_down(egui::Align::LEFT),
                                            |ui| {
                                                r1 = render_params_card(ui, &mut app, col_w, top_card_min_h).rect;
                                            },
                                        );
                                        ui.allocate_ui_with_layout(
                                            Vec2::new(col_w, 0.0),
                                            Layout::top_down(egui::Align::LEFT),
                                            |ui| {
                                                r2 = render_backup_card(ui, &mut app, col_w, top_card_min_h).rect;
                                            },
                                        );
                                    });
                                    eprintln!("Card 1 rect: {:?}, h={}", r1, r1.height());
                                    eprintln!("Card 2 rect: {:?}, h={}", r2, r2.height());

                                    assert_eq!(r1.height(), r2.height(), "Card 1 and Card 2 must have identical heights");
                                    assert_eq!(r1.min.y, r2.min.y, "Card 1 and Card 2 must align at the top");
                                    assert_eq!(r1.max.y, r2.max.y, "Card 1 and Card 2 must align at the bottom");
                                    assert_eq!(r1.width(), r2.width(), "Card 1 and Card 2 must have identical widths");
                                });
                        });
                });
        });
    }
}
