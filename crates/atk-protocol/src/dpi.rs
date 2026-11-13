/* DPI levels, stages, and validation. */

use thiserror::Error;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DpiValue(pub u16);

#[derive(Error, Debug, PartialEq, Eq)]
pub enum DpiError {
    #[error("DPI {0} is out of supported range ({min} - {max})", min = DpiValue::MIN.0, max = DpiValue::MAX.0)]
    OutOfRange(u16),
    #[error("DPI {0} must be a multiple of step size {1}")]
    InvalidStep(u16, u16),
    #[error("Invalid active DPI stage index {0}, max allowed is {1}")]
    InvalidStageIndex(usize, usize),
    #[error("A minimum of 1 DPI stage is required")]
    EmptyStages,
    #[error("Exceeded maximum of {0} DPI stages")]
    TooManyStages(usize),
}

impl DpiValue {
    pub const MIN: DpiValue = DpiValue(50);
    pub const MAX: DpiValue = DpiValue(30000);
    pub const STEP: u16 = 50;

    pub fn new(value: u16) -> Result<Self, DpiError> {
        if value < Self::MIN.0 || value > Self::MAX.0 {
            return Err(DpiError::OutOfRange(value));
        }
        if value % Self::STEP != 0 {
            return Err(DpiError::InvalidStep(value, Self::STEP));
        }
        Ok(DpiValue(value))
    }

    pub fn raw(self) -> u16 {
        self.0
    }
}

/* Represents a single DPI stage (which can have an RGB indicator color associated). */
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DpiStage {
    pub dpi: DpiValue,
    /* RGB color for this stage: (R, G, B) */
    pub color: (u8, u8, u8),
}

impl DpiStage {
    pub fn new(dpi: DpiValue, color: (u8, u8, u8)) -> Self {
        Self { dpi, color }
    }
}

/* Collection of DPI stages and the currently active stage index. */
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DpiConfig {
    pub stages: Vec<DpiStage>,
    pub active_stage_idx: usize,
}

impl DpiConfig {
    pub const MAX_STAGES: usize = 6;

    pub fn new(stages: Vec<DpiStage>, active_stage_idx: usize) -> Result<Self, DpiError> {
        if stages.is_empty() {
            return Err(DpiError::EmptyStages);
        }
        if stages.len() > Self::MAX_STAGES {
            return Err(DpiError::TooManyStages(Self::MAX_STAGES));
        }
        if active_stage_idx >= stages.len() {
            return Err(DpiError::InvalidStageIndex(active_stage_idx, stages.len() - 1));
        }
        Ok(Self {
            stages,
            active_stage_idx,
        })
    }

    pub fn active_stage(&self) -> &DpiStage {
        &self.stages[self.active_stage_idx]
    }

    pub fn default_r1_pro_max() -> Self {
        Self {
            stages: vec![
                DpiStage::new(DpiValue(400), (255, 0, 0)),
                DpiStage::new(DpiValue(800), (0, 0, 255)),
                DpiStage::new(DpiValue(1600), (0, 255, 0)),
                DpiStage::new(DpiValue(3200), (255, 255, 0)),
            ],
            active_stage_idx: 1,
        }
    }
}

/* Encode a single axis DPI value into (byte, nibble) */
pub fn encode_dpi_axis(dpi: u32) -> (u8, u8) {
    let doubled = dpi > 30000;
    let step = if doubled { 100 } else { 50 };
    let count = (dpi / step).saturating_sub(1);
    let byte = (count & 0xFF) as u8;
    let nibble = ((((count >> 8) & 0x03) << 2) | (if doubled { 1 } else { 0 })) as u8;
    (byte, nibble)
}

/* Decode a single axis DPI value from (byte, nibble) */
pub fn decode_dpi_axis(byte: u8, nibble: u8) -> u32 {
    let count = (byte as u32) | (((nibble as u32 >> 2) & 0x03) << 8);
    let doubled = (nibble & 1) != 0;
    let step = if doubled { 100 } else { 50 };
    (count + 1) * step
}

/* Pack X and Y DPI into the 4-byte EEPROM record */
pub fn pack_dpi_stage(x: u32, y: u32) -> [u8; 4] {
    let (xb, xn) = encode_dpi_axis(x);
    let (yb, yn) = encode_dpi_axis(y);
    let mode = ((yn & 0x0F) << 4) | (xn & 0x0F);
    let total = xb.wrapping_add(yb).wrapping_add(mode);
    let check = 0x55u8.wrapping_sub(total);
    [xb, yb, mode, check]
}

/* Unpack 4-byte EEPROM record into (x_dpi, y_dpi) */
pub fn unpack_dpi_stage(data: &[u8]) -> Option<(u32, u32)> {
    if data.len() < 4 {
        return None;
    }
    let sum = data[0].wrapping_add(data[1]).wrapping_add(data[2]).wrapping_add(data[3]);
    if sum != 0x55 {
        return None;
    }
    let x = decode_dpi_axis(data[0], data[2] & 0x0F);
    let y = decode_dpi_axis(data[1], (data[2] >> 4) & 0x0F);
    Some((x, y))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dpi_roundtrip_800() {
        let packed = pack_dpi_stage(800, 800);
        assert_eq!(packed, [0x0F, 0x0F, 0x00, 0x37]);
        let (x, y) = unpack_dpi_stage(&packed).unwrap();
        assert_eq!(x, 800);
        assert_eq!(y, 800);
    }

    #[test]
    fn test_dpi_roundtrip_1600() {
        let packed = pack_dpi_stage(1600, 1600);
        let (x, y) = unpack_dpi_stage(&packed).unwrap();
        assert_eq!(x, 1600);
        assert_eq!(y, 1600);
    }
}
