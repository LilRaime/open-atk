/*
 * vxectl / atk-probe - CLI Control Utility for ATK and VXE Gaming Mice
 *
 * Provides fast access to hardware telemetry, battery, DPI stages,
 * and polling rates, with full support for safe EEPROM write & verification,
 * and multi-device selection with short aliases (-d / --device).
 */

mod cli;
mod commands;
mod device;
mod profile;
mod setup;
mod status;
mod theme;

use atk_device::AtkDevice;
use atk_hid::HidManager;
use clap::Parser;
use cli::{Cli, Commands};
use commands::{
    handle_set_debounce, handle_set_dpi, handle_set_dpilight, handle_set_lod, handle_set_longrange,
    handle_set_motionsync, handle_set_polling, handle_set_ripple, handle_set_sleep,
    handle_set_stage, handle_set_straightline,
};
use device::{
    scan_all_devices, select_device, select_device_for_mutation, show_device_list,
};
use profile::{handle_export_profile, handle_import_profile};
use setup::{handle_completions, handle_dynamic_complete, handle_install_completions, handle_install_udev};
use status::{show_all_batteries, show_single_battery, show_status};
use theme::Theme;

fn main() {
    let cli = Cli::parse();

    /* Fast dynamic shell completion callback */
    if let Some(Commands::Complete { shell, words }) = &cli.command {
        handle_dynamic_complete(shell, words);
        return;
    }

    /* Early commands that do not require HID scan */
    if let Some(Commands::Completions { shell }) = &cli.command {
        handle_completions(shell);
        return;
    }

    if let Some(Commands::InstallCompletions) = cli.command {
        handle_install_completions();
        return;
    }

    if let Some(Commands::InstallUdev) = cli.command {
        handle_install_udev();
        return;
    }

    let hid_mgr = match HidManager::new() {
        Ok(mgr) => mgr,
        Err(e) => {
            eprintln!("[!] Error initializing HID API: {e}");
            std::process::exit(1);
        }
    };

    let theme = Theme::new(cli.no_color);
    let devices = scan_all_devices(&hid_mgr);

    /* Handle `vxectl list` command first without requiring an open target */
    if let Some(Commands::List { json }) = cli.command {
        show_device_list(&devices, theme, json);
        return;
    }

    match cli.command.unwrap_or(Commands::Status { json: false }) {
        Commands::List { .. }
        | Commands::Completions { .. }
        | Commands::InstallCompletions
        | Commands::InstallUdev
        | Commands::Complete { .. } => unreachable!(),
        Commands::Status { json } => {
            if json {
                if let Some(selector) = cli.device.as_deref() {
                    let target = select_device(&devices, Some(selector));
                    let dev = match AtkDevice::open_with_info(&hid_mgr, target.channel.info.clone(), target.mode) {
                        Ok(d) => d,
                        Err(e) => {
                            eprintln!("[!] Failed to open mouse configuration channel for {}: {e}", target.model_name);
                            std::process::exit(1);
                        }
                    };
                    let status = status::get_status_json(&dev, target);
                    println!("{}", serde_json::to_string_pretty(&status).unwrap());
                } else if devices.len() <= 1 {
                    let target = select_device(&devices, None);
                    let dev = match AtkDevice::open_with_info(&hid_mgr, target.channel.info.clone(), target.mode) {
                        Ok(d) => d,
                        Err(e) => {
                            eprintln!("[!] Failed to open mouse configuration channel for {}: {e}", target.model_name);
                            std::process::exit(1);
                        }
                    };
                    let status = status::get_status_json(&dev, target);
                    println!("{}", serde_json::to_string_pretty(&status).unwrap());
                } else {
                    let mut list = Vec::new();
                    for target in &devices {
                        if let Ok(dev) = AtkDevice::open_with_info(&hid_mgr, target.channel.info.clone(), target.mode) {
                            list.push(status::get_status_json(&dev, target));
                        }
                    }
                    println!("{}", serde_json::to_string_pretty(&list).unwrap());
                }
                return;
            }

            if let Some(selector) = cli.device.as_deref() {
                let target = select_device(&devices, Some(selector));
                let dev = match AtkDevice::open_with_info(&hid_mgr, target.channel.info.clone(), target.mode) {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!("[!] Failed to open mouse configuration channel for {}: {e}", target.model_name);
                        std::process::exit(1);
                    }
                };
                show_status(&dev, target, 1, theme);
            } else {
                /* When run without a specific -d target, display status for ALL detected devices */
                if devices.is_empty() {
                    eprintln!("[-] No compatible ATK / VXE mouse detected.");
                    eprintln!("    Make sure your 2.4 GHz wireless dongle or USB-C cable is connected.");
                    std::process::exit(1);
                }
                let total = devices.len();
                for (i, target) in devices.iter().enumerate() {
                    if i > 0 {
                        println!();
                    }
                    match AtkDevice::open_with_info(&hid_mgr, target.channel.info.clone(), target.mode) {
                        Ok(dev) => {
                            show_status(&dev, target, total, theme);
                        }
                        Err(e) => {
                            eprintln!("[!] Failed to open device [{}] {}: {e}", target.index, target.model_name);
                        }
                    }
                }
                if total > 1 {
                    println!("\nUse `vxectl -d <index|alias> <command>` to target a specific device.");
                }
            }
        }
        Commands::Watch { interval } => {
            let interval_secs = interval.max(1);
            loop {
                if !cli.no_color {
                    print!("\x1B[2J\x1B[H");
                }
                let live_devices = scan_all_devices(&hid_mgr);
                if let Some(selector) = cli.device.as_deref() {
                    if live_devices.is_empty() {
                        show_device_list(&live_devices, theme, false);
                    } else {
                        let target = select_device(&live_devices, Some(selector));
                        if let Ok(dev) = AtkDevice::open_with_info(&hid_mgr, target.channel.info.clone(), target.mode) {
                            show_status(&dev, target, 1, theme);
                        }
                    }
                } else if live_devices.is_empty() {
                    show_device_list(&live_devices, theme, false);
                } else {
                    let total = live_devices.len();
                    for (i, target) in live_devices.iter().enumerate() {
                        if i > 0 {
                            println!();
                        }
                        if let Ok(dev) = AtkDevice::open_with_info(&hid_mgr, target.channel.info.clone(), target.mode) {
                            show_status(&dev, target, total, theme);
                        }
                    }
                }
                println!(
                    "\n{}",
                    theme.dim(&format!(
                        "Live monitor: refreshing every {}s (Ctrl+C to exit)",
                        interval_secs
                    ))
                );
                let _ = std::io::Write::flush(&mut std::io::stdout());
                std::thread::sleep(std::time::Duration::from_secs(interval_secs));
            }
        }
        Commands::Battery { json } => {
            if let Some(selector) = cli.device.as_deref() {
                let target = select_device(&devices, Some(selector));
                let dev = match AtkDevice::open_with_info(&hid_mgr, target.channel.info.clone(), target.mode) {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!("[!] Failed to open mouse configuration channel for {}: {e}", target.model_name);
                        std::process::exit(1);
                    }
                };
                show_single_battery(&dev, json, &target.model_name, theme);
            } else if devices.len() <= 1 {
                let target = select_device(&devices, None);
                let dev = match AtkDevice::open_with_info(&hid_mgr, target.channel.info.clone(), target.mode) {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!("[!] Failed to open mouse configuration channel for {}: {e}", target.model_name);
                        std::process::exit(1);
                    }
                };
                show_single_battery(&dev, json, &target.model_name, theme);
            } else {
                show_all_batteries(&devices, &hid_mgr, json, theme);
            }
        }
        Commands::SetStage { stage } => {
            let target = select_device_for_mutation(&devices, cli.device.as_deref());
            let dev = match AtkDevice::open_with_info(&hid_mgr, target.channel.info.clone(), target.mode) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("[!] Failed to open {}: {e}", target.model_name);
                    std::process::exit(1);
                }
            };
            handle_set_stage(&dev, stage);
        }
        Commands::SetDpi { stage, dpi, y } => {
            let target = select_device_for_mutation(&devices, cli.device.as_deref());
            let dev = match AtkDevice::open_with_info(&hid_mgr, target.channel.info.clone(), target.mode) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("[!] Failed to open {}: {e}", target.model_name);
                    std::process::exit(1);
                }
            };
            handle_set_dpi(&dev, stage, dpi, y);
        }
        Commands::SetPolling { rate } => {
            let target = select_device_for_mutation(&devices, cli.device.as_deref());
            let dev = match AtkDevice::open_with_info(&hid_mgr, target.channel.info.clone(), target.mode) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("[!] Failed to open {}: {e}", target.model_name);
                    std::process::exit(1);
                }
            };
            handle_set_polling(&dev, rate);
        }
        Commands::SetLod { lod } => {
            let target = select_device_for_mutation(&devices, cli.device.as_deref());
            let dev = match AtkDevice::open_with_info(&hid_mgr, target.channel.info.clone(), target.mode) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("[!] Failed to open {}: {e}", target.model_name);
                    std::process::exit(1);
                }
            };
            handle_set_lod(&dev, lod);
        }
        Commands::SetDebounce { ms } => {
            let target = select_device_for_mutation(&devices, cli.device.as_deref());
            let dev = match AtkDevice::open_with_info(&hid_mgr, target.channel.info.clone(), target.mode) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("[!] Failed to open {}: {e}", target.model_name);
                    std::process::exit(1);
                }
            };
            handle_set_debounce(&dev, ms);
        }
        Commands::SetMotionSync { state } => {
            let target = select_device_for_mutation(&devices, cli.device.as_deref());
            let dev = match AtkDevice::open_with_info(&hid_mgr, target.channel.info.clone(), target.mode) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("[!] Failed to open {}: {e}", target.model_name);
                    std::process::exit(1);
                }
            };
            handle_set_motionsync(&dev, state);
        }
        Commands::SetStraightLine { state } => {
            let target = select_device_for_mutation(&devices, cli.device.as_deref());
            let dev = match AtkDevice::open_with_info(&hid_mgr, target.channel.info.clone(), target.mode) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("[!] Failed to open {}: {e}", target.model_name);
                    std::process::exit(1);
                }
            };
            handle_set_straightline(&dev, state);
        }
        Commands::SetRipple { state } => {
            let target = select_device_for_mutation(&devices, cli.device.as_deref());
            let dev = match AtkDevice::open_with_info(&hid_mgr, target.channel.info.clone(), target.mode) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("[!] Failed to open {}: {e}", target.model_name);
                    std::process::exit(1);
                }
            };
            handle_set_ripple(&dev, state);
        }
        Commands::SetLongRange { state } => {
            let target = select_device_for_mutation(&devices, cli.device.as_deref());
            let dev = match AtkDevice::open_with_info(&hid_mgr, target.channel.info.clone(), target.mode) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("[!] Failed to open {}: {e}", target.model_name);
                    std::process::exit(1);
                }
            };
            handle_set_longrange(&dev, state);
        }
        Commands::SetSleep { time } => {
            let target = select_device_for_mutation(&devices, cli.device.as_deref());
            let dev = match AtkDevice::open_with_info(&hid_mgr, target.channel.info.clone(), target.mode) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("[!] Failed to open {}: {e}", target.model_name);
                    std::process::exit(1);
                }
            };
            handle_set_sleep(&dev, time);
        }
        Commands::SetDpiLight { mode } => {
            let target = select_device_for_mutation(&devices, cli.device.as_deref());
            let dev = match AtkDevice::open_with_info(&hid_mgr, target.channel.info.clone(), target.mode) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("[!] Failed to open {}: {e}", target.model_name);
                    std::process::exit(1);
                }
            };
            handle_set_dpilight(&dev, mode);
        }
        Commands::Export { file } => {
            let target = select_device_for_mutation(&devices, cli.device.as_deref());
            let dev = match AtkDevice::open_with_info(&hid_mgr, target.channel.info.clone(), target.mode) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("[!] Failed to open {}: {e}", target.model_name);
                    std::process::exit(1);
                }
            };
            handle_export_profile(&dev, &target.model_name, file);
        }
        Commands::Import { file } => {
            let target = select_device_for_mutation(&devices, cli.device.as_deref());
            let dev = match AtkDevice::open_with_info(&hid_mgr, target.channel.info.clone(), target.mode) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("[!] Failed to open {}: {e}", target.model_name);
                    std::process::exit(1);
                }
            };
            handle_import_profile(&dev, file);
        }
    }
}
