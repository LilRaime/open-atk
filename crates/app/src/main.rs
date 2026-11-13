/*
 * OpenATK - Native Desktop GUI Configuration Suite
 * Cross-platform native configuration app for ATK / VXE gaming mice.
 */

/* Hide black console window in release builds */
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod device_selector;
mod icons;
mod macro_player;
mod models;
mod preferences;
mod sidebar;
mod state;
mod theme;
mod tray;
mod views;
mod worker;

use state::{AtkControlApp, NavItem};
use std::time::Duration;

use eframe::egui::{self, Color32, RichText, Rounding};

/* Embedded high-res photos for all supported mouse families (White & Black variants) */
const EMBEDDED_VXE_R1_WHITE_PNG: &[u8] = include_bytes!("../assets/mice/vxe_r1_white.png");
const EMBEDDED_VXE_R1_BLACK_PNG: &[u8] = include_bytes!("../assets/mice/vxe_r1_black.png");
const EMBEDDED_VGN_F1_WHITE_PNG: &[u8] = include_bytes!("../assets/mice/vgn_f1_white.png");
const EMBEDDED_VGN_F1_BLACK_PNG: &[u8] = include_bytes!("../assets/mice/vgn_f1_black.png");
const EMBEDDED_ATK_MADR_WHITE_PNG: &[u8] = include_bytes!("../assets/mice/atk_madr_white.png");
const EMBEDDED_ATK_MADR_BLACK_PNG: &[u8] = include_bytes!("../assets/mice/atk_madr_black.png");
/* Embedded official ATK vector logo rendered in high resolution */
const EMBEDDED_ATK_LOGO_PNG: &[u8] = include_bytes!("../assets/atk_logo.png");
/* Embedded application window/taskbar icon (256x256) */
const EMBEDDED_APP_ICON_PNG: &[u8] = include_bytes!("../assets/app_icon.png");

fn load_window_icon() -> Option<egui::IconData> {
    if let Ok(img) = image::load_from_memory(EMBEDDED_APP_ICON_PNG) {
        let rgba = img.to_rgba8();
        let (width, height) = (img.width(), img.height());
        Some(egui::IconData {
            rgba: rgba.into_raw(),
            width,
            height,
        })
    } else {
        None
    }
}

fn main() -> eframe::Result<()> {
    /* Initialize Windows native dark mode early for standard Win32 menus and controls */
    tray::enable_dark_mode_for_app();

    let start_minimized = std::env::args().any(|arg| arg == "--minimized" || arg == "-m");

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1160.0, 780.0])
        .with_min_inner_size([980.0, 640.0])
        .with_title("OpenATK");

    if start_minimized {
        viewport = viewport.with_visible(false);
    }

    if let Some(icon) = load_window_icon() {
        viewport = viewport.with_icon(icon);
    }

    /* Initialize background system tray icon */
    tray::init_tray();

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "OpenATK",
        options,
        Box::new(|cc| {
            theme::setup_custom_theme(&cc.egui_ctx);
            tray::register_egui_ctx(cc.egui_ctx.clone());
            worker::register_egui_ctx(cc.egui_ctx.clone());

            /* Start background thread to flush DirectX 12 driver init heaps once the window is up */
            std::thread::spawn(|| {
                std::thread::sleep(std::time::Duration::from_millis(2000));
                tray::trim_working_set();
            });

            /* Start global low-level Windows hook for bound macro reproduction */
            macro_player::start_mouse_macro_hook();

            let mut app = AtkControlApp::new();
            if start_minimized {
                app.is_minimized_to_tray = true;
            }

            /* Load and register embedded mouse textures for all supported families */
            let load_tex = |name: &str, bytes: &[u8]| -> Option<egui::TextureHandle> {
                if let Ok(img) = image::load_from_memory(bytes) {
                    let rgba = img.to_rgba8();
                    let size = [img.width() as usize, img.height() as usize];
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_flat_samples().as_slice());
                    Some(cc.egui_ctx.load_texture(name, color_image, Default::default()))
                } else {
                    None
                }
            };

            let r1_w = load_tex("vxe_r1_white", EMBEDDED_VXE_R1_WHITE_PNG);
            let r1_b = load_tex("vxe_r1_black", EMBEDDED_VXE_R1_BLACK_PNG);
            let f1_w = load_tex("vgn_f1_white", EMBEDDED_VGN_F1_WHITE_PNG);
            let f1_b = load_tex("vgn_f1_black", EMBEDDED_VGN_F1_BLACK_PNG);
            let madr_w = load_tex("atk_madr_white", EMBEDDED_ATK_MADR_WHITE_PNG);
            let madr_b = load_tex("atk_madr_black", EMBEDDED_ATK_MADR_BLACK_PNG);

            app.vxe_r1_white_texture = r1_w.clone();
            app.vxe_r1_black_texture = r1_b.clone();
            app.mouse_white_texture = r1_w.clone();
            app.mouse_texture = r1_w;
            app.mouse_black_texture = r1_b;

            app.vgn_f1_white_texture = f1_w;
            app.vgn_f1_black_texture = f1_b;

            app.atk_madr_white_texture = madr_w;
            app.atk_madr_black_texture = madr_b;

            /* Load and register the embedded ATK logo texture */
            if let Ok(img) = image::load_from_memory(EMBEDDED_ATK_LOGO_PNG) {
                let rgba = img.to_rgba8();
                let size = [img.width() as usize, img.height() as usize];
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_flat_samples().as_slice());
                let texture = cc.egui_ctx.load_texture("atk_logo", color_image, Default::default());
                app.atk_logo_texture = Some(texture);
            }

            Ok(Box::new(app))
        }),
    )
}

impl eframe::App for AtkControlApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        /* Track foreground focus state so background worker pauses USB polling while user is in Chrome Web Hub */
        let is_window_focused = ctx.input(|i| i.focused);
        state::APP_FOCUSED.store(is_window_focused, std::sync::atomic::Ordering::Relaxed);

        /* Check if user requested window restore via system tray icon */
        if tray::should_restore_window() {
            self.is_minimized_to_tray = false;
            state::APP_FOCUSED.store(true, std::sync::atomic::Ordering::Relaxed);
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            ctx.request_repaint();
        }

        /* Check if user selected Exit from system tray context menu */
        if tray::should_exit() {
            self.preferences.save();
            tray::remove_tray();
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            std::process::exit(0);
        }

        /* Minimize to tray on close if enabled in preferences, otherwise exit application completely */
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.preferences.minimize_to_tray {
                self.is_minimized_to_tray = true;
                state::APP_FOCUSED.store(false, std::sync::atomic::Ordering::Relaxed);
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                tray::trim_working_set();
            } else {
                self.preferences.save();
                tray::remove_tray();
                std::process::exit(0);
            }
        }

        /* Check low battery threshold and show desktop toast notification */
        self.check_low_battery_warning();

        /* Update tray tooltip with live model and battery status */
        let tray_tooltip = if self.connected {
            format!("OpenATK: {} ({}%)", self.mouse.model_name, self.mouse.battery_pct)
        } else {
            "OpenATK (Searching for mouse...)".to_string()
        };
        tray::update_tray_tooltip(&tray_tooltip);

        /* Drain updates from background worker thread (zero latency, zero UI blocking) */
        self.drain_device_updates();

        /* If minimized to tray, sleep to prevent winit from busy-waiting on hidden window */
        if self.is_minimized_to_tray {
            std::thread::sleep(Duration::from_millis(200));
            return;
        }

        /* One-time memory trim 1.5s after launch to flush DirectX 12 driver initialization buffers */
        if !self.initial_trim_done && self.startup_time.elapsed() >= Duration::from_millis(1500) {
            self.initial_trim_done = true;
            tray::trim_working_set();
        }

        /* Reactive frame scheduling: avoids continuous 33 FPS GPU rendering while idle */
        if self.status_message.is_some() {
            /* Active toast message: tick down smoothly */
            ctx.request_repaint_after(Duration::from_millis(100));
        } else if is_window_focused {
            /* Gentle 1-second pulse for telemetry/battery while window is in foreground */
            ctx.request_repaint_after(Duration::from_secs(1));
        }

        /* Left sidebar (width 250px for spacious layout) with auto-scrolling when content overflows */
        egui::SidePanel::left("left_sidebar")
            .resizable(false)
            .exact_width(250.0)
            .frame(
                egui::Frame::none()
                    .fill(theme::COLOR_BG_SIDEBAR)
                    .inner_margin(egui::Margin::symmetric(14.0, 10.0)),
            )
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = egui::Vec2::new(0.0, 2.0);

                /* Bottom toolbar is allocated at the bottom first so it never overlaps content */
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.add_space(2.0);
                    sidebar::render_bottom_toolbar(ui, self);
                    ui.add_space(4.0);

                    /* Content area above the toolbar scrolls smoothly if needed */
                    ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
                            .show(ui, |ui| {
                                sidebar::render_sidebar_content(ui, self);
                            });
                    });
                });
            });

        /* Bottom disclaimer inscription & change notifications */
        egui::TopBottomPanel::bottom("bottom_disclaimer")
            .frame(
                egui::Frame::none()
                    .fill(theme::COLOR_BG_CANVAS)
                    .inner_margin(egui::Margin::symmetric(24.0, 8.0)),
            )
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    /* Notification of changes strictly ABOVE the disclaimer text */
                    if let Some((msg, time, is_err)) = &self.status_message {
                        let timeout = if msg.contains("http") {
                            Duration::from_secs(6)
                        } else {
                            Duration::from_secs(3)
                        };
                        if time.elapsed() < timeout {
                            let bg_color = if *is_err {
                                Color32::from_rgb(180, 45, 45)
                            } else {
                                Color32::from_rgb(32, 130, 80)
                            };
                            egui::Frame::none()
                                .fill(bg_color)
                                .rounding(Rounding::same(6.0))
                                .inner_margin(egui::Margin::symmetric(14.0, 5.0))
                                .show(ui, |ui| {
                                    if let Some(idx) = msg.find("https://") {
                                        let (prefix, url) = msg.split_at(idx);
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(prefix).color(Color32::WHITE).strong().size(12.5));
                                            ui.hyperlink_to(
                                                RichText::new(url)
                                                    .color(Color32::from_rgb(205, 240, 255))
                                                    .underline()
                                                    .strong()
                                                    .size(12.5),
                                                url,
                                            );
                                        });
                                    } else {
                                        ui.label(RichText::new(msg).color(Color32::WHITE).strong().size(12.5));
                                    }
                                });
                            ui.add_space(5.0);
                        }
                    }

                    ui.label(
                        egui::RichText::new("Unofficial Community Suite — Not an official ATK product")
                            .size(11.5)
                            .color(Color32::from_gray(115)),
                    );
                });
            });

        /* Main content canvas with vertical ScrollArea so content never gets cut off */
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(theme::COLOR_BG_CANVAS)
                    .inner_margin(egui::Margin {
                        left: 24.0,
                        right: 4.0,
                        top: 24.0,
                        bottom: 0.0,
                    }),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        egui::Frame::none()
                            .inner_margin(egui::Margin {
                                left: 0.0,
                                right: 20.0,
                                top: 0.0,
                                bottom: 12.0,
                            })
                            .show(ui, |ui| {
                                match self.active_nav {
                                    NavItem::KeyChange => views::key_change::render_key_change_view(ui, self),
                                    NavItem::MacroSettings => views::macro_view::render_macro_view(ui, self),
                                    NavItem::DpiSettings => views::dpi::render_dpi_view(ui, self),
                                    NavItem::PerformanceSettings => views::performance::render_performance_view(ui, self),
                                    NavItem::ParameterSettings => views::parameters::render_parameter_view(ui, self),
                                    NavItem::ConfigManagement => views::config::render_config_management_view(ui, self),
                                    NavItem::OtherSettings => views::other::render_other_settings_view(ui, self),
                                }
                            });
                    });
            });
    }
}
