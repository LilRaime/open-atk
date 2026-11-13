/*
 * Other Settings & Diagnostics View
 * Layout:
 * - Top Row (2 equal tiles side-by-side):
 *   1. Device Diagnostics
 *   2. Window & System Tray Behavior
 * - Bottom Row (Full width):
 *   3. Legal Disclaimer & Project Notice
 */

use crate::icons::{draw_icon, IconType};
use crate::state::AtkControlApp;
use crate::theme::{COLOR_ACCENT_ORANGE, COLOR_TEXT_MUTED, COLOR_TEXT_PRIMARY};
use eframe::egui::{self, Color32, Layout, Pos2, RichText, Rounding, Sense, Stroke, Ui, Vec2};

pub fn render_other_settings_view(ui: &mut Ui, app: &mut AtkControlApp) {
    ui.label(
        RichText::new("Other Settings")
            .size(22.0)
            .strong()
            .color(COLOR_TEXT_PRIMARY),
    );
    ui.add(
        egui::Label::new(
            RichText::new("Application behavior, hardware diagnostics, and project information")
                .size(13.0)
                .color(COLOR_TEXT_MUTED),
        )
        .wrap(),
    );
    ui.add_space(16.0);

    let available_w = ui.available_width();
    let gap = 16.0_f32;
    let use_two_cols = available_w >= 720.0;

    /* Top Row: 2 Tiles (1: Device Diagnostics, 2: Window & System Tray Behavior) */
    let total_row_w = if use_two_cols {
        let col_w = ((available_w - gap) / 2.0_f32).floor();

        /* Persistently sync and equalize the height of both top cards across frames */
        let height_id = ui.make_persistent_id("other_top_cards_equal_height");
        let synced_h = ui.ctx().data(|d| d.get_temp::<f32>(height_id)).unwrap_or(330.0_f32);

        let mut diag_h = synced_h;
        let mut tray_h = synced_h;

        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(gap, 0.0);

            /* Tile 1 (Left): Device Diagnostics */
            ui.allocate_ui_with_layout(
                Vec2::new(col_w, 0.0),
                Layout::top_down(egui::Align::LEFT),
                |ui| {
                    let resp = render_diagnostics_card(ui, app, col_w, synced_h);
                    diag_h = resp.rect.height();
                },
            );

            /* Tile 2 (Right): Window & System Tray Behavior */
            ui.allocate_ui_with_layout(
                Vec2::new(col_w, 0.0),
                Layout::top_down(egui::Align::LEFT),
                |ui| {
                    let resp = render_tray_behavior_card(ui, app, col_w, synced_h);
                    tray_h = resp.rect.height();
                },
            );
        });

        let max_card_h = diag_h.max(tray_h).max(330.0_f32);
        ui.ctx().data_mut(|d| d.insert_temp(height_id, max_card_h));

        col_w * 2.0_f32 + gap
    } else {
        let col_w = available_w.floor();
        render_diagnostics_card(ui, app, col_w, 0.0);
        ui.add_space(16.0);
        render_tray_behavior_card(ui, app, col_w, 0.0);
        col_w
    };

    ui.add_space(16.0);

    /* Bottom Row (Full Width): Legal Disclaimer & Project Notice */
    render_disclaimer_card(ui, total_row_w);
}

/* Tile 1: Window & System Tray Behavior */
fn render_tray_behavior_card(ui: &mut Ui, app: &mut AtkControlApp, width: f32, min_height: f32) -> egui::Response {
    let inner_w = (width - 36.0_f32).max(180.0_f32);
    let inner_min_h = (min_height - 36.0_f32).max(0.0_f32);

    card_container(ui, |ui| {
        ui.set_min_width(inner_w);
        ui.set_max_width(inner_w);
        if inner_min_h > 0.0 {
            ui.set_min_height(inner_min_h);
        }

        ui.label(
            RichText::new("Window & System Tray Behavior")
                .size(16.5)
                .strong()
                .color(COLOR_TEXT_PRIMARY),
        );
        ui.add_space(4.0);
        ui.add(
            egui::Label::new(
                RichText::new("Configure close button action, startup behavior, and battery alerts")
                    .size(12.5)
                    .color(COLOR_TEXT_MUTED),
            )
            .wrap(),
        );
        ui.add_space(20.0);

        /* Option 1: Minimize to system tray on close */
        let mut min_tray = app.preferences.minimize_to_tray;
        if preference_toggle_row(
            ui,
            "min_tray_toggle",
            &mut min_tray,
            "Minimize to system tray on close",
            "When disabled, clicking X exits OpenATK completely",
            inner_w,
        ) {
            app.preferences.minimize_to_tray = min_tray;
            app.preferences.save();
            let msg = if min_tray {
                "Close action: Minimize to system tray"
            } else {
                "Close action: Exit application completely"
            };
            app.set_status(msg, false);
        }

        ui.add_space(18.0);

        /* Option 2: Launch on Windows startup */
        let mut startup = app.preferences.run_on_startup;
        if preference_toggle_row(
            ui,
            "startup_toggle",
            &mut startup,
            "Launch on Windows startup",
            "Automatically start minimized in the background on login",
            inner_w,
        ) {
            app.preferences.apply_startup(startup);
            let msg = if startup {
                "Auto-startup enabled"
            } else {
                "Auto-startup disabled"
            };
            app.set_status(msg, false);
        }

        ui.add_space(18.0);

        /* Option 3: Low battery desktop notification */
        let mut bat_warn = app.preferences.low_battery_warning;
        if preference_toggle_row(
            ui,
            "bat_warn_toggle",
            &mut bat_warn,
            "Low battery desktop notification",
            "Show a system toast warning when battery drops below 15%",
            inner_w,
        ) {
            app.preferences.low_battery_warning = bat_warn;
            app.preferences.save();
        }
    })
}

/* Tile 2: Device Diagnostics */
fn render_diagnostics_card(ui: &mut Ui, app: &mut AtkControlApp, width: f32, min_height: f32) -> egui::Response {
    let inner_w = (width - 36.0_f32).max(180.0_f32);
    let inner_min_h = (min_height - 36.0_f32).max(0.0_f32);

    card_container(ui, |ui| {
        ui.set_min_width(inner_w);
        ui.set_max_width(inner_w);
        if inner_min_h > 0.0 {
            ui.set_min_height(inner_min_h);
        }

        ui.label(
            RichText::new("Device Diagnostics")
                .size(16.5)
                .strong()
                .color(COLOR_TEXT_PRIMARY),
        );
        ui.add_space(12.0);

        diag_row(ui, "Hardware Model:", &app.mouse.model_name);
        let conn_str = match app.mouse.mode {
            atk_device::ConnectionMode::Wired => "Wired (USB-C Cable)",
            atk_device::ConnectionMode::WirelessDongle => "Wireless (2.4 GHz Receiver)",
            atk_device::ConnectionMode::Bluetooth => "Bluetooth (BLE)",
            atk_device::ConnectionMode::Unknown => "Wireless / Auto",
        };
        diag_row(ui, "Connection Mode:", conn_str);

        let charge_str = if app.mouse.is_charging {
            format!("Charging - {}%", app.mouse.battery_pct)
        } else {
            format!("Discharging - {}%", app.mouse.battery_pct)
        };
        diag_row(ui, "Power State:", &charge_str);
        diag_row(ui, "Firmware Version:", &app.mouse.firmware);
        if app.mouse.cid > 0 || app.mouse.mid > 0 {
            diag_row(
                ui,
                "Hardware Identity:",
                &format!("CID {}, MID {}", app.mouse.cid, app.mouse.mid),
            );
        }
        diag_row(
            ui,
            "Battery Voltage:",
            &format!("{:.2} V ({} mV)", app.mouse.voltage_mv as f32 / 1000.0, app.mouse.voltage_mv),
        );
        diag_row(ui, "Receiver RF ID:", &app.mouse.rf_id);

        let vid_pid_desc = match app.mouse.product_id {
            0xF58A => "0x3554 / 0xF58A (Compx Nordic 1K Dongle)".to_string(),
            0xF58C => "0x3554 / 0xF58C (Nordic 4K/8K Dongle)".to_string(),
            0xF58E => "0x3554 / 0xF58E (VXE R1SE+ Wireless)".to_string(),
            0xF58F => "0x3554 / 0xF58F (VXE R1SE+ Wired)".to_string(),
            0xF588 => "0x3554 / 0xF588 (VXE R1 Pro Wireless)".to_string(),
            0xF589 => "0x3554 / 0xF589 (VXE R1 Pro Wired)".to_string(),
            0xF580 => "0x3554 / 0xF580 (VXE R1 SE Wireless)".to_string(),
            0xF581 => "0x3554 / 0xF581 (VXE R1 SE Wired)".to_string(),
            0x1085 => "0x3554 / 0x1085 (VXE Wireless Dongle)".to_string(),
            _ if app.mouse.vendor_id > 0 => format!("0x{:04X} / 0x{:04X}", app.mouse.vendor_id, app.mouse.product_id),
            _ => "0x3554 / 0xF58A (Compx Nordic 1K)".to_string(),
        };
        diag_row(ui, "USB VID / PID:", &vid_pid_desc);

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(
                Vec2::new(160.0, 18.0),
                Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.label(RichText::new("Mouse Theme:").size(13.0).color(COLOR_TEXT_MUTED));
                },
            );
            if ui
                .selectable_label(
                    app.mouse_color == crate::state::MouseColor::White,
                    "White",
                )
                .clicked()
            {
                app.set_active_mouse_color(crate::state::MouseColor::White);
            }
            if ui
                .selectable_label(
                    app.mouse_color == crate::state::MouseColor::Black,
                    "Black",
                )
                .clicked()
            {
                app.set_active_mouse_color(crate::state::MouseColor::Black);
            }
        });
    })
}

/* Bottom Tile (Full Width): Legal Disclaimer & Project Notice */
fn render_disclaimer_card(ui: &mut Ui, width: f32) -> egui::Response {
    let inner_w = (width - 36.0_f32).max(200.0_f32);

    card_container(ui, |ui| {
        ui.set_min_width(inner_w);
        ui.set_max_width(inner_w);

        ui.label(
            RichText::new("Legal Disclaimer & Project Notice")
                .size(16.5)
                .strong()
                .color(COLOR_TEXT_PRIMARY),
        );
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            let (icon_rect, _) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::hover());
            draw_icon(ui.painter(), icon_rect, IconType::Warning, COLOR_ACCENT_ORANGE);
            ui.add_space(6.0);
            ui.add(
                egui::Label::new(
                    RichText::new("This application is an independent open-source project and is NOT an official product of ATK or VXE.")
                        .size(13.5)
                        .strong()
                        .color(COLOR_ACCENT_ORANGE),
                )
                .wrap(),
            );
        });
        ui.add_space(6.0);
        ui.add(
            egui::Label::new(
                RichText::new("This software has no affiliation, connection, sponsorship, or endorsement from https://hub.atk.pro/ or https://www.atk.store/.")
                    .size(12.5)
                    .color(COLOR_TEXT_MUTED),
            )
            .wrap(),
        );
        ui.add_space(4.0);
        ui.add(
            egui::Label::new(
                RichText::new("All hardware identifiers and trademarks belong to their respective owners and are referenced solely for compatibility purposes.")
                    .size(12.0)
                    .color(Color32::from_gray(140)),
            )
            .wrap(),
        );
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("GitHub Repository & Issues:").size(12.0).color(Color32::from_gray(140)));
            ui.hyperlink_to(
                RichText::new("https://github.com/LilRaime/open-atk")
                    .size(12.0)
                    .color(Color32::from_rgb(130, 195, 255))
                    .underline(),
                "https://github.com/LilRaime/open-atk",
            );
        });
    })
}

fn diag_row(ui: &mut Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.allocate_ui_with_layout(
            Vec2::new(160.0, 18.0),
            Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.label(RichText::new(label).size(13.0).color(COLOR_TEXT_MUTED));
            },
        );
        ui.label(RichText::new(value).size(13.0).strong().color(COLOR_TEXT_PRIMARY));
    });
    ui.add_space(3.0);
}

/* Row with title + subtitle on the left and a high-contrast animated toggle switch on the right */
fn preference_toggle_row(
    ui: &mut Ui,
    id_salt: &str,
    value: &mut bool,
    title: &str,
    subtitle: &str,
    row_width: f32,
) -> bool {
    let mut changed = false;
    let text_w = (row_width - 52.0_f32).max(120.0_f32);

    ui.horizontal(|ui| {
        ui.set_min_width(row_width);
        ui.set_max_width(row_width);

        ui.allocate_ui_with_layout(
            Vec2::new(text_w, 0.0),
            Layout::top_down(egui::Align::LEFT),
            |ui| {
                ui.set_max_width(text_w);
                ui.label(RichText::new(title).size(13.5).strong().color(COLOR_TEXT_PRIMARY));
                ui.add(
                    egui::Label::new(RichText::new(subtitle).size(11.5).color(COLOR_TEXT_MUTED))
                        .wrap(),
                );
            },
        );

        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            if toggle_switch(ui, id_salt, value) {
                changed = true;
            }
        });
    });

    changed
}

fn toggle_switch(ui: &mut Ui, id_salt: &str, on: &mut bool) -> bool {
    let desired_size = egui::vec2(38.0, 20.0);
    let id = ui.make_persistent_id(id_salt);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::click());
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool_responsive(id, *on);
        let bg_color = if *on {
            Color32::from_rgb(245, 245, 250)
        } else {
            Color32::from_rgb(48, 48, 58)
        };
        ui.painter().rect_filled(rect, 10.0, bg_color);

        let circle_x = egui::lerp((rect.left() + 10.0)..=(rect.right() - 10.0), how_on);
        let center = Pos2::new(circle_x, rect.center().y);
        let knob_color = if *on {
            Color32::from_rgb(18, 18, 22)
        } else {
            Color32::from_rgb(140, 140, 155)
        };
        ui.painter().circle_filled(center, 7.5, knob_color);
    }
    response.changed()
}

fn card_container<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> egui::Response {
    let frame = egui::Frame::none()
        .fill(Color32::from_rgb(20, 20, 24))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(34, 34, 42)))
        .rounding(Rounding::same(8.0))
        .inner_margin(egui::Margin::same(18.0));
    frame.show(ui, add_contents).response
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{Pos2, Rect, Vec2};

    #[test]
    fn test_other_view_layout_at_1920() {
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
                                    let top_card_min_h = 330.0_f32;

                                    let mut r1 = Rect::NOTHING;
                                    let mut r2 = Rect::NOTHING;
                                    ui.horizontal_top(|ui| {
                                        ui.spacing_mut().item_spacing = Vec2::new(gap, 0.0);
                                        ui.allocate_ui_with_layout(
                                            Vec2::new(col_w, 0.0),
                                            Layout::top_down(egui::Align::LEFT),
                                            |ui| {
                                                r1 = render_diagnostics_card(ui, &mut app, col_w, top_card_min_h).rect;
                                            },
                                        );
                                        ui.allocate_ui_with_layout(
                                            Vec2::new(col_w, 0.0),
                                            Layout::top_down(egui::Align::LEFT),
                                            |ui| {
                                                r2 = render_tray_behavior_card(ui, &mut app, col_w, top_card_min_h).rect;
                                            },
                                        );
                                    });

                                    let total_row_w = col_w * 2.0_f32 + gap;
                                    let r3 = render_disclaimer_card(ui, total_row_w).rect;

                                    eprintln!("Other Card 1 rect: {:?}, h={}", r1, r1.height());
                                    eprintln!("Other Card 2 rect: {:?}, h={}", r2, r2.height());
                                    eprintln!("Other Card 3 rect: {:?}, w={}", r3, r3.width());

                                    assert_eq!(r1.height(), r2.height(), "Card 1 and Card 2 must have identical heights");
                                    assert_eq!(r1.min.y, r2.min.y, "Card 1 and Card 2 must align at the top");
                                    assert_eq!(r1.max.y, r2.max.y, "Card 1 and Card 2 must align at the bottom");
                                    assert_eq!(r1.min.x, r3.min.x, "Disclaimer must align with Left Card on the left");
                                    assert_eq!(r2.max.x, r3.max.x, "Disclaimer must align with Right Card on the right");
                                });
                        });
                });
        });
    }
}
