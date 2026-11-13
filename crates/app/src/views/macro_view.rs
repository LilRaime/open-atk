/* Macro Settings View & Visual Sequence Editor */

use crate::icons::{draw_icon, IconType};
use crate::state::{AtkControlApp, MacroItem, MacroStep, MacroTriggerMode};
use crate::theme::{COLOR_ACCENT_ORANGE, COLOR_CARD_BG, COLOR_TEXT_MUTED, COLOR_TEXT_PRIMARY};
use eframe::egui::{
    Align, Align2, Color32, FontId, Layout, Pos2, Rect, RichText, Rounding, Sense, Stroke, Ui,
    Vec2,
};

pub fn render_macro_view(ui: &mut Ui, app: &mut AtkControlApp) {
    /* 1. Handle keyboard key recording if active */
    if app.recording_key_mode {
        let mut recorded_key = None;
        let mut cancelled = false;

        ui.input(|i| {
            for event in &i.events {
                match event {
                    eframe::egui::Event::Key { key, pressed: true, .. } => {
                        if *key == eframe::egui::Key::Escape {
                            cancelled = true;
                        } else {
                            recorded_key = Some(format_egui_key(*key));
                        }
                        break;
                    }
                    eframe::egui::Event::Text(t) => {
                        let text = t.trim();
                        if !text.is_empty() {
                            recorded_key = Some(text.to_uppercase());
                            break;
                        }
                    }
                    _ => {}
                }
            }
        });

        if cancelled {
            app.recording_key_mode = false;
            app.set_status("Key recording cancelled", false);
        } else if let Some(key_name) = recorded_key {
            if let Some(sel_idx) = app.selected_macro_idx {
                if let Some(m) = app.macros.get_mut(sel_idx) {
                    m.steps.push(MacroStep::KeyDown(key_name.clone()));
                    m.steps.push(MacroStep::DelayMs(20));
                    m.steps.push(MacroStep::KeyUp(key_name.clone()));
                }
            }
            app.recording_key_mode = false;
            app.set_status(format!("Recorded Key: {}", key_name), false);
        }
    }

    ui.label(RichText::new("Macro Settings").size(22.0).strong().color(COLOR_TEXT_PRIMARY));
    ui.label(RichText::new("Record, edit, and bind custom keyboard and mouse sequences").size(13.5).color(COLOR_TEXT_MUTED));
    ui.add_space(18.0);

    ui.horizontal(|ui| {
        /* Left Column: Macro List */
        ui.vertical(|ui| {
            ui.set_width(260.0);

            ui.horizontal(|ui| {
                ui.label(RichText::new("Macros").size(16.0).strong().color(COLOR_TEXT_PRIMARY));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button(RichText::new("+ New Macro").size(12.5).strong()).clicked() {
                        let new_id = format!("macro_{}", app.macros.len() + 1);
                        let new_name = format!("Macro {}", app.macros.len() + 1);
                        app.macros.push(MacroItem {
                            id: new_id,
                            name: new_name,
                            repeat_count: 1,
                            trigger_mode: MacroTriggerMode::Once,
                            steps: vec![
                                MacroStep::KeyDown("Space".to_string()),
                                MacroStep::DelayMs(20),
                                MacroStep::KeyUp("Space".to_string()),
                            ],
                        });
                        app.selected_macro_idx = Some(app.macros.len() - 1);
                        app.set_status("Created new macro", false);
                    }
                });
            });

            ui.add_space(8.0);

            let mut delete_idx = None;

            for (idx, m) in app.macros.iter().enumerate() {
                let is_selected = app.selected_macro_idx == Some(idx);
                let (rect, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 44.0), Sense::click());
                let hovered = resp.hovered();

                let bg = if is_selected {
                    Color32::from_rgb(38, 44, 56)
                } else if hovered {
                    Color32::from_rgb(30, 30, 36)
                } else {
                    COLOR_CARD_BG
                };

                ui.painter().rect_filled(rect, Rounding::same(7.0), bg);
                if is_selected {
                    ui.painter().rect_stroke(rect, Rounding::same(7.0), Stroke::new(1.2_f32, COLOR_ACCENT_ORANGE));
                }

                ui.painter().text(
                    eframe::egui::Pos2::new(rect.min.x + 12.0, rect.min.y + 14.0),
                    eframe::egui::Align2::LEFT_CENTER,
                    &m.name,
                    eframe::egui::FontId::proportional(13.0),
                    if is_selected { COLOR_TEXT_PRIMARY } else { Color32::from_gray(190) },
                );

                ui.painter().text(
                    eframe::egui::Pos2::new(rect.min.x + 12.0, rect.min.y + 30.0),
                    eframe::egui::Align2::LEFT_CENTER,
                    format!("{} steps", m.steps.len()),
                    eframe::egui::FontId::proportional(11.0),
                    COLOR_TEXT_MUTED,
                );

                /* Delete button on the right */
                let del_rect = Rect::from_center_size(
                    eframe::egui::Pos2::new(rect.max.x - 16.0, rect.center().y),
                    Vec2::splat(18.0),
                );
                let del_resp = ui.allocate_rect(del_rect, Sense::click());
                if del_resp.hovered() {
                    ui.painter().circle_filled(del_rect.center(), 9.0, Color32::from_rgb(180, 40, 40));
                }
                ui.painter().text(
                    del_rect.center(),
                    eframe::egui::Align2::CENTER_CENTER,
                    "X",
                    eframe::egui::FontId::proportional(11.0),
                    Color32::WHITE,
                );

                if del_resp.clicked() {
                    delete_idx = Some(idx);
                } else if resp.clicked() {
                    app.selected_macro_idx = Some(idx);
                }

                ui.add_space(4.0);
            }

            if let Some(idx) = delete_idx {
                if idx < app.macros.len() {
                    app.macros.remove(idx);
                    app.selected_macro_idx = if app.macros.is_empty() {
                        None
                    } else {
                        Some(0.min(app.macros.len() - 1))
                    };
                    app.set_status("Macro deleted", false);
                }
            }

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button(RichText::new("Export JSON").size(12.0)).clicked() {
                    app.export_macros_to_disk();
                }
                if ui.button(RichText::new("Import JSON").size(12.0)).clicked() {
                    app.import_macros_from_disk();
                }
            });
        });

        ui.add_space(16.0);

        /* Right Column: Macro Sequence Editor */
        ui.vertical(|ui| {
            if let Some(sel_idx) = app.selected_macro_idx {
                if let Some(m) = app.macros.get_mut(sel_idx) {
                    card_container(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Macro Name:").size(14.0).strong().color(COLOR_TEXT_PRIMARY));
                            ui.text_edit_singleline(&mut m.name);
                        });

                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Trigger Mode:").size(13.5).color(COLOR_TEXT_MUTED));
                            ui.selectable_value(&mut m.trigger_mode, MacroTriggerMode::Once, "Single Run");
                            ui.selectable_value(&mut m.trigger_mode, MacroTriggerMode::HoldToRepeat, "Hold to Repeat");
                            ui.selectable_value(&mut m.trigger_mode, MacroTriggerMode::Toggle, "Toggle Loop");
                        });

                        if m.trigger_mode == MacroTriggerMode::Once {
                            ui.add_space(6.0);
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Repeat Count:").size(13.5).color(COLOR_TEXT_MUTED));
                                ui.add(eframe::egui::Slider::new(&mut m.repeat_count, 1..=50));
                            });
                        }

                        ui.add_space(14.0);
                        ui.label(RichText::new("Sequence Steps:").size(14.0).strong().color(COLOR_TEXT_PRIMARY));
                        ui.add_space(6.0);

                        let mut step_to_remove = None;
                        let mut step_to_move_up = None;
                        let mut step_to_move_down = None;

                        if m.steps.is_empty() {
                            ui.label(RichText::new("Sequence is empty. Record or add actions below.").size(12.5).italics().color(COLOR_TEXT_MUTED));
                        }

                        for (step_i, step) in m.steps.iter().enumerate() {
                            ui.horizontal(|ui| {
                                let (label, icon_col) = match step {
                                    MacroStep::MouseDown(b) => (format!("Mouse Down [{}]", b), Color32::from_rgb(0, 200, 255)),
                                    MacroStep::MouseUp(b) => (format!("Mouse Up [{}]", b), Color32::from_rgb(0, 160, 220)),
                                    MacroStep::KeyDown(k) => (format!("Key Down [{}]", k), COLOR_ACCENT_ORANGE),
                                    MacroStep::KeyUp(k) => (format!("Key Up [{}]", k), Color32::from_rgb(220, 130, 30)),
                                    MacroStep::KeyPress(k) => (format!("Key Press [{}]", k), Color32::from_rgb(255, 190, 50)),
                                    MacroStep::DelayMs(ms) => (format!("Wait [{} ms]", ms), Color32::from_rgb(140, 220, 100)),
                                };

                                let frame = eframe::egui::Frame::none()
                                    .fill(Color32::from_rgb(34, 34, 42))
                                    .rounding(Rounding::same(5.0))
                                    .inner_margin(eframe::egui::Margin::symmetric(10.0, 5.0));

                                frame.show(ui, |ui| {
                                    ui.label(RichText::new(format!("{}. ", step_i + 1)).size(12.0).color(Color32::from_gray(120)));
                                    ui.label(RichText::new(label).size(12.5).strong().color(icon_col));
                                });

                                if step_i > 0 && ui.small_button("^").clicked() {
                                    step_to_move_up = Some(step_i);
                                }
                                if step_i + 1 < m.steps.len() && ui.small_button("v").clicked() {
                                    step_to_move_down = Some(step_i);
                                }
                                if ui.small_button("X").clicked() {
                                    step_to_remove = Some(step_i);
                                }
                            });
                            ui.add_space(3.0);
                        }

                        if let Some(i) = step_to_remove {
                            if i < m.steps.len() {
                                m.steps.remove(i);
                            }
                        }
                        if let Some(i) = step_to_move_up {
                            if i > 0 && i < m.steps.len() {
                                m.steps.swap(i, i - 1);
                            }
                        }
                        if let Some(i) = step_to_move_down {
                            if i + 1 < m.steps.len() {
                                m.steps.swap(i, i + 1);
                            }
                        }

                        ui.add_space(14.0);

                        /* Key Recording Section */
                        ui.label(RichText::new("Record Keyboard Key:").size(13.0).strong().color(COLOR_TEXT_PRIMARY));
                        ui.add_space(4.0);

                        ui.horizontal(|ui| {
                            if app.recording_key_mode {
                                let rec_frame = eframe::egui::Frame::none()
                                    .fill(Color32::from_rgb(80, 40, 20))
                                    .stroke(Stroke::new(1.5_f32, COLOR_ACCENT_ORANGE))
                                    .rounding(Rounding::same(6.0))
                                    .inner_margin(eframe::egui::Margin::symmetric(14.0, 8.0));

                                rec_frame.show(ui, |ui| {
                                    ui.label(
                                        RichText::new(">> PRESS ANY KEY ON YOUR KEYBOARD NOW (ESC to cancel) <<")
                                            .size(13.0)
                                            .strong()
                                            .color(COLOR_ACCENT_ORANGE),
                                    );
                                });
                            } else {
                                /* Custom button with vector Record icon & orange text */
                                let btn_size = Vec2::new(178.0, 32.0);
                                let (rect, resp) = ui.allocate_exact_size(btn_size, Sense::click());
                                let is_hover = resp.hovered();
                                let is_down = resp.is_pointer_button_down_on();

                                let bg_color = if is_down {
                                    Color32::from_rgb(60, 35, 15)
                                } else if is_hover {
                                    Color32::from_rgb(46, 46, 56)
                                } else {
                                    Color32::from_rgb(34, 34, 42)
                                };
                                ui.painter().rect_filled(rect, Rounding::same(7.0), bg_color);
                                ui.painter().rect_stroke(
                                    rect,
                                    Rounding::same(7.0),
                                    Stroke::new(
                                        1.0_f32,
                                        if is_hover {
                                            COLOR_ACCENT_ORANGE
                                        } else {
                                            Color32::from_rgb(65, 65, 78)
                                        },
                                    ),
                                );

                                /* Vector Record icon: glowing ring + filled center dot */
                                let icon_rect = Rect::from_center_size(
                                    Pos2::new(rect.min.x + 18.0, rect.center().y),
                                    Vec2::splat(14.0),
                                );
                                draw_icon(ui.painter(), icon_rect, IconType::Record, COLOR_ACCENT_ORANGE);

                                /* Button label */
                                ui.painter().text(
                                    Pos2::new(rect.min.x + 32.0, rect.center().y),
                                    Align2::LEFT_CENTER,
                                    "Record Physical Key",
                                    FontId::proportional(13.0),
                                    COLOR_ACCENT_ORANGE,
                                );

                                if resp.clicked() {
                                    app.recording_key_mode = true;
                                }
                                ui.label(RichText::new("Click to record any keyboard key automatically").size(12.0).color(COLOR_TEXT_MUTED));
                            }
                        });

                        ui.add_space(12.0);

                        /* Manual Key Input Row */
                        ui.label(RichText::new("Type Any Custom Key:").size(13.0).strong().color(COLOR_TEXT_PRIMARY));
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.add(eframe::egui::TextEdit::singleline(&mut app.new_macro_key_input).hint_text("e.g. G, F1, Space, Enter, Ctrl...").desired_width(120.0));

                            let key_to_add = app.new_macro_key_input.trim().to_string();
                            if !key_to_add.is_empty() {
                                if ui.button("+ Add Key Press").clicked() {
                                    m.steps.push(MacroStep::KeyDown(key_to_add.clone()));
                                    m.steps.push(MacroStep::DelayMs(20));
                                    m.steps.push(MacroStep::KeyUp(key_to_add.clone()));
                                }
                                if ui.button("+ Key Down").clicked() {
                                    m.steps.push(MacroStep::KeyDown(key_to_add.clone()));
                                }
                                if ui.button("+ Key Up").clicked() {
                                    m.steps.push(MacroStep::KeyUp(key_to_add.clone()));
                                }
                            }
                        });

                        ui.add_space(8.0);

                        /* Quick Gaming Keys Chips */
                        ui.label(RichText::new("Quick Gaming Keys:").size(12.0).color(COLOR_TEXT_MUTED));
                        ui.horizontal_wrapped(|ui| {
                            for &k in &["Q", "W", "E", "R", "F", "G", "C", "V", "1", "2", "3", "4", "Space", "Shift", "Ctrl", "Alt", "Tab", "Enter"] {
                                if ui.button(k).clicked() {
                                    m.steps.push(MacroStep::KeyDown(k.to_string()));
                                    m.steps.push(MacroStep::DelayMs(20));
                                    m.steps.push(MacroStep::KeyUp(k.to_string()));
                                }
                            }
                        });

                        ui.add_space(12.0);

                        /* Mouse Clicks & Delays Row */
                        ui.label(RichText::new("Mouse Actions & Delays:").size(13.0).strong().color(COLOR_TEXT_PRIMARY));
                        ui.add_space(4.0);
                        ui.horizontal_wrapped(|ui| {
                            if ui.button("+ Left Click").clicked() {
                                m.steps.push(MacroStep::MouseDown("Left".to_string()));
                                m.steps.push(MacroStep::DelayMs(20));
                                m.steps.push(MacroStep::MouseUp("Left".to_string()));
                            }
                            if ui.button("+ Right Click").clicked() {
                                m.steps.push(MacroStep::MouseDown("Right".to_string()));
                                m.steps.push(MacroStep::DelayMs(20));
                                m.steps.push(MacroStep::MouseUp("Right".to_string()));
                            }
                            if ui.button("+ Middle Click").clicked() {
                                m.steps.push(MacroStep::MouseDown("Middle".to_string()));
                                m.steps.push(MacroStep::DelayMs(20));
                                m.steps.push(MacroStep::MouseUp("Middle".to_string()));
                            }
                            if ui.button("+ 15ms").clicked() {
                                m.steps.push(MacroStep::DelayMs(15));
                            }
                            if ui.button("+ 30ms").clicked() {
                                m.steps.push(MacroStep::DelayMs(30));
                            }
                            if ui.button("+ 50ms").clicked() {
                                m.steps.push(MacroStep::DelayMs(50));
                            }
                            if ui.button("+ 100ms").clicked() {
                                m.steps.push(MacroStep::DelayMs(100));
                            }
                            if ui.button("Clear Steps").clicked() {
                                m.steps.clear();
                            }
                        });
                    });
                }
            } else {
                card_container(ui, |ui| {
                    ui.label(RichText::new("No macro selected. Create or select a macro from the left list.").size(13.5).color(COLOR_TEXT_MUTED));
                });
            }
        });
    });
}

fn format_egui_key(key: eframe::egui::Key) -> String {
    use eframe::egui::Key;
    match key {
        Key::Space => "Space".to_string(),
        Key::Enter => "Enter".to_string(),
        Key::Tab => "Tab".to_string(),
        Key::Backspace => "Backspace".to_string(),
        Key::Delete => "Delete".to_string(),
        Key::Insert => "Insert".to_string(),
        Key::Home => "Home".to_string(),
        Key::End => "End".to_string(),
        Key::PageUp => "PageUp".to_string(),
        Key::PageDown => "PageDown".to_string(),
        Key::ArrowUp => "Up".to_string(),
        Key::ArrowDown => "Down".to_string(),
        Key::ArrowLeft => "Left".to_string(),
        Key::ArrowRight => "Right".to_string(),
        Key::Num0 => "0".to_string(),
        Key::Num1 => "1".to_string(),
        Key::Num2 => "2".to_string(),
        Key::Num3 => "3".to_string(),
        Key::Num4 => "4".to_string(),
        Key::Num5 => "5".to_string(),
        Key::Num6 => "6".to_string(),
        Key::Num7 => "7".to_string(),
        Key::Num8 => "8".to_string(),
        Key::Num9 => "9".to_string(),
        Key::A => "A".to_string(),
        Key::B => "B".to_string(),
        Key::C => "C".to_string(),
        Key::D => "D".to_string(),
        Key::E => "E".to_string(),
        Key::F => "F".to_string(),
        Key::G => "G".to_string(),
        Key::H => "H".to_string(),
        Key::I => "I".to_string(),
        Key::J => "J".to_string(),
        Key::K => "K".to_string(),
        Key::L => "L".to_string(),
        Key::M => "M".to_string(),
        Key::N => "N".to_string(),
        Key::O => "O".to_string(),
        Key::P => "P".to_string(),
        Key::Q => "Q".to_string(),
        Key::R => "R".to_string(),
        Key::S => "S".to_string(),
        Key::T => "T".to_string(),
        Key::U => "U".to_string(),
        Key::V => "V".to_string(),
        Key::W => "W".to_string(),
        Key::X => "X".to_string(),
        Key::Y => "Y".to_string(),
        Key::Z => "Z".to_string(),
        Key::F1 => "F1".to_string(),
        Key::F2 => "F2".to_string(),
        Key::F3 => "F3".to_string(),
        Key::F4 => "F4".to_string(),
        Key::F5 => "F5".to_string(),
        Key::F6 => "F6".to_string(),
        Key::F7 => "F7".to_string(),
        Key::F8 => "F8".to_string(),
        Key::F9 => "F9".to_string(),
        Key::F10 => "F10".to_string(),
        Key::F11 => "F11".to_string(),
        Key::F12 => "F12".to_string(),
        _ => format!("{:?}", key),
    }
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
