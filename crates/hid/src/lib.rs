/* Low-level USB HID transport abstraction over `hidapi`. */

use hidapi::{DeviceInfo as RawDeviceInfo, HidApi, HidDevice};
use thiserror::Error;
use tracing::{debug, trace};

#[derive(Error, Debug)]
pub enum HidError {
    #[error("HID API initialization failed: {0}")]
    ApiInit(#[from] hidapi::HidError),
    #[error("Device not found: VID {0:#06X}, PID {1:#06X}")]
    DeviceNotFound(u16, u16),
    #[error("Failed to open device at path {path}: {source}")]
    OpenFailed {
        path: String,
        source: hidapi::HidError,
    },
    #[error("Read timed out after {0} ms")]
    Timeout(i32),
    #[error("I/O error during HID transfer: {0}")]
    Io(String),
}

/* Clean information struct for an enumerated HID interface. */
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HidDeviceInfo {
    pub path: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub serial_number: Option<String>,
    pub release_number: u16,
    pub manufacturer_string: Option<String>,
    pub product_string: Option<String>,
    pub usage_page: u16,
    pub usage: u16,
    pub interface_number: i32,
}

impl From<&RawDeviceInfo> for HidDeviceInfo {
    fn from(info: &RawDeviceInfo) -> Self {
        let path = info.path().to_str().unwrap_or_default().to_string();
        Self {
            path,
            vendor_id: info.vendor_id(),
            product_id: info.product_id(),
            serial_number: info.serial_number().map(|s| s.to_string()),
            release_number: info.release_number(),
            manufacturer_string: info.manufacturer_string().map(|s| s.to_string()),
            product_string: info.product_string().map(|s| s.to_string()),
            usage_page: info.usage_page(),
            usage: info.usage(),
            interface_number: info.interface_number(),
        }
    }
}

pub struct HidManager {
    api: HidApi,
}

impl HidManager {
    pub fn new() -> Result<Self, HidError> {
        let api = HidApi::new()?;
        Ok(Self { api })
    }

    /* Refresh device cache */
    pub fn refresh(&mut self) -> Result<(), HidError> {
        self.api.refresh_devices()?;
        Ok(())
    }

    /* List all discovered HID interfaces in the system */
    pub fn list_devices(&self) -> Vec<HidDeviceInfo> {
        self.api.device_list().map(HidDeviceInfo::from).collect()
    }

    /* Find all interfaces matching the given VID and PID */
    pub fn find_by_vid_pid(&self, vid: u16, pid: u16) -> Vec<HidDeviceInfo> {
        self.list_devices()
            .into_iter()
            .filter(|d| d.vendor_id == vid && d.product_id == pid)
            .collect()
    }

    /* Open an enumerated HID device by its device path */
    pub fn open_path(&self, path: &str) -> Result<HidDeviceHandle, HidError> {
        debug!(path = %path, "Opening HID device");
        let cpath = std::ffi::CString::new(path).map_err(|e| HidError::Io(e.to_string()))?;
        let device = self
            .api
            .open_path(&cpath)
            .map_err(|source| HidError::OpenFailed {
                path: path.to_string(),
                source,
            })?;

        /* Match enumerated device list case-insensitively by path to retain VID, PID, product string */
        let info = self
            .api
            .device_list()
            .find(|d| d.path().to_string_lossy().eq_ignore_ascii_case(path))
            .map(HidDeviceInfo::from)
            .unwrap_or_else(|| HidDeviceInfo {
                path: path.to_string(),
                vendor_id: 0,
                product_id: 0,
                serial_number: None,
                release_number: 0,
                manufacturer_string: None,
                product_string: None,
                usage_page: 0,
                usage: 0,
                interface_number: -1,
            });

        Ok(HidDeviceHandle { device, info })
    }
}

pub struct HidDeviceHandle {
    device: HidDevice,
    pub info: HidDeviceInfo,
}

impl HidDeviceHandle {
    pub fn info(&self) -> &HidDeviceInfo {
        &self.info
    }

    /* Write report to HID device (first byte must be Report ID) */
    pub fn write(&self, data: &[u8]) -> Result<usize, HidError> {
        trace!(len = data.len(), "Writing HID report");
        self.device
            .write(data)
            .map_err(|e| HidError::Io(e.to_string()))
    }

    /* Read report with timeout in milliseconds */
    pub fn read_timeout(&self, buf: &mut [u8], timeout_ms: i32) -> Result<usize, HidError> {
        let read_bytes = self
            .device
            .read_timeout(buf, timeout_ms)
            .map_err(|e| HidError::Io(e.to_string()))?;

        if read_bytes == 0 && timeout_ms > 0 {
            return Err(HidError::Timeout(timeout_ms));
        }

        trace!(len = read_bytes, "Read HID report");
        Ok(read_bytes)
    }

    /* Send a Feature Report */
    pub fn send_feature_report(&self, data: &[u8]) -> Result<(), HidError> {
        self.device
            .send_feature_report(data)
            .map_err(|e| HidError::Io(e.to_string()))
    }

    /* Get a Feature Report */
    pub fn get_feature_report(&self, buf: &mut [u8]) -> Result<usize, HidError> {
        self.device
            .get_feature_report(buf)
            .map_err(|e| HidError::Io(e.to_string()))
    }
}

