/* Persistent user preferences, OS startup integration, and desktop notifications. */

#![allow(dead_code)]

use crate::models::MouseColor;

/* Returns the OS-appropriate path for persistent user preferences (JSON) */
fn get_prefs_path() -> std::path::PathBuf {
    if let Some(appdata) = std::env::var_os("APPDATA") {
        let dir = std::path::PathBuf::from(appdata).join("open-atk");
        let _ = std::fs::create_dir_all(&dir);
        dir.join("preferences.json")
    } else {
        std::path::PathBuf::from("open_atk_preferences.json")
    }
}

fn default_true() -> bool {
    true
}

/* User preferences persisted to disk */
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppPreferences {
    /* Maps device ID or model name to preferred mouse color */
    #[serde(default)]
    pub device_colors: std::collections::HashMap<String, MouseColor>,
    /* Minimize to system tray on window close button click */
    #[serde(default = "default_true")]
    pub minimize_to_tray: bool,
    /* Launch on operating system startup */
    #[serde(default)]
    pub run_on_startup: bool,
    /* Desktop toast notification when mouse battery drops below 15% */
    #[serde(default = "default_true")]
    pub low_battery_warning: bool,
}

impl Default for AppPreferences {
    fn default() -> Self {
        Self {
            device_colors: std::collections::HashMap::new(),
            minimize_to_tray: true,
            run_on_startup: false,
            low_battery_warning: true,
        }
    }
}

impl AppPreferences {
    pub fn load() -> Self {
        let path = get_prefs_path();
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(prefs) = serde_json::from_str::<AppPreferences>(&content) {
                return prefs;
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        let path = get_prefs_path();
        if let Ok(content) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(&path, content);
        }
    }

    pub fn get_color(&self, id: &str, model_name: &str) -> MouseColor {
        /* Prioritize model_name so color persists seamlessly across wired and wireless connections */
        if !model_name.is_empty() {
            if let Some(&color) = self.device_colors.get(model_name) {
                return color;
            }
        }
        if let Some(&color) = self.device_colors.get(id) {
            color
        } else {
            MouseColor::White
        }
    }

    pub fn set_device_color(&mut self, id: &str, model_name: &str, color: MouseColor) {
        if !id.is_empty() {
            self.device_colors.insert(id.to_string(), color);
        }
        if !model_name.is_empty() {
            self.device_colors.insert(model_name.to_string(), color);
        }
        self.save();
    }

    /* Enables or disables automatic launch on operating system startup */
    pub fn apply_startup(&mut self, enabled: bool) {
        self.run_on_startup = enabled;
        self.save();
        set_system_autorun(enabled);
    }
}

/* Configure operating system startup registry or desktop file */
pub fn set_system_autorun(enabled: bool) {
    #[cfg(windows)]
    {
        if let Ok(exe_path) = std::env::current_exe() {
            let exe_str = exe_path.to_string_lossy();
            if enabled {
                let val = format!("\"{}\" --minimized", exe_str);
                let _ = std::process::Command::new("reg")
                    .args([
                        "add",
                        "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                        "/v",
                        "OpenATK",
                        "/t",
                        "REG_SZ",
                        "/d",
                        &val,
                        "/f",
                    ])
                    .status();
            } else {
                let _ = std::process::Command::new("reg")
                    .args([
                        "delete",
                        "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                        "/v",
                        "OpenATK",
                        "/f",
                    ])
                    .status();
            }
        }
    }

    #[cfg(not(windows))]
    {
        if let Some(config_dir) = std::env::var_os("XDG_CONFIG_HOME")
            .map(std::path::PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".config"))
            })
        {
            let autostart_dir = config_dir.join("autostart");
            let _ = std::fs::create_dir_all(&autostart_dir);
            let dest = autostart_dir.join("open-atk.desktop");
            if enabled {
                let desktop_entry = "[Desktop Entry]\nName=OpenATK\nExec=open-atk --minimized\nType=Application\nHidden=false\n";
                let _ = std::fs::write(dest, desktop_entry);
            } else {
                let _ = std::fs::remove_file(dest);
            }
        }
    }
}

/* Displays a native desktop notification */
pub fn show_desktop_notification(title: &str, message: &str) {
    #[cfg(windows)]
    {
        let script = format!(
            "[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] > $null; \
             $template = [Windows.UI.Notifications.ToastNotificationManager]::GetTemplateContent([Windows.UI.Notifications.ToastTemplateType]::ToastText02); \
             $textNodes = $template.GetElementsByTagName('text'); \
             $textNodes.Item(0).AppendChild($template.CreateTextNode('{}')) > $null; \
             $textNodes.Item(1).AppendChild($template.CreateTextNode('{}')) > $null; \
             $toast = [Windows.UI.Notifications.ToastNotification]::new($template); \
             [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('OpenATK').Show($toast);",
            title.replace('\'', "''"),
            message.replace('\'', "''")
        );
        let _ = std::process::Command::new("powershell")
            .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &script])
            .spawn();
    }

    #[cfg(not(windows))]
    {
        let _ = std::process::Command::new("notify-send")
            .args(["-u", "critical", "-a", "OpenATK", title, message])
            .spawn();
    }
}
