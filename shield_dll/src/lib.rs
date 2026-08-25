//! Shield DLL — injected into target processes to call SetWindowDisplayAffinity.
//! shield_window is called as a thread entry point via CreateRemoteThread,
//! so it MUST match LPTHREAD_START_ROUTINE: fn(LPVOID) -> DWORD.

#[cfg(windows)]
mod imp {
    use winapi::shared::windef::HWND;
    use winapi::um::winuser::{
        SetWindowDisplayAffinity, SetWindowPos, RedrawWindow,
        SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SWP_NOACTIVATE, SWP_FRAMECHANGED,
        RDW_INVALIDATE, RDW_ERASE, RDW_FRAME, RDW_ALLCHILDREN, RDW_UPDATENOW,
    };

    const WDA_NONE: u32 = 0x00000000;
    const WDA_EXCLUDEFROMCAPTURE: u32 = 0x00000011;
    const WDA_MONITOR: u32 = 0x00000001;

    /// Called as a remote thread entry point.
    /// param encodes: bit 63 = enable (1) / disable (0), bits 0-62 = HWND value
    #[no_mangle]
    pub unsafe extern "system" fn shield_window(param: *mut std::ffi::c_void) -> u32 {
        let val = param as usize;
        let enable = (val & (1usize << 63)) != 0;
        let hwnd = (val & !(1usize << 63)) as HWND;

        let affinity = if enable { WDA_EXCLUDEFROMCAPTURE } else { WDA_NONE };
        let mut res = SetWindowDisplayAffinity(hwnd, affinity);

        // Fallback to WDA_MONITOR if WDA_EXCLUDEFROMCAPTURE is unsupported on legacy OS
        if res == 0 && enable {
            res = SetWindowDisplayAffinity(hwnd, WDA_MONITOR);
        }

        // Force DWM compositor to flush the previous visual surface buffer
        SetWindowPos(
            hwnd,
            std::ptr::null_mut(),
            0, 0, 0, 0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
        );

        RedrawWindow(
            hwnd,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            RDW_INVALIDATE | RDW_ERASE | RDW_FRAME | RDW_ALLCHILDREN | RDW_UPDATENOW,
        );

        res as u32
    }
}

#[cfg(not(windows))]
#[no_mangle]
pub unsafe extern "system" fn shield_window(_param: *mut std::ffi::c_void) -> u32 { 0 }
