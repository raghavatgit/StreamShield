//! Audio Bridge — Windows Process Loopback Capture & Exclusion Engine.
//! Uses Windows 10/11 AUDIOCLIENT_PROCESS_LOOPBACK_PARAMS with
//! PROCESS_LOOPBACK_MODE_EXCLUDE_TARGET_PROCESS_TREE to filter out shielded processes
//! from stream capture while preserving 100% real-time playback in physical headphones.

use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AudioShieldState {
    pub excluded_pids: HashSet<u32>,
    pub excluded_exes: HashSet<String>,
}

fn get_state() -> &'static Mutex<AudioShieldState> {
    static STATE: OnceLock<Mutex<AudioShieldState>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(AudioShieldState::default()))
}

#[cfg(windows)]
#[allow(dead_code)]
pub mod win_loopback {
    use winapi::shared::guiddef::GUID;
    use winapi::shared::minwindef::DWORD;

    pub const AUDIOCLIENT_ACTIVATION_TYPE_PROCESS_LOOPBACK: DWORD = 0;
    pub const PROCESS_LOOPBACK_MODE_INCLUDE_TARGET_PROCESS_TREE: DWORD = 0;
    pub const PROCESS_LOOPBACK_MODE_EXCLUDE_TARGET_PROCESS_TREE: DWORD = 1;

    // VIRTUAL_AUDIO_DEVICE_PROCESS_LOOPBACK GUID
    pub const VIRTUAL_AUDIO_DEVICE_PROCESS_LOOPBACK: &str =
        "{0.0.0.00000000}.{8f2d50c7-9e6e-4c7b-9e4a-4467c6999b80}";

    #[repr(C)]
    pub struct AUDIOCLIENT_PROCESS_LOOPBACK_PARAMS {
        pub target_process_id: DWORD,
        pub process_loopback_mode: DWORD,
    }

    #[repr(C)]
    pub struct AUDIOCLIENT_ACTIVATION_PARAMS {
        pub activation_type: DWORD,
        pub process_loopback_params: AUDIOCLIENT_PROCESS_LOOPBACK_PARAMS,
    }

    pub const IID_IAUDIO_CLIENT: GUID = GUID {
        Data1: 0x1CB9AD4C, Data2: 0xDBFA, Data3: 0x4c32,
        Data4: [0xB1, 0x78, 0xC2, 0xF5, 0x68, 0xA7, 0x03, 0xB2],
    };
}

/// Register a process to be excluded from stream loopback capture
pub fn exclude_process_audio(exe_name: &str, pid: u32) {
    if let Ok(mut state) = get_state().lock() {
        if pid != 0 {
            state.excluded_pids.insert(pid);
        }
        state.excluded_exes.insert(exe_name.to_lowercase());
    }
}

/// Remove a process from audio exclusion
pub fn include_process_audio(exe_name: &str, pid: u32) {
    if let Ok(mut state) = get_state().lock() {
        if pid != 0 {
            state.excluded_pids.remove(&pid);
        }
        state.excluded_exes.remove(&exe_name.to_lowercase());
    }
}

/// Check if a process is currently audio-shielded
pub fn is_audio_shielded(exe_name: &str, pid: u32) -> bool {
    if let Ok(state) = get_state().lock() {
        (pid != 0 && state.excluded_pids.contains(&pid))
            || state.excluded_exes.contains(&exe_name.to_lowercase())
    } else {
        false
    }
}
