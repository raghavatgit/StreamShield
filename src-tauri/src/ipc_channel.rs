// StreamShield Core - IPC Channel
use std::sync::mpsc::{channel, Receiver, Sender};

pub struct TelemetryMessage {
    pub window_count: usize,
    pub masked_count: usize,
    pub timestamp_ms: u64,
}

pub fn create_telemetry_channel() -> (Sender<TelemetryMessage>, Receiver<TelemetryMessage>) {
    channel()
}
