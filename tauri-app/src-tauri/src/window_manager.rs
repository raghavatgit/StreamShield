use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub hwnd: usize,
    pub pid: u32,
    pub title: String,
    pub exe_name: String,
    pub is_shielded: bool,
    pub is_audio_shielded: bool,
    pub icon_base64: Option<String>,
}

#[cfg(windows)]
pub fn enumerate_windows(
    shielded_exes: &std::collections::HashSet<String>,
    audio_shielded_exes: &std::collections::HashSet<String>,
) -> Vec<WindowInfo> {
    use std::collections::{HashMap, HashSet};
    use winapi::shared::minwindef::{BOOL, LPARAM};
    use winapi::shared::windef::HWND;
    use winapi::um::winuser::{
        EnumWindows, GetWindowLongW, GetWindowTextLengthW, GetWindowTextW,
        GetWindowThreadProcessId, IsWindowVisible, GWL_EXSTYLE, GWL_STYLE,
        WS_EX_TOOLWINDOW, WS_VISIBLE,
    };
    use winapi::um::dwmapi::{DwmGetWindowAttribute, DWMWA_CLOAKED, DWM_CLOAKED_APP, DWM_CLOAKED_SHELL, DWM_CLOAKED_INHERITED};

    // System processes that should never be listed in the app
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
    ];

    unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        if IsWindowVisible(hwnd) == 0 {
            return 1;
        }

        // Filter out zero-length window titles
        if GetWindowTextLengthW(hwnd) == 0 {
            return 1;
        }

        // Filter out Tool Windows (floating palettes, tooltip hosts)
        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
        if (ex_style & WS_EX_TOOLWINDOW) != 0 {
            return 1;
        }

        // Filter out invisible styles
        let style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
        if (style & WS_VISIBLE) == 0 {
            return 1;
        }

        // Filter out Cloaked UWP/Windows 10/11 dummy windows
        let mut cloaked: u32 = 0;
        let hr = DwmGetWindowAttribute(
            hwnd,
            DWMWA_CLOAKED,
            &mut cloaked as *mut _ as *mut _,
            std::mem::size_of::<u32>() as u32,
        );
        if hr == 0 && (cloaked & (DWM_CLOAKED_APP | DWM_CLOAKED_SHELL | DWM_CLOAKED_INHERITED)) != 0 {
            return 1;
        }

        let list = &mut *(lparam as *mut Vec<HWND>);
        list.push(hwnd);
        1
    }

    let mut hwnds: Vec<HWND> = Vec::new();
    unsafe { EnumWindows(Some(enum_proc), &mut hwnds as *mut Vec<HWND> as LPARAM); }

    let mut results: Vec<WindowInfo> = Vec::new();
    let mut seen_exe: HashSet<String> = HashSet::new();
    let mut pid_exe_cache: HashMap<u32, (String, String)> = HashMap::new(); // PID -> (exe_name, full_path)
    let mut icon_cache: HashMap<String, Option<String>> = HashMap::new(); // exe_name -> base64_icon

    for hwnd in hwnds {
        let mut title_buf = vec![0u16; 512];
        let title_len = unsafe { GetWindowTextW(hwnd, title_buf.as_mut_ptr(), 512) };
        if title_len == 0 { continue; }
        let title = String::from_utf16_lossy(&title_buf[..title_len as usize]).trim().to_string();
        if title.is_empty() || title == "Program Manager" { continue; }

        let mut pid: u32 = 0;
        unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
        if pid == 0 { continue; }

        let (exe_name, full_path) = pid_exe_cache.entry(pid).or_insert_with(|| get_process_info(pid)).clone();
        let exe_lower = exe_name.to_lowercase();

        // Check against system exclusion list
        if EXCLUDED_PROCESSES.contains(&exe_lower.as_str()) {
            continue;
        }

        // Filter ApplicationFrameHost if title is generic
        if exe_lower == "applicationframehost.exe" && (title.is_empty() || title == "ApplicationFrameHost") {
            continue;
        }

        if seen_exe.contains(&exe_lower) {
            continue;
        }
        seen_exe.insert(exe_lower.clone());

        // Ground-truth live OS display affinity check
        let mut is_shielded = query_live_affinity(hwnd as usize);

        // AUTO-REAPPLY: If this executable was previously shielded by user config,
        // but this window instance isn't shielded yet (e.g. freshly launched app),
        // automatically inject and reapply the shield immediately!
        let should_be_shielded = shielded_exes.iter().any(|s| s.eq_ignore_ascii_case(&exe_name) || s.eq_ignore_ascii_case(&exe_lower));
        if should_be_shielded && !is_shielded {
            let _ = crate::injector::set_window_affinity(hwnd as usize, true);
            is_shielded = query_live_affinity(hwnd as usize);
        }

        // Audio Stream Exclusion State
        let is_audio_shielded = audio_shielded_exes.iter().any(|s| s.eq_ignore_ascii_case(&exe_name) || s.eq_ignore_ascii_case(&exe_lower))
            || crate::audio_bridge::is_audio_shielded(&exe_name, pid);

        if is_audio_shielded {
            crate::audio_bridge::exclude_process_audio(&exe_name, pid);
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
            is_audio_shielded,
            icon_base64,
        });
    }

    // Sort: shielded apps first, then alphabetically
    results.sort_by(|a, b| {
        b.is_shielded.cmp(&a.is_shielded)
            .then_with(|| a.exe_name.to_lowercase().cmp(&b.exe_name.to_lowercase()))
    });

    results
}

/// Helper for background watchdog to auto-shield freshly opened windows without overhead
#[cfg(windows)]
pub fn auto_reapply_shields(shielded_exes: &std::collections::HashSet<String>, audio_shielded_exes: &std::collections::HashSet<String>) {
    if shielded_exes.is_empty() && audio_shielded_exes.is_empty() {
        return;
    }
    use winapi::shared::minwindef::{BOOL, LPARAM};
    use winapi::shared::windef::HWND;
    use winapi::um::winuser::{EnumWindows, GetWindowThreadProcessId, IsWindowVisible};

    unsafe extern "system" fn watch_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        if IsWindowVisible(hwnd) == 0 { return 1; }
        let (shielded_set, audio_set) = &*(lparam as *const (std::collections::HashSet<String>, std::collections::HashSet<String>));

        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == 0 { return 1; }

        let (exe_name, _) = get_process_info(pid);
        let exe_lower = exe_name.to_lowercase();

        let should_shield = shielded_set.iter().any(|s| s.eq_ignore_ascii_case(&exe_name) || s.eq_ignore_ascii_case(&exe_lower));
        if should_shield && !query_live_affinity(hwnd as usize) {
            let _ = crate::injector::set_window_affinity(hwnd as usize, true);
        }

        let should_audio_shield = audio_set.iter().any(|s| s.eq_ignore_ascii_case(&exe_name) || s.eq_ignore_ascii_case(&exe_lower));
        if should_audio_shield {
            crate::audio_bridge::exclude_process_audio(&exe_name, pid);
        }

        1
    }

    let payload = (shielded_exes.clone(), audio_shielded_exes.clone());
    unsafe {
        EnumWindows(Some(watch_proc), &payload as *const _ as LPARAM);
    }
}

/// Query the true, ground-truth Display Affinity from Windows OS
#[cfg(windows)]
fn query_live_affinity(hwnd: usize) -> bool {
    use winapi::um::winuser::GetWindowDisplayAffinity;
    const WDA_EXCLUDEFROMCAPTURE: u32 = 0x00000011;
    let mut affinity: u32 = 0;
    let ok = unsafe { GetWindowDisplayAffinity(hwnd as _, &mut affinity) };
    ok != 0 && affinity == WDA_EXCLUDEFROMCAPTURE
}

#[cfg(windows)]
fn get_process_info(pid: u32) -> (String, String) {
    use winapi::um::processthreadsapi::OpenProcess;
    use winapi::um::psapi::GetProcessImageFileNameW;
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::winnt::PROCESS_QUERY_LIMITED_INFORMATION;

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return (format!("PID {pid}"), String::new());
        }

        let mut buf = vec![0u16; 1024];
        let len = GetProcessImageFileNameW(handle, buf.as_mut_ptr(), 1024);
        CloseHandle(handle);

        if len == 0 {
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

        // 1. Try WM_GETICON (ICON_BIG) with 100ms timeout
        let mut result: usize = 0;
        if SendMessageTimeoutW(hwnd, WM_GETICON, ICON_BIG as _, 0, SMTO_ABORTIFHUNG, 100, &mut result as *mut _ as _) != 0 && result != 0 {
            hicon = result as HICON;
        }

        // 2. Try WM_GETICON (ICON_SMALL)
        if hicon.is_null() {
            if SendMessageTimeoutW(hwnd, WM_GETICON, ICON_SMALL as _, 0, SMTO_ABORTIFHUNG, 100, &mut result as *mut _ as _) != 0 && result != 0 {
                hicon = result as HICON;
            }
        }

        // 3. Try WM_GETICON (ICON_SMALL2)
        if hicon.is_null() {
            if SendMessageTimeoutW(hwnd, WM_GETICON, ICON_SMALL2 as _, 0, SMTO_ABORTIFHUNG, 100, &mut result as *mut _ as _) != 0 && result != 0 {
                hicon = result as HICON;
            }
        }

        // 4. Try GetClassLongPtrW (GCLP_HICON)
        if hicon.is_null() {
            let class_icon = GetClassLongPtrW(hwnd, GCLP_HICON);
            if class_icon != 0 {
                hicon = class_icon as HICON;
            }
        }

        // 5. Try GetClassLongPtrW (GCLP_HICONSM)
        if hicon.is_null() {
            let class_icon_sm = GetClassLongPtrW(hwnd, GCLP_HICONSM);
            if class_icon_sm != 0 {
                hicon = class_icon_sm as HICON;
            }
        }

        // 6. Fallback: ExtractIconExW from the executable file path
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

    // Encode to PNG bytes using image crate
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
pub fn enumerate_windows(
    _shielded_exes: &std::collections::HashSet<String>,
    _audio_shielded_exes: &std::collections::HashSet<String>,
) -> Vec<WindowInfo> { vec![] }

#[cfg(not(windows))]
pub fn auto_reapply_shields(
    _shielded_exes: &std::collections::HashSet<String>,
    _audio_shielded_exes: &std::collections::HashSet<String>,
) {}