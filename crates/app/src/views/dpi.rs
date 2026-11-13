/*
 * DPI Settings View (1:1 Native Recreation of ATK Web Hub)
 * Supports:
 * - 1 column (vertical stack) on small/normal windows (< 860px available width)
 * - 2 columns on large/maximized windows (>= 860px available width) using top-aligned rows
 * - Identical fixed card widths across active and inactive stages
 * - Full-width sliders via explicit ui.spacing_mut().slider_width calculation
 * - Pill badge [ (●) DPI N ] for each stage with custom color dot and quick stage switching
 * - Clean headers: only the active card displays the Independent X/Y toggle and delete button
 * - Side-by-side X and Y sliders when independent X/Y mode is active
 * - Exact ATK 2D Saturation/Value + Hue + RGB color picker popup (matching official hub)
 * - Vector-rendered stepper chevrons and trash icon (zero missing font glyph boxes)
 */

use crate::state::AtkControlApp;
use crate::theme::{COLOR_TEXT_MUTED, COLOR_TEXT_PRIMARY};
use crate::views::dpi_widgets::{
    dpi_stepper_input, render_floating_color_picker, toggle_switch_custom,
};
use eframe::egui::{self, Color32, Layout, Pos2, RichText, Rounding, Stroke, Ui, Vec2};

pub fn render_dpi_view(ui: &mut Ui, app: &mut AtkControlApp) {
    /* Header Row: Title on the left, "+ Add DPI file" on the right */
    ui.horizontal(|ui| {
        ui.label(RichText::new("DPI Settings").size(22.0).strong().color(COLOR_TEXT_PRIMARY));
        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            let can_add = app.mouse.stage_count < 6;
            let add_btn = egui::Button::new(
                RichText::new("+ Add DPI file")
                    .size(13.0)
                    .color(if can_add { Color32::WHITE } else { Color32::from_gray(100) }),
            )
            .fill(if can_add { Color32::from_rgb(32, 32, 38) } else { Color32::from_rgb(22, 22, 26) })
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(48, 48, 56)))
            .rounding(Rounding::same(6.0))
            .min_size(Vec2::new(110.0, 28.0));

            if ui.add_enabled(can_add, add_btn).clicked() {
                app.apply_add_stage();
            }
        });
    });

    ui.add(
        egui::Label::new(
            RichText::new("Customize sensitivity stages, independent X/Y axes, and stage indicator colors")
                .size(13.0)
                .color(COLOR_TEXT_MUTED),
        )
        .wrap(),
    );
    ui.add_space(14.0);

    let stage_count = app.mouse.stage_count;
    let stages_indices: Vec<u8> = (0..stage_count).collect();

    /* 1 column for smaller windows (< 860px content width), 2 columns for large windows (>= 860px) */
    let available_w = ui.available_width();
    let use_two_cols = available_w >= 860.0;

    if use_two_cols {
        let gap = 16.0_f32;
        let col_w = ((available_w - gap) / 2.0_f32).floor();

        for chunk in stages_indices.chunks(2) {
            /* Use horizontal_top so left and right cards always align at their top edges */
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(gap, 0.0);

                let left_stage = chunk[0];
                ui.allocate_ui_with_layout(
                    Vec2::new(col_w, 0.0),
                    Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        render_dpi_card(ui, app, left_stage, col_w);
                    },
                );

                if chunk.len() > 1 {
                    let right_stage = chunk[1];
                    ui.allocate_ui_with_layout(
                        Vec2::new(col_w, 0.0),
                        Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            render_dpi_card(ui, app, right_stage, col_w);
                        },
                    );
                }
            });
            ui.add_space(14.0);
        }
    } else {
        /* Single column layout for smaller windows: full width, identical card dimensions */
        let col_w = available_w.floor();
        for &s in &stages_indices {
            render_dpi_card(ui, app, s, col_w);
            ui.add_space(14.0);
        }
    }
}

/* Individual DPI Stage Card with strictly enforced width */
fn render_dpi_card(ui: &mut Ui, app: &mut AtkControlApp, s: u8, card_width: f32) {
    ui.push_id(s, |ui| {
        let is_active = s == app.mouse.active_stage;
        let (current_x, current_y) = app.mouse.stages_dpi.get(s as usize).copied().unwrap_or((800, 800));
        let (r, g, b) = app.mouse.stage_colors.get(s as usize).copied().unwrap_or((255, 255, 255));
        let is_independent = app.mouse.independent_xy.get(s as usize).copied().unwrap_or(false);
        let is_color_picker_open = app.color_picker_open_stage == Some(s);

        let card_frame = egui::Frame::none()
            .fill(Color32::from_rgb(20, 20, 24))
            .stroke(Stroke::new(
                if is_active { 1.5_f32 } else { 1.0_f32 },
                if is_active { Color32::from_rgb(240, 240, 245) } else { Color32::from_rgb(34, 34, 42) },
            ))
            .rounding(Rounding::same(8.0))
            .inner_margin(egui::Margin::symmetric(16.0, 14.0));

        let mut circle_click_rect = egui::Rect::NOTHING;
        let mut pill_click_rect = egui::Rect::NOTHING;
        let mut opened_this_frame = false;
        let inner_w = (card_width - 35.0_f32).max(200.0_f32);

        card_frame.show(ui, |ui| {
            /* Enforce identical inner width on both active and inactive cards */
            ui.set_min_width(inner_w);
            ui.set_max_width(inner_w);

            /* Card Header: Pill Badge on the left, active controls on the right */
            ui.horizontal(|ui| {
                ui.set_min_width(inner_w);
                ui.set_max_width(inner_w);

                /* ATK Hub Style Stage Pill Badge: [ (●) DPI N ] with non-overlapping hit rects */
                let (pill_rect, dot_hit_rect, title_hit_rect) = render_stage_pill(
                    ui,
                    s,
                    is_active,
                    (r, g, b),
                    is_color_picker_open,
                );

                circle_click_rect = dot_hit_rect;
                pill_click_rect = pill_rect;

                /* Left portion of pill: Color Dot click toggles floating Color Picker */
                let dot_resp = ui.interact(dot_hit_rect, ui.id().with("dot_click"), egui::Sense::click());
                if dot_resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if dot_resp.clicked() {
                    if is_color_picker_open {
                        app.color_picker_open_stage = None;
                    } else {
                        app.color_picker_open_stage = Some(s);
                        opened_this_frame = true;
                    }
                    if !is_active {
                        app.apply_active_stage(s);
                    }
                }
                dot_resp.on_hover_text("Click to configure indicator color for this DPI stage");

                /* Right portion of pill: Stage Title click activates stage (or opens color picker if already active) */
                let title_resp = ui.interact(title_hit_rect, ui.id().with("title_click"), egui::Sense::click());
                if title_resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if title_resp.clicked() {
                    if !is_active {
                        app.apply_active_stage(s);
                    } else if is_color_picker_open {
                        app.color_picker_open_stage = None;
                    } else {
                        app.color_picker_open_stage = Some(s);
                        opened_this_frame = true;
                    }
                }
                if !is_active {
                    title_resp.on_hover_text("Click to activate this DPI stage");
                } else {
                    title_resp.on_hover_text("Click to configure indicator color for this DPI stage");
                }

                /* Right Header: ONLY shown on the ACTIVE card (matching ATK Web Hub 1:1) */
                if is_active {
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        if app.mouse.stage_count > 1 {
                            if render_trash_button(ui).on_hover_text("Delete this DPI stage").clicked() {
                                app.apply_delete_stage(s);
                                return;
                            }
                            ui.add_space(6.0);
                        }

                        /* Independent X/Y Toggle Switch */
                        let mut indep_val = is_independent;
                        if toggle_switch_custom(ui, ("indep_switch", s), &mut indep_val) {
                            if let Some(val) = app.mouse.independent_xy.get_mut(s as usize) {
                                *val = indep_val;
                            }
                            if !indep_val {
                                app.apply_dpi(s, current_x, current_x);
                            }
                        }

                        ui.add_space(6.0);
                        ui.label(RichText::new("X/Y Axis Independent Settings").size(11.5).color(Color32::from_gray(140)));
                    });
                }
            });

            ui.add_space(12.0);

            /* Card Body: Sliders and Steppers */
            if !is_independent {
                render_dpi_slider_row(ui, app, s, "X/Y", current_x, true, false, is_active, inner_w);
            } else if inner_w >= 380.0 {
                /* Side-by-side X and Y sliders in one row (matching ATK Web Hub) */
                let gap = 16.0_f32;
                let half_w = ((inner_w - gap) / 2.0_f32).floor();
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(gap, 0.0);
                    ui.allocate_ui_with_layout(Vec2::new(half_w, 0.0), Layout::top_down(egui::Align::LEFT), |ui| {
                        ui.set_min_width(half_w);
                        ui.set_max_width(half_w);
                        render_dpi_slider_row(ui, app, s, "X", current_x, false, false, is_active, half_w);
                    });
                    ui.allocate_ui_with_layout(Vec2::new(half_w, 0.0), Layout::top_down(egui::Align::LEFT), |ui| {
                        ui.set_min_width(half_w);
                        ui.set_max_width(half_w);
                        render_dpi_slider_row(ui, app, s, "Y", current_y, false, true, is_active, half_w);
                    });
                });
            } else {
                render_dpi_slider_row(ui, app, s, "X", current_x, false, false, is_active, inner_w);
                ui.add_space(8.0);
                render_dpi_slider_row(ui, app, s, "Y", current_y, false, true, is_active, inner_w);
            }
        });

        /* Floating 2D Color Picker Popup (matching ATK Web Hub Screenshot 9) */
        if app.color_picker_open_stage == Some(s) && pill_click_rect != egui::Rect::NOTHING {
            render_floating_color_picker(
                ui.ctx(),
                app,
                s,
                pill_click_rect,
                opened_this_frame,
                r,
                g,
                b,
            );
        }
    });
}

/* Vector-drawn trash can button so it never renders as a missing glyph box */
fn render_trash_button(ui: &mut Ui) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(20.0, 20.0), egui::Sense::click());
    if ui.is_rect_visible(rect) {
        let color = if resp.hovered() {
            Color32::from_rgb(235, 95, 95)
        } else {
            Color32::from_rgb(175, 75, 75)
        };
        let stroke = Stroke::new(1.3_f32, color);
        let c = rect.center();

        /* Lid line */
        ui.painter().line_segment([Pos2::new(c.x - 5.5, c.y - 4.0), Pos2::new(c.x + 5.5, c.y - 4.0)], stroke);
        /* Lid handle */
        ui.painter().line_segment([Pos2::new(c.x - 2.0, c.y - 5.5), Pos2::new(c.x + 2.0, c.y - 5.5)], stroke);
        /* Can body */
        let body_rect = egui::Rect::from_min_max(Pos2::new(c.x - 4.0, c.y - 3.5), Pos2::new(c.x + 4.0, c.y + 5.5));
        ui.painter().rect_stroke(body_rect, Rounding::same(1.0), stroke);
        /* Inner vertical ribs */
        ui.painter().line_segment([Pos2::new(c.x - 1.3, c.y - 1.5), Pos2::new(c.x - 1.3, c.y + 3.5)], stroke);
        ui.painter().line_segment([Pos2::new(c.x + 1.3, c.y - 1.5), Pos2::new(c.x + 1.3, c.y + 3.5)], stroke);
    }
    resp
}

/* Stage Pill Badge [ (●) DPI N ] drawn with exact ATK capsule styling and disjoint hit rects */
fn render_stage_pill(
    ui: &mut Ui,
    stage_idx: u8,
    is_active: bool,
    color_rgb: (u8, u8, u8),
    picker_open: bool,
) -> (egui::Rect, egui::Rect, egui::Rect) {
    let pill_h = 26.0_f32;
    let pill_w = 84.0_f32;
    let (pill_rect, _) = ui.allocate_exact_size(Vec2::new(pill_w, pill_h), egui::Sense::hover());

    let (r, g, b) = color_rgb;
    let bg_color = if is_active {
        Color32::from_rgb(34, 34, 42)
    } else {
        Color32::from_rgb(26, 26, 32)
    };
    let stroke_color = if is_active {
        Color32::from_rgb(70, 70, 85)
    } else {
        Color32::from_rgb(40, 40, 48)
    };

    ui.painter().rect(
        pill_rect,
        Rounding::same(13.0),
        bg_color,
        Stroke::new(1.0_f32, stroke_color),
    );

    /* Disjoint left hit rect for Color Dot (0..26px) and right hit rect for Title (26..84px) */
    let split_x = pill_rect.left() + 26.0_f32;
    let dot_hit_rect = egui::Rect::from_min_max(pill_rect.min, Pos2::new(split_x, pill_rect.bottom()));
    let title_hit_rect = egui::Rect::from_min_max(Pos2::new(split_x, pill_rect.top()), pill_rect.max);

    /* Left Color Dot */
    let dot_center = Pos2::new(pill_rect.left() + 14.0, pill_rect.center().y);
    let dot_radius = 6.0_f32;

    ui.painter().circle_filled(dot_center, dot_radius, Color32::from_rgb(r, g, b));
    let ring_stroke = if picker_open {
        Stroke::new(1.8_f32, Color32::WHITE)
    } else {
        Stroke::new(1.2_f32, Color32::from_white_alpha(90))
    };
    ui.painter().circle_stroke(dot_center, dot_radius, ring_stroke);

    /* Right Text "DPI N" */
    let text_pos = Pos2::new(pill_rect.left() + 28.0, pill_rect.center().y);
    let text_color = if is_active {
        Color32::WHITE
    } else {
        Color32::from_rgb(190, 190, 200)
    };
    ui.painter().text(
        text_pos,
        egui::Align2::LEFT_CENTER,
        format!("DPI {}", stage_idx + 1),
        egui::FontId::proportional(13.0),
        text_color,
    );

    (pill_rect, dot_hit_rect, title_hit_rect)
}

/* Horizontal row containing axis label, full-width slider, and number box with vector steppers */
fn render_dpi_slider_row(
    ui: &mut Ui,
    app: &mut AtkControlApp,
    stage: u8,
    axis_label: &str,
    current_val: u32,
    both_axes: bool,
    is_y_axis: bool,
    is_active: bool,
    row_width: f32,
) {
    ui.horizontal(|ui| {
        ui.set_min_width(row_width);
        ui.set_max_width(row_width);
        ui.spacing_mut().item_spacing = Vec2::new(8.0, 0.0);

        /* Fixed-width Axis Label so sliders start at the exact same X offset */
        let label_w = if both_axes { 26.0_f32 } else { 14.0_f32 };
        ui.allocate_ui_with_layout(
            Vec2::new(label_w, 22.0),
            Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.label(RichText::new(axis_label).size(12.5).strong().color(Color32::from_gray(170)));
            },
        );

        /* Explicitly set ui.spacing_mut().slider_width so egui::Slider stretches its track rail */
        let stepper_width = 66.0_f32;
        let slider_width = (row_width - label_w - stepper_width - 16.0_f32).max(50.0_f32);
        ui.spacing_mut().slider_width = slider_width;

        let mut edit_val = current_val;

        if is_active {
            ui.visuals_mut().selection.bg_fill = Color32::from_rgb(235, 235, 245);
        }

        let slider_resp = ui.add_sized(
            Vec2::new(slider_width, 22.0),
            egui::Slider::new(&mut edit_val, 50..=30000)
                .step_by(50.0)
                .show_value(false),
        );

        let (stepper_changed, stepper_finished) = dpi_stepper_input(ui, &mut edit_val);

        if slider_resp.changed() || stepper_changed {
            if let Some(st) = app.mouse.stages_dpi.get_mut(stage as usize) {
                if both_axes {
                    *st = (edit_val, edit_val);
                } else if is_y_axis {
                    st.1 = edit_val;
                } else {
                    st.0 = edit_val;
                }
            }
        }

        /* Apply immediately on drag release, track click, or stepper change */
        let clicked_track = slider_resp.changed() && !slider_resp.dragged();
        if slider_resp.drag_stopped() || clicked_track || stepper_finished {
            let (x, y) = app.mouse.stages_dpi.get(stage as usize).copied().unwrap_or((edit_val, edit_val));
            app.apply_dpi(stage, x, y);
        }
    });
}

