// StreamShield Core - Config Manager
pub struct AppConfig {
    pub auto_shield_on_stream: bool,
    pub poll_interval_ms: u64,
    pub target_processes: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            auto_shield_on_stream: true,
            poll_interval_ms: 200,
            target_processes: vec!["discord.exe".to_string(), "signal.exe".to_string()],
        }
    }
}
