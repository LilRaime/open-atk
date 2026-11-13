/* Packet structures and framing for ATK HID communication. */

use crate::checksum::calculate_compx_checksum;
use crate::constants::{ATK_PAYLOAD_SIZE, ATK_REPORT_ID_VENDOR, ATK_WIRE_REPORT_SIZE};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum PacketError {
    #[error("Packet length {0} is smaller than required minimum {1}")]
    TooShort(usize, usize),
    #[error("Checksum verification failed: expected {expected:#04X}, got {actual:#04X}")]
    ChecksumMismatch { expected: u8, actual: u8 },
    #[error("Invalid Report ID {0:#04X}, expected {1:#04X}")]
    InvalidReportId(u8, u8),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet {
    pub report_id: u8,
    pub payload: [u8; ATK_PAYLOAD_SIZE],
}

impl Packet {
    pub fn new(report_id: u8) -> Self {
        Self {
            report_id,
            payload: [0u8; ATK_PAYLOAD_SIZE],
        }
    }

    /* Construct a command packet with automatic checksum calculation */
    pub fn new_command(cmd: u8, body: &[u8]) -> Self {
        let mut payload = [0u8; ATK_PAYLOAD_SIZE];
        payload[0] = cmd;
        let copy_len = body.len().min(14);
        payload[1..1 + copy_len].copy_from_slice(&body[..copy_len]);

        /* Wire representation: report_id at index 0, payload at index 1..16 */
        /* checksum is calculated so sum of all 17 bytes equals 0x55 */
        let mut wire_prefix = [0u8; 16];
        wire_prefix[0] = ATK_REPORT_ID_VENDOR;
        wire_prefix[1..16].copy_from_slice(&payload[..15]);
        payload[15] = calculate_compx_checksum(&wire_prefix);

        Self {
            report_id: ATK_REPORT_ID_VENDOR,
            payload,
        }
    }

    /* Parse from wire bytes (17 bytes: report_id + 16 payload bytes) */
    pub fn from_wire_bytes(bytes: &[u8]) -> Result<Self, PacketError> {
        if bytes.len() < ATK_WIRE_REPORT_SIZE {
            return Err(PacketError::TooShort(bytes.len(), ATK_WIRE_REPORT_SIZE));
        }

        let report_id = bytes[0];
        if report_id != ATK_REPORT_ID_VENDOR {
            return Err(PacketError::InvalidReportId(report_id, ATK_REPORT_ID_VENDOR));
        }

        let mut payload = [0u8; ATK_PAYLOAD_SIZE];
        payload.copy_from_slice(&bytes[1..1 + ATK_PAYLOAD_SIZE]);

        let expected_csum = calculate_compx_checksum(&bytes[..16]);
        if payload[15] != expected_csum {
            return Err(PacketError::ChecksumMismatch {
                expected: expected_csum,
                actual: payload[15],
            });
        }

        Ok(Self { report_id, payload })
    }

    /* Convert to full wire buffer including Report ID at index 0 */
    pub fn to_hid_buffer(&self) -> [u8; ATK_WIRE_REPORT_SIZE] {
        let mut buf = [0u8; ATK_WIRE_REPORT_SIZE];
        buf[0] = self.report_id;
        buf[1..17].copy_from_slice(&self.payload);
        buf
    }
}
