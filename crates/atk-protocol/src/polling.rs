/* Polling rate settings and conversion. */

use thiserror::Error;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum PollingRate {
    Hz125 = 125,
    Hz250 = 250,
    Hz500 = 500,
    Hz1000 = 1000,
    Hz2000 = 2000,
    Hz4000 = 4000,
    Hz8000 = 8000,
}

#[derive(Error, Debug, PartialEq, Eq)]
#[error("Unsupported polling rate {0} Hz")]
pub struct UnsupportedPollingRateError(pub u16);

impl PollingRate {
    pub fn from_hz(hz: u16) -> Result<Self, UnsupportedPollingRateError> {
        match hz {
            125 => Ok(PollingRate::Hz125),
            250 => Ok(PollingRate::Hz250),
            500 => Ok(PollingRate::Hz500),
            1000 => Ok(PollingRate::Hz1000),
            2000 => Ok(PollingRate::Hz2000),
            4000 => Ok(PollingRate::Hz4000),
            8000 => Ok(PollingRate::Hz8000),
            other => Err(UnsupportedPollingRateError(other)),
        }
    }

    pub fn to_hz(self) -> u16 {
        self as u16
    }
}

