/* Battery status and charging states. */

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BatteryStatus {
    /* Percentage from 0 to 100 */
    pub percentage: u8,
    /* Whether the device is currently charging (plugged in via cable) */
    pub is_charging: bool,
}

impl BatteryStatus {
    pub fn new(percentage: u8, is_charging: bool) -> Self {
        Self {
            percentage: percentage.min(100),
            is_charging,
        }
    }
}

