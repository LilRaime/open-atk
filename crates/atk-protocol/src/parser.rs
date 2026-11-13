/* Parser for incoming reports from ATK mice. */

use crate::battery::BatteryStatus;
use crate::constants::*;
use crate::packet::Packet;
use crate::response::{IdentityInfo, ReceiverInfo, Response};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ParseError {
    #[error("Device reported error status: {0:#04X}")]
    DeviceError(u8),
    #[error("Response length {0} is smaller than expected")]
    ResponseTooShort(usize),
}

pub struct Parser;

impl Parser {
    pub fn parse_packet(packet: &Packet) -> Result<Response, ParseError> {
        let cmd = packet.payload[0];
        let status = packet.payload[1];

        if status != 0 && cmd != CMD_BATTERY {
            return Err(ParseError::DeviceError(status));
        }

        match cmd {
            CMD_BATTERY => {
                let battery_pct = packet.payload[5];
                let is_charging = packet.payload[6] != 0;
                let voltage_mv = ((packet.payload[7] as u16) << 8) | (packet.payload[8] as u16);
                Ok(Response::Battery {
                    status: BatteryStatus::new(battery_pct, is_charging),
                    voltage_mv,
                })
            }
            CMD_CID_MID => {
                let cid = packet.payload[5];
                let mid = packet.payload[6];
                let model_name = resolve_model_name(cid, mid);
                Ok(Response::Identity(IdentityInfo {
                    cid,
                    mid,
                    model_name,
                }))
            }
            CMD_VERSION => {
                let major = packet.payload[5];
                let minor = packet.payload[6];
                Ok(Response::Firmware(format!("{major:x}.{minor:02x}")))
            }
            CMD_READ_EEPROM => {
                let addr = ((packet.payload[2] as u16) << 8) | (packet.payload[3] as u16);
                let len = (packet.payload[4] as usize).min(10);
                let data = packet.payload[5..5 + len].to_vec();
                Ok(Response::RegisterData { addr, data })
            }
            CMD_RECEIVER_ONLINE => {
                let online = packet.payload[5] == 1;
                let rf_id = format!(
                    "{:02X} {:02X} {:02X}",
                    packet.payload[8], packet.payload[7], packet.payload[6]
                );
                Ok(Response::Receiver(ReceiverInfo { online, rf_id }))
            }
            CMD_WRITE_EEPROM | CMD_SET_PROFILE => Ok(Response::Ack),
            _ => Ok(Response::Raw(packet.payload.to_vec())),
        }
    }
}

pub fn resolve_model_name(cid: u8, mid: u8) -> String {
    match (cid, mid) {
        (2, 27) | (2, 28) => "VXE R1 Pro Max".to_string(),
        (2, 7) | (2, 8) => "VXE R1 Pro".to_string(),
        (2, 11) | (2, 12) => "VXE R1".to_string(),
        (2, 13) | (2, 14) => "VXE R1 SE".to_string(),
        (2, 15) | (2, 16) | (2, 32) | (2, 85) => "VXE R1SE+".to_string(),
        (2, 114) => "VXE R1 Ultra".to_string(),
        (2, 125) => "VXE R1 Pro (PAW3950Ultra)".to_string(),
        (2, 126) => "VXE R1 Pro Max (PAW3950Ultra)".to_string(),
        (2, 127) => "VXE R1 (PAW3950Ultra)".to_string(),
        (2, 47) => "ATK MAD R MAJOR".to_string(),
        (2, 49) => "ATK MAD R".to_string(),
        (2, 54) => "ATK MAD R +".to_string(),
        (2, 55) => "ATK MAD R MAJOR+".to_string(),
        (2, 133) => "ATK MAD R (PAW3950Ultra)".to_string(),
        (2, 39) => "ATK X1 Pro Max".to_string(),
        (2, 83) => "ATK A9 Plus NearLink".to_string(),
        _ => "ATK Gaming Mouse".to_string(),
    }
}
