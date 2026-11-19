// Always hide console window - no terminal alongside the app
#![windows_subsystem = "windows"]

mod window_manager;
mod injector;
mod config;
mod nvidia_bypass;

use std::sync::Mutex;
use serde::Serialize;
use tauri::{
    menu::{IsMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    webview::WebviewWindowBuilder,
    Emitter, LogicalSize, Manager, PhysicalPosition, PhysicalRect, Size, State, WebviewUrl, WebviewWindow,
};
use window_manager::WindowInfo;
use config::{load_config, save_config, AppSettings, ShieldConfig};

/// Identical WebView2 browser flags shared by ALL windows (main + widget).
/// WebView2 requires every window sharing a UserDataFolder to pass identical browser arguments,
/// otherwise secondary WebView2 creation fails with HRESULT 0x8007139F (ERROR_INVALID_STATE).
const WEBVIEW_BROWSER_ARGS: &str = "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection --disable-background-timer-throttling --disable-renderer-backgrounding --disable-backgrounding-occluded-windows";

#[derive(Debug, Clone, Copy, Serialize, Default)]
struct BadgeExpandResult {
    expand_up: bool,
    expand_left: bool,
}

#[derive(Debug, Clone, Serialize)]
struct OsInfo {
    build: u32,
    is_win11: bool,
    supports_exclude_from_capture: bool,
    display_name: String,
}

#[derive(Serialize)]
struct WidgetInitPayload {
    style: String,
    theme: String,
    self_stealth: bool,
    widget_stealth: bool,
}

struct AppState {
    config: Mutex<ShieldConfig>,
    tray_status: Mutex<Option<MenuItem<tauri::Wry>>>,
    last_widget_pos: Mutex<Option<(i32, i32)>>,
    widget_badge_expanded: Mutex<bool>,
    badge_pre_expand_pos: Mutex<Option<(i32, i32)>>,
    badge_last_dir: Mutex<BadgeExpandResult>,
}

// ── Windows Version & DWM helpers ───────────────────────────────────────────

#[cfg(windows)]
fn read_windows_build_number() -> u32 {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::winnt::{KEY_QUERY_VALUE, REG_SZ};
    use winapi::um::winreg::{RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY_LOCAL_MACHINE};

    fn wide(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    }

    let subkey = wide(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion");
    let val_name = wide("CurrentBuildNumber");
    unsafe {
        let mut hkey = std::ptr::null_mut();
        if RegOpenKeyExW(HKEY_LOCAL_MACHINE, subkey.as_ptr(), 0, KEY_QUERY_VALUE, &mut hkey) != 0 {
            return 19045;
        }
        let mut data_type: u32 = 0;
        let mut buf = [0u16; 64];
        let mut bytes_len: u32 = (buf.len() * 2) as u32;
        let ok = RegQueryValueExW(
            hkey,
            val_name.as_ptr(),
            std::ptr::null_mut(),
            &mut data_type,
            buf.as_mut_ptr() as *mut u8,
            &mut bytes_len,
        );
        RegCloseKey(hkey);
        if ok == 0 && data_type == REG_SZ && bytes_len >= 2 {
            let u16_len = (bytes_len as usize) / 2;
            let trimmed = &buf[..u16_len.min(buf.len())];
            let end = trimmed.iter().position(|&c| c == 0).unwrap_or(trimmed.len());
            if let Ok(parsed) = String::from_utf16_lossy(&trimmed[..end]).trim().parse::<u32>() {
                return parsed;
            }
        }
        19045
    }
}

#[cfg(not(windows))]
fn read_windows_build_number() -> u32 { 0 }

fn query_os_info() -> OsInfo {
    let build = read_windows_build_number();
    let is_win11 = build >= 22000;
    let supports_exclude_from_capture = build >= 19041;
    let display_name = if build == 0 {
        "Non-Windows".to_string()
    } else if is_win11 {
        format!("Windows 11 (Build {})", build)
    } else {
        format!("Windows 10 (Build {})", build)
    };
    OsInfo {
        build,
        is_win11,
        supports_exclude_from_capture,
        display_name,
    }
}

/// Apply crisp Windows 11 DWM rounded corners (or sharp square when maximized). No-op on Windows 10.
#[cfg(windows)]
fn configure_main_window_dwm(hwnd: usize, maximized: bool) {
    if read_windows_build_number() < 22000 {
        return;
    }
    use winapi::um::dwmapi::DwmSetWindowAttribute;
    const DWMWA_WINDOW_CORNER_PREFERENCE: u32 = 33;
    const DWMWCP_DONOTROUND: u32 = 1;
    const DWMWCP_ROUND: u32 = 2;
    let pref = if maximized { DWMWCP_DONOTROUND } else { DWMWCP_ROUND };
    unsafe {
        let _ = DwmSetWindowAttribute(
            hwnd as _,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &pref as *const u32 as *const _,
            std::mem::size_of::<u32>() as u32,
        );
    }
}

/// Configure Win32 extended styles & Win11 DWM attributes for the frameless transparent widget:
/// - Ensures WS_EX_TOOLWINDOW and strips WS_EX_APPWINDOW so the widget stays out of Alt+Tab.
/// - Disables Win11 DWM border stroke & forced window corner rounding so shaped CSS radii render cleanly.
#[cfg(windows)]
fn configure_widget_window_win32(hwnd: usize) {
    use winapi::um::winuser::{
        GetWindowLongW, SetWindowLongW, GWL_EXSTYLE, WS_EX_APPWINDOW, WS_EX_TOOLWINDOW,
    };
    unsafe {
        let ex_style = GetWindowLongW(hwnd as _, GWL_EXSTYLE) as u32;
        let new_ex = (ex_style | WS_EX_TOOLWINDOW) & !WS_EX_APPWINDOW;
        if new_ex != ex_style {
            SetWindowLongW(hwnd as _, GWL_EXSTYLE, new_ex as i32);
        }
    }

    if read_windows_build_number() >= 22000 {
        use winapi::um::dwmapi::DwmSetWindowAttribute;
        const DWMWA_WINDOW_CORNER_PREFERENCE: u32 = 33;
        const DWMWA_BORDER_COLOR: u32 = 34;
        const DWMWCP_DONOTROUND: u32 = 1;
        const DWMWA_COLOR_NONE: u32 = 0xFFFFFFFE;
        unsafe {
            let pref = DWMWCP_DONOTROUND;
            let _ = DwmSetWindowAttribute(
                hwnd as _,
                DWMWA_WINDOW_CORNER_PREFERENCE,
                &pref as *const u32 as *const _,
                std::mem::size_of::<u32>() as u32,
            );
            let border_none = DWMWA_COLOR_NONE;
            let _ = DwmSetWindowAttribute(
                hwnd as _,
                DWMWA_BORDER_COLOR,
                &border_none as *const u32 as *const _,
                std::mem::size_of::<u32>() as u32,
            );
        }
    }
}

// ── Admin & Power helpers ───────────────────────────────────────────────────

#[cfg(windows)]
fn is_admin() -> bool {
    use winapi::um::processthreadsapi::{GetCurrentProcess, OpenProcessToken};
    use winapi::um::securitybaseapi::GetTokenInformation;
    use winapi::um::winnt::{TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY, HANDLE};
    use winapi::um::handleapi::CloseHandle;
    unsafe {
        let mut token: HANDLE = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 { return false; }
        let mut elev = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut sz = std::mem::size_of::<TOKEN_ELEVATION>() as u32;
        let ok = GetTokenInformation(token, TokenElevation, &mut elev as *mut _ as *mut _, sz, &mut sz);
        CloseHandle(token);
        ok != 0 && elev.TokenIsElevated != 0
    }
}

#[cfg(windows)]
fn relaunch_as_admin(start_minimized: bool) {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::shellapi::ShellExecuteW;
    use winapi::um::winuser::{SW_SHOW, SW_HIDE};
    fn wide(s: &str) -> Vec<u16> { OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect() }
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => return,
    };
    let exe_w = wide(exe.to_str().unwrap_or_default());
    let verb_w = wide("runas");

    let mut raw_args: Vec<String> = std::env::args().skip(1).collect();
    let has_minimized_flag = raw_args.iter().any(|a| a == "--minimized" || a == "-m");
    let is_minimized = start_minimized || has_minimized_flag;
    if is_minimized && !has_minimized_flag {
        raw_args.push("--minimized".to_string());
    }
    let show_cmd = if is_minimized { SW_HIDE } else { SW_SHOW };

    let args_str = raw_args.join(" ");
    let args_w = wide(&args_str);
    let params_ptr = if raw_args.is_empty() { std::ptr::null() } else { args_w.as_ptr() };

    unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            verb_w.as_ptr(),
            exe_w.as_ptr(),
            params_ptr,
            std::ptr::null(),
            show_cmd as i32,
        );
    }
}

/// Disable Windows 10/11 EcoQoS / Efficiency Mode throttling on the StreamShield process
/// without preventing the user's PC from entering sleep when idle.
#[cfg(windows)]
fn disable_power_throttling() {
    use winapi::um::processthreadsapi::{GetCurrentProcess, SetProcessInformation};

    #[allow(non_snake_case)]
    #[repr(C)]
    struct PROCESS_POWER_THROTTLING_STATE {
        Version: u32,
        ControlMask: u32,
        StateMask: u32,
    }

    // Correct Windows SDK value from PROCESS_INFORMATION_CLASS enum: ProcessPowerThrottling == 4
    const PROCESS_POWER_THROTTLING: u32 = 4;
    const PROCESS_POWER_THROTTLING_CURRENT_VERSION: u32 = 1;
    const PROCESS_POWER_THROTTLING_EXECUTION_SPEED: u32 = 0x1;
    const PROCESS_POWER_THROTTLING_IGNORE_TIMER_RESOLUTION: u32 = 0x4;

    unsafe {
        // Try both EXECUTION_SPEED + IGNORE_TIMER_RESOLUTION (Win11+), fallback to EXECUTION_SPEED (Win10)
        let mut throttling = PROCESS_POWER_THROTTLING_STATE {
            Version: PROCESS_POWER_THROTTLING_CURRENT_VERSION,
            ControlMask: PROCESS_POWER_THROTTLING_EXECUTION_SPEED
                | PROCESS_POWER_THROTTLING_IGNORE_TIMER_RESOLUTION,
            StateMask: 0,
        };
        let ok = SetProcessInformation(
            GetCurrentProcess(),
            PROCESS_POWER_THROTTLING as _,
            &mut throttling as *mut _ as *mut _,
            std::mem::size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
        );
        if ok == 0 {
            let mut fallback = PROCESS_POWER_THROTTLING_STATE {
                Version: PROCESS_POWER_THROTTLING_CURRENT_VERSION,
                ControlMask: PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
                StateMask: 0,
            };
            let _ = SetProcessInformation(
                GetCurrentProcess(),
                PROCESS_POWER_THROTTLING as _,
                &mut fallback as *mut _ as *mut _,
                std::mem::size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
            );
        }
    }
}

/// Enforces single-instance execution and creates a cross-integrity wake event so
/// double-clicking the executable while docked to the tray or started `--minimized`
/// immediately restores the main window across Windows UIPI boundaries.
#[cfg(windows)]
fn ensure_single_instance() -> Option<usize> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::shared::winerror::ERROR_ALREADY_EXISTS;
    use winapi::um::errhandlingapi::GetLastError;
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::minwinbase::SECURITY_ATTRIBUTES;
    use winapi::um::synchapi::{CreateEventW, CreateMutexW, OpenEventW, SetEvent};
    use winapi::um::winbase::LocalFree;
    use winapi::um::winnt::{EVENT_MODIFY_STATE, PSECURITY_DESCRIPTOR};
    use winapi::um::winuser::{FindWindowW, SetForegroundWindow, ShowWindow, SW_RESTORE};

    #[link(name = "advapi32")]
    extern "system" {
        fn ConvertStringSecurityDescriptorToSecurityDescriptorW(
            StringSecurityDescriptor: *const u16,
            StringSDRevision: u32,
            SecurityDescriptor: *mut PSECURITY_DESCRIPTOR,
            SecurityDescriptorSize: *mut u32,
        ) -> i32;
    }

    fn wide(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    }

    let mutex_name = wide("Global\\StreamShield_SingleInstance_Mutex");
    let wake_name = wide("Local\\StreamShield_WakeEvent");

    unsafe {
        let mut p_sd: PSECURITY_DESCRIPTOR = std::ptr::null_mut();
        // Allow Everyone (WD) SYNCHRONIZE/EVENT_MODIFY_STATE + Low Mandatory Label so standard-user clicks can signal the elevated primary
        let sddl = wide("D:(A;;GA;;;SY)(A;;GA;;;BA)(A;;0x00120002;;;WD)S:(ML;;NW;;;LW)");
        let has_sd = ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl.as_ptr(),
            1,
            &mut p_sd,
            std::ptr::null_mut(),
        ) != 0;

        let mut sa = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: if has_sd { p_sd } else { std::ptr::null_mut() },
            bInheritHandle: 0,
        };
        let sa_ptr = if has_sd { &mut sa as *mut SECURITY_ATTRIBUTES } else { std::ptr::null_mut() };

        let mutex = CreateMutexW(sa_ptr, 1, mutex_name.as_ptr());
        let last_err = GetLastError();
        if mutex.is_null() || last_err == ERROR_ALREADY_EXISTS {
            if !mutex.is_null() {
                CloseHandle(mutex);
            }
            if has_sd {
                LocalFree(p_sd as _);
            }
            // Signal already-running instance (even if minimized/tray-only/elevated)
            let ev = OpenEventW(EVENT_MODIFY_STATE, 0, wake_name.as_ptr());
            if !ev.is_null() {
                SetEvent(ev);
                CloseHandle(ev);
            } else {
                let title = wide("StreamShield");
                let hwnd = FindWindowW(std::ptr::null(), title.as_ptr());
                if !hwnd.is_null() {
                    ShowWindow(hwnd, SW_RESTORE);
                    SetForegroundWindow(hwnd);
                }
            }
            return None;
        }

        // Primary instance holds mutex handle until process exit
        let wake_event = CreateEventW(sa_ptr, 0, 0, wake_name.as_ptr());
        if has_sd {
            LocalFree(p_sd as _);
        }
        let _ = mutex;
        Some(if wake_event.is_null() { 0 } else { wake_event as usize })
    }
}

#[cfg(not(windows))]
fn ensure_single_instance() -> Option<usize> { Some(0) }

#[cfg(not(windows))]
fn disable_power_throttling() {}

// ── Windows Autostart Registry helpers ───────────────────────────────────────

#[cfg(windows)]
fn set_autostart_registry(enable: bool, start_minimized: bool) -> Result<(), String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::winreg::{RegOpenKeyExW, RegSetValueExW, RegDeleteValueW, RegCloseKey, HKEY_CURRENT_USER};
    use winapi::um::winnt::{KEY_SET_VALUE, REG_SZ};

    fn wide(s: &str) -> Vec<u16> { OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect() }

    let subkey = wide(r"Software\Microsoft\Windows\CurrentVersion\Run");
    let app_name = wide("StreamShield");

    unsafe {
        let mut hkey = std::ptr::null_mut();
        let status = RegOpenKeyExW(HKEY_CURRENT_USER, subkey.as_ptr(), 0, KEY_SET_VALUE, &mut hkey);
        if status != 0 {
            return Err(format!("RegOpenKeyExW failed with error code: {}", status));
        }

        let res = if enable {
            let current_exe = std::env::current_exe().map_err(|e| e.to_string())?;
            let cmd = if start_minimized {
                format!("\"{}\" --minimized", current_exe.to_string_lossy())
            } else {
                format!("\"{}\"", current_exe.to_string_lossy())
            };
            let cmd_w = wide(&cmd);
            let bytes_len = (cmd_w.len() * std::mem::size_of::<u16>()) as u32;
            let set_res = RegSetValueExW(
                hkey,
                app_name.as_ptr(),
                0,
                REG_SZ,
                cmd_w.as_ptr() as *const _,
                bytes_len,
            );
            if set_res != 0 {
                Err(format!("RegSetValueExW failed with error: {}", set_res))
            } else {
                Ok(())
            }
        } else {
            let del_res = RegDeleteValueW(hkey, app_name.as_ptr());
            if del_res != 0 && del_res != 2 {
                Err(format!("RegDeleteValueW failed with error: {}", del_res))
            } else {
                Ok(())
            }
        };

        RegCloseKey(hkey);
        res
    }
}

#[cfg(not(windows))]
fn set_autostart_registry(_enable: bool, _start_minimized: bool) -> Result<(), String> {
    Ok(())
}

#[cfg(windows)]
fn get_autostart_registry() -> bool {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::winreg::{RegOpenKeyExW, RegQueryValueExW, RegCloseKey, HKEY_CURRENT_USER};
    use winapi::um::winnt::KEY_QUERY_VALUE;

    fn wide(s: &str) -> Vec<u16> { OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect() }

    let subkey = wide(r"Software\Microsoft\Windows\CurrentVersion\Run");
    let app_name = wide("StreamShield");

    unsafe {
        let mut hkey = std::ptr::null_mut();
        if RegOpenKeyExW(HKEY_CURRENT_USER, subkey.as_ptr(), 0, KEY_QUERY_VALUE, &mut hkey) != 0 {
            return false;
        }
        let mut data_type = 0u32;
        let mut data_len = 0u32;
        let query_res = RegQueryValueExW(
            hkey,
            app_name.as_ptr(),
            std::ptr::null_mut(),
            &mut data_type,
            std::ptr::null_mut(),
            &mut data_len,
        );
        RegCloseKey(hkey);
        query_res == 0 && data_len > 2
    }
}

#[cfg(not(windows))]
fn get_autostart_registry() -> bool {
    false
}

// ── Tray status & menu helpers ──────────────────────────────────────────────

fn tray_status_text(count: usize) -> String {
    if count == 0 {
        "No apps shielded".to_string()
    } else {
        format!("🛡️  {} app{} hidden from capture", count, if count == 1 { "" } else { "s" })
    }
}

fn update_tray_status_count(state: &AppState, active_count: usize) {
    if let Ok(guard) = state.tray_status.lock() {
        if let Some(mi) = guard.as_ref() {
            let _ = mi.set_text(tray_status_text(active_count));
        }
    }
}

fn build_tray_menu_from_windows(
    app: &tauri::AppHandle,
    wins: &[WindowInfo],
) -> Result<Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let state = match app.try_state::<AppState>() {
        Some(s) => s,
        None => return Err("AppState not found".into()),
    };

    let (shielded, mode, is_compact, widget_en, widget_st) = {
        let cfg = state.config.lock().map_err(|_| "Config lock poisoned")?;
        (
            cfg.shielded_exes.clone(),
            cfg.settings.shield_mode.clone(),
            cfg.settings.compact_mode,
            cfg.settings.widget_enabled,
            cfg.settings.widget_style.clone(),
        )
    };

    let active_count = wins.iter().filter(|w| w.is_shielded).count();

    let show = MenuItem::with_id(app, "show", "🛡️  Open StreamShield", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let status_text = tray_status_text(active_count);
    let status = MenuItem::with_id(app, "status", status_text, false, None::<&str>)?;
    let sep2 = PredefinedMenuItem::separator(app)?;

    if let Ok(mut guard) = state.tray_status.lock() {
        *guard = Some(status.clone());
    }

    // ── Apps Toggle Submenu: unique running applications ──
    let mut app_items: Vec<MenuItem<tauri::Wry>> = Vec::new();
    let mut seen_exes = std::collections::HashSet::new();
    for w in wins {
        let exe_lower = w.exe_name.to_lowercase();
        if !seen_exes.insert(exe_lower.clone()) {
            continue;
        }
        let is_sh = w.is_shielded || shielded.contains(&exe_lower);
        let prefix = if is_sh { "✓  " } else { "○  " };
        let label = format!("{}{}", prefix, w.exe_name);
        let item_id = format!("toggle_app:{}", w.exe_name);
        if let Ok(item) = MenuItem::with_id(app, item_id, label, true, None::<&str>) {
            app_items.push(item);
        }
        if app_items.len() >= 15 {
            break;
        }
    }

    let item_refs: Vec<&dyn IsMenuItem<tauri::Wry>> =
        app_items.iter().map(|i| i as &dyn IsMenuItem<tauri::Wry>).collect();
    let empty_item = MenuItem::with_id(app, "noop", "(No detectable windows)", false, None::<&str>)?;
    let apps_submenu = if app_items.is_empty() {
        Submenu::with_items(app, "📱  Toggle Apps", true, &[&empty_item])?
    } else {
        Submenu::with_items(app, "📱  Toggle Apps", true, &item_refs)?
    };

    // ── Quick Actions Submenu ──
    let shield_all = MenuItem::with_id(app, "tray_shield_all", "🛡️  Shield All Open Apps", true, None::<&str>)?;
    let unshield_all = MenuItem::with_id(app, "tray_unshield_all", "🔓  Unshield All", true, None::<&str>)?;
    let reapply = MenuItem::with_id(app, "tray_reapply", "🔄  Reapply Protection", true, None::<&str>)?;
    let quick_actions = Submenu::with_items(app, "⚡  Quick Actions", true, &[&shield_all, &unshield_all, &reapply])?;

    // ── Capture Mode Submenu ──
    let mode_exclude_label = if mode == "exclude" {
        "✓  Transparent (OBS / Discord)"
    } else {
        "○  Transparent (OBS / Discord)"
    };
    let mode_monitor_label = if mode == "monitor" {
        "✓  Black Screen Mask (ShadowPlay / Medal)"
    } else {
        "○  Black Screen Mask (ShadowPlay / Medal)"
    };
    let m_exclude = MenuItem::with_id(app, "tray_mode_exclude", mode_exclude_label, true, None::<&str>)?;
    let m_monitor = MenuItem::with_id(app, "tray_mode_monitor", mode_monitor_label, true, None::<&str>)?;
    let mode_submenu = Submenu::with_items(app, "🎯  Shield Mode", true, &[&m_exclude, &m_monitor])?;

    // ── Compact Mode Toggle ──
    let compact_label = if is_compact { "✓  Compact Mode" } else { "○  Compact Mode" };
    let m_compact = MenuItem::with_id(app, "tray_toggle_compact", compact_label, true, None::<&str>)?;

    // ── Desktop Widget Submenu ──
    let w_toggle_label = if widget_en { "✓  Enable Desktop Widget" } else { "○  Enable Desktop Widget" };
    let m_widget_toggle = MenuItem::with_id(app, "tray_widget_toggle", w_toggle_label, true, None::<&str>)?;
    let sep_w = PredefinedMenuItem::separator(app)?;
    let m_w_pill = MenuItem::with_id(
        app,
        "tray_widget_style_pill",
        if widget_st == "pill" { "●  Dynamic Pill" } else { "○  Dynamic Pill" },
        true,
        None::<&str>,
    )?;
    let m_w_dock = MenuItem::with_id(
        app,
        "tray_widget_style_dock",
        if widget_st == "dock" { "●  Compact Dock" } else { "○  Compact Dock" },
        true,
        None::<&str>,
    )?;
    let m_w_badge = MenuItem::with_id(
        app,
        "tray_widget_style_badge",
        if widget_st == "badge" { "●  Corner Badge" } else { "○  Corner Badge" },
        true,
        None::<&str>,
    )?;

    let widget_submenu = Submenu::with_items(
        app,
        "🖥️  Desktop Widget",
        true,
        &[&m_widget_toggle, &sep_w, &m_w_pill, &m_w_dock, &m_w_badge],
    )?;

    let sep3 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit StreamShield", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &show,
            &sep1,
            &status,
            &sep2,
            &apps_submenu,
            &quick_actions,
            &mode_submenu,
            &m_compact,
            &widget_submenu,
            &sep3,
            &quit,
        ],
    )?;

    Ok(menu)
}

fn create_tray_menu(app: &tauri::AppHandle) -> Result<Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let (shielded, mode, auto_reapply) = app
        .try_state::<AppState>()
        .and_then(|s| {
            s.config
                .lock()
                .ok()
                .map(|c| (c.shielded_exes.clone(), c.settings.shield_mode.clone(), c.settings.auto_reapply))
        })
        .unwrap_or_default();
    let wins = window_manager::enumerate_windows(&shielded, &mode, auto_reapply);
    build_tray_menu_from_windows(app, &wins)
}

fn rebuild_tray_menu_with_windows(app: &tauri::AppHandle, wins: &[WindowInfo]) {
    if let Some(tray) = app.tray_by_id("main") {
        if let Ok(menu) = build_tray_menu_from_windows(app, wins) {
            let _ = tray.set_menu(Some(menu));
        }
    }
}

fn rebuild_tray_menu(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(tray) = app.tray_by_id("main") {
        let menu = create_tray_menu(app)?;
        tray.set_menu(Some(menu))?;
    }
    Ok(())
}

// ── Tauri commands ───────────────────────────────────────────────────────────

#[tauri::command]
fn get_os_info() -> OsInfo {
    query_os_info()
}

#[tauri::command]
async fn get_windows(app: tauri::AppHandle) -> Vec<WindowInfo> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let (shielded, mode, auto_reapply) = state
            .config
            .lock()
            .map(|c| {
                (
                    c.shielded_exes.clone(),
                    c.settings.shield_mode.clone(),
                    c.settings.auto_reapply,
                )
            })
            .unwrap_or_default();

        let wins = window_manager::enumerate_windows(&shielded, &mode, auto_reapply);
        let active_count = wins.iter().filter(|w| w.is_shielded).count();
        update_tray_status_count(&state, active_count);
        wins
    })
    .await
    .unwrap_or_default()
}

#[tauri::command]
async fn toggle_shield(
    app: tauri::AppHandle,
    exe_name: String,
    hwnd: usize,
    #[allow(unused_variables)] pid: u32,
    enable: bool,
) -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let shield_mode = state
            .config
            .lock()
            .map(|c| c.settings.shield_mode.clone())
            .unwrap_or_else(|_| "exclude".to_string());

        // Shield/unshield ALL top-level processes belonging to this executable (Chrome, Edge, Discord, etc.)
        let mut targets = window_manager::find_all_windows_for_exe(&exe_name);
        if hwnd != 0 && !targets.iter().any(|(h, _)| *h == hwnd) {
            targets.insert(0, (hwnd, pid));
        }

        let mut any_ok = false;
        let mut last_err: Option<String> = None;
        for (target_hwnd, _) in targets {
            match injector::set_window_affinity(target_hwnd, enable, Some(&shield_mode)) {
                Ok(()) => any_ok = true,
                Err(e) => last_err = Some(e),
            }
        }

        if !any_ok {
            return Err(last_err.unwrap_or_else(|| "No valid target window found".to_string()));
        }

        let (shielded, auto_reapply) = {
            let mut config = state.config.lock().map_err(|e| e.to_string())?;
            let normalized = exe_name.to_lowercase();
            config.shielded_exes.retain(|x| {
                let x_lower = x.to_lowercase();
                if x_lower == normalized || x.eq_ignore_ascii_case(&exe_name) {
                    return false;
                }
                if !enable && normalized.starts_with("whatsapp") && x_lower.starts_with("whatsapp") {
                    return false;
                }
                true
            });

            if enable {
                config.shielded_exes.insert(normalized);
            }
            save_config(&config);
            (config.shielded_exes.clone(), config.settings.auto_reapply)
        };

        let wins = window_manager::enumerate_windows(&shielded, &shield_mode, auto_reapply);
        rebuild_tray_menu_with_windows(&app, &wins);
        let _ = app.emit("shield_state_changed", ());
        Ok(true)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn shield_all_windows(app: tauri::AppHandle) -> Result<usize, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let (prev_shielded, mode) = state
            .config
            .lock()
            .map(|c| (c.shielded_exes.clone(), c.settings.shield_mode.clone()))
            .unwrap_or_default();

        let current_wins = window_manager::enumerate_windows(&prev_shielded, &mode, false);
        let mut newly_shielded = 0usize;
        let mut added_exes: Vec<String> = Vec::new();
        let mut last_err: Option<String> = None;

        for w in &current_wins {
            let mut targets = window_manager::find_all_windows_for_exe(&w.exe_name);
            if w.hwnd != 0 && !targets.iter().any(|(h, _)| *h == w.hwnd) {
                targets.insert(0, (w.hwnd, w.pid));
            }
            let mut exe_ok = false;
            for (target_hwnd, _) in targets {
                match injector::set_window_affinity(target_hwnd, true, Some(&mode)) {
                    Ok(()) => exe_ok = true,
                    Err(e) => last_err = Some(e),
                }
            }
            if exe_ok {
                added_exes.push(w.exe_name.to_lowercase());
                if !w.is_shielded {
                    newly_shielded += 1;
                }
            }
        }

        if added_exes.is_empty() && !current_wins.is_empty() {
            if let Some(err) = last_err {
                return Err(err);
            }
        }

        let (shielded, auto_reapply) = {
            let mut config = state.config.lock().map_err(|e| e.to_string())?;
            for exe in added_exes {
                config.shielded_exes.insert(exe);
            }
            save_config(&config);
            (config.shielded_exes.clone(), config.settings.auto_reapply)
        };

        let updated_wins = window_manager::enumerate_windows(&shielded, &mode, auto_reapply);
        rebuild_tray_menu_with_windows(&app, &updated_wins);
        let _ = app.emit("shield_state_changed", ());
        Ok(newly_shielded)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn get_shielded_exes(state: State<AppState>) -> Vec<String> {
    state
        .config
        .lock()
        .map(|c| c.shielded_exes.iter().cloned().collect())
        .unwrap_or_default()
}

#[tauri::command]
async fn reapply_shields(app: tauri::AppHandle) -> Vec<String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let (exes, mode, auto_reapply) = state
            .config
            .lock()
            .map(|c| (c.shielded_exes.clone(), c.settings.shield_mode.clone(), c.settings.auto_reapply))
            .unwrap_or_default();

        let wins = window_manager::enumerate_windows(&exes, &mode, auto_reapply);
        let res: Vec<String> = wins
            .iter()
            .filter(|w| exes.contains(&w.exe_name.to_lowercase()))
            .filter(|w| injector::set_window_affinity(w.hwnd, true, Some(&mode)).is_ok())
            .map(|w| w.exe_name.clone())
            .collect();

        rebuild_tray_menu_with_windows(&app, &wins);
        let _ = app.emit("shield_state_changed", ());
        res
    })
    .await
    .unwrap_or_default()
}

// ── MPO (Multiplane Overlay) Registry Helpers ─────────────────────────────────

#[cfg(windows)]
fn set_mpo_fix_registry(enable: bool) -> Result<(), String> {
    nvidia_bypass::set_mpo_fix_registry(enable)
}

#[cfg(not(windows))]
fn set_mpo_fix_registry(_enable: bool) -> Result<(), String> {
    Ok(())
}

#[cfg(windows)]
fn get_mpo_fix_registry() -> bool {
    nvidia_bypass::is_mpo_fix_active()
}

#[cfg(not(windows))]
fn get_mpo_fix_registry() -> bool {
    false
}

#[tauri::command]
fn get_settings(state: State<AppState>) -> AppSettings {
    let mut settings = state
        .config
        .lock()
        .map(|c| c.settings.clone())
        .unwrap_or_default();
    #[cfg(windows)]
    {
        settings.autostart = get_autostart_registry();
        settings.mpo_fix = get_mpo_fix_registry();
    }
    settings
}

#[tauri::command]
async fn update_settings(mut settings: AppSettings, app: tauri::AppHandle) -> Result<AppSettings, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();

        let (prev_start_minimized, prev_shield_mode, prev_compact_mode, prev_self_stealth, saved_wx, saved_wy) = {
            let cfg = state.config.lock().map_err(|e| e.to_string())?;
            (
                cfg.settings.start_minimized,
                cfg.settings.shield_mode.clone(),
                cfg.settings.compact_mode,
                cfg.settings.self_stealth,
                cfg.settings.widget_x,
                cfg.settings.widget_y,
            )
        };

        // Preserve live persisted widget coordinates unless explicitly supplied
        if settings.widget_x.is_none() {
            settings.widget_x = saved_wx;
        }
        if settings.widget_y.is_none() {
            settings.widget_y = saved_wy;
        }

        #[cfg(windows)]
        {
            let current_autostart = get_autostart_registry();
            if settings.autostart != current_autostart
                || (settings.autostart && settings.start_minimized != prev_start_minimized)
            {
                let _ = set_autostart_registry(settings.autostart, settings.start_minimized);
            }

            let current_mpo = get_mpo_fix_registry();
            if settings.mpo_fix != current_mpo {
                set_mpo_fix_registry(settings.mpo_fix)?;
            }
        }

        let (shielded_exes, auto_reapply) = {
            let mut config = state.config.lock().map_err(|e| e.to_string())?;
            config.settings = settings.clone();
            save_config(&config);
            (config.shielded_exes.clone(), config.settings.auto_reapply)
        };

        // Live-sync compact mode if changed via settings modal
        if settings.compact_mode != prev_compact_mode {
            if let Some(w) = app.get_webview_window("main") {
                let _ = apply_window_compact_mode(&w, settings.compact_mode);
                let _ = app.emit("compact_mode_changed", settings.compact_mode);
            }
        }

        // Live-sync app self-stealth if changed
        if settings.self_stealth != prev_self_stealth {
            if let Some(main_win) = app.get_webview_window("main") {
                #[cfg(windows)]
                if let Ok(hwnd) = main_win.hwnd() {
                    let _ = injector::apply_own_window_stealth(hwnd.0 as _, settings.self_stealth);
                }
            }
            let _ = app.emit("self_shield_changed", settings.self_stealth);
        }

        // Reapply active shields immediately if shield_mode switched (exclude <-> monitor)
        if settings.shield_mode != prev_shield_mode && !shielded_exes.is_empty() {
            for exe in &shielded_exes {
                for (hwnd, _) in window_manager::find_all_windows_for_exe(exe) {
                    let _ = injector::set_window_affinity(hwnd, true, Some(&settings.shield_mode));
                }
            }
            let _ = app.emit("shield_state_changed", ());
        }

        open_or_update_widget(&app)?;

        let wins = window_manager::enumerate_windows(&shielded_exes, &settings.shield_mode, auto_reapply);
        rebuild_tray_menu_with_windows(&app, &wins);
        let _ = app.emit("settings_changed", &settings);
        Ok(settings)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn reset_settings(app: tauri::AppHandle) -> Result<AppSettings, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let default_settings = AppSettings::default();
        #[cfg(windows)]
        {
            let _ = set_autostart_registry(false, false);
            let _ = set_mpo_fix_registry(false);
        }

        let shielded_exes = {
            let mut config = state.config.lock().map_err(|e| e.to_string())?;
            config.settings = default_settings.clone();
            save_config(&config);
            config.shielded_exes.clone()
        };

        if let Ok(mut pos_guard) = state.last_widget_pos.lock() {
            *pos_guard = None;
        }

        if let Some(main_win) = app.get_webview_window("main") {
            let _ = apply_window_compact_mode(&main_win, default_settings.compact_mode);
            #[cfg(windows)]
            if let Ok(hwnd) = main_win.hwnd() {
                let _ = injector::apply_own_window_stealth(hwnd.0 as _, default_settings.self_stealth);
            }
        }

        let _ = open_or_update_widget(&app);
        let wins = window_manager::enumerate_windows(
            &shielded_exes,
            &default_settings.shield_mode,
            default_settings.auto_reapply,
        );
        rebuild_tray_menu_with_windows(&app, &wins);

        let _ = app.emit("compact_mode_changed", default_settings.compact_mode);
        let _ = app.emit("self_shield_changed", default_settings.self_stealth);
        let _ = app.emit("widget_state_changed", default_settings.widget_enabled);
        let _ = app.emit("settings_changed", &default_settings);
        Ok(default_settings)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn clear_all_shields(app: tauri::AppHandle) -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let (exes, mode) = state
            .config
            .lock()
            .map(|c| (c.shielded_exes.clone(), c.settings.shield_mode.clone()))
            .unwrap_or_default();

        // Clear all windows for all configured shielded exes as well as any currently shielded window
        for exe in &exes {
            for (hwnd, _) in window_manager::find_all_windows_for_exe(exe) {
                let _ = injector::set_window_affinity(hwnd, false, None);
            }
        }
        let wins = window_manager::enumerate_windows(&exes, &mode, false);
        for w in &wins {
            if w.is_shielded {
                let _ = injector::set_window_affinity(w.hwnd, false, None);
            }
        }

        let auto_reapply = {
            let mut config = state.config.lock().map_err(|e| e.to_string())?;
            config.shielded_exes.clear();
            save_config(&config);
            config.settings.auto_reapply
        };

        let empty = std::collections::HashSet::new();
        let updated_wins = window_manager::enumerate_windows(&empty, &mode, auto_reapply);
        rebuild_tray_menu_with_windows(&app, &updated_wins);
        let _ = app.emit("shield_state_changed", ());
        Ok(true)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn check_admin() -> bool {
    #[cfg(windows)] { is_admin() }
    #[cfg(not(windows))] { true }
}

#[tauri::command]
fn hide_to_tray(window: WebviewWindow) -> Result<(), String> {
    window.hide().map_err(|e| e.to_string())
}

#[tauri::command]
fn minimize_window(window: WebviewWindow) -> Result<(), String> {
    window.minimize().map_err(|e| e.to_string())
}

#[tauri::command]
fn toggle_maximize_window(window: WebviewWindow) -> Result<bool, String> {
    let is_max = window.is_maximized().map_err(|e| e.to_string())?;
    if is_max {
        window.unmaximize().map_err(|e| e.to_string())?;
        #[cfg(windows)]
        if let Ok(hwnd) = window.hwnd() {
            configure_main_window_dwm(hwnd.0 as _, false);
        }
        Ok(false)
    } else {
        window.maximize().map_err(|e| e.to_string())?;
        #[cfg(windows)]
        if let Ok(hwnd) = window.hwnd() {
            configure_main_window_dwm(hwnd.0 as _, true);
        }
        Ok(true)
    }
}

#[tauri::command]
fn toggle_self_shield(enable: bool, app: tauri::AppHandle, state: State<AppState>) -> Result<bool, String> {
    let updated_settings = {
        let mut cfg = state.config.lock().map_err(|e| e.to_string())?;
        cfg.settings.self_stealth = enable;
        save_config(&cfg);
        cfg.settings.clone()
    };

    #[cfg(windows)]
    {
        if let Some(main_win) = app.get_webview_window("main") {
            if let Ok(hwnd) = main_win.hwnd() {
                injector::apply_own_window_stealth(hwnd.0 as _, enable)?;
            }
        }
        if let Some(widget_win) = app.get_webview_window("widget") {
            if let Ok(hwnd) = widget_win.hwnd() {
                let effective_widget_stealth = enable || updated_settings.widget_stealth;
                let _ = injector::apply_own_window_stealth(hwnd.0 as _, effective_widget_stealth);
            }
        }
    }

    let _ = app.emit("self_shield_changed", enable);
    let _ = app.emit("settings_changed", &updated_settings);
    let _ = rebuild_tray_menu(&app);
    Ok(enable)
}

#[tauri::command]
fn is_self_shielded(state: State<AppState>) -> bool {
    state
        .config
        .lock()
        .map(|c| c.settings.self_stealth)
        .unwrap_or(false)
}

fn is_valid_external_url(url: &str) -> bool {
    let trimmed = url.trim();
    trimmed.starts_with("https://") || trimmed.starts_with("http://")
}

fn is_trusted_update_url(url: &str) -> bool {
    let trimmed = url.trim();
    trimmed.starts_with("https://github.com/raghavatgit/StreamShield/")
        && !trimmed
            .chars()
            .any(|c| c.is_whitespace() || matches!(c, '"' | '\'' | '`' | ';' | '$' | '&' | '|' | '<' | '>'))
}

#[tauri::command]
fn open_external_url(url: String) -> Result<(), String> {
    let trimmed = url.trim();
    if !is_valid_external_url(trimmed) {
        return Err("Invalid URL scheme".to_string());
    }
    #[cfg(windows)]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use winapi::um::shellapi::ShellExecuteW;
        use winapi::um::winuser::SW_SHOWNORMAL;
        fn wide(s: &str) -> Vec<u16> { OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect() }
        let url_w = wide(trimmed);
        let verb_w = wide("open");
        unsafe {
            ShellExecuteW(
                std::ptr::null_mut(),
                verb_w.as_ptr(),
                url_w.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                SW_SHOWNORMAL,
            );
        }
    }
    Ok(())
}

#[tauri::command]
fn install_update_silent(download_url: String) -> Result<(), String> {
    let trimmed = download_url.trim();
    if !is_trusted_update_url(trimmed) {
        return Err("Untrusted or malformed update download URL".to_string());
    }

    #[cfg(windows)]
    {
        let is_msi = trimmed.to_lowercase().ends_with(".msi");
        let ps_cmd = if is_msi {
            format!(
                "$temp = \"$env:TEMP\\StreamShield_Setup.msi\"; \
                 [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; \
                 Invoke-WebRequest -Uri '{}' -OutFile $temp; \
                 Start-Process -FilePath 'msiexec.exe' -ArgumentList \"/i `\"$temp`\" /passive\"",
                trimmed
            )
        } else {
            format!(
                "$temp = \"$env:TEMP\\StreamShield_Setup.exe\"; \
                 [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; \
                 Invoke-WebRequest -Uri '{}' -OutFile $temp; \
                 Start-Process -FilePath $temp -ArgumentList '/S'",
                trimmed
            )
        };
        std::process::Command::new("powershell")
            .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &ps_cmd])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn start_window_drag(app: tauri::AppHandle, window_label: Option<String>) {
    let label = window_label.unwrap_or_else(|| "main".to_string());
    if let Some(w) = app.get_webview_window(&label) {
        // Non-blocking native Win32 drag (uses GetCursorPos + ReleaseCapture + PostMessageW in tao)
        let _ = w.start_dragging();
    }
}

#[tauri::command]
fn set_widget_size(app: tauri::AppHandle, width: f64, height: f64) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("widget") {
        let _ = w.set_size(Size::Logical(LogicalSize::new(width, height)));
    }
    Ok(())
}

fn get_widget_dimensions(style: &str) -> (f64, f64) {
    match style {
        "dock" => (292.0, 248.0),
        "badge" => (76.0, 76.0),
        _ => (416.0, 68.0), // "pill"
    }
}

fn clamp_to_work_area(
    wa: &PhysicalRect<i32, u32>,
    x: i32,
    y: i32,
    w_phys: i32,
    h_phys: i32,
) -> (i32, i32) {
    let min_x = wa.position.x;
    let min_y = wa.position.y;
    let max_x = (wa.position.x + wa.size.width as i32 - w_phys).max(min_x);
    let max_y = (wa.position.y + wa.size.height as i32 - h_phys).max(min_y);
    (x.clamp(min_x, max_x), y.clamp(min_y, max_y))
}

fn find_monitor_for_point(app: &tauri::AppHandle, x: i32, y: i32) -> Option<tauri::Monitor> {
    if let Ok(monitors) = app.available_monitors() {
        for m in &monitors {
            let wa = m.work_area();
            if x >= wa.position.x
                && x < wa.position.x + wa.size.width as i32
                && y >= wa.position.y
                && y < wa.position.y + wa.size.height as i32
            {
                return Some(m.clone());
            }
        }
        // Check full monitor rects in case point is inside taskbar
        for m in &monitors {
            let pos = m.position();
            let sz = m.size();
            if x >= pos.x
                && x < pos.x + sz.width as i32
                && y >= pos.y
                && y < pos.y + sz.height as i32
            {
                return Some(m.clone());
            }
        }
    }
    app.primary_monitor().ok().flatten()
}

fn persist_last_widget_position(state: &AppState) {
    let pos_opt = state.last_widget_pos.lock().ok().and_then(|g| *g);
    if let Some((wx, wy)) = pos_opt {
        if let Ok(mut cfg) = state.config.lock() {
            if cfg.settings.widget_x != Some(wx) || cfg.settings.widget_y != Some(wy) {
                cfg.settings.widget_x = Some(wx);
                cfg.settings.widget_y = Some(wy);
                save_config(&cfg);
            }
        }
    }
}

#[tauri::command]
async fn set_widget_badge_expanded(
    app: tauri::AppHandle,
    expanded: bool,
) -> Result<BadgeExpandResult, String> {
    let state = app.state::<AppState>();
    let win = match app.get_webview_window("widget") {
        Some(w) => w,
        None => return Ok(BadgeExpandResult::default()),
    };

    let scale = win.scale_factor().unwrap_or(1.0);
    let col_w = (76.0 * scale).round() as i32;
    let col_h = (76.0 * scale).round() as i32;
    let exp_w = (208.0 * scale).round() as i32;
    let exp_h = (286.0 * scale).round() as i32;

    if expanded {
        let cur_pos = win.outer_position().map_err(|e| e.to_string())?;
        let already_expanded = state.widget_badge_expanded.lock().map(|g| *g).unwrap_or(false);
        if already_expanded {
            return Ok(state.badge_last_dir.lock().map(|g| *g).unwrap_or_default());
        }

        let anchor_x = cur_pos.x;
        let anchor_y = cur_pos.y;
        let monitor = find_monitor_for_point(&app, anchor_x + col_w / 2, anchor_y + col_h / 2);

        let (expand_up, expand_left, new_x, new_y) = if let Some(m) = monitor {
            let wa = m.work_area();
            let mid_x = wa.position.x + (wa.size.width as i32) / 2;
            let mid_y = wa.position.y + (wa.size.height as i32) / 2;
            let e_left = (anchor_x + col_w / 2) >= mid_x;
            let e_up = (anchor_y + col_h / 2) >= mid_y;
            let raw_x = if e_left { anchor_x - (exp_w - col_w) } else { anchor_x };
            let raw_y = if e_up { anchor_y - (exp_h - col_h) } else { anchor_y };
            let (cx, cy) = clamp_to_work_area(wa, raw_x, raw_y, exp_w, exp_h);
            (e_up, e_left, cx, cy)
        } else {
            (true, true, anchor_x - (exp_w - col_w), anchor_y - (exp_h - col_h))
        };

        if let Ok(mut g) = state.badge_pre_expand_pos.lock() {
            *g = Some((anchor_x, anchor_y));
        }
        if let Ok(mut g) = state.widget_badge_expanded.lock() {
            *g = true;
        }
        let dir = BadgeExpandResult { expand_up, expand_left };
        if let Ok(mut g) = state.badge_last_dir.lock() {
            *g = dir;
        }

        let _ = win.set_position(PhysicalPosition::new(new_x, new_y));
        let _ = win.set_size(Size::Logical(LogicalSize::new(208.0, 286.0)));
        Ok(dir)
    } else {
        let was_expanded = state.widget_badge_expanded.lock().map(|g| *g).unwrap_or(false);
        if let Ok(mut g) = state.widget_badge_expanded.lock() {
            *g = false;
        }
        let dir = state.badge_last_dir.lock().map(|g| *g).unwrap_or_default();
        let anchor_opt = state.badge_pre_expand_pos.lock().ok().and_then(|mut g| g.take());

        let _ = win.set_size(Size::Logical(LogicalSize::new(76.0, 76.0)));
        if was_expanded {
            if let Some((ax, ay)) = anchor_opt {
                let _ = win.set_position(PhysicalPosition::new(ax, ay));
            }
        }
        Ok(dir)
    }
}

fn open_or_update_widget(app: &tauri::AppHandle) -> Result<(), String> {
    let state = match app.try_state::<AppState>() {
        Some(s) => s,
        None => return Ok(()),
    };

    let (enabled, style, always_on_top, widget_stealth, self_stealth, theme, saved_x, saved_y) = {
        let cfg = state.config.lock().map_err(|_| "Config lock error".to_string())?;
        (
            cfg.settings.widget_enabled,
            cfg.settings.widget_style.clone(),
            cfg.settings.widget_always_on_top,
            cfg.settings.widget_stealth,
            cfg.settings.self_stealth,
            cfg.settings.theme.clone(),
            cfg.settings.widget_x,
            cfg.settings.widget_y,
        )
    };

    if !enabled {
        let was_expanded = state.widget_badge_expanded.lock().map(|g| *g).unwrap_or(false);
        if was_expanded {
            if let Some(ax_ay) = state.badge_pre_expand_pos.lock().ok().and_then(|mut g| g.take()) {
                if let Ok(mut lp) = state.last_widget_pos.lock() {
                    *lp = Some(ax_ay);
                }
            }
            if let Ok(mut g) = state.widget_badge_expanded.lock() {
                *g = false;
            }
        }
        persist_last_widget_position(&state);
        if let Some(w) = app.get_webview_window("widget") {
            let _ = w.hide();
        }
        return Ok(());
    }

    let effective_stealth = widget_stealth || self_stealth;
    let (w_log, h_log) = get_widget_dimensions(&style);

    if let Some(win) = app.get_webview_window("widget") {
        // Restore pre-expand anchor if collapsing from badge expansion before style switch
        let was_expanded = state.widget_badge_expanded.lock().map(|g| *g).unwrap_or(false);
        if let Ok(mut g) = state.widget_badge_expanded.lock() {
            *g = false;
        }
        if was_expanded {
            if let Some((ax, ay)) = state.badge_pre_expand_pos.lock().ok().and_then(|mut g| g.take()) {
                let _ = win.set_position(PhysicalPosition::new(ax, ay));
            }
        }

        let scale = win.scale_factor().unwrap_or(1.0);
        let w_phys = (w_log * scale).round() as i32;
        let h_phys = (h_log * scale).round() as i32;

        let _ = win.set_size(Size::Logical(LogicalSize::new(w_log, h_log)));

        // Clamp existing position to monitor work_area so switching Badge/Dock -> Pill stays onscreen
        if let Ok(pos) = win.outer_position() {
            if let Some(m) = find_monitor_for_point(app, pos.x + 16, pos.y + 16) {
                let (cx, cy) = clamp_to_work_area(m.work_area(), pos.x, pos.y, w_phys, h_phys);
                if cx != pos.x || cy != pos.y {
                    let _ = win.set_position(PhysicalPosition::new(cx, cy));
                }
            }
        }

        let _ = win.set_always_on_top(always_on_top);
        #[cfg(windows)]
        if let Ok(hwnd) = win.hwnd() {
            configure_widget_window_win32(hwnd.0 as _);
            let _ = injector::apply_own_window_stealth(hwnd.0 as _, effective_stealth);
        }
        let _ = win.show();
        let _ = win.emit("widget_style_changed", &style);
        return Ok(());
    }

    // Determine initial physical coordinates inside monitor work_area
    let target_monitor = match (saved_x, saved_y) {
        (Some(sx), Some(sy)) => find_monitor_for_point(app, sx, sy),
        _ => app.primary_monitor().ok().flatten(),
    };

    let (init_px, init_py, init_scale) = if let Some(m) = target_monitor {
        let scale = m.scale_factor();
        let wa = m.work_area();
        let w_phys = (w_log * scale).round() as i32;
        let h_phys = (h_log * scale).round() as i32;
        let margin = (24.0 * scale).round() as i32;

        let (raw_x, raw_y) = match (saved_x, saved_y) {
            (Some(sx), Some(sy)) => (sx, sy),
            _ => match style.as_str() {
                "dock" => (
                    wa.position.x + wa.size.width as i32 - w_phys - margin,
                    wa.position.y + margin,
                ),
                "badge" => (
                    wa.position.x + wa.size.width as i32 - w_phys - margin,
                    wa.position.y + wa.size.height as i32 - h_phys - margin,
                ),
                _ => (
                    wa.position.x + (wa.size.width as i32 - w_phys) / 2,
                    wa.position.y + margin,
                ),
            },
        };
        let (cx, cy) = clamp_to_work_area(wa, raw_x, raw_y, w_phys, h_phys);
        (cx, cy, scale)
    } else {
        (300, 36, 1.0)
    };

    let init_payload = serde_json::to_string(&WidgetInitPayload {
        style: style.clone(),
        theme,
        self_stealth,
        widget_stealth,
    })
    .unwrap_or_else(|_| "{}".to_string());

    let init_script = format!(
        "window.__STREAMSHIELD_WIDGET_INIT__ = {}; \
         if (document.documentElement) {{ document.documentElement.classList.add('is-widget-window'); }}",
        init_payload
    );

    let mut builder = WebviewWindowBuilder::new(app, "widget", WebviewUrl::default())
        .title("StreamShield Widget")
        .inner_size(w_log, h_log)
        .position(init_px as f64 / init_scale, init_py as f64 / init_scale)
        .decorations(false)
        .transparent(true)
        .background_color(tauri::window::Color(0, 0, 0, 0))
        .always_on_top(always_on_top)
        .skip_taskbar(true)
        .resizable(false)
        .shadow(false)
        .visible(true)
        .additional_browser_args(WEBVIEW_BROWSER_ARGS)
        .initialization_script(&init_script);

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone()).map_err(|e| e.to_string())?;
    }

    let win = builder
        .build()
        .map_err(|e| format!("Failed to create desktop widget: {}", e))?;

    let _ = win.set_position(PhysicalPosition::new(init_px, init_py));
    if let Ok(mut lp) = state.last_widget_pos.lock() {
        *lp = Some((init_px, init_py));
    }

    #[cfg(windows)]
    if let Ok(hwnd) = win.hwnd() {
        configure_widget_window_win32(hwnd.0 as _);
        let _ = injector::apply_own_window_stealth(hwnd.0 as _, effective_stealth);
    }

    Ok(())
}

#[tauri::command]
async fn show_main_window(app: tauri::AppHandle) -> Result<(), String> {
    create_main_window(&app).map_err(|e| e.to_string())
}

#[tauri::command]
async fn toggle_widget(
    app: tauri::AppHandle,
    enable: Option<bool>,
    style: Option<String>,
) -> Result<bool, String> {
    let (new_enabled, updated_settings) = {
        let state = app.state::<AppState>();
        let mut cfg = state.config.lock().map_err(|_| "Lock failed".to_string())?;
        if let Some(e) = enable {
            cfg.settings.widget_enabled = e;
        } else {
            cfg.settings.widget_enabled = !cfg.settings.widget_enabled;
        }
        if let Some(s) = style {
            cfg.settings.widget_style = s;
        }
        let en = cfg.settings.widget_enabled;
        save_config(&cfg);
        (en, cfg.settings.clone())
    };

    if let Err(e) = open_or_update_widget(&app) {
        // Revert flag if widget window failed to build
        if new_enabled {
            if let Ok(mut cfg) = app.state::<AppState>().config.lock() {
                cfg.settings.widget_enabled = false;
                save_config(&cfg);
            }
        }
        let _ = rebuild_tray_menu(&app);
        return Err(e);
    }

    let _ = rebuild_tray_menu(&app);
    let _ = app.emit("widget_state_changed", new_enabled);
    let _ = app.emit("settings_changed", &updated_settings);
    Ok(new_enabled)
}

#[tauri::command]
async fn get_capture_environment() -> nvidia_bypass::CaptureEnvironment {
    tauri::async_runtime::spawn_blocking(nvidia_bypass::detect_capture_environment)
        .await
        .unwrap_or_else(|_| nvidia_bypass::CaptureEnvironment {
            nvidia_detected: false,
            obs_detected: false,
            discord_detected: false,
            medal_detected: false,
            mpo_active: false,
            recommended_mode: "exclude".to_string(),
            summary: "Detection unavailable".to_string(),
        })
}

fn apply_window_compact_mode(win: &WebviewWindow, compact: bool) -> Result<(), Box<dyn std::error::Error>> {
    if compact {
        let _ = win.set_min_size(Some(Size::Logical(LogicalSize::new(260.0, 340.0))));
        let _ = win.set_size(Size::Logical(LogicalSize::new(280.0, 420.0)));
    } else {
        let _ = win.set_min_size(Some(Size::Logical(LogicalSize::new(440.0, 580.0))));
        let _ = win.set_size(Size::Logical(LogicalSize::new(520.0, 720.0)));
    }
    Ok(())
}

#[tauri::command]
fn set_compact_mode(app: tauri::AppHandle, compact: bool) -> Result<bool, String> {
    let updated_settings = if let Some(state) = app.try_state::<AppState>() {
        let settings_opt = if let Ok(mut cfg) = state.config.lock() {
            cfg.settings.compact_mode = compact;
            save_config(&cfg);
            Some(cfg.settings.clone())
        } else {
            None
        };
        let _ = rebuild_tray_menu(&app);
        settings_opt
    } else {
        None
    };
    if let Some(w) = app.get_webview_window("main") {
        let _ = apply_window_compact_mode(&w, compact);
    }
    let _ = app.emit("compact_mode_changed", compact);
    if let Some(s) = updated_settings {
        let _ = app.emit("settings_changed", &s);
    }
    Ok(compact)
}

// ── Main Window Creator ──────────────────────────────────────────────────────

/// Create the main webview window on demand or restore/focus if already created.
fn create_main_window(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
        #[cfg(windows)]
        if let Ok(hwnd) = w.hwnd() {
            use winapi::um::winuser::{ShowWindow, SetForegroundWindow, BringWindowToTop, SW_RESTORE};
            unsafe {
                ShowWindow(hwnd.0 as _, SW_RESTORE);
                BringWindowToTop(hwnd.0 as _);
                SetForegroundWindow(hwnd.0 as _);
            }
            let is_max = w.is_maximized().unwrap_or(false);
            configure_main_window_dwm(hwnd.0 as _, is_max);
        }
        let _ = w.emit("app_wake", ());
        return Ok(());
    }

    let (is_compact, is_stealth) = app
        .try_state::<AppState>()
        .and_then(|s| {
            s.config
                .lock()
                .ok()
                .map(|c| (c.settings.compact_mode, c.settings.self_stealth))
        })
        .unwrap_or((false, false));

    let (w, h, min_w, min_h) = if is_compact {
        (280.0, 420.0, 260.0, 340.0)
    } else {
        (520.0, 720.0, 440.0, 580.0)
    };

    let mut builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
        .title("StreamShield")
        .inner_size(w, h)
        .min_inner_size(min_w, min_h)
        .resizable(true)
        .center()
        .decorations(false)
        .shadow(true)
        .background_color(tauri::window::Color(7, 9, 14, 255))
        .visible(true)
        .additional_browser_args(WEBVIEW_BROWSER_ARGS);

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone())?;
    }

    let win = builder.build()?;
    let _ = win.set_focus();

    #[cfg(windows)]
    if let Ok(hwnd) = win.hwnd() {
        use winapi::um::winuser::{ShowWindow, SetForegroundWindow, BringWindowToTop, SW_SHOWNORMAL};
        configure_main_window_dwm(hwnd.0 as _, false);
        unsafe {
            ShowWindow(hwnd.0 as _, SW_SHOWNORMAL);
            BringWindowToTop(hwnd.0 as _);
            SetForegroundWindow(hwnd.0 as _);
        }
        if is_stealth {
            let _ = injector::apply_own_window_stealth(hwnd.0 as _, true);
        }
    }

    Ok(())
}

// ── Entry Point ──────────────────────────────────────────────────────────────

fn main() {
    let wake_event_handle = match ensure_single_instance() {
        Some(h) => h,
        None => std::process::exit(0),
    };

    let initial_config = load_config();

    #[cfg(windows)]
    if !is_admin() {
        let args: Vec<String> = std::env::args().collect();
        let is_minimized = args.iter().any(|a| a == "--minimized" || a == "-m");
        relaunch_as_admin(is_minimized);
        std::process::exit(0);
    }

    disable_power_throttling();
    injector::cleanup_stale_dlls();

    tauri::Builder::default()
        .manage(AppState {
            config: Mutex::new(initial_config),
            tray_status: Mutex::new(None),
            last_widget_pos: Mutex::new(None),
            widget_badge_expanded: Mutex::new(false),
            badge_pre_expand_pos: Mutex::new(None),
            badge_last_dir: Mutex::new(BadgeExpandResult::default()),
        })
        .invoke_handler(tauri::generate_handler![
            get_windows,
            toggle_shield,
            shield_all_windows,
            get_shielded_exes,
            reapply_shields,
            get_settings,
            update_settings,
            reset_settings,
            clear_all_shields,
            check_admin,
            get_os_info,
            hide_to_tray,
            minimize_window,
            toggle_maximize_window,
            toggle_self_shield,
            is_self_shielded,
            get_capture_environment,
            open_external_url,
            install_update_silent,
            set_compact_mode,
            start_window_drag,
            toggle_widget,
            show_main_window,
            set_widget_size,
            set_widget_badge_expanded,
        ])
        .setup(move |app| {
            let args: Vec<String> = std::env::args().collect();
            let launched_by_autostart = args.iter().any(|a| a == "--minimized" || a == "-m");

            if !launched_by_autostart {
                create_main_window(app.handle())?;
            }

            // Spawn single-instance wake event listener (restores main window cleanly across UIPI)
            #[cfg(windows)]
            if wake_event_handle != 0 {
                let app_wake = app.handle().clone();
                std::thread::spawn(move || {
                    use winapi::um::synchapi::WaitForSingleObject;
                    use winapi::um::winbase::{INFINITE, WAIT_OBJECT_0};
                    let handle = wake_event_handle as winapi::um::winnt::HANDLE;
                    loop {
                        let res = unsafe { WaitForSingleObject(handle, INFINITE) };
                        if res == WAIT_OBJECT_0 {
                            let _ = create_main_window(&app_wake);
                        } else {
                            break;
                        }
                    }
                });
            }

            // Restore desktop widget if configured to be enabled
            if let Err(e) = open_or_update_widget(app.handle()) {
                eprintln!("[StreamShield] Initial widget setup error: {}", e);
            }

            // ── Dynamic Tray menu with toggleable options ────────────────
            let menu = create_tray_menu(app.handle())?;

            let icon = app
                .default_window_icon()
                .ok_or("no window icon")?
                .clone();

            TrayIconBuilder::with_id("main")
                .icon(icon)
                .tooltip("StreamShield")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            if w.is_visible().unwrap_or(false) && !w.is_minimized().unwrap_or(false) {
                                let _ = w.hide();
                            } else {
                                let _ = create_main_window(app);
                            }
                        } else {
                            let _ = create_main_window(app);
                        }
                    }
                })
                .on_menu_event(|app, event| {
                    let id = event.id.as_ref().to_string();
                    let app_h = app.clone();
                    match id.as_str() {
                        "show" => {
                            let _ = create_main_window(&app_h);
                        }
                        "tray_shield_all" => {
                            tauri::async_runtime::spawn(async move {
                                let _ = shield_all_windows(app_h.clone()).await;
                                let _ = app_h.emit("tray_action_completed", "shield_all");
                            });
                        }
                        "tray_unshield_all" => {
                            tauri::async_runtime::spawn(async move {
                                let _ = clear_all_shields(app_h.clone()).await;
                                let _ = app_h.emit("tray_action_completed", "unshield_all");
                            });
                        }
                        "tray_reapply" => {
                            tauri::async_runtime::spawn(async move {
                                let _ = reapply_shields(app_h).await;
                            });
                        }
                        "tray_mode_exclude" | "tray_mode_monitor" => {
                            let new_mode = if id == "tray_mode_exclude" { "exclude" } else { "monitor" };
                            tauri::async_runtime::spawn_blocking(move || {
                                let state = app_h.state::<AppState>();
                                let (exes, auto_reapply) = {
                                    let mut cfg = match state.config.lock() {
                                        Ok(c) => c,
                                        Err(_) => return,
                                    };
                                    cfg.settings.shield_mode = new_mode.to_string();
                                    save_config(&cfg);
                                    (cfg.shielded_exes.clone(), cfg.settings.auto_reapply)
                                };
                                for exe in &exes {
                                    for (hwnd, _) in window_manager::find_all_windows_for_exe(exe) {
                                        let _ = injector::set_window_affinity(hwnd, true, Some(new_mode));
                                    }
                                }
                                let wins = window_manager::enumerate_windows(&exes, new_mode, auto_reapply);
                                rebuild_tray_menu_with_windows(&app_h, &wins);
                                if let Ok(cfg) = state.config.lock() {
                                    let _ = app_h.emit("settings_changed", &cfg.settings);
                                }
                                let _ = app_h.emit("shield_state_changed", ());
                                let _ = app_h.emit("tray_action_completed", "mode_changed");
                            });
                        }
                        "tray_toggle_compact" => {
                            let current = app_h
                                .try_state::<AppState>()
                                .and_then(|s| s.config.lock().ok().map(|c| c.settings.compact_mode))
                                .unwrap_or(false);
                            let _ = set_compact_mode(app_h.clone(), !current);
                            let _ = app_h.emit("tray_action_completed", "compact_toggled");
                        }
                        "tray_widget_toggle" => {
                            tauri::async_runtime::spawn(async move {
                                let _ = toggle_widget(app_h, None, None).await;
                            });
                        }
                        "tray_widget_style_pill" => {
                            tauri::async_runtime::spawn(async move {
                                let _ = toggle_widget(app_h, Some(true), Some("pill".to_string())).await;
                            });
                        }
                        "tray_widget_style_dock" => {
                            tauri::async_runtime::spawn(async move {
                                let _ = toggle_widget(app_h, Some(true), Some("dock".to_string())).await;
                            });
                        }
                        "tray_widget_style_badge" => {
                            tauri::async_runtime::spawn(async move {
                                let _ = toggle_widget(app_h, Some(true), Some("badge".to_string())).await;
                            });
                        }
                        "quit" => {
                            if let Some(state) = app_h.try_state::<AppState>() {
                                persist_last_widget_position(&state);
                            }
                            let _ = app_h.remove_tray_by_id("main");
                            app_h.exit(0);
                        }
                        other if other.starts_with("toggle_app:") => {
                            let target_exe = other.trim_start_matches("toggle_app:").to_string();
                            tauri::async_runtime::spawn_blocking(move || {
                                let state = app_h.state::<AppState>();
                                let (mode, new_state) = {
                                    let cfg = match state.config.lock() {
                                        Ok(c) => c,
                                        Err(_) => return,
                                    };
                                    let norm = target_exe.to_lowercase();
                                    (cfg.settings.shield_mode.clone(), !cfg.shielded_exes.contains(&norm))
                                };

                                let targets = window_manager::find_all_windows_for_exe(&target_exe);
                                for (hwnd, _) in targets {
                                    let _ = injector::set_window_affinity(hwnd, new_state, Some(&mode));
                                }

                                let (exes, auto_reapply) = {
                                    let mut cfg = match state.config.lock() {
                                        Ok(c) => c,
                                        Err(_) => return,
                                    };
                                    let norm = target_exe.to_lowercase();
                                    if new_state {
                                        cfg.shielded_exes.insert(norm);
                                    } else {
                                        cfg.shielded_exes.remove(&norm);
                                    }
                                    save_config(&cfg);
                                    (cfg.shielded_exes.clone(), cfg.settings.auto_reapply)
                                };

                                let wins = window_manager::enumerate_windows(&exes, &mode, auto_reapply);
                                rebuild_tray_menu_with_windows(&app_h, &wins);
                                let _ = app_h.emit("shield_state_changed", ());
                                let _ = app_h.emit("tray_action_completed", "app_toggled");
                            });
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            // ── Auto-Shield & Live Tray Update Background Watchdog Daemon ──
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                loop {
                    std::thread::sleep(std::time::Duration::from_millis(5000));
                    if let Some(state) = app_handle.try_state::<AppState>() {
                        let (shielded, mode, auto_reapply) = state
                            .config
                            .lock()
                            .map(|guard| {
                                (
                                    guard.shielded_exes.clone(),
                                    guard.settings.shield_mode.clone(),
                                    guard.settings.auto_reapply,
                                )
                            })
                            .unwrap_or_else(|_| (std::collections::HashSet::new(), "exclude".to_string(), false));

                        if auto_reapply && !shielded.is_empty() {
                            window_manager::auto_reapply_shields(&shielded, &mode);
                        }
                    }
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    let _ = window.hide();
                    if window.label() == "widget" {
                        let app = window.app_handle();
                        if let Some(state) = app.try_state::<AppState>() {
                            persist_last_widget_position(&state);
                            let updated_settings = if let Ok(mut cfg) = state.config.lock() {
                                cfg.settings.widget_enabled = false;
                                save_config(&cfg);
                                Some(cfg.settings.clone())
                            } else {
                                None
                            };
                            let _ = rebuild_tray_menu(app);
                            let _ = app.emit("widget_state_changed", false);
                            if let Some(s) = updated_settings {
                                let _ = app.emit("settings_changed", &s);
                            }
                        }
                    }
                }
                tauri::WindowEvent::Moved(pos) => {
                    if window.label() == "widget" {
                        let app = window.app_handle();
                        if let Some(state) = app.try_state::<AppState>() {
                            let is_expanded = state
                                .widget_badge_expanded
                                .lock()
                                .map(|g| *g)
                                .unwrap_or(false);
                            if !is_expanded {
                                if let Ok(mut g) = state.last_widget_pos.lock() {
                                    *g = Some((pos.x, pos.y));
                                }
                            }
                        }
                    }
                }
                tauri::WindowEvent::Focused(false) => {
                    if window.label() == "widget" {
                        let app = window.app_handle();
                        if let Some(state) = app.try_state::<AppState>() {
                            persist_last_widget_position(&state);
                        }
                        let _ = window.emit("widget_blur", ());
                    }
                }
                tauri::WindowEvent::Resized(_) => {
                    if window.label() == "main" {
                        #[cfg(windows)]
                        if let Ok(hwnd) = window.hwnd() {
                            let is_max = window.is_maximized().unwrap_or(false);
                            configure_main_window_dwm(hwnd.0 as _, is_max);
                        }
                    }
                }
                _ => {}
            }
        })
        .run(tauri::generate_context!())
        .expect("StreamShield error");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tauri::PhysicalSize;

    #[test]
    fn test_widget_dimensions() {
        assert_eq!(get_widget_dimensions("pill"), (416.0, 68.0));
        assert_eq!(get_widget_dimensions("dock"), (292.0, 248.0));
        assert_eq!(get_widget_dimensions("badge"), (76.0, 76.0));
        assert_eq!(get_widget_dimensions("unknown_style"), (416.0, 68.0));
    }

    #[test]
    fn test_clamp_to_work_area() {
        let primary = PhysicalRect {
            position: PhysicalPosition::new(0, 0),
            size: PhysicalSize::new(1920, 1040),
        };
        assert_eq!(clamp_to_work_area(&primary, 500, 300, 416, 68), (500, 300));
        assert_eq!(clamp_to_work_area(&primary, -120, -40, 416, 68), (0, 0));
        assert_eq!(clamp_to_work_area(&primary, 1850, 1020, 416, 68), (1504, 972));

        let secondary_left = PhysicalRect {
            position: PhysicalPosition::new(-1920, 0),
            size: PhysicalSize::new(1920, 1080),
        };
        assert_eq!(clamp_to_work_area(&secondary_left, -2500, 500, 76, 76), (-1920, 500));
        assert_eq!(clamp_to_work_area(&secondary_left, 100, 2000, 76, 76), (-76, 1004));
    }

    #[test]
    fn test_external_and_update_url_validation() {
        assert!(is_valid_external_url("https://github.com/raghavatgit/StreamShield"));
        assert!(is_valid_external_url("  http://example.com  "));
        assert!(!is_valid_external_url("file:///C:/Windows/System32/cmd.exe"));
        assert!(!is_valid_external_url("javascript:alert(1)"));
        assert!(!is_valid_external_url("cmd.exe"));

        assert!(is_trusted_update_url(
            "https://github.com/raghavatgit/StreamShield/releases/download/v0.2.4/StreamShield_0.2.4_x64-setup.exe"
        ));
        assert!(!is_trusted_update_url("https://evil.example.com/setup.exe"));
        assert!(!is_trusted_update_url(
            "https://github.com/raghavatgit/StreamShield/releases/download/v0.2.4/setup.exe';calc.exe'"
        ));
        assert!(!is_trusted_update_url(
            "https://github.com/raghavatgit/StreamShield/releases/download/v0.2.4/bad name.exe"
        ));
    }
}
