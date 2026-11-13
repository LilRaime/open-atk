/*
 * Pure ATK protocol layer: commands, responses, packets, checksum, and validation.
 *
 * This crate does NOT depend on HID libraries or UI frameworks.
 */

pub mod battery;
pub mod buttons;
pub mod checksum;
pub mod commands;
pub mod constants;
pub mod dpi;
pub mod packet;
pub mod parser;
pub mod polling;
pub mod response;

pub use battery::BatteryStatus;
pub use buttons::{ButtonAction, MouseButton};
pub use checksum::*;
pub use commands::Command;
pub use constants::*;
pub use dpi::{
    decode_dpi_axis, encode_dpi_axis, pack_dpi_stage, unpack_dpi_stage, DpiConfig, DpiError,
    DpiStage, DpiValue,
};
pub use packet::{Packet, PacketError};
pub use parser::{resolve_model_name, ParseError, Parser};
pub use polling::{PollingRate, UnsupportedPollingRateError};
pub use response::{IdentityInfo, ReceiverInfo, Response};
