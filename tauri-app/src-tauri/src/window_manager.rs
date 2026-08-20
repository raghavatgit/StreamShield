use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub hwnd: usize,
    pub pid: u32,
    pub title: String,
    pub exe_name: String,
}

#[cfg(windows)]
pub fn enumerate_windows() -> Vec<WindowInfo> {
    use std::collections::HashSet;
    use winapi::shared::minwindef::{BOOL, LPARAM};
    use winapi::shared::windef::HWND;
    use winapi::um::winuser::{
        EnumWindows, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
        IsWindowVisible,
    };

    unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        if IsWindowVisible(hwnd) == 0 { return 1; }
        if GetWindowTextLengthW(hwnd) == 0 { return 1; }
        let list = &mut *(lparam as *mut Vec<HWND>);
        list.push(hwnd);
        1
    }

    let mut hwnds: Vec<HWND> = Vec::new();
    unsafe { EnumWindows(Some(enum_proc), &mut hwnds as *mut Vec<HWND> as LPARAM); }

    let mut results: Vec<WindowInfo> = Vec::new();
    let mut seen_exe: HashSet<String> = HashSet::new();
    let mut pid_cache: std::collections::HashMap<u32, String> = std::collections::HashMap::new();

    for hwnd in hwnds {
        let mut title_buf = vec![0u16; 512];
        let title_len = unsafe { GetWindowTextW(hwnd, title_buf.as_mut_ptr(), 512) };
        if title_len == 0 { continue; }
        let title = String::from_utf16_lossy(&title_buf[..title_len as usize]);
        if title.is_empty() { continue; }

        let mut pid: u32 = 0;
        unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };

        let exe_name = pid_cache.entry(pid).or_insert_with(|| get_exe_name(pid)).clone();
        if exe_name == "streamshield.exe" { continue; }
        if seen_exe.contains(&exe_name) { continue; }
        seen_exe.insert(exe_name.clone());

        results.push(WindowInfo { hwnd: hwnd as usize, pid, title, exe_name });
    }
    results
}

#[cfg(windows)]
fn get_exe_name(pid: u32) -> String {
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::processthreadsapi::OpenProcess;
    use winapi::um::psapi::GetModuleFileNameExW;
    use winapi::um::winnt::{PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, pid);
        if handle.is_null() { return "unknown".to_string(); }
        let mut buf = vec![0u16; 512];
        let len = GetModuleFileNameExW(handle, std::ptr::null_mut(), buf.as_mut_ptr(), 512);
        CloseHandle(handle);
        if len == 0 { return "unknown".to_string(); }
        let full = String::from_utf16_lossy(&buf[..len as usize]);
        full.split('\\').last().unwrap_or("unknown").to_string()
    }
}

#[cfg(not(windows))]
pub fn enumerate_windows() -> Vec<WindowInfo> { vec![] }