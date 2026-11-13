/* Macro engine: recording, sequencing, and software reproduction. */

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MacroError {
    #[error("Playback was cancelled")]
    Cancelled,
    #[error("System input simulation failed: {0}")]
    InputSimulation(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MacroAction {
    KeyDown(u16),
    KeyUp(u16),
    MouseDown(u8),
    MouseUp(u8),
    DelayMs(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Macro {
    pub id: String,
    pub name: String,
    pub actions: Vec<MacroAction>,
    pub repeat_count: u32,
}

impl Macro {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            actions: Vec::new(),
            repeat_count: 1,
        }
    }
}

