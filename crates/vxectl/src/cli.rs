/*
 * Command line interface definition for vxectl.
 */

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "vxectl", author = "OpenATK Contributors", version = "0.1.0")]
#[command(about = "CLI control utility for ATK / VXE gaming mice", long_about = None)]
pub struct Cli {
    /* Target mouse index (0, 1...) or model alias (e.g. 'r1pm', 'r1p', 'r1se', 'f1pm', 'madr') */
    #[arg(short, long, global = true, value_name = "DEVICE")]
    pub device: Option<String>,

    /* Disable ANSI color formatting and decorative card borders */
    #[arg(long, global = true)]
    pub no_color: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /* List all connected ATK / VXE mice, dongles, and their short aliases */
    #[command(name = "list", alias = "devices", about = "List connected mice, dongles, and short aliases")]
    List {
        /* Output formatted as JSON */
        #[arg(long)]
        json: bool,
    },

    /* Show current mouse status, battery, polling rate, and DPI stages (default) */
    #[command(about = "Show mouse status, battery, polling rate, and DPI stages")]
    Status {
        /* Output formatted as JSON */
        #[arg(long)]
        json: bool,
    },

    /* Continuously monitor live battery, DPI stage, and sleep state in the terminal */
    #[command(name = "watch", alias = "monitor", about = "Live terminal monitor for battery, DPI stage, and telemetry")]
    Watch {
        /* Refresh interval in seconds (default: 2) */
        #[arg(short, long, default_value_t = 2, value_name = "SECS")]
        interval: u64,
    },

    /* Show current battery percentage and charging state */
    #[command(about = "Show battery percentage and charging state")]
    Battery {
        /* Output formatted as JSON */
        #[arg(long)]
        json: bool,
    },

    /* Set active DPI stage (1-based index: 1, 2, 3, 4...) */
    #[command(name = "set-stage", alias = "stage", about = "Set active DPI stage (1..6)")]
    SetStage {
        /* Stage number to activate (1 to 4/6) */
        #[arg(value_name = "STAGE")]
        stage: Option<u8>,
    },

    /* Set DPI for a stage: set-dpi <STAGE> <DPI> [--y <Y>] */
    #[command(name = "set-dpi", alias = "dpi", about = "Set DPI for a stage: set-dpi <STAGE> <DPI> [--y <Y>]")]
    SetDpi {
        /* Target stage number (1-based index: 1, 2, 3, 4...) */
        #[arg(value_name = "STAGE")]
        stage: u8,

        /* DPI value (e.g. 400, 800, 1200, 1600, 3200...) */
        #[arg(value_name = "DPI")]
        dpi: u32,

        /* Optional separate Y-axis DPI value (defaults to same as DPI) */
        #[arg(short, long, value_name = "Y_DPI")]
        y: Option<u32>,
    },

    /* Set polling rate (Hz: 125, 250, 500, 1000) */
    #[command(name = "set-polling", alias = "polling", about = "Set polling rate in Hz (125..8000)")]
    SetPolling {
        /* Target polling rate in Hz (125, 250, 500, 1000) */
        #[arg(value_name = "HZ")]
        rate: Option<u16>,
    },

    /* Set Lift-off Distance (LOD: 1 = 1.0 mm, 2 = 2.0 mm) */
    #[command(name = "set-lod", alias = "lod", about = "Set Lift-off Distance (1 = 1.0mm, 2 = 2.0mm)")]
    SetLod {
        /* LOD level: 1 (1.0 mm Low) or 2 (2.0 mm High) */
        #[arg(value_name = "MM")]
        lod: Option<u8>,
    },

    /* Set switch debounce stabilization delay (0 to 20 ms) */
    #[command(name = "set-debounce", alias = "debounce", about = "Set switch debounce delay (0..20 ms)")]
    SetDebounce {
        /* Debounce delay in milliseconds (0..=20) */
        #[arg(value_name = "MS")]
        ms: Option<u8>,
    },

    /* Enable or disable sensor Motion Synchronization (on / off) */
    #[command(name = "set-motionsync", alias = "motionsync", about = "Enable or disable sensor Motion Sync (on/off)")]
    SetMotionSync {
        /* Target state ('on', 'off', 'true', 'false', '1', '0') */
        #[arg(value_name = "STATE")]
        state: Option<String>,
    },

    /* Enable or disable Straight Line Correction / Angle Snapping (on / off) */
    #[command(name = "set-straightline", alias = "straightline", about = "Enable or disable Straight Line Correction (on/off)")]
    SetStraightLine {
        /* Target state ('on', 'off', 'true', 'false', '1', '0') */
        #[arg(value_name = "STATE")]
        state: Option<String>,
    },

    /* Enable or disable Ripple Control smoothing (on / off) */
    #[command(name = "set-ripple", alias = "ripple", about = "Enable or disable Ripple Control smoothing (on/off)")]
    SetRipple {
        /* Target state ('on', 'off', 'true', 'false', '1', '0') */
        #[arg(value_name = "STATE")]
        state: Option<String>,
    },

    /* Enable or disable Ultra Long Range wireless mode (on / off) */
    #[command(name = "set-longrange", alias = "longrange", about = "Enable or disable Ultra Long Range mode (on/off)")]
    SetLongRange {
        /* Target state ('on', 'off', 'true', 'false', '1', '0') */
        #[arg(value_name = "STATE")]
        state: Option<String>,
    },

    /* Set mouse sleep timeout (30s, 1m, 2m, 3m, 5m, 20m, 25m, 30m) */
    #[command(name = "set-sleep", alias = "sleep", about = "Set sleep timeout (30s, 1m, 2m, 3m, 5m, 20m, 25m, 30m)")]
    SetSleep {
        /* Timeout value ('30s', '1m', '2m', '3m', '5m', '20m', '25m', '30m') */
        #[arg(value_name = "TIME")]
        time: Option<String>,
    },

    /* Set DPI indicator LED effect (off, steady, breathing) */
    #[command(name = "set-dpilight", alias = "dpilight", about = "Set DPI indicator LED mode (off, steady, breathing)")]
    SetDpiLight {
        /* Mode ('off', 'steady', 'breathing') */
        #[arg(value_name = "MODE")]
        mode: Option<String>,
    },

    /* Export full mouse hardware profile to JSON (to file or stdout) */
    #[command(name = "export", alias = "dump", about = "Export mouse hardware profile to JSON")]
    Export {
        /* Output JSON file path (prints to stdout if omitted) */
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,
    },

    /* Import and apply a mouse profile from a JSON file */
    #[command(name = "import", alias = "load", about = "Import and apply a mouse profile from JSON")]
    Import {
        /* Path to input JSON profile file */
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },

    /* Generate shell auto-completion script for vxectl (powershell, bash, zsh, fish) */
    #[command(name = "completions", about = "Generate shell auto-completion script")]
    Completions {
        /* Target shell ('powershell', 'bash', 'zsh', 'fish') */
        #[arg(value_name = "SHELL")]
        shell: String,
    },

    /* Automatically install shell auto-completions into user shell profile or system directory */
    #[command(name = "install-completions", alias = "setup", about = "Automatically install shell auto-completions")]
    InstallCompletions,

    /* Install udev rules on Linux (/etc/udev/rules.d/71-open-atk.rules) */
    #[command(name = "install-udev", about = "Install Linux udev rules for rootless USB HID access")]
    InstallUdev,

    /* Internal dynamic completion engine invoked by shell Tab-completion hooks */
    #[command(name = "_complete", hide = true)]
    Complete {
        /* Target shell format ('powershell', 'bash', 'zsh', 'fish') */
        #[arg(long, default_value = "plain")]
        shell: String,

        /* Command-line words after the binary name */
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        words: Vec<String>,
    },
}

