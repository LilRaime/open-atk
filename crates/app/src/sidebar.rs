/* Sidebar view rendering for OpenATK */

use crate::device_selector::render_device_selector_panel;
use crate::icons::{draw_icon, IconType};
use crate::state::{AtkControlApp, NavItem};
use crate::theme::{COLOR_ACCENT_ORANGE, COLOR_BG_SIDEBAR, COLOR_TEXT_MUTED, COLOR_TEXT_PRIMARY};
use eframe::egui::{
    Align2, Color32, FontId, Pos2, Rect, RichText, Rounding, Sense, Stroke, Ui, Vec2,
};

/* Renders all top and middle sidebar content (status, active mouse, device selector, nav items) */
pub fn render_sidebar_content(ui: &mut Ui, app: &mut AtkControlApp) {
    ui.spacing_mut().item_spacing = Vec2::new(0.0, 2.0);

    /* 1. Top status line: Battery & Signal (perfectly aligned on shared center line) */
    let (status_rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 20.0), Sense::hover());
    let cy = status_rect.center().y;
    let painter = ui.painter();

    if app.connected {
        let (bat_pct, is_chg, mode) = if let Some(entry) = app.device_list.get(app.selected_device_idx) {
            (
                if entry.battery_pct > 0 {
                    entry.battery_pct
                } else {
                    app.mouse.battery_pct
                },
                entry.is_charging || app.mouse.is_charging || entry.mode == atk_device::ConnectionMode::Wired,
                entry.mode,
            )
        } else {
            (
                app.mouse.battery_pct,
                app.mouse.is_charging || app.mouse.mode == atk_device::ConnectionMode::Wired,
                app.mouse.mode,
            )
        };

        /* Battery Icon */
        let bat_w = 20.0;
        let bat_h = 10.5;
        let bat_rect = Rect::from_center_size(
            Pos2::new(status_rect.min.x + bat_w * 0.5 + 2.0, cy),
            Vec2::new(bat_w, bat_h),
        );
        draw_icon(
            painter,
            bat_rect,
            IconType::Battery {
                pct: bat_pct,
                charging: is_chg,
            },
            if is_chg {
                Color32::from_rgb(0, 220, 120)
            } else if bat_pct < 20 {
                Color32::from_rgb(235, 75, 75)
            } else {
                Color32::from_gray(190)
            },
        );

        /* Battery percentage text (e.g. "95%") */
        let text_x = bat_rect.max.x + 7.0;
        let bat_text = format!("{}%", bat_pct);
        let bat_text_color = if is_chg {
            Color32::from_rgb(0, 220, 120)
        } else if bat_pct < 20 {
            Color32::from_rgb(235, 75, 75)
        } else {
            Color32::from_gray(210)
        };
        painter.text(
            Pos2::new(text_x, cy),
            Align2::LEFT_CENTER,
            &bat_text,
            FontId::proportional(12.5),
            bat_text_color,
        );

        /* Connection Mode Icon (Cable if Wired, Bluetooth if BT, Wireless otherwise) */
        let text_len = bat_text.chars().count();
        let text_w = text_len as f32 * 7.5;
        let conn_center_x = text_x + text_w + 12.0;
        let conn_rect = Rect::from_center_size(
            Pos2::new(conn_center_x, cy),
            Vec2::new(16.0, 14.0),
        );
        let conn_icon = match mode {
            atk_device::ConnectionMode::Wired => IconType::Cable,
            atk_device::ConnectionMode::Bluetooth => IconType::Bluetooth,
            _ => IconType::Wireless,
        };
        draw_icon(painter, conn_rect, conn_icon, Color32::from_gray(190));
    } else {
        /* Disconnected status line: muted outline battery & "--%" text */
        let bat_w = 20.0;
        let bat_h = 10.5;
        let bat_rect = Rect::from_center_size(
            Pos2::new(status_rect.min.x + bat_w * 0.5 + 2.0, cy),
            Vec2::new(bat_w, bat_h),
        );
        draw_icon(
            painter,
            bat_rect,
            IconType::Battery {
                pct: 0,
                charging: false,
            },
            Color32::from_gray(100),
        );

        let text_x = bat_rect.max.x + 7.0;
        painter.text(
            Pos2::new(text_x, cy),
            Align2::LEFT_CENTER,
            "--%",
            FontId::proportional(12.5),
            Color32::from_gray(120),
        );

        let status_label_x = text_x + 34.0;
        painter.text(
            Pos2::new(status_label_x, cy),
            Align2::LEFT_CENTER,
            "Disconnected",
            FontId::proportional(11.5),
            Color32::from_gray(115),
        );
    }

    ui.add_space(4.0);

    /* 2. Device Name: e.g. "VXE R1 PRO MAX  >" or "VXE R1SE+  >" */
    let (dev_rect, dev_resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 28.0), Sense::click());
    let dev_hover = dev_resp.hovered();

    if dev_hover {
        ui.painter().rect_filled(
            dev_rect,
            Rounding::same(6.0),
            Color32::from_rgb(28, 28, 36),
        );
    }

    /* Guaranteed selection sync: display exactly the currently selected device from device_list */
    let dev_title = if app.connected {
        app.active_device_name()
    } else {
        "No Device Connected"
    };
    let dev_color = if app.connected {
        COLOR_TEXT_PRIMARY
    } else {
        Color32::from_gray(150)
    };

    /* Reserve room on right for chevron icon (24px) so text never collides with it */
    let text_clip_rect = Rect::from_min_max(
        Pos2::new(dev_rect.min.x + 2.0, dev_rect.min.y),
        Pos2::new(dev_rect.max.x - 24.0, dev_rect.max.y),
    );

    ui.painter().with_clip_rect(text_clip_rect).text(
        Pos2::new(text_clip_rect.min.x, dev_rect.center().y),
        Align2::LEFT_CENTER,
        dev_title,
        FontId::proportional(14.0),
        dev_color,
    );

    let chev_rect = Rect::from_center_size(
        Pos2::new(dev_rect.max.x - 10.0, dev_rect.center().y),
        Vec2::splat(13.0),
    );
    draw_icon(
        ui.painter(),
        chev_rect,
        IconType::ChevronRight,
        if dev_hover || app.device_selector_open {
            Color32::WHITE
        } else if app.connected {
            Color32::from_gray(130)
        } else {
            Color32::from_gray(90)
        },
    );

    if dev_resp.clicked() {
        app.device_selector_open = !app.device_selector_open;
    }

    /* Device Selection Dropdown Panel (Select Device) */
    if app.device_selector_open {
        ui.add_space(2.0);
        render_device_selector_panel(ui, app);
        ui.add_space(2.0);
    }

    ui.add_space(4.0);

    /* 3. Section header: "Mouse Configuration" */
    ui.label(
        RichText::new("Mouse Configuration")
            .size(11.0)
            .strong()
            .color(Color32::from_gray(115)),
    );
    ui.add_space(3.0);

    /* 5. Navigation Items with crisp custom vector icons */
    sidebar_item(ui, "Key Change Settings", IconType::Mouse, NavItem::KeyChange, &mut app.active_nav);
    sidebar_item(ui, "Macro Settings", IconType::Macro, NavItem::MacroSettings, &mut app.active_nav);
    sidebar_item(ui, "DPI Settings", IconType::Dpi, NavItem::DpiSettings, &mut app.active_nav);
    sidebar_item(ui, "Performance Settings", IconType::Performance, NavItem::PerformanceSettings, &mut app.active_nav);
    sidebar_item(ui, "Parameter Settings", IconType::Parameters, NavItem::ParameterSettings, &mut app.active_nav);
    sidebar_item(ui, "Configuration Management", IconType::Config, NavItem::ConfigManagement, &mut app.active_nav);
    sidebar_item(ui, "Other settings", IconType::Settings, NavItem::OtherSettings, &mut app.active_nav);
}

/* Renders the bottom toolbar (Bug Feedback & Refresh) strictly pinned to the bottom of SidePanel */
pub fn render_bottom_toolbar(ui: &mut Ui, app: &mut AtkControlApp) {
    let total_w = ui.available_width();
    let bar_h = 30.0;
    let (toolbar_rect, _) = ui.allocate_exact_size(Vec2::new(total_w, bar_h), Sense::hover());

    let b1_w = (total_w * 0.52).round();
    let b2_w = (total_w * 0.38).round();
    let b1_rect = Rect::from_min_size(toolbar_rect.min, Vec2::new(b1_w, bar_h));
    let b2_rect = Rect::from_min_size(
        Pos2::new(toolbar_rect.max.x - b2_w, toolbar_rect.min.y),
        Vec2::new(b2_w, bar_h),
    );

    if draw_bottom_button(ui, b1_rect, "Bug Feedback") {
        ui.ctx().open_url(eframe::egui::OpenUrl::new_tab("https://github.com/LilRaime/open-atk"));
        app.set_status("Report issues on GitHub: https://github.com/LilRaime/open-atk", false);
    }
    if draw_bottom_button(ui, b2_rect, "Refresh") {
        app.refresh_hardware();
        app.set_status("Hardware refreshed from EEPROM", false);
    }
}

/* Backward-compatible full sidebar renderer */
#[allow(dead_code)]
pub fn render_sidebar(ui: &mut Ui, app: &mut AtkControlApp) {
    render_sidebar_content(ui, app);
    ui.add_space(8.0);
    render_bottom_toolbar(ui, app);
}

fn sidebar_item(ui: &mut Ui, label: &str, icon: IconType, item: NavItem, current: &mut NavItem) {
    let is_selected = *current == item;

    let (rect, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 34.0), Sense::click());
    let hovered = resp.hovered();

    let bg_color = if is_selected {
        Color32::from_rgb(38, 38, 48)
    } else if hovered {
        Color32::from_rgb(26, 26, 32)
    } else {
        COLOR_BG_SIDEBAR
    };

    let text_color = if is_selected {
        COLOR_TEXT_PRIMARY
    } else if hovered {
        Color32::from_gray(215)
    } else {
        COLOR_TEXT_MUTED
    };

    let icon_color = if is_selected {
        COLOR_ACCENT_ORANGE
    } else if hovered {
        Color32::from_gray(200)
    } else {
        Color32::from_gray(140)
    };

    ui.painter().rect_filled(rect, Rounding::same(7.0), bg_color);
    if is_selected {
        ui.painter().rect_stroke(
            rect,
            Rounding::same(7.0),
            Stroke::new(1.0_f32, Color32::from_rgb(60, 60, 75)),
        );
    }

    /* Draw Vector Icon */
    let icon_rect = Rect::from_center_size(
        Pos2::new(rect.min.x + 18.0, rect.center().y),
        Vec2::splat(16.0),
    );
    draw_icon(ui.painter(), icon_rect, icon, icon_color);

    /* Draw Label text */
    ui.painter().text(
        Pos2::new(rect.min.x + 36.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(13.0),
        text_color,
    );

    if resp.clicked() {
        *current = item;
    }

    ui.add_space(2.0);
}

/* Renders the official ATK brand logo matching the exact user SVG:
 * Black rounded squircle with crisp white A, T, K letterforms.
 */
#[allow(dead_code)]
fn draw_atk_sidebar_logo(
    painter: &eframe::egui::Painter,
    rect: Rect,
    texture: Option<&eframe::egui::TextureHandle>,
) {
    let c = rect.center();
    let badge_size = 46.0;
    let badge_rect = Rect::from_center_size(c, Vec2::splat(badge_size));

    if let Some(tex) = texture {
        painter.image(
            tex.id(),
            badge_rect,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
    } else {
        /* Fallback vector draw */
        let rounding = badge_size * (190.0 / 1024.0);
        painter.rect_filled(badge_rect, Rounding::same(rounding), Color32::from_rgb(0, 0, 0));
        painter.rect_stroke(
            badge_rect,
            Rounding::same(rounding),
            Stroke::new(1.0_f32, Color32::from_rgb(34, 34, 42)),
        );
        painter.text(
            c,
            Align2::CENTER_CENTER,
            "ATK",
            FontId::monospace(18.0),
            Color32::WHITE,
        );
    }
}

fn draw_bottom_button(ui: &mut Ui, rect: Rect, label: &str) -> bool {
    let resp = ui.allocate_rect(rect, Sense::click());
    let hovered = resp.hovered();
    let is_down = resp.is_pointer_button_down_on();

    let bg = if is_down {
        COLOR_ACCENT_ORANGE
    } else if hovered {
        Color32::from_rgb(48, 48, 58)
    } else {
        Color32::from_rgb(34, 34, 42)
    };

    let text_color = if is_down {
        Color32::BLACK
    } else if hovered {
        COLOR_TEXT_PRIMARY
    } else {
        Color32::from_gray(200)
    };

    ui.painter().rect_filled(rect, Rounding::same(7.0), bg);
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        FontId::proportional(12.0),
        text_color,
    );

    resp.clicked()
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{Align, Layout};

    #[test]
    fn test_sidebar_alignment() {
        let ctx = eframe::egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            eframe::egui::SidePanel::left("left_sidebar")
                .exact_width(250.0)
                .frame(
                    eframe::egui::Frame::none()
                        .inner_margin(eframe::egui::Margin::symmetric(14.0, 10.0)),
                )
                .show(ctx, |ui| {
                    let (item_rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 34.0), Sense::click());
                    let mut b1_min_x = 0.0;
                    let mut b2_max_x = 0.0;
                    ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
                        let total_w = ui.available_width();
                        let bar_h = 30.0;
                        let (toolbar_rect, _) = ui.allocate_exact_size(Vec2::new(total_w, bar_h), Sense::hover());
                        let b1_w = (total_w * 0.52).round();
                        let b2_w = (total_w * 0.38).round();
                        let b1_rect = Rect::from_min_size(toolbar_rect.min, Vec2::new(b1_w, bar_h));
                        let b2_rect = Rect::from_min_size(Pos2::new(toolbar_rect.max.x - b2_w, toolbar_rect.min.y), Vec2::new(b2_w, bar_h));
                        b1_min_x = b1_rect.min.x;
                        b2_max_x = b2_rect.max.x;
                    });
                    assert_eq!(item_rect.min.x, b1_min_x, "Bug Feedback left edge must match sidebar_item left edge");
                    assert_eq!(item_rect.max.x, b2_max_x, "Refresh right edge must match sidebar_item right edge");
                });
        });
    }
}
