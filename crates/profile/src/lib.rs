/* Profile definitions, sensor tuning, button mapping, and validation. */

use atk_protocol::{ButtonAction, DpiConfig, MouseButton, PollingRate};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ProfileError {
    #[error("Profile name cannot be empty")]
    EmptyName,
    #[error("Debounce time {0} ms is out of valid range (0 - 30 ms)")]
    InvalidDebounce(u8),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SensorConfig {
    /* Lift-off distance in millimeters (e.g. 1mm or 2mm) */
    pub lod_mm: u8,
    /* Button debounce time in milliseconds (0 - 30 ms) */
    pub debounce_ms: u8,
    /* Motion sync enabled */
    pub motion_sync: bool,
    /* Angle snapping enabled */
    pub angle_snapping: bool,
    /* Ripple control enabled */
    pub ripple_control: bool,
}

impl Default for SensorConfig {
    fn default() -> Self {
        Self {
            lod_mm: 1,
            debounce_ms: 4,
            motion_sync: true,
            angle_snapping: false,
            ripple_control: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub dpi: DpiConfig,
    pub polling_rate: PollingRate,
    pub sensor: SensorConfig,
    pub buttons: HashMap<MouseButton, ButtonAction>,
}

impl Profile {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Result<Self, ProfileError> {
        let name_str = name.into();
        if name_str.trim().is_empty() {
            return Err(ProfileError::EmptyName);
        }

        let mut buttons = HashMap::new();
        buttons.insert(MouseButton::Left, ButtonAction::ClickLeft);
        buttons.insert(MouseButton::Right, ButtonAction::ClickRight);
        buttons.insert(MouseButton::Middle, ButtonAction::ClickMiddle);
        buttons.insert(MouseButton::Forward, ButtonAction::BrowserForward);
        buttons.insert(MouseButton::Back, ButtonAction::BrowserBack);
        buttons.insert(MouseButton::DpiSwitch, ButtonAction::DpiCycle);

        Ok(Self {
            id: id.into(),
            name: name_str,
            dpi: DpiConfig::default_r1_pro_max(),
            polling_rate: PollingRate::Hz1000,
            sensor: SensorConfig::default(),
            buttons,
        })
    }

    pub fn validate(&self) -> Result<(), ProfileError> {
        if self.name.trim().is_empty() {
            return Err(ProfileError::EmptyName);
        }
        if self.sensor.debounce_ms > 30 {
            return Err(ProfileError::InvalidDebounce(self.sensor.debounce_ms));
        }
        Ok(())
    }
}

