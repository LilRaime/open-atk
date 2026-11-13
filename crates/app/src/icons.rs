/*
 * Vector icons for ATK Control
 * Rendered with pure painter geometry so they never fail or depend on system fonts.
 */

#![allow(dead_code)]

use eframe::egui::{Color32, Painter, Pos2, Rect, Rounding, Stroke, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconType {
    Mouse,
    Macro,
    Dpi,
    Performance,
    Parameters,
    Config,
    Settings,
    Battery { pct: u8, charging: bool },
    Wireless,
    Cable,
    Bluetooth,
    Bug,
    Refresh,
    ChevronRight,
    Close,
    Warning,
    AtkLogo,
    Record,
}

pub fn draw_icon(painter: &Painter, rect: Rect, icon: IconType, color: Color32) {
    let c = rect.center();
    let size = rect.size();
    let min_dim = size.x.min(size.y);
    let s = min_dim * 0.5; /* half extent */

    match icon {
        IconType::Mouse => {
            /* Mouse body silhouette */
            let body = Rect::from_center_size(c, Vec2::new(s * 1.1, s * 1.6));
            painter.rect_stroke(body, Rounding::same(s * 0.5), Stroke::new(1.3_f32, color));
            /* Center button split line */
            painter.line_segment(
                [Pos2::new(c.x, c.y - s * 0.8), Pos2::new(c.x, c.y - s * 0.1)],
                Stroke::new(1.2_f32, color),
            );
            /* Scroll wheel dot */
            painter.circle_filled(Pos2::new(c.x, c.y - s * 0.4), 1.4, color);
        }
        IconType::Macro => {
            /* Keyboard/script `< >` icon */
            let stroke = Stroke::new(1.4_f32, color);
            /* Left bracket `<` */
            painter.line_segment([Pos2::new(c.x - s * 0.3, c.y - s * 0.6), Pos2::new(c.x - s * 0.7, c.y)], stroke);
            painter.line_segment([Pos2::new(c.x - s * 0.7, c.y), Pos2::new(c.x - s * 0.3, c.y + s * 0.6)], stroke);
            /* Right bracket `>` */
            painter.line_segment([Pos2::new(c.x + s * 0.3, c.y - s * 0.6), Pos2::new(c.x + s * 0.7, c.y)], stroke);
            painter.line_segment([Pos2::new(c.x + s * 0.7, c.y), Pos2::new(c.x + s * 0.3, c.y + s * 0.6)], stroke);
        }
        IconType::Dpi => {
            /* 2x2 grid of dots/squares */
            let r = s * 0.28;
            let offset = s * 0.45;
            for dx in &[-offset, offset] {
                for dy in &[-offset, offset] {
                    let dot_c = Pos2::new(c.x + dx, c.y + dy);
                    painter.rect_filled(
                        Rect::from_center_size(dot_c, Vec2::splat(r * 2.0)),
                        Rounding::same(1.5),
                        color,
                    );
                }
            }
        }
        IconType::Performance => {
            /* Speedometer gauge arc & needle */
            let stroke = Stroke::new(1.4_f32, color);
            let radius = s * 0.8;
            let segments = 8;
            for i in 0..segments {
                let a1 = std::f32::consts::PI * (0.85 + (i as f32 / segments as f32) * 1.3);
                let a2 = std::f32::consts::PI * (0.85 + ((i + 1) as f32 / segments as f32) * 1.3);
                let p1 = Pos2::new(c.x + radius * a1.cos(), c.y + radius * a1.sin() + s * 0.2);
                let p2 = Pos2::new(c.x + radius * a2.cos(), c.y + radius * a2.sin() + s * 0.2);
                painter.line_segment([p1, p2], stroke);
            }
            /* Pointer needle */
            let needle_end = Pos2::new(c.x + radius * 0.7 * 0.5, c.y - radius * 0.7 * 0.8 + s * 0.2);
            painter.line_segment([Pos2::new(c.x, c.y + s * 0.2), needle_end], Stroke::new(1.5_f32, color));
            painter.circle_filled(Pos2::new(c.x, c.y + s * 0.2), 1.8, color);
        }
        IconType::Parameters => {
            /* 3 horizontal slider bars with knobs */
            let stroke = Stroke::new(1.3_f32, color);
            let dy = s * 0.55;
            for (idx, y_off) in [-dy, 0.0, dy].iter().enumerate() {
                let y = c.y + y_off;
                painter.line_segment([Pos2::new(c.x - s * 0.8, y), Pos2::new(c.x + s * 0.8, y)], stroke);
                /* Knob positions */
                let knob_x = match idx {
                    0 => c.x - s * 0.25,
                    1 => c.x + s * 0.35,
                    _ => c.x - s * 0.05,
                };
                painter.circle_filled(Pos2::new(knob_x, y), 2.2, color);
            }
        }
        IconType::Config => {
            /* 4 layout windows / cards */
            let r = s * 0.36;
            let off = s * 0.44;
            let stroke = Stroke::new(1.2_f32, color);
            for dx in &[-off, off] {
                for dy in &[-off, off] {
                    let card = Rect::from_center_size(Pos2::new(c.x + dx, c.y + dy), Vec2::splat(r * 2.0));
                    painter.rect_stroke(card, Rounding::same(1.5), stroke);
                }
            }
        }
        IconType::Settings => {
            /* Gear / Cog with 6 teeth */
            let outer_r = s * 0.85;
            let inner_r = s * 0.65;
            let hole_r = s * 0.3;
            let teeth = 6;
            let stroke = Stroke::new(1.3_f32, color);

            for i in 0..teeth {
                let angle = (i as f32) * (std::f32::consts::TAU / teeth as f32);
                let p_in = Pos2::new(c.x + inner_r * angle.cos(), c.y + inner_r * angle.sin());
                let p_out = Pos2::new(c.x + outer_r * angle.cos(), c.y + outer_r * angle.sin());
                painter.line_segment([p_in, p_out], Stroke::new(2.4_f32, color));
            }
            painter.circle_stroke(c, inner_r, stroke);
            painter.circle_stroke(c, hole_r, stroke);
        }
        IconType::Battery { pct, charging } => {
            let bat_color = if charging {
                Color32::from_rgb(0, 220, 120) /* Bright green when charging */
            } else if pct < 20 {
                Color32::from_rgb(235, 75, 75) /* Red when low battery (< 20%) */
            } else {
                color
            };

            /* Battery body rectangle (perfectly centered at c.y) */
            let body_w = (rect.width() - 4.0).clamp(16.0, 24.0);
            let body_h = (rect.height() - 4.0).clamp(10.0, 14.0);
            let body_rect = Rect::from_center_size(Pos2::new(c.x - 1.2, c.y), Vec2::new(body_w, body_h));
            painter.rect_stroke(body_rect, Rounding::same(2.5), Stroke::new(1.3_f32, bat_color));

            /* Positive terminal nub */
            let nub = Rect::from_min_size(
                Pos2::new(body_rect.max.x + 1.2, c.y - body_h * 0.25),
                Vec2::new(2.2, body_h * 0.5),
            );
            painter.rect_filled(nub, Rounding::same(0.8), bat_color);

            /* Fill level indicator */
            let fill_pct = (pct as f32 / 100.0).clamp(0.06, 1.0);
            let inner_w = (body_w - 3.4) * fill_pct;
            let fill_rect = Rect::from_min_size(
                Pos2::new(body_rect.min.x + 1.7, body_rect.min.y + 1.7),
                Vec2::new(inner_w, body_h - 3.4),
            );
            painter.rect_filled(fill_rect, Rounding::same(1.2), bat_color);
        }
        IconType::Cable => {
            let stroke = Stroke::new(1.35_f32, color);
            /* Small connector plug */
            let plug_w = s * 0.42;
            let plug_h = s * 0.65;
            let plug = Rect::from_center_size(
                Pos2::new(c.x - s * 0.65, c.y),
                Vec2::new(plug_w, plug_h),
            );
            painter.rect_stroke(plug, Rounding::same(1.5), stroke);
            /* Plug pin connector */
            painter.line_segment(
                [Pos2::new(plug.max.x, c.y), Pos2::new(plug.max.x + 2.2, c.y)],
                stroke,
            );
            /* Smooth S-curve cable wire */
            let start_x = plug.max.x + 2.2;
            let end_x = c.x + s * 0.82;
            let steps = 16;
            for i in 0..steps {
                let f1 = i as f32 / steps as f32;
                let f2 = (i + 1) as f32 / steps as f32;
                let x1 = start_x + (end_x - start_x) * f1;
                let x2 = start_x + (end_x - start_x) * f2;
                let y1 = c.y + (f1 * std::f32::consts::TAU).sin() * (s * 0.42);
                let y2 = c.y + (f2 * std::f32::consts::TAU).sin() * (s * 0.42);
                painter.line_segment([Pos2::new(x1, y1), Pos2::new(x2, y2)], stroke);
            }
        }
        IconType::Bluetooth => {
            let stroke = Stroke::new(1.4_f32, color);
            /* Vertical spine */
            painter.line_segment(
                [Pos2::new(c.x, c.y - s * 0.78), Pos2::new(c.x, c.y + s * 0.78)],
                stroke,
            );
            /* Upper diagonal cross */
            painter.line_segment(
                [Pos2::new(c.x - s * 0.45, c.y - s * 0.38), Pos2::new(c.x + s * 0.45, c.y + s * 0.38)],
                stroke,
            );
            /* Lower diagonal cross */
            painter.line_segment(
                [Pos2::new(c.x - s * 0.45, c.y + s * 0.38), Pos2::new(c.x + s * 0.45, c.y - s * 0.38)],
                stroke,
            );
            /* Upper triangle loop */
            painter.line_segment(
                [Pos2::new(c.x + s * 0.45, c.y - s * 0.38), Pos2::new(c.x, c.y)],
                stroke,
            );
            /* Lower triangle loop */
            painter.line_segment(
                [Pos2::new(c.x + s * 0.45, c.y + s * 0.38), Pos2::new(c.x, c.y)],
                stroke,
            );
        }
        IconType::Wireless => {
            /* High-precision smooth concentric signal arcs, vertically balanced */
            let stroke = Stroke::new(1.35_f32, color);
            let dot_y = c.y + 4.0;
            /* Base transceiver dot */
            painter.circle_filled(Pos2::new(c.x, dot_y), 1.5, color);

            /* Two silky-smooth circular arcs (18 segments each) */
            let arcs = [
                (4.8_f32, 1.25_f32, 1.75_f32),  /* inner arc */
                (8.2_f32, 1.25_f32, 1.75_f32),  /* outer arc */
            ];

            for (r, a_start, a_end) in arcs {
                let segments = 18;
                for i in 0..segments {
                    let a1 = std::f32::consts::PI * (a_start + (i as f32 / segments as f32) * (a_end - a_start));
                    let a2 = std::f32::consts::PI * (a_start + ((i + 1) as f32 / segments as f32) * (a_end - a_start));
                    let p1 = Pos2::new(c.x + r * a1.cos(), dot_y + r * a1.sin());
                    let p2 = Pos2::new(c.x + r * a2.cos(), dot_y + r * a2.sin());
                    painter.line_segment([p1, p2], stroke);
                }
            }
        }
        IconType::Bug => {
            /* Ladybug / Feedback icon */
            let stroke = Stroke::new(1.3_f32, color);
            painter.circle_stroke(c, s * 0.65, stroke);
            painter.line_segment([Pos2::new(c.x, c.y - s * 0.65), Pos2::new(c.x, c.y + s * 0.65)], stroke);
            /* Legs */
            painter.line_segment([Pos2::new(c.x - s * 0.85, c.y), Pos2::new(c.x - s * 0.55, c.y)], stroke);
            painter.line_segment([Pos2::new(c.x + s * 0.55, c.y), Pos2::new(c.x + s * 0.85, c.y)], stroke);
        }
        IconType::Refresh => {
            /* Circular reload arrow */
            let stroke = Stroke::new(1.3_f32, color);
            let r = s * 0.65;
            let segments = 8;
            for i in 0..segments {
                let a1 = std::f32::consts::PI * (0.2 + (i as f32 / segments as f32) * 1.6);
                let a2 = std::f32::consts::PI * (0.2 + ((i + 1) as f32 / segments as f32) * 1.6);
                let p1 = Pos2::new(c.x + r * a1.cos(), c.y + r * a1.sin());
                let p2 = Pos2::new(c.x + r * a2.cos(), c.y + r * a2.sin());
                painter.line_segment([p1, p2], stroke);
            }
            /* Arrowhead at top */
            let tip = Pos2::new(c.x + r * (0.2 * std::f32::consts::PI).cos(), c.y + r * (0.2 * std::f32::consts::PI).sin());
            painter.line_segment([tip, Pos2::new(tip.x - 3.0, tip.y - 3.0)], stroke);
            painter.line_segment([tip, Pos2::new(tip.x + 3.0, tip.y - 2.0)], stroke);
        }
        IconType::ChevronRight => {
            let stroke = Stroke::new(1.4_f32, color);
            painter.line_segment([Pos2::new(c.x - s * 0.3, c.y - s * 0.5), Pos2::new(c.x + s * 0.2, c.y)], stroke);
            painter.line_segment([Pos2::new(c.x + s * 0.2, c.y), Pos2::new(c.x - s * 0.3, c.y + s * 0.5)], stroke);
        }
        IconType::Close => {
            let stroke = Stroke::new(1.5_f32, color);
            painter.line_segment([Pos2::new(c.x - s * 0.5, c.y - s * 0.5), Pos2::new(c.x + s * 0.5, c.y + s * 0.5)], stroke);
            painter.line_segment([Pos2::new(c.x + s * 0.5, c.y - s * 0.5), Pos2::new(c.x - s * 0.5, c.y + s * 0.5)], stroke);
        }
        IconType::Warning => {
            /* Crisp warning triangle with exclamation mark */
            let p_top = Pos2::new(c.x, c.y - s * 0.85);
            let p_left = Pos2::new(c.x - s * 0.90, c.y + s * 0.80);
            let p_right = Pos2::new(c.x + s * 0.90, c.y + s * 0.80);

            let stroke = Stroke::new(1.5_f32, color);
            painter.line_segment([p_top, p_left], stroke);
            painter.line_segment([p_left, p_right], stroke);
            painter.line_segment([p_right, p_top], stroke);

            /* Exclamation mark '!' inside */
            painter.line_segment(
                [Pos2::new(c.x, c.y - s * 0.32), Pos2::new(c.x, c.y + s * 0.18)],
                Stroke::new(1.8_f32, color),
            );
            painter.circle_filled(Pos2::new(c.x, c.y + s * 0.52), 1.3, color);
        }
        IconType::AtkLogo => {
            /* Clean ATK brand logo matching reference: black rounded badge with white ATK */
            painter.rect_filled(rect, Rounding::same(rect.height() * 0.18), Color32::from_rgb(0, 0, 0));
            painter.rect_stroke(rect, Rounding::same(rect.height() * 0.18), Stroke::new(1.0_f32, Color32::from_rgb(32, 32, 40)));
            painter.text(
                c,
                eframe::egui::Align2::CENTER_CENTER,
                "ATK",
                eframe::egui::FontId::monospace(s * 1.1),
                Color32::WHITE,
            );
        }
        IconType::Record => {
            /* Crisp vector record symbol: outer ring and filled center dot */
            painter.circle_stroke(c, s * 0.75, Stroke::new(1.3_f32, color));
            painter.circle_filled(c, s * 0.45, color);
        }
    }
}

#[derive(Clone, Copy)]
pub enum ParamIcon {
    MotionSync,
    StraightLine,
    Ripple,
    LongRange,
    Debounce,
    Sleep,
    Lod,
    DpiLight,
}

/* Crisp custom vector icons for Parameter Settings cards (never relies on missing Unicode glyphs) */
pub fn draw_param_icon(ui: &mut eframe::egui::Ui, icon: ParamIcon) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(18.0, 18.0), eframe::egui::Sense::hover());
    let p = ui.painter();
    let c = rect.center();
    let col = Color32::from_rgb(210, 210, 220);
    let s = Stroke::new(1.5_f32, col);

    match icon {
        ParamIcon::MotionSync => {
            /* Crosshair / 4-way target icon */
            p.circle_stroke(c, 5.2, s);
            p.circle_filled(c, 1.4, col);
            p.line_segment([Pos2::new(c.x, c.y - 7.8), Pos2::new(c.x, c.y - 4.0)], s);
            p.line_segment([Pos2::new(c.x, c.y + 4.0), Pos2::new(c.x, c.y + 7.8)], s);
            p.line_segment([Pos2::new(c.x - 7.8, c.y), Pos2::new(c.x - 4.0, c.y)], s);
            p.line_segment([Pos2::new(c.x + 4.0, c.y), Pos2::new(c.x + 7.8, c.y)], s);
        }
        ParamIcon::StraightLine => {
            /* Set-square triangle ruler */
            let p1 = Pos2::new(c.x - 6.0, c.y + 5.5);
            let p2 = Pos2::new(c.x + 6.5, c.y + 5.5);
            let p3 = Pos2::new(c.x + 6.5, c.y - 6.0);
            p.line_segment([p1, p2], s);
            p.line_segment([p2, p3], s);
            p.line_segment([p3, p1], s);
        }
        ParamIcon::Ripple => {
            /* Smooth sine wave */
            let mut prev = None;
            for i in 0..=16 {
                let t = i as f32 / 16.0;
                let x = c.x - 7.0 + t * 14.0;
                let y = c.y - (t * std::f32::consts::TAU * 1.5).sin() * 3.2;
                let pt = Pos2::new(x, y);
                if let Some(pr) = prev {
                    p.line_segment([pr, pt], s);
                }
                prev = Some(pt);
            }
        }
        ParamIcon::LongRange => {
            /* Wireless antenna waves */
            let base = Pos2::new(c.x, c.y + 3.5);
            p.circle_filled(base, 1.8, col);
            for r in [4.8_f32, 8.0_f32] {
                let mut prev = None;
                for i in 0..=10 {
                    let ang = -std::f32::consts::FRAC_PI_4 - (i as f32 / 10.0) * std::f32::consts::FRAC_PI_2;
                    let pt = Pos2::new(base.x + ang.cos() * r, base.y + ang.sin() * r);
                    if let Some(pr) = prev {
                        p.line_segment([pr, pt], s);
                    }
                    prev = Some(pt);
                }
            }
        }
        ParamIcon::Debounce => {
            /* Stopwatch icon */
            let body_c = Pos2::new(c.x, c.y + 1.0);
            p.circle_stroke(body_c, 5.8, s);
            p.line_segment([Pos2::new(c.x - 2.2, c.y - 6.8), Pos2::new(c.x + 2.2, c.y - 6.8)], s);
            p.line_segment([Pos2::new(c.x, c.y - 6.8), Pos2::new(c.x, c.y - 4.8)], s);
            p.line_segment([body_c, Pos2::new(body_c.x, body_c.y - 3.2)], s);
            p.line_segment([body_c, Pos2::new(body_c.x + 2.6, body_c.y + 1.2)], s);
        }
        ParamIcon::Sleep => {
            /* Crescent moon icon */
            p.circle_filled(c, 5.8, col);
            p.circle_filled(Pos2::new(c.x + 2.6, c.y - 2.0), 5.0, Color32::from_rgb(20, 20, 24));
        }
        ParamIcon::Lod => {
            /* Vertical height gap indicator with downward arrow to surface */
            let surf_y = c.y + 5.8;
            p.line_segment([Pos2::new(c.x - 6.5, surf_y), Pos2::new(c.x + 6.5, surf_y)], s);
            p.line_segment([Pos2::new(c.x, c.y - 6.2), Pos2::new(c.x, surf_y - 2.0)], s);
            p.line_segment([Pos2::new(c.x - 3.2, surf_y - 5.2), Pos2::new(c.x, surf_y - 1.8)], s);
            p.line_segment([Pos2::new(c.x + 3.2, surf_y - 5.2), Pos2::new(c.x, surf_y - 1.8)], s);
            p.line_segment([Pos2::new(c.x - 4.0, c.y - 6.2), Pos2::new(c.x + 4.0, c.y - 6.2)], s);
        }
        ParamIcon::DpiLight => {
            /* Lightbulb / LED glow icon */
            let bulb_c = Pos2::new(c.x, c.y - 1.2);
            p.circle_stroke(bulb_c, 4.4, s);
            p.line_segment([Pos2::new(c.x - 2.2, c.y + 4.8), Pos2::new(c.x + 2.2, c.y + 4.8)], s);
            p.line_segment([Pos2::new(c.x - 1.4, c.y + 6.8), Pos2::new(c.x + 1.4, c.y + 6.8)], s);
        }
    }
}

/* Vector painter graphic showing a mouse hovering above surface with height gap and arrow */
pub fn draw_lod_illustration(ui: &mut eframe::egui::Ui, lod_code: u8) {
    let size = Vec2::new(130.0, 48.0);
    let (rect, _) = ui.allocate_exact_size(size, eframe::egui::Sense::hover());
    let painter = ui.painter_at(rect);

    /* Background panel */
    painter.rect_filled(rect, 6.0, Color32::from_rgb(16, 16, 20));

    let surface_y = rect.bottom() - 10.0;
    let surface_start = rect.left() + 6.0;
    let surface_end = rect.right() - 6.0;

    /* Dotted desk surface line */
    let mut x = surface_start;
    while x < surface_end {
        painter.line_segment(
            [Pos2::new(x, surface_y), Pos2::new(x + 3.0, surface_y)],
            Stroke::new(1.0_f32, Color32::from_rgb(80, 80, 90)),
        );
        x += 6.0;
    }

    /* Mouse profile outline */
    let mouse_lift = if lod_code == 2 { 14.0 } else { 8.0 };
    let mouse_base_y = surface_y - mouse_lift;
    let mouse_center_x = rect.left() + 65.0;

    /* Curved mouse profile outline */
    let stroke = Stroke::new(1.2_f32, Color32::from_rgb(170, 170, 180));
    let pts = vec![
        Pos2::new(mouse_center_x - 30.0, mouse_base_y),
        Pos2::new(mouse_center_x - 22.0, mouse_base_y - 10.0),
        Pos2::new(mouse_center_x - 6.0, mouse_base_y - 15.0),
        Pos2::new(mouse_center_x + 12.0, mouse_base_y - 13.0),
        Pos2::new(mouse_center_x + 28.0, mouse_base_y - 5.0),
        Pos2::new(mouse_center_x + 32.0, mouse_base_y),
        Pos2::new(mouse_center_x - 30.0, mouse_base_y),
    ];
    for w in pts.windows(2) {
        painter.line_segment([w[0], w[1]], stroke);
    }

    /* Mouse optical sensor lens */
    painter.line_segment(
        [Pos2::new(mouse_center_x - 4.0, mouse_base_y), Pos2::new(mouse_center_x + 4.0, mouse_base_y)],
        Stroke::new(2.0_f32, Color32::from_rgb(255, 184, 0)),
    );

    /* Vertical measurement dimension line with arrows */
    let arrow_x = mouse_center_x;
    painter.line_segment(
        [Pos2::new(arrow_x, mouse_base_y + 1.0), Pos2::new(arrow_x, surface_y - 1.0)],
        Stroke::new(1.0_f32, Color32::from_rgb(255, 184, 0)),
    );
    /* Top tick */
    painter.line_segment(
        [Pos2::new(arrow_x - 3.0, mouse_base_y + 1.0), Pos2::new(arrow_x + 3.0, mouse_base_y + 1.0)],
        Stroke::new(1.0_f32, Color32::from_rgb(255, 184, 0)),
    );
    /* Bottom tick */
    painter.line_segment(
        [Pos2::new(arrow_x - 3.0, surface_y - 1.0), Pos2::new(arrow_x + 3.0, surface_y - 1.0)],
        Stroke::new(1.0_f32, Color32::from_rgb(255, 184, 0)),
    );
}
