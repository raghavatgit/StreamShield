use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShieldConfig {
    #[serde(default)]
    pub shielded_exes: HashSet<String>,
    #[serde(default)]
    pub audio_shielded_exes: HashSet<String>,
    #[serde(default = "default_obs_port")]
    pub obs_port: u16,
    #[serde(default)]
    pub obs_password: Option<String>,
    #[serde(default = "default_true")]
    pub obs_auto_sync: bool,
}

fn default_obs_port() -> u16 { 4455 }
fn default_true() -> bool { true }

impl Default for ShieldConfig {
    fn default() -> Self {
        Self {
            shielded_exes: HashSet::new(),
            audio_shielded_exes: HashSet::new(),
            obs_port: 4455,
            obs_password: None,
            obs_auto_sync: true,
        }
    }
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