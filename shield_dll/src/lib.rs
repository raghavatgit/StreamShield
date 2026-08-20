//! Shield DLL — injected into target processes to call SetWindowDisplayAffinity.
//! shield_window is called as a thread entry point via CreateRemoteThread,
//! so it MUST match LPTHREAD_START_ROUTINE: fn(LPVOID) -> DWORD.
//! We pack hwnd (low 32 bits) and enable flag (bit 32) into the single LPVOID param.

#[cfg(windows)]
mod imp {
    use winapi::shared::windef::HWND;
    use winapi::um::winuser::SetWindowDisplayAffinity;

    const WDA_NONE: u32 = 0x00000000;
    const WDA_EXCLUDEFROMCAPTURE: u32 = 0x00000011;

    /// Called as a remote thread entry point.
    /// param encodes: bits 0-31 = HWND value, bit 32 = enable (1) / disable (0)
    #[no_mangle]
    pub unsafe extern "system" fn shield_window(param: *mut std::ffi::c_void) -> u32 {
        let val = param as usize;
        let hwnd = (val & 0xFFFF_FFFF) as HWND;
        let enable = (val >> 32) & 1;
        let affinity = if enable != 0 { WDA_EXCLUDEFROMCAPTURE } else { WDA_NONE };
        SetWindowDisplayAffinity(hwnd, affinity) as u32
    }
}

#[cfg(not(windows))]
#[no_mangle]
pub unsafe extern "system" fn shield_window(_param: *mut std::ffi::c_void) -> u32 { 0 }
