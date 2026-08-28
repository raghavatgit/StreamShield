//! Shield DLL — injected into target processes to call SetWindowDisplayAffinity.
//! shield_window is called as a thread entry point via CreateRemoteThread,
//! so it MUST match LPTHREAD_START_ROUTINE: fn(LPVOID) -> DWORD.

#[cfg(windows)]
mod imp {
    use winapi::shared::minwindef::{BOOL, LPARAM};
    use winapi::shared::windef::HWND;
    use winapi::um::processthreadsapi::GetCurrentProcessId;
    use winapi::um::winuser::{
        EnumWindows, GetWindowThreadProcessId, IsWindowVisible,
        GetWindowDisplayAffinity, SetWindowDisplayAffinity,
        SetWindowPos, RedrawWindow,
        SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SWP_NOACTIVATE, SWP_FRAMECHANGED,
        RDW_INVALIDATE, RDW_ERASE, RDW_FRAME, RDW_ALLCHILDREN, RDW_UPDATENOW,
    };

    const WDA_NONE: u32 = 0x00000000;
    const WDA_EXCLUDEFROMCAPTURE: u32 = 0x00000011;
    const WDA_MONITOR: u32 = 0x00000001;

    /// Apply display affinity to a window. Only force-redraws if affinity actually changed.
    unsafe fn apply_affinity_to_hwnd(hwnd: HWND, affinity: u32, enable: bool) -> u32 {
        // Check current affinity first — skip if already at desired value
        let mut current: u32 = 0;
        if GetWindowDisplayAffinity(hwnd, &mut current) != 0 {
            if enable && current != 0 {
                return 1; // Already shielded, nothing to do
            }
            if !enable && current == 0 {
                return 1; // Already unshielded, nothing to do
            }
        }

        let mut res = SetWindowDisplayAffinity(hwnd, affinity);
        if res == 0 && enable {
            res = SetWindowDisplayAffinity(hwnd, WDA_MONITOR);
        }

        // Only force-redraw if SetWindowDisplayAffinity succeeded (affinity actually changed)
        if res != 0 {
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
        }
        res as u32
    }

    unsafe extern "system" fn enum_all_process_windows(hwnd: HWND, lparam: LPARAM) -> BOOL {
        // CRITICAL: Only process visible windows — hidden/internal windows must not be
        // force-redrawn or they manifest as blank white ghost rectangles on screen.
        if IsWindowVisible(hwnd) == 0 {
            return 1;
        }

        let (my_pid, affinity, enable) = *(lparam as *const (u32, u32, bool));
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == my_pid {
            apply_affinity_to_hwnd(hwnd, affinity, enable);
        }
        1
    }

    /// Called as a remote thread entry point.
    /// param encodes: bit 63 = enable (1) / disable (0), bits 0-62 = primary HWND value
    #[no_mangle]
    pub unsafe extern "system" fn shield_window(param: *mut std::ffi::c_void) -> u32 {
        let val = param as usize;
        let enable = (val & (1usize << 63)) != 0;
        let primary_hwnd = (val & !(1usize << 63)) as HWND;
        let affinity = if enable { WDA_EXCLUDEFROMCAPTURE } else { WDA_NONE };

        let mut res = 1u32;
        if !primary_hwnd.is_null() {
            res = apply_affinity_to_hwnd(primary_hwnd, affinity, enable);
        }

        // Shield ALL visible sibling and popup windows belonging to this process
        let my_pid = GetCurrentProcessId();
        let payload = (my_pid, affinity, enable);
        EnumWindows(Some(enum_all_process_windows), &payload as *const _ as LPARAM);

        res
    }
}

#[cfg(not(windows))]
#[no_mangle]
pub unsafe extern "system" fn shield_window(_param: *mut std::ffi::c_void) -> u32 { 0 }
