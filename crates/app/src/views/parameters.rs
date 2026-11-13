/*
 * Parameter Settings View (1:1 Native Recreation of ATK Web Hub)
 * Supports:
 * - Movement Synchronization (Motion Sync) toggle
 * - Straight Line Correction (Angle Snapping) toggle
 * - Ripple Correction toggle
 * - Ultra Long Range Mode (High RF Power) toggle
 * - Key Debounce Delay segmented pills (0ms to 20ms in 2 balanced rows)
 * - Sleep Time Setting segmented pills (30s to 30min in 2 balanced rows)
 * - LOD Height Tolerance segmented pills (1mm / 2mm) with live vector hover gap illustration
 * - DPI Light Effect (Modes, Brightness, Speed)
 *
 * All card heights and column boundaries are synchronized across frames:
 * - The 4 sister toggle cards have identical heights.
 * - Sleep Time Setting aligns horizontally with the bottom of Straight Line Correction.
 * - Both columns align symmetrically at the bottom horizon.
 */

use crate::icons::{draw_lod_illustration, draw_param_icon, ParamIcon};
use crate::state::AtkControlApp;
use crate::theme::{COLOR_TEXT_MUTED, COLOR_TEXT_PRIMARY};
use eframe::egui::{self, Color32, RichText, Rounding, Stroke, Ui, Vec2};

pub fn render_parameter_view(ui: &mut Ui, app: &mut AtkControlApp) {
    ui.label(RichText::new("Parameter Settings").size(22.0).strong().color(COLOR_TEXT_PRIMARY));
    ui.add(egui::Label::new(
        RichText::new("Sensor tracking, switch debounce, sleep timers, and wireless radio modes")
            .size(13.0)
            .color(COLOR_TEXT_MUTED),
    ).wrap());
    ui.add_space(14.0);

    let available_w = ui.available_width();
    let gap = 16.0_f32;
    let use_two_cols = available_w >= 720.0;

    if use_two_cols {
        let col_w = ((available_w - gap) / 2.0_f32).floor();

        /* Persistently sync and equalize toggle card heights across frames */
        let toggle_h_id = ui.make_persistent_id("param_toggle_cards_equal_height");
        let synced_toggle_h = ui.ctx().data(|d| d.get_temp::<f32>(toggle_h_id)).unwrap_or(82.0_f32);

        /* Persistently sync and equalize total column heights across frames */
        let total_h_id = ui.make_persistent_id("param_columns_equal_height");
        let synced_total_h = ui.ctx().data(|d| d.get_temp::<f32>(total_h_id)).unwrap_or(540.0_f32);

        /* Sleep card target height matches top 2 left cards plus spacing */
        let sleep_h = 2.0_f32 * synced_toggle_h + 12.0_f32;

        let mut h_motion = synced_toggle_h;
        let mut h_straight = synced_toggle_h;
        let mut h_ripple = synced_toggle_h;
        let mut h_range = synced_toggle_h;
        let mut h_debounce = 140.0_f32;

        let mut h_sleep_act = sleep_h;
        let mut h_lod_act = 115.0_f32;
        let mut h_dpi_act = 190.0_f32;

        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(gap, 0.0);

            /* Column 1 (Left): 4 Equal-Height Toggles + Key Debounce Delay */
            ui.allocate_ui_with_layout(
                Vec2::new(col_w, 0.0),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    /* 1. Movement Synchronization */
                    let resp = render_motion_sync_card(ui, app, col_w, synced_toggle_h);
                    h_motion = resp.rect.height();
                    ui.add_space(12.0);

                    /* 2. Straight Line Correction */
                    let resp = render_straight_line_card(ui, app, col_w, synced_toggle_h);
                    h_straight = resp.rect.height();
                    ui.add_space(12.0);

                    /* 3. Ripple Correction */
                    let resp = render_ripple_card(ui, app, col_w, synced_toggle_h);
                    h_ripple = resp.rect.height();
                    ui.add_space(12.0);

                    /* 4. Ultra Long Range Mode */
                    let resp = render_long_range_card(ui, app, col_w, synced_toggle_h);
                    h_range = resp.rect.height();
                    ui.add_space(12.0);

                    /* 5. Key Debounce Delay (expands to balance total column height) */
                    let debounce_min_h = (synced_total_h - 4.0_f32 * synced_toggle_h - 48.0_f32).max(136.0_f32);
                    let resp = render_debounce_card(ui, app, col_w, debounce_min_h);
                    h_debounce = resp.rect.height();
                },
            );

            /* Column 2 (Right): Sleep Time Setting (aligned with card 2), LOD, and DPI Light Effect (aligned with card 5) */
            ui.allocate_ui_with_layout(
                Vec2::new(col_w, 0.0),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    /* 6. Sleep Time Setting (height matches cards 1 & 2 + gap) */
                    let resp = render_sleep_card(ui, app, col_w, sleep_h);
                    h_sleep_act = resp.rect.height();
                    ui.add_space(12.0);

                    /* 7. LOD Height Tolerance */
                    let resp = render_lod_card(ui, app, col_w, 0.0);
                    h_lod_act = resp.rect.height();
                    ui.add_space(12.0);

                    /* 8. DPI Light Effect (expands to perfectly match the bottom of Key Debounce Delay) */
                    let dpi_min_h = (synced_total_h - h_sleep_act - 12.0_f32 - h_lod_act - 12.0_f32).max(0.0_f32);
                    let resp = render_dpi_light_card(ui, app, col_w, dpi_min_h);
                    h_dpi_act = resp.rect.height();
                },
            );
        });

        /* Update dynamic heights */
        let max_toggle_h = h_motion.max(h_straight).max(h_ripple).max(h_range).max(82.0_f32);
        ui.ctx().data_mut(|d| d.insert_temp(toggle_h_id, max_toggle_h));

        let total_left = h_motion + 12.0 + h_straight + 12.0 + h_ripple + 12.0 + h_range + 12.0 + h_debounce;
        let total_right = h_sleep_act + 12.0 + h_lod_act + 12.0 + h_dpi_act;
        let max_total = total_left.max(total_right).max(520.0_f32);
        ui.ctx().data_mut(|d| d.insert_temp(total_h_id, max_total));
    } else {
        let col_w = available_w.floor();
        render_motion_sync_card(ui, app, col_w, 0.0);
        ui.add_space(12.0);
        render_straight_line_card(ui, app, col_w, 0.0);
        ui.add_space(12.0);
        render_ripple_card(ui, app, col_w, 0.0);
        ui.add_space(12.0);
        render_long_range_card(ui, app, col_w, 0.0);
        ui.add_space(12.0);
        render_debounce_card(ui, app, col_w, 0.0);
        ui.add_space(12.0);
        render_sleep_card(ui, app, col_w, 0.0);
        ui.add_space(12.0);
        render_lod_card(ui, app, col_w, 0.0);
        ui.add_space(12.0);
        render_dpi_light_card(ui, app, col_w, 0.0);
    }
}

/* 1. Movement Synchronization Card */
fn render_motion_sync_card(ui: &mut Ui, app: &mut AtkControlApp, width: f32, min_h: f32) -> egui::Response {
    card_container(ui, width, min_h, |ui| {
        ui.horizontal(|ui| {
            draw_param_icon(ui, ParamIcon::MotionSync);
            ui.label(RichText::new("Movement Synchronization").size(15.5).strong().color(COLOR_TEXT_PRIMARY));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let mut val = app.mouse.motion_sync;
                if toggle_switch(ui, &mut val) {
                    app.apply_motion_sync(val);
                }
            });
        });
        ui.add_space(6.0);
        ui.add(egui::Label::new(
            RichText::new("Smooths mouse movement with algorithm correction for accurate trajectories, adding minor latency.")
                .size(12.0)
                .color(COLOR_TEXT_MUTED),
        ).wrap());
    })
}

/* 2. Straight Line Correction Card */
fn render_straight_line_card(ui: &mut Ui, app: &mut AtkControlApp, width: f32, min_h: f32) -> egui::Response {
    card_container(ui, width, min_h, |ui| {
        ui.horizontal(|ui| {
            draw_param_icon(ui, ParamIcon::StraightLine);
            ui.label(RichText::new("Straight Line Correction").size(15.5).strong().color(COLOR_TEXT_PRIMARY));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let mut val = app.mouse.straight_line_correction;
                if toggle_switch(ui, &mut val) {
                    app.apply_straight_line(val);
                }
            });
        });
        ui.add_space(6.0);
        ui.add(egui::Label::new(
            RichText::new("Corrects offset jitter when moving the mouse straight for absolute straight movement, ideal for design and drawing.")
                .size(12.0)
                .color(COLOR_TEXT_MUTED),
        ).wrap());
    })
}

/* 3. Ripple Correction Card */
fn render_ripple_card(ui: &mut Ui, app: &mut AtkControlApp, width: f32, min_h: f32) -> egui::Response {
    card_container(ui, width, min_h, |ui| {
        ui.horizontal(|ui| {
            draw_param_icon(ui, ParamIcon::Ripple);
            ui.label(RichText::new("Ripple Correction").size(15.5).strong().color(COLOR_TEXT_PRIMARY));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let mut val = app.mouse.ripple_control;
                if toggle_switch(ui, &mut val) {
                    app.apply_ripple_control(val);
                }
            });
        });
        ui.add_space(6.0);
        ui.add(egui::Label::new(
            RichText::new("Algorithm correction for high-speed movement to eliminate wave-like jitter.")
                .size(12.0)
                .color(COLOR_TEXT_MUTED),
        ).wrap());
    })
}

/* 4. Ultra Long Range Mode Card */
fn render_long_range_card(ui: &mut Ui, app: &mut AtkControlApp, width: f32, min_h: f32) -> egui::Response {
    card_container(ui, width, min_h, |ui| {
        ui.horizontal(|ui| {
            draw_param_icon(ui, ParamIcon::LongRange);
            ui.label(RichText::new("Ultra Long Range Mode").size(15.5).strong().color(COLOR_TEXT_PRIMARY));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let mut val = app.mouse.ultra_long_range;
                if toggle_switch(ui, &mut val) {
                    app.apply_ultra_long_range(val);
                }
            });
        });
        ui.add_space(6.0);
        ui.add(egui::Label::new(
            RichText::new("Improves wireless performance at long distances but increases power consumption")
                .size(12.0)
                .color(COLOR_TEXT_MUTED),
        ).wrap());
    })
}

/* 5. Key Debounce Delay Card */
fn render_debounce_card(ui: &mut Ui, app: &mut AtkControlApp, width: f32, min_h: f32) -> egui::Response {
    card_container(ui, width, min_h, |ui| {
        ui.horizontal(|ui| {
            draw_param_icon(ui, ParamIcon::Debounce);
            ui.label(RichText::new("Key Debounce Delay").size(15.5).strong().color(COLOR_TEXT_PRIMARY));
        });
        ui.add_space(4.0);
        ui.add(egui::Label::new(
            RichText::new("Prevents repeated or false triggers from rapid button presses or jitter")
                .size(12.0)
                .color(COLOR_TEXT_MUTED),
        ).wrap());
        ui.add_space(12.0);

        /* Two tidy rows so all pills fit comfortably with no horizontal clipping */
        const DEBOUNCE_ROW1: &[(u8, &str)] = &[
            (0, "0ms"),
            (1, "1ms"),
            (2, "2ms"),
            (4, "4ms"),
        ];
        const DEBOUNCE_ROW2: &[(u8, &str)] = &[
            (8, "8ms"),
            (15, "15ms"),
            (20, "20ms"),
        ];

        segmented_pills(ui, DEBOUNCE_ROW1, app.mouse.debounce_ms, |val| {
            app.apply_debounce(val);
        });
        ui.add_space(6.0);
        segmented_pills(ui, DEBOUNCE_ROW2, app.mouse.debounce_ms, |val| {
            app.apply_debounce(val);
        });
    })
}

/* 6. Sleep Time Setting Card */
fn render_sleep_card(ui: &mut Ui, app: &mut AtkControlApp, width: f32, min_h: f32) -> egui::Response {
    card_container(ui, width, min_h, |ui| {
        ui.horizontal(|ui| {
            draw_param_icon(ui, ParamIcon::Sleep);
            ui.label(RichText::new("Sleep Time Setting").size(15.5).strong().color(COLOR_TEXT_PRIMARY));
        });
        ui.add_space(4.0);
        ui.add(egui::Label::new(
            RichText::new("The mouse will automatically enter secondary sleep after a period of inactivity to reduce power consumption and extend battery life (the receiver indicator light going off is not related to this function)")
                .size(12.0)
                .color(COLOR_TEXT_MUTED),
        ).wrap());
        ui.add_space(12.0);

        /* Split into 2 neat rows of 4 options each so 20min, 25min, 30min never overflow */
        const SLEEP_ROW1: &[(usize, &str)] = &[
            (0, "30s"),
            (1, "1min"),
            (2, "2min"),
            (3, "3min"),
        ];
        const SLEEP_ROW2: &[(usize, &str)] = &[
            (4, "5min"),
            (5, "20min"),
            (6, "25min"),
            (7, "30min"),
        ];

        segmented_pills(ui, SLEEP_ROW1, app.mouse.sleep_time_idx, |idx| {
            app.apply_sleep_time(idx);
        });
        ui.add_space(6.0);
        segmented_pills(ui, SLEEP_ROW2, app.mouse.sleep_time_idx, |idx| {
            app.apply_sleep_time(idx);
        });
    })
}

/* 7. LOD Height Tolerance Card */
fn render_lod_card(ui: &mut Ui, app: &mut AtkControlApp, width: f32, min_h: f32) -> egui::Response {
    card_container(ui, width, min_h, |ui| {
        ui.horizontal(|ui| {
            draw_param_icon(ui, ParamIcon::Lod);
            ui.label(RichText::new("LOD Height Tolerance").size(15.5).strong().color(COLOR_TEXT_PRIMARY));
        });
        ui.add_space(4.0);
        ui.add(egui::Label::new(
            RichText::new("Maximum working distance from mouse bottom to surface")
                .size(12.0)
                .color(COLOR_TEXT_MUTED),
        ).wrap());
        ui.add_space(12.0);

        ui.horizontal(|ui| {
            const LOD_OPTIONS: &[(u8, &str)] = &[
                (1, "1mm"),
                (2, "2mm"),
            ];

            let cur_lod = app.mouse.lod_code;
            segmented_pills(ui, LOD_OPTIONS, cur_lod, |val| {
                app.apply_lod(val);
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                draw_lod_illustration(ui, cur_lod);
            });
        });
    })
}

/* 8. DPI Light Effect Card */
fn render_dpi_light_card(ui: &mut Ui, app: &mut AtkControlApp, width: f32, min_h: f32) -> egui::Response {
    card_container(ui, width, min_h, |ui| {
        let active_idx = app.mouse.active_stage as usize;
        let (sr, sg, sb) = app
            .mouse
            .stage_colors
            .get(active_idx)
            .copied()
            .unwrap_or((155, 93, 229));
        let stage_color = Color32::from_rgb(sr, sg, sb);

        ui.horizontal(|ui| {
            draw_param_icon(ui, ParamIcon::DpiLight);
            ui.label(RichText::new("DPI Light Effect").size(15.5).strong().color(COLOR_TEXT_PRIMARY));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new(format!("DPI {}", active_idx + 1))
                        .size(11.5)
                        .strong()
                        .color(COLOR_TEXT_MUTED),
                );
                let (dot_rect, _) = ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
                ui.painter().circle_filled(dot_rect.center(), 5.0, stage_color);
                ui.painter().circle_stroke(
                    dot_rect.center(),
                    5.0,
                    Stroke::new(1.0_f32, Color32::from_white_alpha(90)),
                );
            });
        });
        ui.add_space(4.0);
        ui.add(egui::Label::new(
            RichText::new("Illuminates the mouse DPI indicator LED using the active DPI stage color")
                .size(12.0)
                .color(COLOR_TEXT_MUTED),
        ).wrap());
        ui.add_space(12.0);

        const LIGHT_MODES: &[(u8, &str)] = &[
            (0, "Close"),
            (1, "Always On"),
            (2, "Breathing"),
        ];

        let cur_mode = app.mouse.dpi_light_mode;
        let cur_bright = app.mouse.dpi_light_brightness;
        let cur_speed = app.mouse.dpi_light_speed;

        segmented_pills(ui, LIGHT_MODES, cur_mode, |mode| {
            app.apply_dpi_light(mode, cur_bright, cur_speed);
        });

        if cur_mode != 0 {
            ui.add_space(10.0);
            ui.label(RichText::new("Brightness").size(12.0).color(COLOR_TEXT_MUTED));
            ui.add_space(4.0);

            const BRIGHT_OPTIONS: &[(u8, &str)] = &[
                (0, "Min Brightness"),
                (1, "Medium"),
                (2, "Max Brightness"),
            ];

            segmented_pills(ui, BRIGHT_OPTIONS, cur_bright, |b| {
                app.apply_dpi_light(cur_mode, b, cur_speed);
            });

            /* Speed setting is only available for Breathing animation (mode 2) */
            if cur_mode == 2 {
                ui.add_space(10.0);
                ui.label(RichText::new("Speed").size(12.0).color(COLOR_TEXT_MUTED));
                ui.add_space(4.0);

                const SPEED_OPTIONS: &[(u8, &str)] = &[
                    (0, "Slowest"),
                    (1, "Medium"),
                    (2, "Fastest"),
                ];

                segmented_pills(ui, SPEED_OPTIONS, cur_speed, |s| {
                    app.apply_dpi_light(cur_mode, cur_bright, s);
                });
            }
        }
    })
}

/* Card container matching the ATK graphite hub styling with explicit width and min_height */
fn card_container<R>(
    ui: &mut Ui,
    width: f32,
    min_height: f32,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> egui::Response {
    let inner_w = (width - 32.0_f32).max(180.0_f32);
    let inner_min_h = (min_height - 28.0_f32).max(0.0_f32);

    let frame = egui::Frame::none()
        .fill(Color32::from_rgb(20, 20, 24))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(34, 34, 42)))
        .rounding(Rounding::same(8.0))
        .inner_margin(egui::Margin::symmetric(16.0, 14.0));
    frame
        .show(ui, |ui| {
            ui.set_min_width(inner_w);
            ui.set_max_width(inner_w);
            if inner_min_h > 0.0 {
                ui.set_min_height(inner_min_h);
            }
            add_contents(ui)
        })
        .response
}

/* Custom interactive iOS/ATK toggle switch */
pub fn toggle_switch(ui: &mut Ui, on: &mut bool) -> bool {
    let desired_size = egui::vec2(40.0, 22.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool_responsive(response.id, *on);
        let bg_color = if *on {
            Color32::from_rgb(245, 245, 250)
        } else {
            Color32::from_rgb(46, 46, 54)
        };
        ui.painter().rect_filled(rect, 11.0, bg_color);

        let circle_x = egui::lerp((rect.left() + 11.0)..=(rect.right() - 11.0), how_on);
        let center = egui::pos2(circle_x, rect.center().y);
        let knob_color = if *on {
            Color32::from_rgb(18, 18, 22)
        } else {
            Color32::from_rgb(125, 125, 135)
        };
        ui.painter().circle_filled(center, 8.5, knob_color);
    }
    response.changed()
}

/* Segmented pill buttons (active: pure white background with dark bold text) */
fn segmented_pills<T: Copy + PartialEq>(
    ui: &mut Ui,
    options: &[(T, &str)],
    current: T,
    mut on_select: impl FnMut(T),
) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(5.0, 5.0);
        for &(val, label) in options {
            let is_selected = val == current;
            let (bg, text_color, weight) = if is_selected {
                (Color32::from_rgb(255, 255, 255), Color32::from_rgb(15, 15, 20), true)
            } else {
                (Color32::from_rgb(34, 34, 42), Color32::from_rgb(180, 180, 190), false)
            };

            let text = if weight {
                RichText::new(label).size(12.5).strong().color(text_color)
            } else {
                RichText::new(label).size(12.5).color(text_color)
            };

            let btn = egui::Button::new(text)
                .fill(bg)
                .stroke(Stroke::NONE)
                .rounding(Rounding::same(5.0))
                .min_size(Vec2::new(0.0, 26.0));

            if ui.add(btn).clicked() {
                on_select(val);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{Pos2, Rect, Vec2};

    #[test]
    fn test_parameters_view_layout_at_1920() {
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
                                    let target_toggle_h = 82.0_f32;
                                    let target_sleep_h = 2.0_f32 * target_toggle_h + 12.0_f32;

                                    let mut r_motion = Rect::NOTHING;
                                    let mut r_straight = Rect::NOTHING;
                                    let mut r_ripple = Rect::NOTHING;
                                    let mut r_range = Rect::NOTHING;
                                    let mut r_debounce = Rect::NOTHING;

                                    let mut r_sleep = Rect::NOTHING;
                                    let mut r_lod = Rect::NOTHING;
                                    let mut r_dpi = Rect::NOTHING;

                                    ui.horizontal_top(|ui| {
                                        ui.spacing_mut().item_spacing = Vec2::new(gap, 0.0);

                                        ui.allocate_ui_with_layout(
                                            Vec2::new(col_w, 0.0),
                                            egui::Layout::top_down(egui::Align::LEFT),
                                            |ui| {
                                                r_motion = render_motion_sync_card(ui, &mut app, col_w, target_toggle_h).rect;
                                                ui.add_space(12.0);
                                                r_straight = render_straight_line_card(ui, &mut app, col_w, target_toggle_h).rect;
                                                ui.add_space(12.0);
                                                r_ripple = render_ripple_card(ui, &mut app, col_w, target_toggle_h).rect;
                                                ui.add_space(12.0);
                                                r_range = render_long_range_card(ui, &mut app, col_w, target_toggle_h).rect;
                                                ui.add_space(12.0);
                                                r_debounce = render_debounce_card(ui, &mut app, col_w, 140.0).rect;
                                            },
                                        );

                                        ui.allocate_ui_with_layout(
                                            Vec2::new(col_w, 0.0),
                                            egui::Layout::top_down(egui::Align::LEFT),
                                            |ui| {
                                                r_sleep = render_sleep_card(ui, &mut app, col_w, target_sleep_h).rect;
                                                ui.add_space(12.0);
                                                r_lod = render_lod_card(ui, &mut app, col_w, 0.0).rect;
                                                ui.add_space(12.0);
                                                let dpi_min_h = (r_debounce.max.y - r_lod.max.y - 12.0).max(180.0);
                                                r_dpi = render_dpi_light_card(ui, &mut app, col_w, dpi_min_h).rect;
                                            },
                                        );
                                    });

                                    eprintln!("Motion: {:?}", r_motion);
                                    eprintln!("Straight: {:?}", r_straight);
                                    eprintln!("Ripple: {:?}", r_ripple);
                                    eprintln!("Range: {:?}", r_range);
                                    eprintln!("Debounce: {:?}", r_debounce);
                                    eprintln!("Sleep: {:?}", r_sleep);
                                    eprintln!("LOD: {:?}", r_lod);
                                    eprintln!("DPI: {:?}", r_dpi);

                                    /* All 4 toggle cards must have identical height */
                                    assert_eq!(r_motion.height(), r_straight.height());
                                    assert_eq!(r_ripple.height(), r_range.height());
                                    assert_eq!(r_motion.height(), r_range.height());

                                    /* Sleep card bottom aligns with Straight card bottom */
                                    assert_eq!(r_straight.max.y, r_sleep.max.y, "Straight Line and Sleep Time must align at bottom");

                                    /* DPI Light bottom aligns with Debounce bottom */
                                    assert_eq!(r_debounce.max.y, r_dpi.max.y, "DPI Light and Debounce must align at bottom");

                                    /* Widths must match */
                                    assert_eq!(r_motion.width(), r_sleep.width());
                                });
                        });
                });
        });
    }
}
