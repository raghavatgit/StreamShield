#![allow(non_snake_case)]
use winapi::shared::minwindef::{BOOL, DWORD, HINSTANCE, LPVOID, TRUE};
use winapi::shared::windef::HWND;
use winapi::um::winuser::SetWindowDisplayAffinity;

const WDA_NONE: u32 = 0x0000_0000;
const WDA_EXCLUDEFROMCAPTURE: u32 = 0x0000_0011;

#[no_mangle]
pub extern "system" fn DllMain(_: HINSTANCE, _: DWORD, _: LPVOID) -> BOOL {
    TRUE
}

/// Remote thread entry: lpParameter encodes hwnd (low 32 bits) | enable<<32 (high 32 bits)
/// Called via CreateRemoteThread from the host process.
#[no_mangle]
pub unsafe extern "system" fn shield_window(param: LPVOID) -> DWORD {
    let val = param as usize;
    let hwnd = (val & 0xFFFF_FFFF) as HWND;
    let enable = (val >> 32) & 1;
    let affinity = if enable != 0 { WDA_EXCLUDEFROMCAPTURE } else { WDA_NONE };
    SetWindowDisplayAffinity(hwnd, affinity);
    0
}
