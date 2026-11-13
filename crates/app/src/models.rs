/* Core domain models, IPC message types, and button action codecs. */

#![allow(dead_code)]

use atk_device::ConnectionMode;
use eframe::egui::Vec2;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum NavItem {
    KeyChange,
    MacroSettings,
    DpiSettings,
    PerformanceSettings,
    ParameterSettings,
    ConfigManagement,
    OtherSettings,
}

#[derive(Clone, Debug)]
pub struct ButtonBinding {
    pub id: usize,
    pub title: &'static str,
    pub action: String,
    pub hotspot_offset: Vec2,
    pub badge_offset: Vec2,
}

pub enum DeviceAction {
    SetActiveStage(u8),
    SetStageDpi { stage: u8, x: u32, y: u32 },
    SetStageColor { stage: u8, r: u8, g: u8, b: u8 },
    SetStageCount(u8),
    SetPolling(u16),
    SetDebounce(u8),
    SetMotionSync(bool),
    SetStraightLine(bool),
    SetRipple(bool),
    SetUltraLongRange(bool),
    SetSleepTime(usize),
    SetLod(u8),
    SetDpiLight { mode: u8, brightness: u8, speed: u8 },
    SetButton { btn_idx: u8, key_class: u8, val1: u8, val2: u8 },
    CommitProfile,
    RestoreDefaults,
    RefreshHardware,
    SwitchDevice(String),
}

pub enum DeviceUpdate {
    Disconnected,
    DeviceList(Vec<DeviceEntry>),
    Status {
        device_id: String,
        battery_pct: u8,
        is_charging: bool,
        voltage_mv: u16,
        active_stage: u8,
        stage_count: u8,
    },
    FullRefresh {
        device_id: String,
        model_name: String,
        vendor_id: u16,
        product_id: u16,
        cid: u8,
        mid: u8,
        firmware: String,
        battery_pct: u8,
        is_charging: bool,
        voltage_mv: u16,
        rf_id: String,
        polling_rate: u16,
        active_stage: u8,
        stage_count: u8,
        stages_dpi: Vec<(u32, u32)>,
        stage_colors: Vec<(u8, u8, u8)>,
        lod_code: u8,
        debounce_ms: u8,
        motion_sync: bool,
        straight_line_correction: bool,
        ripple_control: bool,
        ultra_long_range: bool,
        sleep_time_idx: usize,
        dpi_light_mode: u8,
        dpi_light_brightness: u8,
        dpi_light_speed: u8,
        button_mappings: Vec<(u8, u8, u8)>,
    },
    Toast(String, bool),
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MacroStep {
    KeyDown(String),
    KeyUp(String),
    KeyPress(String),
    MouseDown(String),
    MouseUp(String),
    DelayMs(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MacroTriggerMode {
    /* Run specified repeat_count times on button click (default) */
    Once,
    /* Keep repeating continuously while the button is held down */
    HoldToRepeat,
    /* Click once to loop continuously, click again to stop */
    Toggle,
}

impl Default for MacroTriggerMode {
    fn default() -> Self {
        MacroTriggerMode::Once
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MacroItem {
    pub id: String,
    pub name: String,
    pub repeat_count: u32,
    #[serde(default)]
    pub trigger_mode: MacroTriggerMode,
    pub steps: Vec<MacroStep>,
}

pub struct MouseData {
    pub model_name: String,
    pub mode: ConnectionMode,
    pub vendor_id: u16,
    pub product_id: u16,
    pub cid: u8,
    pub mid: u8,
    pub firmware: String,
    pub battery_pct: u8,
    pub is_charging: bool,
    pub voltage_mv: u16,
    pub rf_id: String,
    pub polling_rate: u16,
    pub active_stage: u8,
    pub stage_count: u8,
    pub stages_dpi: Vec<(u32, u32)>,
    pub stage_colors: Vec<(u8, u8, u8)>,
    pub independent_xy: Vec<bool>,
    pub lod_code: u8,
    pub debounce_ms: u8,
    pub motion_sync: bool,
    pub straight_line_correction: bool,
    pub ripple_control: bool,
    pub ultra_long_range: bool,
    pub sleep_time_idx: usize,
    pub dpi_light_mode: u8,
    pub dpi_light_brightness: u8,
    pub dpi_light_speed: u8,
    pub buttons: Vec<ButtonBinding>,
}

impl Default for MouseData {
    fn default() -> Self {
        Self {
            model_name: "No Device Connected".to_string(),
            mode: ConnectionMode::WirelessDongle,
            vendor_id: 0x3554,
            product_id: 0xF58A,
            cid: 0,
            mid: 0,
            firmware: String::new(),
            battery_pct: 0,
            is_charging: false,
            voltage_mv: 0,
            rf_id: "00 00 00".to_string(),
            polling_rate: 1000,
            active_stage: 0,
            stage_count: 4,
            stages_dpi: vec![(400, 400), (800, 800), (1600, 1600), (3200, 3200)],
            stage_colors: vec![
                (155, 93, 229),  /* Purple */
                (255, 255, 255), /* White */
                (0, 180, 255),   /* Cyan/Blue */
                (255, 0, 128),   /* Magenta/Pink */
            ],
            independent_xy: vec![false, false, false, false, false, false],
            lod_code: 1,
            debounce_ms: 4,
            motion_sync: true,
            straight_line_correction: false,
            ripple_control: false,
            ultra_long_range: false,
            sleep_time_idx: 1,
            dpi_light_mode: 0,
            dpi_light_brightness: 2,
            dpi_light_speed: 1,
            buttons: vec![
                ButtonBinding {
                    id: 0,
                    title: "Left Button",
                    action: "Left Button".to_string(),
                    hotspot_offset: Vec2::new(-36.0, -153.0),
                    badge_offset: Vec2::new(-94.0, -153.0),
                },
                ButtonBinding {
                    id: 1,
                    title: "Right Click",
                    action: "Right Click".to_string(),
                    hotspot_offset: Vec2::new(70.0, -44.0),
                    badge_offset: Vec2::new(121.0, -58.0),
                },
                ButtonBinding {
                    id: 2,
                    title: "Middle Key",
                    action: "Middle Key".to_string(),
                    hotspot_offset: Vec2::new(-1.0, -109.0),
                    badge_offset: Vec2::new(57.0, -104.0),
                },
                ButtonBinding {
                    id: 3,
                    title: "Back",
                    action: "Back".to_string(),
                    hotspot_offset: Vec2::new(-98.0, 16.0),
                    badge_offset: Vec2::new(-55.0, 16.0),
                },
                ButtonBinding {
                    id: 4,
                    title: "Advance",
                    action: "Advance".to_string(),
                    hotspot_offset: Vec2::new(-86.0, -41.0),
                    badge_offset: Vec2::new(-139.0, -30.0),
                },
                ButtonBinding {
                    id: 5,
                    title: "DPI Cycle",
                    action: "DPI Cycle".to_string(),
                    hotspot_offset: Vec2::new(49.0, 48.0),
                    badge_offset: Vec2::new(105.0, 54.0),
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MouseColor {
    White,
    Black,
}

impl Default for MouseColor {
    fn default() -> Self {
        MouseColor::White
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceEntry {
    pub id: String,
    pub model_name: String,
    pub mode: ConnectionMode,
    pub battery_pct: u8,
    pub is_charging: bool,
    pub color: MouseColor,
}

pub fn encode_button_action(action: &str) -> (u8, u8, u8) {
    if action.starts_with("Macro:") {
        /* Macro action class on Compx Nordic firmware */
        return (0x05, 0x01, 0x00);
    }
    match action {
        "Left Button" => (0x01, 0x01, 0x00),
        "Right Click" => (0x01, 0x02, 0x00),
        "Middle Key" => (0x01, 0x04, 0x00),
        "Back" => (0x01, 0x08, 0x00),
        "Advance" => (0x01, 0x10, 0x00),
        "DPI Cycle" => (0x02, 0x01, 0x00),
        "DPI Up" => (0x02, 0x02, 0x00),
        "DPI Down" => (0x02, 0x03, 0x00),
        "Stage 1" => (0x02, 0x10, 0x00),
        "Stage 2" => (0x02, 0x20, 0x00),
        "Vol +" => (0x03, 0xE9, 0x00),
        "Vol -" => (0x03, 0xEA, 0x00),
        "Mute" => (0x03, 0xE2, 0x00),
        "Play/Pause" => (0x03, 0xCD, 0x00),
        "Disabled" => (0x00, 0x00, 0x00),
        _ => (0x01, 0x01, 0x00),
    }
}

pub fn decode_button_action(key_class: u8, val1: u8, val2: u8) -> String {
    match (key_class, val1, val2) {
        (0x05, _, _) => "Custom Macro".to_string(),
        (0x01, 0x01, 0x00) => "Left Button".to_string(),
        (0x01, 0x02, 0x00) => "Right Click".to_string(),
        (0x01, 0x04, 0x00) => "Middle Key".to_string(),
        (0x01, 0x08, 0x00) => "Back".to_string(),
        (0x01, 0x10, 0x00) => "Advance".to_string(),
        (0x02, 0x01, 0x00) => "DPI Cycle".to_string(),
        (0x02, 0x02, 0x00) => "DPI Up".to_string(),
        (0x02, 0x03, 0x00) => "DPI Down".to_string(),
        (0x02, 0x10, 0x00) => "Stage 1".to_string(),
        (0x02, 0x20, 0x00) => "Stage 2".to_string(),
        (0x03, 0xE9, 0x00) => "Vol +".to_string(),
        (0x03, 0xEA, 0x00) => "Vol -".to_string(),
        (0x03, 0xE2, 0x00) => "Mute".to_string(),
        (0x03, 0xCD, 0x00) => "Play/Pause".to_string(),
        (0x00, 0x00, 0x00) => "Disabled".to_string(),
        _ => "Custom".to_string(),
    }
}

