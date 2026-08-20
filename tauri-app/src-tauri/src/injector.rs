/! Sets window display affinity using winapi.
//! SetWindowDisplayAffinity with WDA_EXCLUDEFROMCAPTURE makes a window
//! appear black in all screen capture tools while remaining visible to the user.

#[cfg(windows)]
pub fn set_window_affinity(hwnd_val: usize, enable: bool) -> Result<(), String> {
    use winapi::shared::windef::HWND;
    use winapi::um::winuser::SetWindowDisplayAffinity;

    // WDA_EXCLUDEFROMCAPTURE = 0x00000011 (Windows 10 2004+)
    // WDA_NONE = 0x00000000
    const WDA_NONE: u32 = 0x00000000;
    const WDA_EXCLUDEFROMCAPTURE: u32 = 0x00000011;

    let hwnd = hwnd_val as HWND;
    let affinity = if enable { WDA_EXCLUDEFROMCAPTURE } else { WDA_NONE };

    let result = unsafe { SetWindowDisplayAffinity(hwnd, affinity) };
    if result != 0 {
        Ok(())
    } else {
        let err = unsafe { winapi::um::errhandlingapi::GetLastError() };
        Err(format!(
            "SetWindowDisplayAffinity failed (error {}). Try running StreamShield as Administrator.",
            err
        ))
    }
}

#[cfg(not(windows))]
pub fn set_window_affinity(_hwnd_val: usize, _enable: bool) -> Result<(), String> {
    Ok(())
}
