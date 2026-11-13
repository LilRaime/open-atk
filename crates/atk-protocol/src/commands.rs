/* Commands sent from host to device. */

use crate::constants::*;
use crate::packet::Packet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /* Query battery percentage, charging state, and voltage in mV */
    GetBattery,
    /* Query device Model/Client ID (CID, MID) */
    GetIdentity,
    /* Query firmware version */
    GetFirmware,
    /* Query receiver online status */
    GetReceiverStatus,
    /* Read EEPROM register(s) */
    ReadRegister { addr: u16, len: u8 },
    /* Write EEPROM register(s) */
    WriteRegister { addr: u16, data: Vec<u8> },
    /* Switch active profile (0..3) */
    SetProfile(u8),
}

impl Command {
    /* Serializes command into an outbound HID `Packet` */
    pub fn build_packet(&self) -> Packet {
        match self {
            Command::GetBattery => Packet::new_command(CMD_BATTERY, &[]),
            Command::GetIdentity => Packet::new_command(CMD_CID_MID, &[]),
            Command::GetFirmware => Packet::new_command(CMD_VERSION, &[]),
            Command::GetReceiverStatus => Packet::new_command(CMD_RECEIVER_ONLINE, &[]),
            Command::ReadRegister { addr, len } => {
                let body = [
                    0x00,
                    ((addr >> 8) & 0xFF) as u8,
                    (addr & 0xFF) as u8,
                    *len,
                ];
                Packet::new_command(CMD_READ_EEPROM, &body)
            }
            Command::WriteRegister { addr, data } => {
                let mut body = Vec::with_capacity(4 + data.len());
                body.push(0x00);
                body.push(((addr >> 8) & 0xFF) as u8);
                body.push((addr & 0xFF) as u8);
                body.push(data.len() as u8);
                body.extend_from_slice(data);
                Packet::new_command(CMD_WRITE_EEPROM, &body)
            }
            Command::SetProfile(profile_idx) => {
                let body = [0x00, 0x00, 0x00, 0x01, *profile_idx];
                Packet::new_command(CMD_SET_PROFILE, &body)
            }
        }
    }
}
