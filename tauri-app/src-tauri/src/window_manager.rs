use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub hwnd: usize,
    pub pid: u32,
    pub title: String,
    pub exe_name: String,
    pub is_shielded: bool,
    pub icon_base64: Option<String>,
}

#[cfg(windows)]
pub fn enumerate_windows() -> Vec<WindowInfo> {
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
            &mut cloaked as *mut _ as _,
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
        let is_shielded = query_live_affinity(hwnd as usize);

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

    // Sort: shielded apps first, then alphabetically
    results.sort_by(|a, b| {
        b.is_shielded.cmp(&a.is_shielded)
            .then_with(|| a.exe_name.to_lowercase().cmp(&b.exe_name.to_lowercase()))
    });

    results
}

/// Query the true, ground-truth Display Affinity from Windows OS
#[cfg(windows)]
fn query_live_affinity(hwnd: usize) -> bool {
    use winapi::um::winuser::GetWindowDisplayAffinity;
    const WDA_EXCLUDEFROMCAPTURE: u32 = 0x00000011;
    const WDA_MONITOR: u32 = 0x00000001;

    let mut affinity: u32 = 0;
    let ok = unsafe { GetWindowDisplayAffinity(hwnd as _, &mut affinity) };
    ok != 0 && (affinity == WDA_EXCLUDEFROMCAPTURE || affinity == WDA_MONITOR)
}

#[cfg(windows)]
fn get_process_info(pid: u32) -> (String, String) {
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::processthreadsapi::OpenProcess;
    use winapi::um::psapi::GetModuleFileNameExW;
    use winapi::um::winnt::{PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, pid);
        if handle.is_null() {
            return ("Unknown".to_string(), String::new());
        }
        let mut buf = vec![0u16; 1024];
        let len = GetModuleFileNameExW(handle, std::ptr::null_mut(), buf.as_mut_ptr(), 1024);
        CloseHandle(handle);
        if len == 0 {
            return ("Unknown".to_string(), String::new());
        }
        let full = String::from_utf16_lossy(&buf[..len as usize]);
        let exe = full.split('\\').last().unwrap_or("Unknown").to_string();
        (exe, full)
    }
}

/// Extract real HICON from window or executable and convert to Base64 PNG data URL
#[cfg(windows)]
fn extract_window_or_exe_icon(hwnd: winapi::shared::windef::HWND, exe_path: &str) -> Option<String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::shared::windef::HICON;
    use winapi::um::winuser::{
        DestroyIcon, GetClassLongPtrW, SendMessageTimeoutW,
        GCLP_HICON, GCLP_HICONSM, ICON_BIG, ICON_SMALL2, SMTO_ABORTIFHUNG, WM_GETICON,
    };
    use winapi::um::shellapi::ExtractIconExW;

    unsafe {
        let mut hicon: HICON = std::ptr::null_mut();

        // 1. Try SendMessageTimeout with WM_GETICON
        let mut result = 0usize;
        let ok = SendMessageTimeoutW(
            hwnd,
            WM_GETICON,
            ICON_BIG as _,
            0,
            SMTO_ABORTIFHUNG,
            100,
            &mut result,
        );
        if ok != 0 && result != 0 {
            hicon = result as HICON;
        }

        if hicon.is_null() {
            let ok_sm = SendMessageTimeoutW(
                hwnd,
                WM_GETICON,
                ICON_SMALL2 as _,
                0,
                SMTO_ABORTIFHUNG,
                100,
                &mut result,
            );
            if ok_sm != 0 && result != 0 {
                hicon = result as HICON;
            }
        }

        // 2. Try window class icon
        if hicon.is_null() {
            let cls_icon = GetClassLongPtrW(hwnd, GCLP_HICON);
            if cls_icon != 0 {
                hicon = cls_icon as HICON;
            } else {
                let cls_icon_sm = GetClassLongPtrW(hwnd, GCLP_HICONSM);
                if cls_icon_sm != 0 {
                    hicon = cls_icon_sm as HICON;
                }
            }
        }

        // 3. Fallback: Extract from EXE file path
        let mut should_destroy = false;
        if hicon.is_null() && !exe_path.is_empty() {
            let wide_path: Vec<u16> = OsStr::new(exe_path).encode_wide().chain(std::iter::once(0)).collect();
            let mut large_icon: HICON = std::ptr::null_mut();
            let mut small_icon: HICON = std::ptr::null_mut();
            let count = ExtractIconExW(wide_path.as_ptr(), 0, &mut large_icon, &mut small_icon, 1);
            if count > 0 {
                if !large_icon.is_null() {
                    hicon = large_icon;
                    should_destroy = true;
                    if !small_icon.is_null() { DestroyIcon(small_icon); }
                } else if !small_icon.is_null() {
                    hicon = small_icon;
                    should_destroy = true;
                }
            }
        }

        if hicon.is_null() {
            return None;
        }

        let base64_png = hicon_to_base64_png(hicon);
        if should_destroy {
            DestroyIcon(hicon);
        }
        base64_png
    }
}

/// Converts a Win32 HICON to a PNG base64 string
#[cfg(windows)]
unsafe fn hicon_to_base64_png(hicon: winapi::shared::windef::HICON) -> Option<String> {
    use base64::Engine;
    use winapi::shared::windef::HDC;
    use winapi::um::wingdi::{
        CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, GetObjectW,
        BITMAP, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    };
    use winapi::um::winuser::{GetDC, GetIconInfo, ReleaseDC, ICONINFO};

    let mut icon_info: ICONINFO = std::mem::zeroed();
    if GetIconInfo(hicon, &mut icon_info) == 0 {
        return None;
    }

    let hdc_screen = GetDC(std::ptr::null_mut());
    let hdc_mem: HDC = CreateCompatibleDC(hdc_screen);

    let mut bmp: BITMAP = std::mem::zeroed();
    GetObjectW(
        icon_info.hbmColor as _,
        std::mem::size_of::<BITMAP>() as i32,
        &mut bmp as *mut _ as _,
    );

    let width = bmp.bmWidth as u32;
    let height = bmp.bmHeight as u32;
    if width == 0 || height == 0 || width > 512 || height > 512 {
        if !icon_info.hbmColor.is_null() { DeleteObject(icon_info.hbmColor as _); }
        if !icon_info.hbmMask.is_null() { DeleteObject(icon_info.hbmMask as _); }
        DeleteDC(hdc_mem);
        ReleaseDC(std::ptr::null_mut(), hdc_screen);
        return None;
    }

    let mut bi: BITMAPINFO = std::mem::zeroed();
    bi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
    bi.bmiHeader.biWidth = width as i32;
    bi.bmiHeader.biHeight = -(height as i32); // Top-down DIB
    bi.bmiHeader.biPlanes = 1;
    bi.bmiHeader.biBitCount = 32;
    bi.bmiHeader.biCompression = BI_RGB;

    let mut raw_pixels = vec![0u8; (width * height * 4) as usize];
    let lines = GetDIBits(
        hdc_mem,
        icon_info.hbmColor,
        0,
        height,
        raw_pixels.as_mut_ptr() as _,
        &mut bi,
        DIB_RGB_COLORS,
    );

    if !icon_info.hbmColor.is_null() { DeleteObject(icon_info.hbmColor as _); }
    if !icon_info.hbmMask.is_null() { DeleteObject(icon_info.hbmMask as _); }
    DeleteDC(hdc_mem);
    ReleaseDC(std::ptr::null_mut(), hdc_screen);

    if lines == 0 {
        return None;
    }

    // Convert BGRA to RGBA and check if alpha channel has non-zero values
    let mut has_alpha = false;
    for chunk in raw_pixels.chunks_exact_mut(4) {
        let b = chunk[0];
        let r = chunk[2];
        let a = chunk[3];
        chunk[0] = r;
        chunk[2] = b;
        if a > 0 {
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
pub fn enumerate_windows() -> Vec<WindowInfo> { vec![] }