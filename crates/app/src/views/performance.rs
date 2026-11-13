/* Performance Settings View */

use crate::state::AtkControlApp;
use crate::theme::{COLOR_ACCENT_BLUE, COLOR_TEXT_MUTED, COLOR_TEXT_PRIMARY};
use eframe::egui::{Color32, RichText, Rounding, Sense, Stroke, Ui, Vec2};

pub fn render_performance_view(ui: &mut Ui, app: &mut AtkControlApp) {
    ui.label(RichText::new("Performance Settings").size(22.0).strong().color(COLOR_TEXT_PRIMARY));
    ui.label(RichText::new("Receiver polling frequency and communication link speed").size(13.5).color(COLOR_TEXT_MUTED));
    ui.add_space(18.0);

    card_container(ui, |ui| {
        ui.label(RichText::new("Polling Rate").size(16.5).strong().color(COLOR_TEXT_PRIMARY));
        ui.label(RichText::new("Higher polling rate reduces latency. 1000 Hz is standard competitive eSports frequency.").size(13.0).color(COLOR_TEXT_MUTED));
        ui.add_space(14.0);

        let current_hz = app.mouse.polling_rate;
        let mut new_hz = None;

        ui.horizontal(|ui| {
            for &hz in &[125, 250, 500, 1000] {
                let is_current = current_hz == hz;

                let btn_bg = if is_current { Color32::from_rgb(32, 48, 66) } else { Color32::from_rgb(34, 36, 44) };

                let frame = eframe::egui::Frame::none()
                    .fill(btn_bg)
                    .stroke(Stroke::new(1.2_f32, if is_current { COLOR_ACCENT_BLUE } else { Color32::from_rgb(50, 52, 62) }))
                    .rounding(Rounding::same(7.0))
                    .inner_margin(eframe::egui::Margin::symmetric(20.0, 12.0));

                let resp = frame.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if is_current {
                            let (dot_rect, _) = ui.allocate_exact_size(Vec2::splat(8.0), Sense::hover());
                            ui.painter().circle_filled(dot_rect.center(), 3.0, COLOR_ACCENT_BLUE);
                        }
                        let text_color = if is_current { COLOR_ACCENT_BLUE } else { Color32::from_gray(190) };
                        ui.label(RichText::new(format!("{} Hz", hz)).size(14.0).strong().color(text_color));
                    });
                });

                if resp.response.interact(Sense::click()).clicked() {
                    new_hz = Some(hz);
                }
                ui.add_space(10.0);
            }
        });

        if let Some(hz) = new_hz {
            app.apply_polling(hz);
        }
    });
}

fn card_container<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
    eframe::egui::Frame::none()
        .fill(Color32::from_rgb(26, 26, 32))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(42, 42, 50)))
        .rounding(Rounding::same(9.0))
        .inner_margin(eframe::egui::Margin::same(18.0))
        .show(ui, add_contents)
        .inner
}
