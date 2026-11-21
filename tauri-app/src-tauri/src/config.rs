use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

fn default_autostart() -> bool { false }
fn default_start_minimized() -> bool { false }
fn default_auto_reapply() -> bool { true }
fn default_shield_mode() -> String { "exclude".to_string() }
fn default_poll_interval_ms() -> u64 { 3000 }
fn default_theme() -> String { "violet".to_string() }
fn default_compact_mode() -> bool { false }
fn default_show_pid() -> bool { true }
fn default_confirm_batch() -> bool { false }
fn default_mpo_fix() -> bool { false }
fn default_self_stealth() -> bool { false }
fn default_widget_enabled() -> bool { false }
fn default_widget_style() -> String { "pill".to_string() }
fn default_widget_always_on_top() -> bool { true }
fn default_widget_stealth() -> bool { true }
fn default_audio_isolation_enabled() -> bool { false }

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
    #[serde(default = "default_mpo_fix")]
    pub mpo_fix: bool,
    #[serde(default = "default_self_stealth")]
    pub self_stealth: bool,
    #[serde(default = "default_widget_enabled")]
    pub widget_enabled: bool,
    #[serde(default = "default_widget_style")]
    pub widget_style: String,
    #[serde(default = "default_widget_always_on_top")]
    pub widget_always_on_top: bool,
    #[serde(default = "default_widget_stealth")]
    pub widget_stealth: bool,
    #[serde(default)]
    pub widget_x: Option<i32>,
    #[serde(default)]
    pub widget_y: Option<i32>,
    #[serde(default = "default_audio_isolation_enabled")]
    pub audio_isolation_enabled: bool,
    #[serde(default)]
    pub audio_stream_device_id: Option<String>,
    #[serde(default)]
    pub audio_monitor_device_id: Option<String>,
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
            mpo_fix: default_mpo_fix(),
            self_stealth: default_self_stealth(),
            widget_enabled: default_widget_enabled(),
            widget_style: default_widget_style(),
            widget_always_on_top: default_widget_always_on_top(),
            widget_stealth: default_widget_stealth(),
            widget_x: None,
            widget_y: None,
            audio_isolation_enabled: default_audio_isolation_enabled(),
            audio_stream_device_id: None,
            audio_monitor_device_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShieldConfig {
    #[serde(default)]
    pub shielded_exes: HashSet<String>,
    #[serde(default)]
    pub muted_exes: HashSet<String>,
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
    let mut config: ShieldConfig = std::fs::read_to_string(&path)
        .ok()
        .and_then(|d| serde_json::from_str(&d).ok())
        .unwrap_or_default();
    config.shielded_exes = config.shielded_exes.into_iter().map(|s| s.to_lowercase()).collect();
    config.muted_exes = config.muted_exes.into_iter().map(|s| s.to_lowercase()).collect();
    config
}

pub fn save_config(config: &ShieldConfig) {
    let path = config_path();
    if let Ok(data) = serde_json::to_string_pretty(config) {
        let tmp_path = path.with_extension("json.tmp");
        if std::fs::write(&tmp_path, &data).is_ok() {
            if std::fs::rename(&tmp_path, &path).is_ok() {
                return;
            }
            let _ = std::fs::remove_file(&tmp_path);
        }
        let _ = std::fs::write(path, data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_settings_defaults() {
        let s = AppSettings::default();
        assert_eq!(s.theme, "violet");
        assert!(s.widget_stealth);
        assert!(s.auto_reapply);
        assert_eq!(s.shield_mode, "exclude");
        assert!(!s.widget_enabled);
        assert!(s.widget_x.is_none());
        assert!(s.widget_y.is_none());
    }

    #[test]
    fn test_backwards_compatible_config_deserialization() {
        let legacy_json = r#"{
            "shielded_exes": ["Discord.exe", "WHATSAPP.EXE"],
            "settings": {
                "autostart": true,
                "shield_mode": "monitor",
                "theme": "cyberpunk"
            }
        }"#;
        let cfg: ShieldConfig = serde_json::from_str(legacy_json).expect("legacy config parses");
        assert!(cfg.settings.autostart);
        assert_eq!(cfg.settings.shield_mode, "monitor");
        assert_eq!(cfg.settings.theme, "cyberpunk");
        assert!(!cfg.settings.widget_enabled);
        assert!(cfg.settings.widget_stealth);
        assert!(cfg.settings.widget_always_on_top);
        assert!(cfg.settings.widget_x.is_none());
        assert!(cfg.settings.widget_y.is_none());
    }
}
