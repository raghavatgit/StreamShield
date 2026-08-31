//! Capture Compatibility Engine
//!
//! Provides capture environment detection (NVIDIA ShadowPlay, OBS Studio, Discord)
//! and hardware Multiplane Overlay (MPO) registry management for seamless capture shielding.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureEnvironment {
    /// Whether NVIDIA ShadowPlay / GeForce Experience / NVIDIA App is active
    pub nvidia_detected: bool,
    /// Whether OBS Studio / Streamlabs is active
    pub obs_detected: bool,
    /// Whether Discord is active
    pub discord_detected: bool,
    /// Whether MPO overlay plane fix is active in registry
    pub mpo_active: bool,
    /// Recommended display affinity mode for current active capture software
    pub recommended_mode: String,
    /// Human-readable summary of capture environment
    pub summary: String,
}

/// Detect active streaming, recording, and capture processes on the system
#[cfg(windows)]
pub fn detect_capture_environment() -> CaptureEnvironment {
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::tlhelp32::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW,
        PROCESSENTRY32W, TH32CS_SNAPPROCESS,
    };

    let nvidia_targets = [
        "nvcontainer.exe",
        "nvsphelper64.exe",
        "nvidia share.exe",
        "nvidia app.exe",
        "nvidia overlay.exe",
        "nvspcaps64.exe",
    ];

    let obs_targets = [
        "obs64.exe",
        "obs32.exe",
        "obs.exe",
        "streamlabs obs.exe",
    ];

    let discord_targets = [
        "discord.exe",
        "discordcanary.exe",
        "discordptb.exe",
    ];

    let mut nvidia_found = false;
    let mut obs_found = false;
    let mut discord_found = false;

    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if !snapshot.is_null() && snapshot != winapi::um::handleapi::INVALID_HANDLE_VALUE {
            let mut pe: PROCESSENTRY32W = std::mem::zeroed();
            pe.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

            if Process32FirstW(snapshot, &mut pe) != 0 {
                loop {
                    let null_pos = pe.szExeFile.iter().position(|&c| c == 0)
                        .unwrap_or(pe.szExeFile.len());
                    let exe_name = String::from_utf16_lossy(&pe.szExeFile[..null_pos]).to_lowercase();

                    if nvidia_targets.iter().any(|&t| exe_name == t) {
                        nvidia_found = true;
                    }
                    if obs_targets.iter().any(|&t| exe_name == t) {
                        obs_found = true;
                    }
                    if discord_targets.iter().any(|&t| exe_name == t) {
                        discord_found = true;
                    }

                    if Process32NextW(snapshot, &mut pe) == 0 {
                        break;
                    }
                }
            }
            CloseHandle(snapshot);
        }
    }

    let mpo_active = is_mpo_fix_active();

    let (recommended_mode, summary) = if nvidia_found {
        (
            "monitor".to_string(),
            "NVIDIA ShadowPlay active. Black Screen Mask recommended for 100% capture compatibility.".to_string()
        )
    } else if obs_found || discord_found {
        (
            "exclude".to_string(),
            "OBS / Discord active. Transparent Exclude mode recommended for invisible shielding.".to_string()
        )
    } else {
        (
            "exclude".to_string(),
            "Standard Windows capture environment detected.".to_string()
        )
    };

    CaptureEnvironment {
        nvidia_detected: nvidia_found,
        obs_detected: obs_found,
        discord_detected: discord_found,
        mpo_active,
        recommended_mode,
        summary,
    }
}

#[cfg(not(windows))]
pub fn detect_capture_environment() -> CaptureEnvironment {
    CaptureEnvironment {
        nvidia_detected: false,
        obs_detected: false,
        discord_detected: false,
        mpo_active: false,
        recommended_mode: "exclude".to_string(),
        summary: "Non-Windows platform".to_string(),
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// MPO (Multiplane Overlay) GPU Optimization via Registry
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Apply or remove the MPO (Multiplane Overlay) fix in the Windows Registry.
/// Requires elevated Administrator privileges.
#[cfg(windows)]
pub fn set_mpo_fix_registry(enable: bool) -> Result<(), String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::winnt::{KEY_SET_VALUE, REG_DWORD};
    use winapi::um::winreg::{
        RegCloseKey, RegCreateKeyExW, RegDeleteValueW, RegOpenKeyExW, RegSetValueExW,
        HKEY_LOCAL_MACHINE,
    };

    fn wide(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    }

    let dwm_subkey = wide(r"SOFTWARE\Microsoft\Windows\Dwm");
    let dwm_val = wide("OverlayTestMode");

    let gfx_subkey = wide(r"SYSTEM\CurrentControlSet\Control\GraphicsDrivers");
    let gfx_val = wide("DisableOverlays");

    unsafe {
        if enable {
            // Set OverlayTestMode = 5 in DWM
            let mut hkey = std::ptr::null_mut();
            let mut disp = 0u32;
            let status = RegCreateKeyExW(
                HKEY_LOCAL_MACHINE, dwm_subkey.as_ptr(), 0, std::ptr::null_mut(),
                0, KEY_SET_VALUE, std::ptr::null_mut(), &mut hkey, &mut disp,
            );
            if status != 0 {
                return Err(format!("Failed to write DWM registry (error code {}). Administrator privileges required.", status));
            }
            let data: u32 = 5;
            let _ = RegSetValueExW(
                hkey, dwm_val.as_ptr(), 0, REG_DWORD,
                &data as *const u32 as *const _, std::mem::size_of::<u32>() as u32,
            );
            RegCloseKey(hkey);

            // Set DisableOverlays = 1 in GraphicsDrivers (Win 11 25H2+)
            let mut gfx_key = std::ptr::null_mut();
            let gfx_status = RegCreateKeyExW(
                HKEY_LOCAL_MACHINE, gfx_subkey.as_ptr(), 0, std::ptr::null_mut(),
                0, KEY_SET_VALUE, std::ptr::null_mut(), &mut gfx_key, &mut disp,
            );
            if gfx_status == 0 {
                let gfx_data: u32 = 1;
                let _ = RegSetValueExW(
                    gfx_key, gfx_val.as_ptr(), 0, REG_DWORD,
                    &gfx_data as *const u32 as *const _, std::mem::size_of::<u32>() as u32,
                );
                RegCloseKey(gfx_key);
            }
        } else {
            // Remove OverlayTestMode from DWM
            let mut hkey = std::ptr::null_mut();
            if RegOpenKeyExW(HKEY_LOCAL_MACHINE, dwm_subkey.as_ptr(), 0, KEY_SET_VALUE, &mut hkey) == 0 {
                let _ = RegDeleteValueW(hkey, dwm_val.as_ptr());
                RegCloseKey(hkey);
            }

            // Remove DisableOverlays from GraphicsDrivers
            let mut gfx_key = std::ptr::null_mut();
            if RegOpenKeyExW(HKEY_LOCAL_MACHINE, gfx_subkey.as_ptr(), 0, KEY_SET_VALUE, &mut gfx_key) == 0 {
                let _ = RegDeleteValueW(gfx_key, gfx_val.as_ptr());
                RegCloseKey(gfx_key);
            }
        }
    }

    Ok(())
}

#[cfg(not(windows))]
pub fn set_mpo_fix_registry(_enable: bool) -> Result<(), String> {
    Ok(())
}

/// Check if the MPO fix is currently active in the registry.
#[cfg(windows)]
pub fn is_mpo_fix_active() -> bool {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::winnt::KEY_QUERY_VALUE;
    use winapi::um::winreg::{RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY_LOCAL_MACHINE};

    fn wide(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    }

    let subkey = wide(r"SOFTWARE\Microsoft\Windows\Dwm");
    let val_name = wide("OverlayTestMode");

    unsafe {
        let mut hkey = std::ptr::null_mut();
        if RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            subkey.as_ptr(),
            0,
            KEY_QUERY_VALUE,
            &mut hkey,
        ) != 0
        {
            return false;
        }
        let mut data_type = 0u32;
        let mut data: u32 = 0;
        let mut data_len = std::mem::size_of::<u32>() as u32;
        let query_res = RegQueryValueExW(
            hkey,
            val_name.as_ptr(),
            std::ptr::null_mut(),
            &mut data_type,
            &mut data as *mut u32 as *mut _,
            &mut data_len,
        );
        RegCloseKey(hkey);
        query_res == 0 && data == 5
    }
}

#[cfg(not(windows))]
pub fn is_mpo_fix_active() -> bool {
    false
}
