use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default = "default_server")]
    pub server: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_rate")]
    pub rate: u32,
    #[serde(default = "default_channels")]
    pub channels: u16,
    #[serde(default)]
    pub device_id: Option<String>,
    #[serde(default)]
    pub auto_connect: bool,
    #[serde(default = "default_true")]
    pub start_with_windows: bool,
    #[serde(default = "default_true")]
    pub minimize_to_tray: bool,
    #[serde(default)]
    pub mute_local_output: bool,
    #[serde(default = "default_true")]
    pub dark_theme: bool,
    #[serde(default = "default_capture_mode")]
    pub capture_mode: String,
}

fn default_server() -> String {
    String::new()
}
fn default_port() -> u16 {
    4714
}
fn default_rate() -> u32 {
    48000
}
fn default_channels() -> u16 {
    2
}
fn default_true() -> bool {
    true
}
fn default_capture_mode() -> String {
    "loopback".to_string()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            server: default_server(),
            port: default_port(),
            rate: default_rate(),
            channels: default_channels(),
            device_id: None,
            auto_connect: false,
            start_with_windows: true,
            minimize_to_tray: true,
            mute_local_output: false,
            dark_theme: true,
            capture_mode: default_capture_mode(),
        }
    }
}

impl AppSettings {
    fn settings_dir() -> Option<PathBuf> {
        directories::ProjectDirs::from("", "", "PulseStream")
            .map(|d| d.data_local_dir().to_path_buf())
    }

    fn settings_path() -> Option<PathBuf> {
        Self::settings_dir().map(|d| d.join("settings.json"))
    }

    pub fn load() -> Self {
        Self::settings_path()
            .and_then(|p| fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let dir = Self::settings_dir().ok_or("no settings dir")?;
        fs::create_dir_all(&dir)?;
        let path = dir.join("settings.json");
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }
}
