use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

// Process-lifetime caches to eliminate repeated GDI extractions and PNG/Base64 encoding churn.
// GLOBAL_PID_CACHE stores (creation_time_100ns, exe_name, full_path) and prunes dead PIDs every scan.
static GLOBAL_ICON_CACHE: Mutex<Option<HashMap<String, Option<String>>>> = Mutex::new(None);
static GLOBAL_PID_CACHE: Mutex<Option<HashMap<u32, (u64, String, String)>>> = Mutex::new(None);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub hwnd: usize,
    pub pid: u32,
    pub title: String,
    pub exe_name: String,
    pub is_shielded: bool,
    pub icon_base64: Option<String>,
}

// System processes and capture/recording tools that should never be listed in the app
#[cfg(windows)]
const EXCLUDED_PROCESSES: &[&str] = &[
    "textinputhost.exe",
    "shellexperiencehost.exe",
    "startmenuexperiencehost.exe",
    "searchhost.exe",
    "searchapp.exe",
    "systemsettings.exe",
    "lockapp.exe",
    "dwm.exe",
    "csrss.exe",
    "conhost.exe",
    "svchost.exe",
    "streamshield.exe",
    "taskmgr.exe",
    "widgets.exe",
    // Capture, recording and clipping tools that should never be shielded
    "medal.exe",
    "medalencoder.exe",
    "medaloverlay.exe",
    "obs64.exe",
    "obs32.exe",
    "obs.exe",
    "streamlabs obs.exe",
    "nvidia share.exe",
    "nvidia app.exe",
    "nvidia overlay.exe",
    "nvspcaps64.exe",
    "nvcontainer.exe",
    "nvsphelper64.exe",
];

#[cfg(windows)]
unsafe fn is_candidate_window(hwnd: winapi::shared::windef::HWND) -> bool {
    use winapi::shared::windef::RECT;
    use winapi::um::dwmapi::{
        DwmGetWindowAttribute, DWMWA_CLOAKED,
        DWM_CLOAKED_APP, DWM_CLOAKED_INHERITED, DWM_CLOAKED_SHELL,
    };
    use winapi::um::winuser::{
        GetWindowLongW, GetWindowRect, GetWindowTextLengthW, IsWindowVisible,
        GWL_EXSTYLE, GWL_STYLE, WS_EX_TOOLWINDOW, WS_VISIBLE,
    };

    if IsWindowVisible(hwnd) == 0 {
        return false;
    }

    if GetWindowTextLengthW(hwnd) == 0 {
        return false;
    }

    let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
    if (ex_style & WS_EX_TOOLWINDOW) != 0 {
        return false;
    }

    let style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
    if (style & WS_VISIBLE) == 0 {
        return false;
    }

    let mut rect: RECT = std::mem::zeroed();
    if GetWindowRect(hwnd, &mut rect) != 0 {
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        if width <= 0 || height <= 0 {
            return false;
        }
    }

    let mut cloaked: u32 = 0;
    let hr = DwmGetWindowAttribute(
        hwnd,
        DWMWA_CLOAKED,
        &mut cloaked as *mut _ as *mut _,
        std::mem::size_of::<u32>() as u32,
    );
    if hr == 0 && (cloaked & (DWM_CLOAKED_APP | DWM_CLOAKED_SHELL | DWM_CLOAKED_INHERITED)) != 0 {
        return false;
    }

    true
}

#[cfg(windows)]
fn resolve_pid_cached(
    pid_cache: &mut HashMap<u32, (u64, String, String)>,
    pid: u32,
) -> (String, String) {
    let live_ctime = get_process_creation_time(pid);
    if let Some((cached_ctime, exe, path)) = pid_cache.get(&pid) {
        if *cached_ctime == live_ctime && !path.is_empty() {
            return (exe.clone(), path.clone());
        }
        // Check matching unknown-time PIDs only if live_ctime is 0 (protected/inaccessible)
        if *cached_ctime == 0 && live_ctime == 0 {
            return (exe.clone(), path.clone());
        }
    }
    let (exe, path) = get_process_info(pid);
    pid_cache.insert(pid, (live_ctime, exe.clone(), path.clone()));
    (exe, path)
}

#[cfg(windows)]
pub fn enumerate_windows(
    shielded_exes: &HashSet<String>,
    shield_mode: &str,
    auto_reapply: bool,
) -> Vec<WindowInfo> {
    use winapi::shared::minwindef::{BOOL, LPARAM};
    use winapi::shared::windef::HWND;
    use winapi::um::winuser::{EnumWindows, GetWindowTextW, GetWindowThreadProcessId, IsWindow};

    unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        if is_candidate_window(hwnd) {
            let list = &mut *(lparam as *mut Vec<HWND>);
            list.push(hwnd);
        }
        1
    }

    let mut hwnds: Vec<HWND> = Vec::new();
    unsafe { EnumWindows(Some(enum_proc), &mut hwnds as *mut Vec<HWND> as LPARAM); }

    let mut results: Vec<WindowInfo> = Vec::new();
    let mut seen_exe: HashSet<String> = HashSet::new();
    let mut active_pids: HashSet<u32> = HashSet::new();

    let mut pid_cache_guard = GLOBAL_PID_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    let pid_cache = pid_cache_guard.get_or_insert_with(HashMap::new);

    let mut icon_cache_guard = GLOBAL_ICON_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    let icon_cache = icon_cache_guard.get_or_insert_with(HashMap::new);

    for hwnd in hwnds {
        let mut title_buf = vec![0u16; 512];
        let title_len = unsafe { GetWindowTextW(hwnd, title_buf.as_mut_ptr(), 512) };
        if title_len == 0 { continue; }
        let title = String::from_utf16_lossy(&title_buf[..title_len as usize]).trim().to_string();
        if title.is_empty() || title == "Program Manager" { continue; }

        let mut pid: u32 = 0;
        unsafe { GetWindowThreadProcessId(hwnd, &mut pid); }
        if pid == 0 { continue; }
        active_pids.insert(pid);

        let (exe_name, full_path) = resolve_pid_cached(pid_cache, pid);
        let exe_lower = exe_name.to_lowercase();

        if EXCLUDED_PROCESSES.contains(&exe_lower.as_str()) {
            continue;
        }

        if exe_lower == "applicationframehost.exe" && (title.is_empty() || title == "ApplicationFrameHost") {
            continue;
        }

        if seen_exe.contains(&exe_lower) {
            continue;
        }
        seen_exe.insert(exe_lower.clone());

        // Ground-truth live OS display affinity check
        let mut is_shielded = query_live_affinity(hwnd as usize);

        // AUTO-REAPPLY: If executable is configured as shielded, ensure window has desired affinity
        if auto_reapply {
            let should_be_shielded = shielded_exes
                .iter()
                .any(|s| s.eq_ignore_ascii_case(&exe_name) || s.eq_ignore_ascii_case(&exe_lower));
            if should_be_shielded {
                let desired = if shield_mode.eq_ignore_ascii_case("monitor") { 0x00000001 } else { 0x00000011 };
                let current_raw = get_live_affinity_raw(hwnd as usize);
                if (!is_shielded || (current_raw != desired && current_raw != 0x00000001))
                    && unsafe { IsWindow(hwnd) } != 0
                {
                    let _ = crate::injector::set_window_affinity(hwnd as usize, true, Some(shield_mode));
                    is_shielded = query_live_affinity(hwnd as usize);
                }
            }
        }

        // Fetch icon (cached per exe to avoid redundant GDI calls)
        let icon_base64 = icon_cache.entry(exe_lower).or_insert_with(|| {
            extract_window_or_exe_icon(hwnd, &full_path)
        }).clone();

        results.push(WindowInfo {
            hwnd: hwnd as usize,
            pid,
            title,
            exe_name,
            is_shielded,
            icon_base64,
        });
    }

    // Prevent memory leaks & stale PID-reuse bugs by pruning terminated PIDs and bounding icon cache
    pid_cache.retain(|pid, _| active_pids.contains(pid));
    if icon_cache.len() > 128 {
        icon_cache.retain(|exe, _| seen_exe.contains(exe));
    }

    // Sort: shielded apps first, then alphabetically
    results.sort_by(|a, b| {
        b.is_shielded.cmp(&a.is_shielded)
            .then_with(|| a.exe_name.to_lowercase().cmp(&b.exe_name.to_lowercase()))
    });

    results
}

/// Find one visible top-level window per distinct PID matching `target_exe`.
/// Ensures multi-process apps (Chrome, Edge, VS Code, Discord) have all windows shielded/unshielded.
#[cfg(windows)]
pub fn find_all_windows_for_exe(target_exe: &str) -> Vec<(usize, u32)> {
    use winapi::shared::minwindef::{BOOL, LPARAM};
    use winapi::shared::windef::HWND;
    use winapi::um::winuser::{EnumWindows, GetWindowThreadProcessId};

    unsafe extern "system" fn collect_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        if is_candidate_window(hwnd) {
            let mut pid: u32 = 0;
            GetWindowThreadProcessId(hwnd, &mut pid);
            if pid != 0 {
                let list = &mut *(lparam as *mut Vec<(usize, u32)>);
                list.push((hwnd as usize, pid));
            }
        }
        1
    }

    let mut candidates: Vec<(usize, u32)> = Vec::new();
    unsafe { EnumWindows(Some(collect_proc), &mut candidates as *mut Vec<(usize, u32)> as LPARAM); }

    let norm_target = target_exe.to_lowercase();
    let mut pid_cache_guard = GLOBAL_PID_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    let pid_cache = pid_cache_guard.get_or_insert_with(HashMap::new);

    let mut seen_pids: HashSet<u32> = HashSet::new();
    let mut matched: Vec<(usize, u32)> = Vec::new();

    for (hwnd, pid) in candidates {
        if seen_pids.contains(&pid) {
            continue;
        }
        let (exe_name, _) = resolve_pid_cached(pid_cache, pid);
        let exe_lower = exe_name.to_lowercase();
        let is_match = exe_lower == norm_target
            || (norm_target.starts_with("whatsapp") && exe_lower.starts_with("whatsapp"));
        if is_match {
            seen_pids.insert(pid);
            matched.push((hwnd, pid));
        }
    }

    matched
}

/// Background watchdog helper: auto-shields freshly opened windows belonging to shielded executables.
/// Collects candidate handles cleanly inside `EnumWindows`, then injects outside the callback.
#[cfg(windows)]
pub fn auto_reapply_shields(shielded_exes: &HashSet<String>, shield_mode: &str) {
    if shielded_exes.is_empty() {
        return;
    }
    use winapi::shared::minwindef::{BOOL, LPARAM};
    use winapi::shared::windef::HWND;
    use winapi::um::winuser::{EnumWindows, GetWindowThreadProcessId, IsWindow};

    unsafe extern "system" fn collect_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        if is_candidate_window(hwnd) {
            let mut pid: u32 = 0;
            GetWindowThreadProcessId(hwnd, &mut pid);
            if pid != 0 {
                let list = &mut *(lparam as *mut Vec<(usize, u32)>);
                list.push((hwnd as usize, pid));
            }
        }
        1
    }

    let mut candidates: Vec<(usize, u32)> = Vec::new();
    unsafe { EnumWindows(Some(collect_proc), &mut candidates as *mut Vec<(usize, u32)> as LPARAM); }

    let mut targets: Vec<(usize, u32)> = Vec::new();
    let mut seen_pids: HashSet<u32> = HashSet::new();

    {
        let mut pid_cache_guard = GLOBAL_PID_CACHE.lock().unwrap_or_else(|e| e.into_inner());
        let pid_cache = pid_cache_guard.get_or_insert_with(HashMap::new);

        for (hwnd, pid) in candidates {
            if !seen_pids.insert(pid) {
                continue;
            }
            let (exe_name, _) = resolve_pid_cached(pid_cache, pid);
            let exe_lower = exe_name.to_lowercase();
            if EXCLUDED_PROCESSES.contains(&exe_lower.as_str()) {
                continue;
            }
            let should_shield = shielded_exes
                .iter()
                .any(|s| s.eq_ignore_ascii_case(&exe_name) || s.eq_ignore_ascii_case(&exe_lower));
            if should_shield && !query_live_affinity(hwnd) {
                targets.push((hwnd, pid));
            }
        }
    }

    for (hwnd, _pid) in targets {
        if unsafe { IsWindow(hwnd as _) } != 0 {
            let _ = crate::injector::set_window_affinity(hwnd, true, Some(shield_mode));
        }
    }
}

/// Query raw OS display affinity flag (0 = None, 1 = Monitor, 0x11 = ExcludeFromCapture).
#[cfg(windows)]
pub fn get_live_affinity_raw(hwnd: usize) -> u32 {
    use winapi::um::winuser::GetWindowDisplayAffinity;
    let mut affinity: u32 = 0;
    let ok = unsafe { GetWindowDisplayAffinity(hwnd as _, &mut affinity) };
    if ok != 0 { affinity } else { 0 }
}

/// Query ground-truth Display Affinity from Windows OS.
/// Returns true if any non-zero affinity is set.
#[cfg(windows)]
pub fn query_live_affinity(hwnd: usize) -> bool {
    get_live_affinity_raw(hwnd) != 0
}

#[cfg(windows)]
fn get_process_creation_time(pid: u32) -> u64 {
    use winapi::shared::minwindef::FILETIME;
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::processthreadsapi::{GetProcessTimes, OpenProcess};
    use winapi::um::winnt::PROCESS_QUERY_LIMITED_INFORMATION;

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return 0;
        }
        let mut creation: FILETIME = std::mem::zeroed();
        let mut exit: FILETIME = std::mem::zeroed();
        let mut kernel: FILETIME = std::mem::zeroed();
        let mut user: FILETIME = std::mem::zeroed();
        let ok = GetProcessTimes(handle, &mut creation, &mut exit, &mut kernel, &mut user);
        CloseHandle(handle);
        if ok != 0 {
            ((creation.dwHighDateTime as u64) << 32) | (creation.dwLowDateTime as u64)
        } else {
            0
        }
    }
}

#[cfg(windows)]
fn get_process_info(pid: u32) -> (String, String) {
    use winapi::um::processthreadsapi::OpenProcess;
    use winapi::um::winbase::QueryFullProcessImageNameW;
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::winnt::PROCESS_QUERY_LIMITED_INFORMATION;

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return (format!("PID {pid}"), String::new());
        }

        let mut buf = vec![0u16; 1024];
        let mut len = 1024u32;
        let ok = QueryFullProcessImageNameW(handle, 0, buf.as_mut_ptr(), &mut len);
        CloseHandle(handle);

        if ok == 0 || len == 0 {
            return (format!("PID {pid}"), String::new());
        }

        let full_path = String::from_utf16_lossy(&buf[..len as usize]);
        let exe_name = std::path::Path::new(&full_path)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| format!("PID {pid}"));

        (exe_name, full_path)
    }
}

/// Extracts the native icon from a window or executable, returning a Base64-encoded PNG data URL.
#[cfg(windows)]
fn extract_window_or_exe_icon(hwnd: winapi::shared::windef::HWND, exe_path: &str) -> Option<String> {
    use std::ptr::null_mut;
    use winapi::shared::windef::HICON;
    use winapi::um::winuser::{
        SendMessageTimeoutW, GetClassLongPtrW, DestroyIcon,
        WM_GETICON, ICON_BIG, ICON_SMALL, ICON_SMALL2, GCLP_HICON, GCLP_HICONSM, SMTO_ABORTIFHUNG,
    };
    use winapi::um::shellapi::ExtractIconExW;
    use std::os::windows::ffi::OsStrExt;
    use std::ffi::OsStr;

    unsafe {
        let mut hicon: HICON = null_mut();

        let mut result: usize = 0;
        if SendMessageTimeoutW(hwnd, WM_GETICON, ICON_BIG as _, 0, SMTO_ABORTIFHUNG, 100, &mut result as *mut _ as _) != 0 && result != 0 {
            hicon = result as HICON;
        }

        if hicon.is_null() {
            if SendMessageTimeoutW(hwnd, WM_GETICON, ICON_SMALL as _, 0, SMTO_ABORTIFHUNG, 100, &mut result as *mut _ as _) != 0 && result != 0 {
                hicon = result as HICON;
            }
        }

        if hicon.is_null() {
            if SendMessageTimeoutW(hwnd, WM_GETICON, ICON_SMALL2 as _, 0, SMTO_ABORTIFHUNG, 100, &mut result as *mut _ as _) != 0 && result != 0 {
                hicon = result as HICON;
            }
        }

        if hicon.is_null() {
            let class_icon = GetClassLongPtrW(hwnd, GCLP_HICON);
            if class_icon != 0 {
                hicon = class_icon as HICON;
            }
        }

        if hicon.is_null() {
            let class_icon_sm = GetClassLongPtrW(hwnd, GCLP_HICONSM);
            if class_icon_sm != 0 {
                hicon = class_icon_sm as HICON;
            }
        }

        let mut extracted_from_file = false;
        if hicon.is_null() && !exe_path.is_empty() {
            let wide_path: Vec<u16> = OsStr::new(exe_path).encode_wide().chain(std::iter::once(0)).collect();
            let mut large_icon: HICON = null_mut();
            let mut small_icon: HICON = null_mut();
            let count = ExtractIconExW(wide_path.as_ptr(), 0, &mut large_icon, &mut small_icon, 1);
            if count > 0 {
                if !large_icon.is_null() {
                    hicon = large_icon;
                    extracted_from_file = true;
                    if !small_icon.is_null() { DestroyIcon(small_icon); }
                } else if !small_icon.is_null() {
                    hicon = small_icon;
                    extracted_from_file = true;
                }
            }
        }

        if hicon.is_null() {
            return None;
        }

        let b64_str = hicon_to_base64_png(hicon);
        if extracted_from_file {
            DestroyIcon(hicon);
        }

        b64_str
    }
}

/// Converts a Win32 HICON into Base64 PNG bytes
#[cfg(windows)]
unsafe fn hicon_to_base64_png(hicon: winapi::shared::windef::HICON) -> Option<String> {
    use winapi::um::winuser::{GetIconInfo, ICONINFO};
    use winapi::um::wingdi::{
        CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, GetObjectW,
        BITMAP, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    };
    use std::ptr::null_mut;
    use base64::Engine;

    let mut icon_info: ICONINFO = std::mem::zeroed();
    if GetIconInfo(hicon, &mut icon_info) == 0 {
        return None;
    }

    let hbm_color = icon_info.hbmColor;
    let hbm_mask = icon_info.hbmMask;

    if hbm_color.is_null() {
        if !hbm_mask.is_null() { DeleteObject(hbm_mask as _); }
        return None;
    }

    let mut bm: BITMAP = std::mem::zeroed();
    if GetObjectW(hbm_color as _, std::mem::size_of::<BITMAP>() as _, &mut bm as *mut _ as _) == 0 {
        DeleteObject(hbm_color as _);
        if !hbm_mask.is_null() { DeleteObject(hbm_mask as _); }
        return None;
    }

    let width = bm.bmWidth as u32;
    let height = bm.bmHeight as u32;
    if width == 0 || height == 0 || width > 256 || height > 256 {
        DeleteObject(hbm_color as _);
        if !hbm_mask.is_null() { DeleteObject(hbm_mask as _); }
        return None;
    }

    let hdc = CreateCompatibleDC(null_mut());
    if hdc.is_null() {
        DeleteObject(hbm_color as _);
        if !hbm_mask.is_null() { DeleteObject(hbm_mask as _); }
        return None;
    }

    let mut bmi: BITMAPINFO = std::mem::zeroed();
    bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
    bmi.bmiHeader.biWidth = width as i32;
    bmi.bmiHeader.biHeight = -(height as i32); // Top-down
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 32;
    bmi.bmiHeader.biCompression = BI_RGB;

    let mut raw_pixels = vec![0u8; (width * height * 4) as usize];
    let lines = GetDIBits(
        hdc,
        hbm_color,
        0,
        height,
        raw_pixels.as_mut_ptr() as _,
        &mut bmi,
        DIB_RGB_COLORS,
    );

    DeleteDC(hdc);
    DeleteObject(hbm_color as _);
    if !hbm_mask.is_null() { DeleteObject(hbm_mask as _); }

    if lines == 0 {
        return None;
    }

    // BGRA -> RGBA conversion
    let mut has_alpha = false;
    for chunk in raw_pixels.chunks_exact_mut(4) {
        let b = chunk[0];
        let r = chunk[2];
        chunk[0] = r;
        chunk[2] = b;
        if chunk[3] != 0 {
            has_alpha = true;
        }
    }

    // If icon has no alpha channel (common in older win32 24bpp icons), fill alpha to 255
    if !has_alpha {
        for chunk in raw_pixels.chunks_exact_mut(4) {
            chunk[3] = 255;
        }
    }

    let img_buffer = image::RgbaImage::from_raw(width, height, raw_pixels)?;
    let mut png_bytes: Vec<u8> = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut png_bytes);
    if img_buffer.write_to(&mut cursor, image::ImageFormat::Png).is_err() {
        return None;
    }

    let b64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);
    Some(format!("data:image/png;base64,{}", b64))
}

#[cfg(not(windows))]
pub fn enumerate_windows(_shielded_exes: &HashSet<String>, _shield_mode: &str, _auto_reapply: bool) -> Vec<WindowInfo> { vec![] }

#[cfg(not(windows))]
pub fn find_all_windows_for_exe(_target_exe: &str) -> Vec<(usize, u32)> { vec![] }

#[cfg(not(windows))]
pub fn auto_reapply_shields(_shielded_exes: &HashSet<String>, _shield_mode: &str) {}

#[cfg(not(windows))]
pub fn query_live_affinity(_hwnd: usize) -> bool { false }

#[cfg(not(windows))]
pub fn get_live_affinity_raw(_hwnd: usize) -> u32 { 0 }
