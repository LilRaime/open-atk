/* High-level device discovery, connection management, and communication. */

pub mod scanner;
pub use scanner::{ConnectionMode, DeviceScanner, DiscoveredDevice};

use atk_hid::{HidDeviceHandle, HidDeviceInfo, HidError, HidManager};
use atk_protocol::{
    constants::{
        ADDR_ADVANCED, ADDR_BUTTON_BASE, ADDR_DPI_BASE, ADDR_DPI_LIGHT, ADDR_LIFT_OFF,
        ADDR_SENSOR_MODE, ADDR_STAGE_COLOR_BASE, ADDR_SYSTEM,
    },
    BatteryStatus, Command, IdentityInfo, Packet, ParseError, Parser, PollingRate, ReceiverInfo,
    Response,
};
use thiserror::Error;
use tracing::{info, warn};

#[derive(Error, Debug)]
pub enum DeviceError {
    #[error("HID transport error: {0}")]
    Hid(#[from] HidError),
    #[error("Protocol parse error: {0}")]
    Protocol(#[from] ParseError),
    #[error("No compatible ATK device found")]
    DeviceNotFound,
    #[error("Device is not connected")]
    NotConnected,
    #[error("Unexpected response from device")]
    UnexpectedResponse,
}

pub struct AtkDevice {
    handle: HidDeviceHandle,
    mode: ConnectionMode,
}

impl AtkDevice {
    pub fn open(hid_mgr: &HidManager, device_path: &str, mode: ConnectionMode) -> Result<Self, DeviceError> {
        let handle = hid_mgr.open_path(device_path)?;
        info!(
            path = %device_path,
            mode = ?mode,
            product = ?handle.info().product_string,
            "Successfully opened ATK device"
        );
        Ok(Self { handle, mode })
    }

    pub fn open_with_info(hid_mgr: &HidManager, info: HidDeviceInfo, mode: ConnectionMode) -> Result<Self, DeviceError> {
        let mut handle = hid_mgr.open_path(&info.path)?;
        handle.info = info;
        info!(
            path = %handle.info().path,
            mode = ?mode,
            product = ?handle.info().product_string,
            "Successfully opened ATK device with info"
        );
        Ok(Self { handle, mode })
    }

    pub fn mode(&self) -> ConnectionMode {
        self.mode
    }

    pub fn info(&self) -> &HidDeviceInfo {
        self.handle.info()
    }

    /* Send a command to the mouse and await response with a timeout */
    pub fn send_command(&self, cmd: Command, timeout_ms: i32) -> Result<Response, DeviceError> {
        /* Non-blocking drain of any stale packets before sending */
        let mut drain_buf = [0u8; 65];
        while self.handle.read_timeout(&mut drain_buf, 0).unwrap_or(0) > 0 {}

        let pkt = cmd.build_packet();
        let wire_buffer = pkt.to_hid_buffer();

        self.handle.write(&wire_buffer)?;

        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms.max(50) as u64);

        while start.elapsed() < timeout {
            let mut read_buf = [0u8; 65];
            let remaining_ms = (timeout.saturating_sub(start.elapsed()).as_millis() as i32).max(1);
            let bytes_read = match self.handle.read_timeout(&mut read_buf, remaining_ms) {
                Ok(n) if n > 0 => n,
                Ok(_) => continue,
                Err(e) => return Err(DeviceError::Hid(e)),
            };

            if let Ok(resp_pkt) = Packet::from_wire_bytes(&read_buf[..bytes_read]) {
                if let Ok(resp) = Parser::parse_packet(&resp_pkt) {
                    /* Match response strictly to the requested command type and register address */
                    match (&cmd, &resp) {
                        (
                            Command::ReadRegister { addr: req_addr, .. },
                            Response::RegisterData { addr: resp_addr, .. },
                        ) => {
                            if req_addr == resp_addr {
                                return Ok(resp);
                            }
                            continue;
                        }
                        (Command::ReadRegister { .. }, _) => continue,
                        (Command::GetBattery, Response::Battery { .. }) => return Ok(resp),
                        (Command::GetBattery, _) => continue,
                        (Command::GetIdentity, Response::Identity(_)) => return Ok(resp),
                        (Command::GetIdentity, _) => continue,
                        (Command::GetFirmware, Response::Firmware(_)) => return Ok(resp),
                        (Command::GetFirmware, _) => continue,
                        (Command::GetReceiverStatus, Response::Receiver(_)) => return Ok(resp),
                        (Command::GetReceiverStatus, _) => continue,
                        _ => return Ok(resp),
                    }
                }
            }
        }

        Err(DeviceError::UnexpectedResponse)
    }

    /* Safe read-only: Query battery status and voltage (with fast retry if dongle returns 0% on initial wake) */
    pub fn query_battery(&self) -> Result<(BatteryStatus, u16), DeviceError> {
        let mut last_ok = None;
        let mut last_err = DeviceError::UnexpectedResponse;
        for attempt in 0..3 {
            match self.send_command(Command::GetBattery, 250) {
                Ok(Response::Battery { status, voltage_mv }) => {
                    if status.percentage > 0 && status.percentage <= 100 {
                        return Ok((status, voltage_mv));
                    }
                    last_ok = Some((status, voltage_mv));
                }
                Ok(_) => {
                    last_err = DeviceError::UnexpectedResponse;
                }
                Err(e) => {
                    last_err = e;
                }
            }
            if attempt < 2 {
                std::thread::sleep(std::time::Duration::from_millis(25));
            }
        }
        if let Some(res) = last_ok {
            Ok(res)
        } else {
            warn!("Failed to query battery: {last_err}");
            Err(last_err)
        }
    }

    /* Safe read-only: Query device Model/Client Identity (CID, MID) */
    pub fn query_identity(&self) -> Result<IdentityInfo, DeviceError> {
        match self.send_command(Command::GetIdentity, 600) {
            Ok(Response::Identity(info)) => Ok(info),
            Ok(_) => Err(DeviceError::UnexpectedResponse),
            Err(e) => {
                warn!("Failed to query identity: {e}");
                Err(e)
            }
        }
    }

    /* Safe read-only: Query device firmware version */
    pub fn query_firmware(&self) -> Result<String, DeviceError> {
        match self.send_command(Command::GetFirmware, 600) {
            Ok(Response::Firmware(ver)) => Ok(ver),
            Ok(_) => Err(DeviceError::UnexpectedResponse),
            Err(e) => {
                warn!("Failed to query firmware: {e}");
                Err(e)
            }
        }
    }

    /* Safe read-only: Query receiver online status */
    pub fn query_receiver(&self) -> Result<ReceiverInfo, DeviceError> {
        match self.send_command(Command::GetReceiverStatus, 600) {
            Ok(Response::Receiver(info)) => Ok(info),
            Ok(_) => Err(DeviceError::UnexpectedResponse),
            Err(e) => {
                warn!("Failed to query receiver: {e}");
                Err(e)
            }
        }
    }

    /* Read EEPROM register chunk (up to 10 bytes) */
    pub fn read_register(&self, addr: u16, len: u8) -> Result<Vec<u8>, DeviceError> {
        match self.send_command(Command::ReadRegister { addr, len: len.min(10) }, 600) {
            Ok(Response::RegisterData { addr: resp_addr, data }) => {
                if resp_addr == addr {
                    Ok(data)
                } else {
                    Err(DeviceError::UnexpectedResponse)
                }
            }
            Ok(_) => Err(DeviceError::UnexpectedResponse),
            Err(e) => Err(e),
        }
    }

    /* Write EEPROM register chunk (up to 10 bytes) */
    pub fn write_register(&self, addr: u16, data: &[u8]) -> Result<(), DeviceError> {
        /* Non-blocking drain of any stale packets before sending */
        let mut drain_buf = [0u8; 65];
        while self.handle.read_timeout(&mut drain_buf, 0).unwrap_or(0) > 0 {}

        let pkt = Command::WriteRegister {
            addr,
            data: data.to_vec(),
        }
        .build_packet();
        self.handle.write(&pkt.to_hid_buffer())?;

        /* Read the ACK packet back from the device (up to 30ms) so USB buffer stays clean */
        let _ = self.handle.read_timeout(&mut drain_buf, 30);
        Ok(())
    }

    /* Set active DPI stage (0-indexed) */
    pub fn set_active_stage(&self, stage_idx: u8) -> Result<(), DeviceError> {
        let pair = [stage_idx, 0x55u8.wrapping_sub(stage_idx)];
        self.write_register(ADDR_SYSTEM + 4, &pair)
    }

    /* Set total active DPI stage count (1 to 6) */
    pub fn set_stage_count(&self, count: u8) -> Result<(), DeviceError> {
        let count_clamped = count.clamp(1, 6);
        let pair = [count_clamped, 0x55u8.wrapping_sub(count_clamped)];
        self.write_register(ADDR_SYSTEM + 2, &pair)
    }

    /* Set DPI for a specific stage (0-indexed) */
    pub fn set_stage_dpi(&self, stage_idx: u8, x: u32, y: u32) -> Result<(), DeviceError> {
        let packed = atk_protocol::pack_dpi_stage(x, y);
        let addr = ADDR_DPI_BASE + (stage_idx as u16) * 4;
        self.write_register(addr, &packed)
    }

    /* Set RGB color for a DPI stage (0-indexed, 4-byte EEPROM record inside an 8-byte two-stage group at 0x002C) */
    pub fn set_stage_color(&self, stage_idx: u8, r: u8, g: u8, b: u8) -> Result<(), DeviceError> {
        let sum = r.wrapping_add(g).wrapping_add(b);
        let check = 0x55u8.wrapping_sub(sum);
        let record = [r, g, b, check];
        let group_addr = ADDR_STAGE_COLOR_BASE + ((stage_idx as u16) / 2) * 8;
        let slot_offset = ((stage_idx as usize) % 2) * 4;
        if let Ok(mut group) = self.read_register(group_addr, 8) {
            if group.len() >= 8 {
                group[slot_offset..slot_offset + 4].copy_from_slice(&record);
                return self.write_register(group_addr, &group[..8]);
            }
        }
        let addr = ADDR_STAGE_COLOR_BASE + (stage_idx as u16) * 4;
        self.write_register(addr, &record)
    }

    /* Read stage RGB colors from EEPROM (stored in 8-byte groups of two 4-byte records at 0x002C) */
    pub fn read_stage_colors(&self, count: u8) -> Result<Vec<(u8, u8, u8)>, DeviceError> {
        let default_colors = [
            (155, 93, 229),  /* Stage 1: Purple */
            (255, 255, 255), /* Stage 2: White */
            (0, 180, 255),   /* Stage 3: Cyan/Blue */
            (255, 0, 128),   /* Stage 4: Magenta/Pink */
            (0, 230, 118),   /* Stage 5: Green */
            (255, 214, 0),   /* Stage 6: Yellow */
        ];
        let total = count.clamp(1, 6) as usize;
        let mut colors = Vec::with_capacity(total);
        let groups = (total + 1) / 2;
        for g in 0..groups {
            let group_addr = ADDR_STAGE_COLOR_BASE + (g as u16) * 8;
            let raw_group = self.read_register(group_addr, 8).ok();
            for slot in 0..2 {
                let idx = g * 2 + slot;
                if idx >= total {
                    break;
                }
                let mut found = None;
                if let Some(ref grp) = raw_group {
                    let offset = slot * 4;
                    if grp.len() >= offset + 4 {
                        let rec = &grp[offset..offset + 4];
                        let valid = rec[0]
                            .wrapping_add(rec[1])
                            .wrapping_add(rec[2])
                            .wrapping_add(rec[3])
                            == 0x55;
                        if valid {
                            found = Some((rec[0], rec[1], rec[2]));
                        }
                    }
                }
                if let Some(rgb) = found {
                    colors.push(rgb);
                } else {
                    let def = default_colors.get(idx).copied().unwrap_or((255, 255, 255));
                    colors.push(def);
                }
            }
        }
        Ok(colors)
    }

    /* Set polling rate */
    pub fn set_polling_rate(&self, rate: PollingRate) -> Result<(), DeviceError> {
        let code = match rate {
            PollingRate::Hz125 => 0x08,
            PollingRate::Hz250 => 0x04,
            PollingRate::Hz500 => 0x02,
            PollingRate::Hz1000 => 0x01,
            PollingRate::Hz2000 => 0x10,
            PollingRate::Hz4000 => 0x20,
            PollingRate::Hz8000 => 0x40,
        };

        let pair = [code, 0x55u8.wrapping_sub(code)];
        self.write_register(ADDR_SYSTEM, &pair)
    }

    /* Set Lift-off Distance (LOD): 1 for 1.0 mm (Low/Standard), 2 for 2.0 mm (High) */
    pub fn set_lod(&self, lod_code: u8) -> Result<(), DeviceError> {
        let pair = [lod_code, 0x55u8.wrapping_sub(lod_code)];
        self.write_register(ADDR_LIFT_OFF, &pair)
    }

    /* Read Lift-off Distance (LOD) code: 1 = 1.0 mm, 2 = 2.0 mm */
    pub fn get_lod(&self) -> Result<u8, DeviceError> {
        let data = self.read_register(ADDR_LIFT_OFF, 2)?;
        if data.len() >= 2 && data[0].wrapping_add(data[1]) == 0x55 {
            Ok(data[0])
        } else {
            Err(DeviceError::UnexpectedResponse)
        }
    }

    /* Update a 2-byte [val, 0x55 - val] pair inside the 10-byte ADDR_ADVANCED (0x00A9) block */
    fn write_advanced_pair(&self, offset: usize, val: u8) -> Result<(), DeviceError> {
        let pair = [val, 0x55u8.wrapping_sub(val)];
        if let Ok(mut block) = self.read_register(ADDR_ADVANCED, 10) {
            if block.len() >= 10 && offset + 2 <= 10 {
                block[offset..offset + 2].copy_from_slice(&pair);
                return self.write_register(ADDR_ADVANCED, &block[..10]);
            }
        }
        self.write_register(ADDR_ADVANCED + (offset as u16), &pair)
    }

    /* Set switch debounce stabilization delay (0 to 20 ms) at ADDR_ADVANCED offset 0 */
    pub fn set_debounce(&self, debounce_ms: u8) -> Result<(), DeviceError> {
        self.write_advanced_pair(0, debounce_ms)
    }

    /* Enable or disable sensor Motion Synchronization at ADDR_ADVANCED offset 2 */
    pub fn set_motion_sync(&self, enabled: bool) -> Result<(), DeviceError> {
        self.write_advanced_pair(2, if enabled { 1 } else { 0 })
    }

    /* Enable or disable Straight Line Correction (Angle Snapping) at ADDR_ADVANCED offset 6 */
    pub fn set_straight_line(&self, enabled: bool) -> Result<(), DeviceError> {
        self.write_advanced_pair(6, if enabled { 1 } else { 0 })
    }

    /* Enable or disable Ripple Correction (Anti-jitter smoothing) at ADDR_ADVANCED offset 8 */
    pub fn set_ripple_control(&self, enabled: bool) -> Result<(), DeviceError> {
        self.write_advanced_pair(8, if enabled { 1 } else { 0 })
    }

    /* Enable or disable Ultra Long Range / High Performance Sensor Mode at ADDR_SENSOR_MODE (0x00B5) */
    pub fn set_ultra_long_range(&self, enabled: bool) -> Result<(), DeviceError> {
        let mode = if enabled { 1u8 } else { 0u8 };
        let pair = [mode, 0x55u8.wrapping_sub(mode)];
        if let Ok(mut row) = self.read_register(ADDR_SENSOR_MODE, 6) {
            if row.len() >= 6 {
                row[4..6].copy_from_slice(&pair);
                return self.write_register(ADDR_SENSOR_MODE, &row[..6]);
            }
        }
        let fallback = [0x00, 0x55, 0x06, 0x4F, pair[0], pair[1]];
        self.write_register(ADDR_SENSOR_MODE, &fallback)
    }

    /* Set Sleep Time setting code (0: 30s, 1: 1m, 2: 2m, 3: 3m, 4: 5m, 5: 20m, 6: 25m, 7: 30m) at ADDR_ADVANCED offset 4 (10s units) */
    pub fn set_sleep_time(&self, code: u8) -> Result<(), DeviceError> {
        let units = match code {
            0 => 3,   /* 30s */
            1 => 6,   /* 1min (60s) */
            2 => 12,  /* 2min (120s) */
            3 => 18,  /* 3min (180s) */
            4 => 30,  /* 5min (300s) */
            5 => 120, /* 20min (1200s) */
            6 => 150, /* 25min (1500s) */
            7 => 180, /* 30min (1800s) */
            u => u,
        };
        self.write_advanced_pair(4, units)
    }

    /* Set DPI indicator LED effect (mode: 0=Close, 1=Always On, 2=Breathing; brightness: 0..=2; speed: 0..=2) */
    pub fn set_dpi_light(&self, mode: u8, brightness: u8, speed: u8) -> Result<(), DeviceError> {
        let effect = match mode {
            0 => 0u8,
            2 => 2u8,
            _ => 1u8,
        };
        let bright_raw = if effect == 0 {
            0u8
        } else {
            match brightness {
                0 => 0x10u8,
                1 => 0x80u8,
                _ => 0xFFu8,
            }
        };
        let speed_raw = match speed {
            0 => 1u8,
            1 => 3u8,
            _ => 5u8,
        };
        let enabled = if effect == 0 { 0u8 } else { 1u8 };
        let block = [
            effect,
            0x55u8.wrapping_sub(effect),
            bright_raw,
            0x55u8.wrapping_sub(bright_raw),
            speed_raw,
            0x55u8.wrapping_sub(speed_raw),
            enabled,
            0x55u8.wrapping_sub(enabled),
        ];
        self.write_register(ADDR_DPI_LIGHT, &block)
    }

    /* Read DPI indicator LED settings from 0x004C (8-byte block of 4 value/checksum pairs) */
    pub fn get_dpi_light(&self) -> Result<(u8, u8, u8), DeviceError> {
        let raw = self.read_register(ADDR_DPI_LIGHT, 8)?;
        if raw.len() < 8 {
            return Err(DeviceError::UnexpectedResponse);
        }
        let decode_pair = |v: u8, c: u8| -> Option<u8> {
            if v == 0 && c == 0 {
                Some(0)
            } else if v.wrapping_add(c) == 0x55 {
                Some(v)
            } else {
                None
            }
        };
        let effect = decode_pair(raw[0], raw[1]).unwrap_or(0);
        let bright_raw = decode_pair(raw[2], raw[3]).unwrap_or(0xFF);
        let speed_raw = decode_pair(raw[4], raw[5]).unwrap_or(3);
        let enabled = decode_pair(raw[6], raw[7]).unwrap_or(0);

        let mode = if effect == 0 || enabled == 0 {
            0
        } else if effect == 2 {
            2
        } else {
            1
        };
        let brightness = if bright_raw == 0 {
            2
        } else if bright_raw <= 0x10 {
            0
        } else if bright_raw <= 0x80 {
            1
        } else {
            2
        };
        let speed = if speed_raw >= 5 {
            2
        } else if speed_raw >= 3 {
            1
        } else {
            0
        };
        Ok((mode, brightness, speed))
    }

    /* Read advanced hardware parameters: returns (debounce_ms, motion_sync, ripple_control) */
    pub fn get_advanced_settings(&self) -> Result<(u8, bool, bool), DeviceError> {
        let adv = self.read_register(ADDR_ADVANCED, 10)?;
        if adv.len() >= 10 {
            let debounce = adv[0];
            let motion_sync = adv[2] == 1;
            let ripple = adv[8] == 1;
            Ok((debounce, motion_sync, ripple))
        } else {
            Err(DeviceError::UnexpectedResponse)
        }
    }

    /* Read all advanced parameters: (debounce_ms, motion_sync, straight_line, ripple, ultra_long_range) */
    pub fn get_all_advanced_settings(&self) -> Result<(u8, bool, bool, bool, bool), DeviceError> {
        let adv = self.read_register(ADDR_ADVANCED, 10)?;
        if adv.len() >= 10 {
            let ultra = self
                .read_register(ADDR_SENSOR_MODE, 6)
                .ok()
                .and_then(|r| if r.len() >= 6 && r[4].wrapping_add(r[5]) == 0x55 { Some(r[4] != 0) } else { None })
                .unwrap_or(false);
            Ok((
                adv[0],
                adv[2] == 1,
                adv[6] == 1,
                adv[8] == 1,
                ultra,
            ))
        } else {
            Err(DeviceError::UnexpectedResponse)
        }
    }

    /* Read Sleep Time in seconds from ADDR_ADVANCED offset 4 (10s units) */
    pub fn get_sleep_time_seconds(&self) -> Result<u16, DeviceError> {
        let adv = self.read_register(ADDR_ADVANCED, 10)?;
        if adv.len() >= 6 {
            Ok((adv[4] as u16) * 10)
        } else {
            Err(DeviceError::UnexpectedResponse)
        }
    }

    /* Safe no-op: Compx Nordic MCU automatically commits register writes to flash memory */
    pub fn commit_profile(&self) -> Result<(), DeviceError> {
        Ok(())
    }

    /* Write a 4-byte button mapping record to EEPROM (ADDR_BUTTON_BASE + btn_idx * 4) */
    pub fn set_button_mapping(&self, btn_index: u8, key_class: u8, val1: u8, val2: u8) -> Result<(), DeviceError> {
        let sum = key_class.wrapping_add(val1).wrapping_add(val2);
        let check = 0x55u8.wrapping_sub(sum);
        let record = [key_class, val1, val2, check];
        let addr = ADDR_BUTTON_BASE + (btn_index as u16) * 4;
        self.write_register(addr, &record)
    }

    /* Read all 6 button mappings from EEPROM */
    pub fn read_button_mappings(&self) -> Result<Vec<(u8, u8, u8)>, DeviceError> {
        let mut mappings = Vec::new();
        for i in 0..6 {
            let addr = ADDR_BUTTON_BASE + (i as u16) * 4;
            if let Ok(rec) = self.read_register(addr, 4) {
                if rec.len() >= 4 {
                    let valid = rec[0].wrapping_add(rec[1]).wrapping_add(rec[2]).wrapping_add(rec[3]) == 0x55;
                    if valid {
                        mappings.push((rec[0], rec[1], rec[2]));
                        continue;
                    }
                }
            }
            mappings.push((0x01, 1 << i, 0));
        }
        Ok(mappings)
    }
}
