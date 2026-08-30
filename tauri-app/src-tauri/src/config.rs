use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

fn default_autostart() -> bool { false }
fn default_start_minimized() -> bool { false }
fn default_auto_reapply() -> bool { true }
fn default_shield_mode() -> String { "exclude".to_string() }
fn default_poll_interval_ms() -> u64 { 3000 }
fn default_theme() -> String { "cyberpunk".to_string() }
fn default_compact_mode() -> bool { false }
fn default_show_pid() -> bool { true }
fn default_confirm_batch() -> bool { false }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default = "default_autostart")]
    pub autostart: bool,
    #[serde(default = "default_start_minimized")]
    pub start_minimized: bool,
    #[serde(default = "default_auto_reapply")]
    pub auto_reapply: bool,
    #[serde(default = "default_shield_mode")]
    pub shield_mode: String,
    #[serde(default = "default_poll_interval_ms")]
    pub poll_interval_ms: u64,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_compact_mode")]
    pub compact_mode: bool,
    #[serde(default = "default_show_pid")]
    pub show_pid: bool,
    #[serde(default = "default_confirm_batch")]
    pub confirm_batch: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            autostart: default_autostart(),
            start_minimized: default_start_minimized(),
            auto_reapply: default_auto_reapply(),
            shield_mode: default_shield_mode(),
            poll_interval_ms: default_poll_interval_ms(),
            theme: default_theme(),
            compact_mode: default_compact_mode(),
            show_pid: default_show_pid(),
            confirm_batch: default_confirm_batch(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShieldConfig {
    #[serde(default)]
    pub shielded_exes: HashSet<String>,
    #[serde(default)]
    pub settings: AppSettings,
}

fn config_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("StreamShield");
    std::fs::create_dir_all(&path).ok();
    path.push("config.json");
    path
}

pub fn load_config() -> ShieldConfig {
    let path = config_path();
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|d| serde_json::from_str(&d).ok())
        .unwrap_or_default()
}

pub fn save_config(config: &ShieldConfig) {
    let path = config_path();
    if let Ok(data) = serde_json::to_string_pretty(config) {
        let _ = std::fs::write(path, data);
    }
}