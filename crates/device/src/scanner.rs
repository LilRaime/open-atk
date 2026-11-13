/* Device discovery and model/connection mode resolution for ATK / VXE hardware. */

use atk_hid::{HidDeviceInfo, HidManager};
use atk_protocol::constants::{
    ATK_USAGE_PAGE_CONFIG, PRODUCT_ID_R1_PRO_MAX_WIRED, PRODUCT_ID_R1_PRO_MAX_WIRELESS,
    VENDOR_ID_ATK, VENDOR_ID_ATK_ALT,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionMode {
    Wired,
    WirelessDongle,
    Bluetooth,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct DiscoveredDevice {
    pub info: HidDeviceInfo,
    pub mode: ConnectionMode,
}

pub struct DeviceScanner;

impl DeviceScanner {
    /* Discovers all ATK / VXE devices, filtering specifically for the configuration collection (UsagePage 0xFF02) */
    pub fn scan_config_channels(hid_mgr: &HidManager) -> Vec<DiscoveredDevice> {
        let mut found = Vec::new();

        for dev in hid_mgr.list_devices() {
            let is_atk_vid = dev.vendor_id == VENDOR_ID_ATK || dev.vendor_id == VENDOR_ID_ATK_ALT;
            let is_config_channel = dev.usage_page == ATK_USAGE_PAGE_CONFIG;

            if is_atk_vid && is_config_channel {
                let path_lower = dev.path.to_lowercase();
                let is_bluetooth = path_lower.contains("bth") || path_lower.contains("bthenum");
                let is_wired_pid = dev.product_id == PRODUCT_ID_R1_PRO_MAX_WIRED
                    || dev.product_id == 0xF58F
                    || dev.product_id == 0xF589
                    || dev.product_id == 0xF585
                    || dev.product_id == 0xF581;

                let mode = if is_bluetooth {
                    ConnectionMode::Bluetooth
                } else if is_wired_pid {
                    ConnectionMode::Wired
                } else {
                    ConnectionMode::WirelessDongle
                };
                found.push(DiscoveredDevice { info: dev, mode });
            }
        }

        found
    }

    /* Resolves model name and connection mode from USB VID, PID, and product string */
    pub fn resolve_device_name_and_mode(
        _vid: u16,
        pid: u16,
        product_string: Option<&str>,
    ) -> (String, ConnectionMode) {
        let is_wired = pid == PRODUCT_ID_R1_PRO_MAX_WIRED
            || pid == 0xF58F
            || pid == 0xF589
            || pid == 0xF585
            || pid == 0xF581;

        let mode = if is_wired {
            ConnectionMode::Wired
        } else {
            ConnectionMode::WirelessDongle
        };

        /* If product string has a clear model name, use it (skip generic receiver / dongle names) */
        if let Some(prod) = product_string {
            let trimmed = prod.trim();
            let lower = trimmed.to_lowercase();
            let is_dongle_or_generic = lower == "usb device"
                || lower == "hid device"
                || lower.contains("dongle")
                || lower.contains("receiver")
                || lower.contains("wireless mouse")
                || lower.contains("2.4g")
                || lower.contains("nordicmouse")
                || lower.contains("compx");

            if !trimmed.is_empty() && !is_dongle_or_generic {
                return (trimmed.to_string(), mode);
            }
        }

        let name = match pid {
            0xF58E | 0xF58F => "VXE R1SE+".to_string(),
            0xF588 | 0xF589 => "VXE R1 PRO".to_string(),
            0xF584 | 0xF585 => "VXE R1".to_string(),
            0xF580 | 0xF581 => "VXE R1 SE".to_string(),
            0xF58C => "VXE R1 PRO MAX".to_string(),
            0xF58A | 0x1085 => "VXE Gaming Mouse".to_string(),
            _ => "ATK Gaming Mouse".to_string(),
        };

        (name, mode)
    }

    /* Discovers all candidate ATK VXE interfaces connected to the system */
    pub fn scan_all(hid_mgr: &HidManager) -> Vec<DiscoveredDevice> {
        let mut found = Vec::new();

        for dev in hid_mgr.list_devices() {
            if dev.vendor_id == VENDOR_ID_ATK || dev.vendor_id == VENDOR_ID_ATK_ALT {
                let mode = match dev.product_id {
                    PRODUCT_ID_R1_PRO_MAX_WIRED => ConnectionMode::Wired,
                    PRODUCT_ID_R1_PRO_MAX_WIRELESS => ConnectionMode::WirelessDongle,
                    _ => ConnectionMode::Unknown,
                };
                found.push(DiscoveredDevice { info: dev, mode });
            }
        }

        found
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_generic_dongle_string() {
        /* When Windows reports generic dongle name, scanner must reject it and use fallback PID name */
        let (name, mode) = DeviceScanner::resolve_device_name_and_mode(
            0x3554,
            0xF58A,
            Some("Wireless mouse -1k dongle"),
        );
        assert_eq!(mode, ConnectionMode::WirelessDongle);
        assert_eq!(name, "VXE Gaming Mouse");
    }

    #[test]
    fn test_wired_se_plus_resolution() {
        let (name, mode) = DeviceScanner::resolve_device_name_and_mode(
            0x3554,
            0xF58F,
            Some("USB Device"),
        );
        assert_eq!(mode, ConnectionMode::Wired);
        assert_eq!(name, "VXE R1SE+");
    }
}

