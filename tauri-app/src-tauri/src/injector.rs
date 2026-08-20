#[cfg(windows)]
pub fn set_window_affinity(hwnd_val: usize, enable: bool) -> Result<(), String> {
    use winapi::shared::windef::HWND;
    use winapi::um::errhandlingapi::GetLastError;
    use winapi::um::winuser::SetWindowDisplayAffinity;
    const WDA_NONE: u32 = 0x00000000;
    const WDA_EXCLUDEFROMCAPTURE: u32 = 0x00000011;
    let hwnd = hwnd_val as HWND;
    let affinity = if enable { WDA_EXCLUDEFROMCAPTURE } else { WDA_NONE };
    let result = unsafe { SetWindowDisplayAffinity(hwnd, affinity) };
    if result != 0 { Ok(()) } else {
        let code = unsafe { GetLastError() };
        Err(format!("Shield failed (code {}). Try Administrator.", code))
    }
}

#[cfg(not(windows))]
pub fn set_window_affinity(_hwnd_val: usize, _enable: bool) -> Result<(), String> { Ok(()) }
