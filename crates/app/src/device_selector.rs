/* Device selector dropdown panel rendered inside the left sidebar. */

use crate::icons::{draw_icon, IconType};
use crate::state::AtkControlApp;
use crate::theme::COLOR_TEXT_PRIMARY;
use eframe::egui::{Align2, Color32, FontId, Pos2, Rect, Rounding, Sense, Stroke, Ui, Vec2};

pub fn render_device_selector_panel(ui: &mut Ui, app: &mut AtkControlApp) {
    let total_w = ui.available_width();

    /* Header row: "Select Device" and "Device List >" */
    let (header_rect, _) = ui.allocate_exact_size(Vec2::new(total_w, 18.0), Sense::hover());
    let painter = ui.painter();
    painter.text(
        Pos2::new(header_rect.min.x + 2.0, header_rect.center().y),
        Align2::LEFT_CENTER,
        "Select Device",
        FontId::proportional(12.5),
        COLOR_TEXT_PRIMARY,
    );
    painter.text(
        Pos2::new(header_rect.max.x - 2.0, header_rect.center().y),
        Align2::RIGHT_CENTER,
        "Device List >",
        FontId::proportional(11.0),
        Color32::from_gray(140),
    );

    ui.add_space(4.0);

    /* If no devices are connected, show clean empty state card */
    if app.device_list.is_empty() {
        let (empty_rect, _) = ui.allocate_exact_size(Vec2::new(total_w, 52.0), Sense::hover());
        let painter = ui.painter();
        painter.rect_filled(
            empty_rect,
            Rounding::same(7.0),
            Color32::from_rgb(22, 22, 28),
        );
        painter.rect_stroke(
            empty_rect,
            Rounding::same(7.0),
            Stroke::new(1.0_f32, Color32::from_rgb(38, 38, 48)),
        );
        painter.text(
            Pos2::new(empty_rect.center().x, empty_rect.center().y - 7.0),
            Align2::CENTER_CENTER,
            "No devices detected",
            FontId::proportional(12.5),
            Color32::from_gray(160),
        );
        painter.text(
            Pos2::new(empty_rect.center().x, empty_rect.center().y + 9.0),
            Align2::CENTER_CENTER,
            "Connect via 2.4G receiver or USB-C",
            FontId::proportional(10.5),
            Color32::from_gray(105),
        );
        ui.add_space(2.0);
        return;
    }

    /* Render Device Cards matching user reference */
    let card_h = 46.0;
    for (idx, dev) in app.device_list.clone().iter().enumerate() {
        let is_selected = idx == app.selected_device_idx;
        let (card_rect, resp) = ui.allocate_exact_size(Vec2::new(total_w, card_h), Sense::click());
        let hovered = resp.hovered();

        let painter = ui.painter();

        /* Background */
        let bg_color = if hovered {
            Color32::from_rgb(32, 32, 40)
        } else {
            Color32::from_rgb(22, 22, 28)
        };
        painter.rect_filled(card_rect, Rounding::same(7.0), bg_color);

        /* Border stroke: selected has crisp white border */
        let border_stroke = if is_selected {
            Stroke::new(1.4_f32, Color32::from_gray(225))
        } else if hovered {
            Stroke::new(1.0_f32, Color32::from_rgb(65, 65, 80))
        } else {
            Stroke::new(1.0_f32, Color32::from_rgb(38, 38, 48))
        };
        painter.rect_stroke(card_rect, Rounding::same(7.0), border_stroke);

        /* Thumbnail rounded box on left */
        let thumb_rect = Rect::from_min_size(
            Pos2::new(card_rect.min.x + 6.0, card_rect.min.y + 6.0),
            Vec2::new(34.0, 34.0),
        );
        painter.rect_filled(thumb_rect, Rounding::same(5.0), Color32::from_rgb(34, 34, 42));

        /* Mouse thumbnail image dynamically resolved by model family & color */
        let mouse_tex = app.mouse_texture_for(&dev.model_name, dev.color);
        if let Some(tex) = mouse_tex {
            let img_rect = Rect::from_center_size(thumb_rect.center(), Vec2::new(17.0, 30.0));
            painter.image(
                tex.id(),
                img_rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        }

        /* Right content area */
        let content_x = thumb_rect.max.x + 8.0;
        let row1_y = card_rect.min.y + 13.5;
        let row2_y = card_rect.min.y + 31.5;

        /* Row 1: Connection icon (strictly by connection mode) */
        let conn_icon_rect = Rect::from_center_size(Pos2::new(content_x + 7.0, row1_y), Vec2::new(14.0, 12.0));
        let conn_icon = match dev.mode {
            atk_device::ConnectionMode::Wired => IconType::Cable,
            atk_device::ConnectionMode::Bluetooth => IconType::Bluetooth,
            _ => IconType::Wireless,
        };
        draw_icon(painter, conn_icon_rect, conn_icon, Color32::from_gray(190));

        /* Row 1: Battery Icon (green if charging, red if < 20%) */
        let bat_rect = Rect::from_center_size(Pos2::new(content_x + 28.0, row1_y), Vec2::new(18.0, 10.0));
        draw_icon(
            painter,
            bat_rect,
            IconType::Battery {
                pct: dev.battery_pct,
                charging: dev.is_charging,
            },
            if dev.is_charging {
                Color32::from_rgb(0, 220, 120)
            } else if dev.battery_pct < 20 {
                Color32::from_rgb(235, 75, 75)
            } else {
                Color32::from_gray(190)
            },
        );

        /* Row 1: Battery % text */
        let card_bat_color = if dev.is_charging {
            Color32::from_rgb(0, 220, 120)
        } else if dev.battery_pct < 20 {
            Color32::from_rgb(235, 75, 75)
        } else {
            Color32::from_gray(210)
        };
        painter.text(
            Pos2::new(bat_rect.max.x + 5.0, row1_y),
            Align2::LEFT_CENTER,
            format!("{}%", dev.battery_pct),
            FontId::proportional(11.5),
            card_bat_color,
        );

        /* Row 2: Model Name in bold white text (clipped to card) */
        let card_clip = Rect::from_min_max(
            Pos2::new(content_x, card_rect.min.y),
            Pos2::new(card_rect.max.x - 6.0, card_rect.max.y),
        );
        painter.with_clip_rect(card_clip).text(
            Pos2::new(content_x, row2_y),
            Align2::LEFT_CENTER,
            &dev.model_name,
            FontId::proportional(13.0),
            COLOR_TEXT_PRIMARY,
        );

        if resp.clicked() {
            app.select_device(idx);
        }

        ui.add_space(3.0);
    }
}

