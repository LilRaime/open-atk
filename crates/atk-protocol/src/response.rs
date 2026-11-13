/* Responses decoded from incoming HID reports. */

use crate::battery::BatteryStatus;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityInfo {
    pub cid: u8,
    pub mid: u8,
    pub model_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiverInfo {
    pub online: bool,
    pub rf_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Response {
    Battery {
        status: BatteryStatus,
        voltage_mv: u16,
    },
    Identity(IdentityInfo),
    Firmware(String),
    Receiver(ReceiverInfo),
    RegisterData {
        addr: u16,
        data: Vec<u8>,
    },
    Ack,
    Raw(Vec<u8>),
}
