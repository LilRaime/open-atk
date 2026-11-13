/* Local storage for user profiles and app configuration. */

use atk_profile::{Profile, ProfileError};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;
use tracing::info;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] toml::ser::Error),
    #[error("Deserialization error: {0}")]
    Deserialization(#[from] toml::de::Error),
    #[error("Profile validation failed: {0}")]
    Validation(#[from] ProfileError),
    #[error("Profile not found: {0}")]
    NotFound(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub active_profile_id: String,
    pub start_minimized: bool,
    pub close_to_tray: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            active_profile_id: "default".to_string(),
            start_minimized: false,
            close_to_tray: true,
        }
    }
}

pub struct StorageManager {
    base_dir: PathBuf,
}

impl StorageManager {
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }

    /* Default OS-specific config folder path */
    pub fn default_path() -> PathBuf {
        let base = dirs_next().unwrap_or_else(|| PathBuf::from("."));
        base.join("open-atk")
    }

    pub fn init(&self) -> Result<(), StorageError> {
        let profiles_dir = self.base_dir.join("profiles");
        fs::create_dir_all(&profiles_dir)?;
        Ok(())
    }

    pub fn save_profile(&self, profile: &Profile) -> Result<(), StorageError> {
        profile.validate()?;
        self.init()?;

        let file_path = self.base_dir.join("profiles").join(format!("{}.toml", profile.id));
        let content = toml::to_string_pretty(profile)?;
        fs::write(&file_path, content)?;

        info!(id = %profile.id, path = %file_path.display(), "Profile saved successfully");
        Ok(())
    }

    pub fn load_profile(&self, id: &str) -> Result<Profile, StorageError> {
        let file_path = self.base_dir.join("profiles").join(format!("{id}.toml"));
        if !file_path.exists() {
            return Err(StorageError::NotFound(id.to_string()));
        }

        let content = fs::read_to_string(&file_path)?;
        let profile: Profile = toml::from_str(&content)?;
        profile.validate()?;
        Ok(profile)
    }

    pub fn list_profiles(&self) -> Result<Vec<Profile>, StorageError> {
        self.init()?;
        let profiles_dir = self.base_dir.join("profiles");
        let mut profiles = Vec::new();

        for entry in fs::read_dir(profiles_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("toml") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(profile) = toml::from_str::<Profile>(&content) {
                        profiles.push(profile);
                    }
                }
            }
        }

        if profiles.is_empty() {
            let default_prof = Profile::new("default", "Default Profile")?;
            self.save_profile(&default_prof)?;
            profiles.push(default_prof);
        }

        Ok(profiles)
    }
}

fn dirs_next() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("APPDATA").map(PathBuf::from)
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config"))
    }
}

