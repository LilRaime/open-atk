/* Application Core: state orchestration, device management, and high-level use cases. */

use atk_device::{AtkDevice, ConnectionMode, DeviceScanner};
use atk_hid::HidManager;
use atk_profile::Profile;
use atk_protocol::BatteryStatus;
use atk_storage::{AppConfig, StorageManager};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tracing::info;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Device error: {0}")]
    Device(#[from] atk_device::DeviceError),
    #[error("Storage error: {0}")]
    Storage(#[from] atk_storage::StorageError),
    #[error("HID error: {0}")]
    Hid(#[from] atk_hid::HidError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceStatus {
    Disconnected,
    Connecting,
    Connected {
        mode: ConnectionMode,
        product_name: String,
        battery: Option<BatteryStatus>,
    },
}

pub struct AppState {
    pub device_status: DeviceStatus,
    pub active_profile: Profile,
    pub profiles: Vec<Profile>,
    pub config: AppConfig,
}

pub struct CoreService {
    hid_mgr: Arc<Mutex<HidManager>>,
    _storage_mgr: StorageManager,
    state: Arc<Mutex<AppState>>,
    device: Option<AtkDevice>,
}

impl CoreService {
    pub fn new() -> Result<Self, CoreError> {
        let hid_mgr = Arc::new(Mutex::new(HidManager::new()?));
        let storage_mgr = StorageManager::new(StorageManager::default_path());
        let profiles = storage_mgr.list_profiles()?;
        let active_profile = profiles.first().cloned().unwrap_or_else(|| {
            Profile::new("default", "Default Profile").expect("Valid default profile")
        });

        let state = Arc::new(Mutex::new(AppState {
            device_status: DeviceStatus::Disconnected,
            active_profile,
            profiles,
            config: AppConfig::default(),
        }));

        Ok(Self {
            hid_mgr,
            _storage_mgr: storage_mgr,
            state,
            device: None,
        })
    }

    pub fn state(&self) -> Arc<Mutex<AppState>> {
        Arc::clone(&self.state)
    }

    /* Scan for connected ATK VXE devices and connect to the configuration channel */
    pub fn scan_and_connect(&mut self) -> Result<bool, CoreError> {
        let mgr = self.hid_mgr.lock().unwrap();
        let devices = DeviceScanner::scan_config_channels(&mgr);

        if let Some(target) = devices.first() {
            info!(path = %target.info.path, mode = ?target.mode, "Found config channel, connecting...");
            let atk_dev = AtkDevice::open(&mgr, &target.info.path, target.mode)?;
            let battery = atk_dev.query_battery().ok().map(|(b, _)| b);

            let mut state = self.state.lock().unwrap();
            state.device_status = DeviceStatus::Connected {
                mode: target.mode,
                product_name: target.info.product_string.clone().unwrap_or_else(|| "ATK Mouse".into()),
                battery,
            };

            self.device = Some(atk_dev);
            Ok(true)
        } else {
            let mut state = self.state.lock().unwrap();
            state.device_status = DeviceStatus::Disconnected;
            self.device = None;
            Ok(false)
        }
    }
}

