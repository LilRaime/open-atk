/*
 * Key Change Settings View
 * Renders the official photo of VXE R1 Pro Max, amber radar hotspots,
 * callout badges, button remapping modal, and defaults restore.
 */

use crate::state::AtkControlApp;
use crate::theme::{COLOR_ACCENT_ORANGE, COLOR_TEXT_MUTED, COLOR_TEXT_PRIMARY};
use eframe::egui::{
    self, Align, Align2, Color32, FontId, Layout, Pos2, Rect, RichText, Rounding, Sense, Stroke,
    Ui, Vec2,
};

pub fn render_key_change_view(ui: &mut Ui, app: &mut AtkControlApp) {
    let available_size = ui.available_size();
    
    /* Centered mouse position exactly as in reference layout */
    let center = Pos2::new(
        ui.min_rect().min.x + available_size.x * 0.5,
        ui.min_rect().min.y + available_size.y * 0.44,
    );

    let total_h = available_size.y.max(center.y - ui.min_rect().min.y + 335.0);
    let (_response, painter) = ui.allocate_painter(Vec2::new(available_size.x, total_h), Sense::hover());

    /* 1. Mouse image dimensions (matches 1:1 reference aspect ratio 234:442) */
    let mouse_h = 440.0;
    let mouse_w = 234.0;
    let mouse_rect = Rect::from_center_size(center, Vec2::new(mouse_w, mouse_h));

    /* Render official transparent high-res PNG (pre-rendered natural drop shadow) */
    if let Some(texture) = app.current_mouse_texture() {
        painter.image(
            texture.id(),
            mouse_rect,
            Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
    }

    /* 2. Button Hotspots & Badges (1:1 pixel-accurate layout from user reference) */
    /* (index, hotspot_offset, badge_offset, default_size, default_title) */
    let button_coords = [
        /* 0: Left Button (LKM - locked from remapping) */
        (0, Vec2::new(-36.0, -177.0), Vec2::new(-105.0, -180.0), Vec2::new(100.0, 50.0), "Left Button"),
        /* 1: Right Click */
        (1, Vec2::new(87.0, -51.0), Vec2::new(146.0, -52.0), Vec2::new(104.0, 50.0), "Right Click"),
        /* 2: Middle Key (directly on scroll wheel!) */
        (2, Vec2::new(5.0, -126.0), Vec2::new(64.0, -126.0), Vec2::new(102.0, 50.0), "Middle Key"),
        /* 3: Back (lower side button - badge placed to right on mouse body!) */
        (3, Vec2::new(-108.0, 18.0), Vec2::new(-32.0, 14.0), Vec2::new(104.0, 50.0), "Back"),
        /* 4: Advance (upper side button - badge placed to left outside mouse!) */
        (4, Vec2::new(-94.0, -47.0), Vec2::new(-148.0, -42.0), Vec2::new(96.0, 50.0), "Advance"),
        /* 5: DPI Cycle (palm button on right) */
        (5, Vec2::new(63.0, 55.0), Vec2::new(126.0, 60.0), Vec2::new(92.0, 50.0), "DPI Cycle"),
    ];

    let mut clicked_button = None;

    for &(btn_idx, hotspot_off, badge_off, badge_size, default_title) in &button_coords {
        let hotspot_pos = center + hotspot_off;

        /* Draw dotted radar hotspot */
        draw_dotted_hotspot(&painter, hotspot_pos);

        /* Action title from app state */
        let action_text = app
            .mouse
            .buttons
            .get(btn_idx)
            .map(|b| b.action.as_str())
            .unwrap_or(default_title);

        /* Format macro text cleanly to prevent overflow */
        let (display_title, clean_action) = if let Some(stripped) = action_text.strip_prefix("Macro: ") {
            (format!("{} (Macro)", default_title), stripped)
        } else {
            (default_title.to_string(), action_text)
        };

        /* Dynamically compute width to fit text with comfortable padding */
        let sub_len = clean_action.chars().count();
        let title_len = display_title.chars().count();
        let max_len = sub_len.max(title_len);
        let needed_w = (max_len as f32 * 7.6 + 24.0).max(badge_size.x);
        let actual_w = needed_w.min(165.0);

        /* Truncate action text gracefully if it still exceeds max width */
        let final_action = if clean_action.chars().count() > 18 {
            let mut s: String = clean_action.chars().take(17).collect();
            s.push('…');
            s
        } else {
            clean_action.to_string()
        };

        /* Position badge so it expands gracefully without overlapping hotspots */
        let badge_pos = if btn_idx == 3 {
            /* Back button badge: anchored right of side hotspot so it expands rightward across the mouse */
            let left_edge = hotspot_pos.x + 24.0;
            Pos2::new(left_edge + actual_w * 0.5, center.y + badge_off.y)
        } else if badge_off.x < 0.0 {
            /* Left badges (Left Button, Advance): expand leftwards */
            let right_edge = center.x + badge_off.x + badge_size.x * 0.5;
            Pos2::new(right_edge - actual_w * 0.5, center.y + badge_off.y)
        } else {
            /* Right badges (Middle Key, Right Click, DPI Cycle): expand rightwards */
            let left_edge = center.x + badge_off.x - badge_size.x * 0.5;
            Pos2::new(left_edge + actual_w * 0.5, center.y + badge_off.y)
        };

        let dynamic_size = Vec2::new(actual_w, badge_size.y);
        let is_selected = app.selected_button_idx == Some(btn_idx);

        /* Draw callout badge */
        if draw_callout_badge(ui, badge_pos, dynamic_size, &display_title, &final_action, is_selected) {
            if btn_idx == 0 {
                /* Left click is locked to prevent disabling primary click */
                app.set_status("Left Button cannot be remapped (reserved for primary click)", true);
            } else {
                clicked_button = Some(btn_idx);
            }
        }
    }

    if let Some(id) = clicked_button {
        app.selected_button_idx = if app.selected_button_idx == Some(id) {
            None
        } else {
            Some(id)
        };
    }

    /* 3. Color switcher: White / Black */
    let modal_open = app.selected_button_idx.is_some();
    let is_white = app.mouse_color == crate::state::MouseColor::White;
    let is_black = app.mouse_color == crate::state::MouseColor::Black;

    let color_bar_rect = Rect::from_center_size(
        Pos2::new(center.x, center.y + 256.0),
        Vec2::new(170.0, 28.0),
    );
    let half_w = (color_bar_rect.width() - 4.0) * 0.5;
    let white_rect = Rect::from_min_size(color_bar_rect.min, Vec2::new(half_w, color_bar_rect.height()));
    let black_rect = Rect::from_min_size(
        Pos2::new(color_bar_rect.max.x - half_w, color_bar_rect.min.y),
        Vec2::new(half_w, color_bar_rect.height()),
    );

    let white_resp = ui.allocate_rect(white_rect, Sense::click());
    let black_resp = ui.allocate_rect(black_rect, Sense::click());

    /* White Button */
    let white_bg = if is_white {
        Color32::from_rgb(52, 52, 64)
    } else if white_resp.hovered() && !modal_open {
        Color32::from_rgb(38, 38, 46)
    } else {
        Color32::from_rgb(24, 24, 30)
    };
    painter.rect_filled(white_rect, Rounding::same(6.0), white_bg);
    if is_white {
        painter.rect_stroke(
            white_rect,
            Rounding::same(6.0),
            Stroke::new(1.2_f32, Color32::from_gray(190)),
        );
    }
    let white_dot_c = Pos2::new(white_rect.center().x - 22.0, white_rect.center().y);
    painter.circle_filled(white_dot_c, 4.0, Color32::WHITE);
    painter.circle_stroke(white_dot_c, 4.0, Stroke::new(1.0_f32, Color32::from_gray(160)));
    painter.text(
        Pos2::new(white_rect.center().x - 12.0, white_rect.center().y),
        Align2::LEFT_CENTER,
        "White",
        FontId::proportional(12.5),
        if is_white { Color32::WHITE } else { Color32::from_gray(160) },
    );

    /* Black Button */
    let black_bg = if is_black {
        Color32::from_rgb(52, 52, 64)
    } else if black_resp.hovered() && !modal_open {
        Color32::from_rgb(38, 38, 46)
    } else {
        Color32::from_rgb(24, 24, 30)
    };
    painter.rect_filled(black_rect, Rounding::same(6.0), black_bg);
    if is_black {
        painter.rect_stroke(
            black_rect,
            Rounding::same(6.0),
            Stroke::new(1.2_f32, Color32::from_gray(190)),
        );
    }
    let black_dot_c = Pos2::new(black_rect.center().x - 22.0, black_rect.center().y);
    painter.circle_filled(black_dot_c, 4.0, Color32::from_gray(24));
    painter.circle_stroke(black_dot_c, 4.0, Stroke::new(1.2_f32, Color32::from_gray(180)));
    painter.text(
        Pos2::new(black_rect.center().x - 12.0, black_rect.center().y),
        Align2::LEFT_CENTER,
        "Black",
        FontId::proportional(12.5),
        if is_black { Color32::WHITE } else { Color32::from_gray(160) },
    );

    if !modal_open && white_resp.clicked() {
        app.set_active_mouse_color(crate::state::MouseColor::White);
    }
    if !modal_open && black_resp.clicked() {
        app.set_active_mouse_color(crate::state::MouseColor::Black);
    }

    /* 4. Large "Restore Default" button centered directly below the mouse chassis */
    let btn_rect = Rect::from_center_size(
        Pos2::new(center.x, center.y + 302.0),
        Vec2::new(170.0, 34.0),
    );
    let btn_resp = ui.allocate_rect(btn_rect, Sense::click());
    let btn_hover = btn_resp.hovered() && !modal_open;

    painter.rect_filled(
        btn_rect,
        Rounding::same(7.0),
        if btn_hover { Color32::from_rgb(52, 52, 62) } else { Color32::from_rgb(34, 34, 40) },
    );
    painter.rect_stroke(
        btn_rect,
        Rounding::same(7.0),
        Stroke::new(1.0_f32, Color32::from_rgb(65, 65, 78)),
    );
    painter.text(
        btn_rect.center(),
        Align2::CENTER_CENTER,
        "Restore Default",
        FontId::proportional(13.5),
        Color32::from_gray(225),
    );

    if !modal_open && btn_resp.clicked() {
        app.restore_defaults();
    }

    /* 4. Render Button Remap Modal if an allowed button is selected (rendered LAST so backdrop covers canvas) */
    if let Some(selected_id) = app.selected_button_idx {
        if selected_id != 0 {
            render_button_remap_modal(ui, app, selected_id, center);
        } else {
            app.selected_button_idx = None;
        }
    }
}

/* Draws an amber dotted ring radar hotspot */
fn draw_dotted_hotspot(painter: &egui::Painter, pos: Pos2) {
    let color = COLOR_ACCENT_ORANGE;
    let radius = 8.5;
    let num_dots = 10;

    for i in 0..num_dots {
        let angle = (i as f32) * (std::f32::consts::TAU / num_dots as f32);
        let dot_pos = Pos2::new(pos.x + radius * angle.cos(), pos.y + radius * angle.sin());
        painter.circle_filled(dot_pos, 1.4, color);
    }
}

/* Draws an interactive callout badge matching 1:1 user reference visual style */
fn draw_callout_badge(
    ui: &mut Ui,
    center: Pos2,
    size: Vec2,
    title: &str,
    subtitle: &str,
    is_selected: bool,
) -> bool {
    let rect = Rect::from_center_size(center, size);

    let resp = ui.allocate_rect(rect, Sense::click());
    let hovered = resp.hovered();

    let bg = if is_selected {
        Color32::from_rgb(58, 58, 66)
    } else if hovered {
        Color32::from_rgb(56, 56, 60)
    } else {
        Color32::from_rgb(46, 46, 48) /* Clean sleek charcoal gray matching reference */
    };

    ui.painter().rect_filled(rect, Rounding::same(7.0), bg);

    let stroke = if is_selected {
        Stroke::new(1.3_f32, COLOR_ACCENT_ORANGE)
    } else {
        Stroke::new(0.9_f32, Color32::from_rgb(62, 62, 68))
    };
    ui.painter().rect_stroke(rect, Rounding::same(7.0), stroke);

    /* Top title line (muted gray label) */
    ui.painter().text(
        Pos2::new(rect.center().x, rect.min.y + 14.5),
        Align2::CENTER_CENTER,
        title,
        FontId::proportional(11.0),
        Color32::from_rgb(160, 160, 168),
    );

    /* Bottom action line (bold white action text) */
    ui.painter().text(
        Pos2::new(rect.center().x, rect.min.y + 32.5),
        Align2::CENTER_CENTER,
        subtitle,
        FontId::proportional(12.5),
        Color32::from_gray(245),
    );

    resp.clicked()
}

/* Renders a spacious button remapping modal properly sized to fully contain all buttons */
fn render_button_remap_modal(
    ui: &mut Ui,
    app: &mut AtkControlApp,
    selected_id: usize,
    canvas_center: Pos2,
) {
    /* Generously sized to completely bound all button categories without clipping */
    let modal_rect = Rect::from_center_size(
        Pos2::new(canvas_center.x, canvas_center.y),
        Vec2::new(430.0, 520.0),
    );

    let painter = ui.painter();

    /* Dark semi-transparent backdrop behind modal */
    let backdrop = Rect::from_center_size(canvas_center, Vec2::new(1200.0, 1000.0));
    painter.rect_filled(backdrop, Rounding::ZERO, Color32::from_rgba_unmultiplied(0, 0, 0, 130));

    /* Modal background with orange accent border */
    painter.rect_filled(modal_rect, Rounding::same(10.0), Color32::from_rgb(24, 24, 30));
    painter.rect_stroke(
        modal_rect,
        Rounding::same(10.0),
        Stroke::new(1.5_f32, COLOR_ACCENT_ORANGE),
    );

    let btn_title = app.mouse.buttons.get(selected_id).map(|b| b.title).unwrap_or("Button");

    let mut new_action = None;
    let mut close_modal = false;

    ui.allocate_new_ui(
        egui::UiBuilder::new().max_rect(modal_rect.shrink(18.0)),
        |child_ui| {
            child_ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("Remap {}", btn_title))
                        .size(17.0)
                        .strong()
                        .color(COLOR_TEXT_PRIMARY),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button(RichText::new("X").size(13.0).strong()).clicked() {
                        close_modal = true;
                    }
                });
            });

            child_ui.add_space(8.0);
            child_ui.label(
                RichText::new("Choose an action to assign to mouse hardware:")
                    .size(13.0)
                    .color(COLOR_TEXT_MUTED),
            );
            child_ui.add_space(8.0);

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(child_ui, |scroll_ui| {
                    /* MOUSE ACTIONS */
                    scroll_ui.label(
                        RichText::new("MOUSE ACTIONS")
                            .size(11.5)
                            .strong()
                            .color(COLOR_ACCENT_ORANGE),
                    );
                    scroll_ui.add_space(4.0);
                    scroll_ui.horizontal_wrapped(|ui| {
                        for &act in &["Left Button", "Right Click", "Middle Key", "Advance", "Back"] {
                            if ui.button(RichText::new(act).size(13.0)).clicked() {
                                new_action = Some(act.to_string());
                            }
                        }
                    });

                    scroll_ui.add_space(14.0);

                    /* DPI CONTROLS */
                    scroll_ui.label(
                        RichText::new("DPI CONTROLS")
                            .size(11.5)
                            .strong()
                            .color(Color32::from_rgb(0, 200, 255)),
                    );
                    scroll_ui.add_space(4.0);
                    scroll_ui.horizontal_wrapped(|ui| {
                        for &act in &["DPI Cycle", "DPI Up", "DPI Down", "Stage 1", "Stage 2"] {
                            if ui.button(RichText::new(act).size(13.0)).clicked() {
                                new_action = Some(act.to_string());
                            }
                        }
                    });

                    scroll_ui.add_space(14.0);

                    /* CUSTOM MACROS */
                    scroll_ui.label(
                        RichText::new("CUSTOM MACROS")
                            .size(11.5)
                            .strong()
                            .color(Color32::from_rgb(255, 175, 40)),
                    );
                    scroll_ui.add_space(4.0);
                    if app.macros.is_empty() {
                        scroll_ui.label(
                            RichText::new("No macros found. Create one in Macro Settings!")
                                .size(12.5)
                                .italics()
                                .color(COLOR_TEXT_MUTED),
                        );
                    } else {
                        scroll_ui.horizontal_wrapped(|ui| {
                            for m in &app.macros {
                                let btn_text = format!("Macro: {}", m.name);
                                if ui.button(RichText::new(&btn_text).size(12.5).strong()).clicked() {
                                    new_action = Some(btn_text);
                                }
                            }
                        });
                    }

                    scroll_ui.add_space(14.0);

                    /* MEDIA & UTILITY */
                    scroll_ui.label(
                        RichText::new("MEDIA & UTILITY")
                            .size(11.5)
                            .strong()
                            .color(Color32::from_rgb(140, 220, 100)),
                    );
                    scroll_ui.add_space(4.0);
                    scroll_ui.horizontal_wrapped(|ui| {
                        for &act in &["Vol +", "Vol -", "Mute", "Play/Pause", "Disabled"] {
                            if ui.button(RichText::new(act).size(13.0)).clicked() {
                                new_action = Some(act.to_string());
                            }
                        }
                    });
                });
        },
    );

    if let Some(act) = new_action {
        /* Real hardware write and flash commit! */
        app.apply_button_mapping(selected_id, &act);
        app.selected_button_idx = None;
    }

    if close_modal {
        app.selected_button_idx = None;
    }
}
