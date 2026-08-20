#[cfg(windows)]
mod imp {
    use winapi::shared::windef::HWND;
    use winapi::um::winuser::SetWindowDisplayAffinity;

    const WDA_NONE: u32 = 0x00000000;
    const WDA_EXCLUDEFROMCAPTURE: u32 = 0x00000011;

    #[no_mangle]
    pub extern "C" fn shield_window(hwnd_val: usize, enable: u32) -> i32 {
        let hwnd = hwnd_val as HWND;
        let affinity = if enable != 0 { WDA_EXCLUDEFROMCAPTURE } else { WDA_NONE };
        let result = unsafe { SetWindowDisplayAffinity(hwnd, affinity) };
        if result != 0 { 1 } else { 0 }
    }
}

#[cfg(not(windows))]
#[no_mangle]
pub extern "C" fn shield_window(_hwnd_val: usize, _enable: u32) -> i32 { 0 }
