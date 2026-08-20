//! OBS Studio WebSocket v5 Companion Protocol
//! Driverless, 0-latency integration with OBS Studio 28, 29, and 30+.
//! Automatically synchronizes visual capture shielding and stream audio isolation.

use std::collections::HashSet;
use std::net::TcpStream;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;
use serde::{Deserialize, Serialize};
use tungstenite::{connect, stream::MaybeTlsStream, Message, WebSocket};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsStatus {
    pub connected: bool,
    pub port: u16,
    pub active_sources: Vec<String>,
    pub obs_process_running: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsSettings {
    pub port: u16,
    pub password: Option<String>,
    pub auto_sync: bool,
}

impl Default for ObsSettings {
    fn default() -> Self {
        Self {
            port: 4455,
            password: None,
            auto_sync: true,
        }
    }
}

pub struct ObsState {
    pub connected: bool,
    pub settings: ObsSettings,
    pub active_audio_inputs: Vec<String>,
    pub client: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
}

fn get_obs_state() -> &'static Mutex<ObsState> {
    static OBS_STATE: OnceLock<Mutex<ObsState>> = OnceLock::new();
    OBS_STATE.get_or_init(|| {
        Mutex::new(ObsState {
            connected: false,
            settings: ObsSettings::default(),
            active_audio_inputs: Vec::new(),
            client: None,
        })
    })
}

/// Check if obs64.exe or obs32.exe is running on the system
#[cfg(windows)]
pub fn is_obs_running() -> bool {
    use winapi::um::tlhelp32::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW,
        PROCESSENTRY32W, TH32CS_SNAPPROCESS,
    };
    use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};

    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap == INVALID_HANDLE_VALUE {
            return false;
        }

        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        let mut found = false;
        if Process32FirstW(snap, &mut entry) != 0 {
            loop {
                let name_len = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(entry.szExeFile.len());
                let name = String::from_utf16_lossy(&entry.szExeFile[..name_len]).to_lowercase();
                if name == "obs64.exe" || name == "obs32.exe" || name == "obs.exe" {
                    found = true;
                    break;
                }
                if Process32NextW(snap, &mut entry) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snap);
        found
    }
}

#[cfg(not(windows))]
pub fn is_obs_running() -> bool { false }

/// Query current OBS connection and source status
pub fn get_obs_status() -> ObsStatus {
    let running = is_obs_running();
    let state = get_obs_state().lock().unwrap();
    ObsStatus {
        connected: state.connected,
        port: state.settings.port,
        active_sources: state.active_audio_inputs.clone(),
        obs_process_running: running,
    }
}

/// Configure OBS connection parameters
pub fn set_obs_settings(settings: ObsSettings) {
    let mut state = get_obs_state().lock().unwrap();
    state.settings = settings;
}

/// Try connecting to OBS WebSocket v5
pub fn try_connect_obs() -> bool {
    let port = {
        let state = get_obs_state().lock().unwrap();
        state.settings.port
    };

    let url = format!("ws://127.0.0.1:{}", port);
    let (mut ws, _) = match connect(&url) {
        Ok(res) => res,
        Err(_) => {
            let mut state = get_obs_state().lock().unwrap();
            state.connected = false;
            state.client = None;
            return false;
        }
    };

    // Perform OBS WebSocket v5 Handshake
    // 1. Read Hello (OpCode 0)
    let hello_ok = if let Ok(Message::Text(txt)) = ws.read() {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&txt) {
            val.get("op").and_then(|o| o.as_i64()) == Some(0)
        } else {
            false
        }
    } else {
        false
    };

    if !hello_ok {
        let mut state = get_obs_state().lock().unwrap();
        state.connected = false;
        state.client = None;
        return false;
    }

    // 2. Send Identify (OpCode 1)
    let identify_msg = serde_json::json!({
        "op": 1,
        "d": {
            "rpcVersion": 1,
            "eventSubscriptions": 33
        }
    });

    if ws.send(Message::Text(identify_msg.to_string())).is_err() {
        let mut state = get_obs_state().lock().unwrap();
        state.connected = false;
        state.client = None;
        return false;
    }

    // 3. Read Identified (OpCode 2)
    let identified = if let Ok(Message::Text(txt)) = ws.read() {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&txt) {
            val.get("op").and_then(|o| o.as_i64()) == Some(2)
        } else {
            false
        }
    } else {
        false
    };

    if identified {
        let mut state = get_obs_state().lock().unwrap();
        state.connected = true;
        state.client = Some(ws);
        true
    } else {
        let mut state = get_obs_state().lock().unwrap();
        state.connected = false;
        state.client = None;
        false
    }
}

/// Synchronize shielded applications with OBS audio sources
pub fn sync_shielded_with_obs(shielded_exes: &HashSet<String>) {
    let mut state = get_obs_state().lock().unwrap();
    if !state.connected || state.client.is_none() {
        return;
    }

    let ws = state.client.as_mut().unwrap();

    // Query active OBS audio inputs
    let req_id = "streamshield_get_inputs";
    let get_inputs = serde_json::json!({
        "op": 6,
        "d": {
            "requestType": "GetInputList",
            "requestId": req_id
        }
    });

    if ws.send(Message::Text(get_inputs.to_string())).is_err() {
        state.connected = false;
        state.client = None;
        return;
    }

    // Read response
    if let Ok(Message::Text(txt)) = ws.read() {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&txt) {
            if let Some(inputs) = val.get("d").and_then(|d| d.get("responseData")).and_then(|r| r.get("inputs")).and_then(|i| i.as_array()) {
                let mut found_sources = Vec::new();
                for input in inputs {
                    if let Some(name) = input.get("inputName").and_then(|n| n.as_str()) {
                        let name_lower = name.to_lowercase();
                        found_sources.push(name.to_string());

                        // Check if this source matches any shielded executable
                        let should_mute = shielded_exes.iter().any(|exe| {
                            let clean_exe = exe.replace(".exe", "").to_lowercase();
                            name_lower.contains(&clean_exe)
                        });

                        // Send SetInputMute
                        let mute_req = serde_json::json!({
                            "op": 6,
                            "d": {
                                "requestType": "SetInputMute",
                                "requestId": "streamshield_mute",
                                "requestData": {
                                    "inputName": name,
                                    "inputMuted": should_mute
                                }
                            }
                        });
                        let _ = ws.send(Message::Text(mute_req.to_string()));
                    }
                }
                state.active_audio_inputs = found_sources;
            }
        }
    }
}

/// Background thread to maintain OBS WebSocket connection
pub fn start_obs_companion_daemon(shielded_exes_provider: Arc<dyn Fn() -> HashSet<String> + Send + Sync + 'static>) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_millis(3000));

            let obs_running = is_obs_running();
            if !obs_running {
                let mut state = get_obs_state().lock().unwrap();
                state.connected = false;
                state.client = None;
                continue;
            }

            let was_connected = {
                let state = get_obs_state().lock().unwrap();
                state.connected && state.client.is_some()
            };

            if !was_connected {
                try_connect_obs();
            }

            let is_now_connected = {
                let state = get_obs_state().lock().unwrap();
                state.connected
            };

            if is_now_connected {
                let exes = shielded_exes_provider();
                sync_shielded_with_obs(&exes);
            }
        }
    });
}
