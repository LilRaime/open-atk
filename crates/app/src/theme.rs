/* Custom Dark Theme and styling for ATK Control
 * Matches the dark graphite style of the official ATK Hub with scaled typography.
 */

use eframe::egui::{Color32, Context, Rounding, Stroke, TextStyle, Vec2, Visuals};

pub const COLOR_BG_SIDEBAR: Color32 = Color32::from_rgb(14, 14, 18);
pub const COLOR_BG_CANVAS: Color32 = Color32::from_rgb(20, 20, 24);
pub const COLOR_CARD_BG: Color32 = Color32::from_rgb(28, 28, 34);
pub const COLOR_ACCENT_ORANGE: Color32 = Color32::from_rgb(255, 160, 40);
pub const COLOR_ACCENT_BLUE: Color32 = Color32::from_rgb(0, 200, 255);
pub const COLOR_TEXT_PRIMARY: Color32 = Color32::WHITE;
pub const COLOR_TEXT_MUTED: Color32 = Color32::from_rgb(155, 155, 168);

pub fn setup_custom_theme(ctx: &Context) {
    let mut visuals = Visuals::dark();

    visuals.panel_fill = COLOR_BG_SIDEBAR;
    visuals.window_fill = COLOR_BG_CANVAS;

    visuals.widgets.noninteractive.bg_fill = COLOR_CARD_BG;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, Color32::from_gray(190));

    visuals.widgets.inactive.bg_fill = Color32::from_rgb(34, 34, 42);
    visuals.widgets.inactive.rounding = Rounding::same(7.0);

    visuals.widgets.hovered.bg_fill = Color32::from_rgb(48, 48, 58);
    visuals.widgets.hovered.rounding = Rounding::same(7.0);

    visuals.widgets.active.bg_fill = COLOR_ACCENT_ORANGE;
    visuals.widgets.active.rounding = Rounding::same(7.0);

    ctx.set_visuals(visuals);

    /* Make all text elements and interactive widgets larger and spacious */
    ctx.style_mut(|style| {
        style.spacing.item_spacing = Vec2::new(10.0, 10.0);
        style.spacing.button_padding = Vec2::new(14.0, 8.0);
        style.spacing.interact_size = Vec2::new(42.0, 26.0);
        style.spacing.slider_width = 280.0;

        /* Ultra-sleek slim scrollbar (width 3px, floating overlay) */
        style.spacing.scroll.floating = true;
        style.spacing.scroll.bar_width = 3.0;
        style.spacing.scroll.floating_width = 3.0;
        style.spacing.scroll.floating_allocated_width = 0.0;
        style.spacing.scroll.bar_inner_margin = 0.0;
        style.spacing.scroll.bar_outer_margin = 0.0;
        style.spacing.scroll.handle_min_length = 24.0;
        style.spacing.scroll.dormant_background_opacity = 0.0;
        style.spacing.scroll.active_background_opacity = 0.0;
        style.spacing.scroll.interact_background_opacity = 0.0;
        style.spacing.scroll.dormant_handle_opacity = 0.0;
        style.spacing.scroll.active_handle_opacity = 0.5;
        style.spacing.scroll.interact_handle_opacity = 0.8;

        /* Custom font scale */
        style.text_styles.insert(TextStyle::Heading, eframe::egui::FontId::proportional(22.0));
        style.text_styles.insert(TextStyle::Body, eframe::egui::FontId::proportional(14.0));
        style.text_styles.insert(TextStyle::Button, eframe::egui::FontId::proportional(13.5));
        style.text_styles.insert(TextStyle::Small, eframe::egui::FontId::proportional(11.5));
    });

    /* Global UI scale factor (+15% larger for better ergonomics and high-DPI displays) */
    ctx.set_zoom_factor(1.15);
}
