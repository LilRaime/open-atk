/* Reusable widgets for the DPI Settings view: stepper input, custom toggle, and floating HSV/RGB color picker. */

use crate::state::AtkControlApp;
use eframe::egui::{self, Color32, Layout, Pos2, RichText, Rounding, Stroke, Ui, Vec2};

/* Number input box with vector-drawn up/down chevrons (no missing font glyphs) */
pub fn dpi_stepper_input(ui: &mut Ui, val: &mut u32) -> (bool, bool) {
    let mut changed = false;
    let mut finished = false;

    let frame = egui::Frame::none()
        .fill(Color32::from_rgb(26, 26, 32))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(48, 48, 58)))
        .rounding(Rounding::same(4.0))
        .inner_margin(egui::Margin::symmetric(4.0, 2.0));

    frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(2.0, 0.0);

            let drag = egui::DragValue::new(val)
                .range(50..=30000)
                .speed(10.0);
            let drag_resp = ui.add_sized(Vec2::new(42.0, 18.0), drag);
            if drag_resp.changed() {
                changed = true;
            }
            if drag_resp.lost_focus() || drag_resp.drag_stopped() {
                finished = true;
            }

            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(0.0, 1.0);

                /* Up Chevron Button */
                let (up_rect, up_resp) = ui.allocate_exact_size(Vec2::new(12.0, 8.5), egui::Sense::click());
                if up_resp.clicked() {
                    *val = (*val + 50).min(30000);
                    changed = true;
                    finished = true;
                }
                if ui.is_rect_visible(up_rect) {
                    let stroke = Stroke::new(
                        1.3_f32,
                        if up_resp.hovered() { Color32::WHITE } else { Color32::from_gray(165) },
                    );
                    let c = up_rect.center();
                    ui.painter().line_segment([Pos2::new(c.x - 3.0, c.y + 1.5), Pos2::new(c.x, c.y - 1.5)], stroke);
                    ui.painter().line_segment([Pos2::new(c.x, c.y - 1.5), Pos2::new(c.x + 3.0, c.y + 1.5)], stroke);
                }

                /* Down Chevron Button */
                let (dn_rect, dn_resp) = ui.allocate_exact_size(Vec2::new(12.0, 8.5), egui::Sense::click());
                if dn_resp.clicked() {
                    *val = val.saturating_sub(50).max(50);
                    changed = true;
                    finished = true;
                }
                if ui.is_rect_visible(dn_rect) {
                    let stroke = Stroke::new(
                        1.3_f32,
                        if dn_resp.hovered() { Color32::WHITE } else { Color32::from_gray(165) },
                    );
                    let c = dn_rect.center();
                    ui.painter().line_segment([Pos2::new(c.x - 3.0, c.y - 1.5), Pos2::new(c.x, c.y + 1.5)], stroke);
                    ui.painter().line_segment([Pos2::new(c.x, c.y + 1.5), Pos2::new(c.x + 3.0, c.y - 1.5)], stroke);
                }
            });
        });
    });

    (changed, finished)
}

/* Custom interactive iOS/ATK toggle switch with unique persistent salt */
pub fn toggle_switch_custom(ui: &mut Ui, id_salt: impl std::hash::Hash, on: &mut bool) -> bool {
    let desired_size = egui::vec2(36.0, 20.0);
    let id = ui.make_persistent_id(id_salt);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool_responsive(id, *on);
        let bg_color = if *on {
            Color32::from_rgb(245, 245, 250)
        } else {
            Color32::from_rgb(46, 46, 54)
        };
        ui.painter().rect_filled(rect, 10.0, bg_color);

        let circle_x = egui::lerp((rect.left() + 10.0)..=(rect.right() - 10.0), how_on);
        let center = Pos2::new(circle_x, rect.center().y);
        let knob_color = if *on {
            Color32::from_rgb(18, 18, 22)
        } else {
            Color32::from_rgb(125, 125, 135)
        };
        ui.painter().circle_filled(center, 7.5, knob_color);
    }
    response.changed()
}

/* Floating 2D Saturation/Value + Hue + RGB Color Picker matching ATK Web Hub */
pub fn render_floating_color_picker(
    ctx: &egui::Context,
    app: &mut AtkControlApp,
    stage: u8,
    anchor_rect: egui::Rect,
    opened_this_frame: bool,
    mut r: u8,
    mut g: u8,
    mut b: u8,
) {
    let popup_pos = anchor_rect.left_bottom() + Vec2::new(0.0, 6.0);
    let area_id = egui::Id::new(("floating_color_picker", stage));

    let mut should_close = false;

    egui::Area::new(area_id)
        .order(egui::Order::Tooltip)
        .interactable(true)
        .fixed_pos(popup_pos)
        .show(ctx, |ui| {
            let frame = egui::Frame::none()
                .fill(Color32::from_rgb(22, 22, 26))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(65, 65, 78)))
                .rounding(Rounding::same(8.0))
                .inner_margin(egui::Margin::same(12.0));

            let popup_resp = frame.show(ui, |ui| {
                ui.set_width(220.0);

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Stage Color").size(12.5).strong().color(Color32::from_rgb(220, 220, 225)));
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        /* Vector-drawn close X button */
                        let (x_rect, x_resp) = ui.allocate_exact_size(Vec2::splat(18.0), egui::Sense::click());
                        if x_resp.clicked() {
                            should_close = true;
                        }
                        if ui.is_rect_visible(x_rect) {
                            let c = x_rect.center();
                            let stroke = Stroke::new(
                                1.4_f32,
                                if x_resp.hovered() { Color32::WHITE } else { Color32::from_gray(150) },
                            );
                            ui.painter().line_segment([Pos2::new(c.x - 3.5, c.y - 3.5), Pos2::new(c.x + 3.5, c.y + 3.5)], stroke);
                            ui.painter().line_segment([Pos2::new(c.x + 3.5, c.y - 3.5), Pos2::new(c.x - 3.5, c.y + 3.5)], stroke);
                        }
                    });
                });
                ui.add_space(8.0);

                /* 2D Saturation/Value gradient picker + Hue rainbow slider */
                let mut hsva = egui::ecolor::Hsva::from_srgb([r, g, b]);
                if egui::color_picker::color_picker_hsva_2d(ui, &mut hsva, egui::color_picker::Alpha::Opaque) {
                    let [nr, ng, nb] = hsva.to_srgb();
                    r = nr;
                    g = ng;
                    b = nb;
                    app.apply_stage_color(stage, r, g, b);
                }

                ui.add_space(8.0);

                /* Bottom Row: Circle swatch + R, G, B number inputs */
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(8.0, 0.0);

                    /* Color circle swatch */
                    let (swatch_rect, _) = ui.allocate_exact_size(Vec2::splat(24.0), egui::Sense::hover());
                    ui.painter().circle_filled(swatch_rect.center(), 11.0, Color32::from_rgb(r, g, b));
                    ui.painter().circle_stroke(swatch_rect.center(), 11.0, Stroke::new(1.5_f32, Color32::WHITE));

                    /* R, G, B input fields with labels underneath */
                    let mut input_changed = false;

                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(0.0, 2.0);
                        if ui.add_sized(Vec2::new(42.0, 20.0), egui::DragValue::new(&mut r).range(0..=255)).changed() {
                            input_changed = true;
                        }
                        ui.label(RichText::new("R").size(10.5).color(Color32::from_gray(140)));
                    });

                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(0.0, 2.0);
                        if ui.add_sized(Vec2::new(42.0, 20.0), egui::DragValue::new(&mut g).range(0..=255)).changed() {
                            input_changed = true;
                        }
                        ui.label(RichText::new("G").size(10.5).color(Color32::from_gray(140)));
                    });

                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(0.0, 2.0);
                        if ui.add_sized(Vec2::new(42.0, 20.0), egui::DragValue::new(&mut b).range(0..=255)).changed() {
                            input_changed = true;
                        }
                        ui.label(RichText::new("B").size(10.5).color(Color32::from_gray(140)));
                    });

                    if input_changed {
                        app.apply_stage_color(stage, r, g, b);
                    }
                });
            });

            /* Dismiss popup on Escape key or primary click outside both the popup and the stage pill */
            if !opened_this_frame {
                if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                    should_close = true;
                } else if let Some(pos) = ctx.pointer_latest_pos() {
                    if ctx.input(|i| i.pointer.primary_clicked()) {
                        let popup_rect = popup_resp.response.rect.expand(6.0);
                        let anchor_expanded = anchor_rect.expand(4.0);
                        if !popup_rect.contains(pos) && !anchor_expanded.contains(pos) {
                            should_close = true;
                        }
                    }
                }
            }
        });

    if should_close {
        app.color_picker_open_stage = None;
    }
}

