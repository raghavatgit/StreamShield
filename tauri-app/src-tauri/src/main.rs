// Always hide console window - no terminal alongside the app
#![windows_subsystem = "windows"]

mod window_manager;
mod injector;
mod config;
mod nvidia_bypass;

use std::sync::Mutex;
use tauri::{
    menu::{IsMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    webview::WebviewWindowBuilder,
    Emitter, LogicalSize, Manager, Size, State, WebviewUrl, WebviewWindow,
};
use window_manager::WindowInfo;
use config::{load_config, save_config, AppSettings, ShieldConfig};

struct AppState {
    config: Mutex<ShieldConfig>,
    tray_status: Mutex<Option<MenuItem<tauri::Wry>>>,
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
    let exe = std::env::current_exe().unwrap();
    let exe_w = wide(exe.to_str().unwrap());
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

#[cfg(windows)]
fn disable_power_throttling() {
    use winapi::um::winbase::SetThreadExecutionState;
    use winapi::um::winnt::{ES_CONTINUOUS, ES_SYSTEM_REQUIRED};
    use winapi::um::processthreadsapi::{GetCurrentProcess, SetProcessInformation};

    #[allow(non_snake_case)]
    #[repr(C)]
    struct PROCESS_POWER_THROTTLING_STATE {
        Version: u32,
        ControlMask: u32,
        StateMask: u32,
    }

    const PROCESS_POWER_THROTTLING: u32 = 0x2A;
    const PROCESS_POWER_THROTTLING_CURRENT_VERSION: u32 = 1;
    const PROCESS_POWER_THROTTLING_EXECUTION_SPEED: u32 = 0x1;

    unsafe {
        // Prevent Windows from suspending background watchdog thread and timer execution
        SetThreadExecutionState(ES_CONTINUOUS | ES_SYSTEM_REQUIRED);

        // Disable Windows 11 EcoQoS / Efficiency Mode on StreamShield process
        let mut throttling = PROCESS_POWER_THROTTLING_STATE {
            Version: PROCESS_POWER_THROTTLING_CURRENT_VERSION,
            ControlMask: PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
            StateMask: 0,
        };
        SetProcessInformation(
            GetCurrentProcess(),
            PROCESS_POWER_THROTTLING as _,
            &mut throttling as *mut _ as *mut _,
            std::mem::size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
        );
    }
}

#[cfg(windows)]
fn ensure_single_instance() -> bool {
    use winapi::um::synchapi::CreateMutexW;
    use winapi::um::errhandlingapi::GetLastError;
    use winapi::shared::winerror::ERROR_ALREADY_EXISTS;
    use winapi::um::winuser::{FindWindowW, SetForegroundWindow, ShowWindow, SW_RESTORE};

    let name: Vec<u16> = "Global\\StreamShield_SingleInstance_Mutex\0".encode_utf16().collect();
    let handle = unsafe { CreateMutexW(std::ptr::null_mut(), 1, name.as_ptr()) };
    if handle.is_null() || unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        // Bring existing window to front if it exists
        let title: Vec<u16> = "StreamShield\0".encode_utf16().collect();
        let hwnd = unsafe { FindWindowW(std::ptr::null(), title.as_ptr()) };
        if !hwnd.is_null() {
            unsafe {
                ShowWindow(hwnd, SW_RESTORE);
                SetForegroundWindow(hwnd);
            }
        }
        return false;
    }
    // Prevent compiler warning for copy type
    let _ = handle;
    true
}

#[cfg(not(windows))]
fn ensure_single_instance() -> bool { true }

#[cfg(not(windows))]
fn disable_power_throttling() {}

// ── Windows Autostart Registry helpers ───────────────────────────────────────

#[cfg(windows)]
fn set_autostart_registry(enable: bool) -> Result<(), String> {
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
            let cmd = format!("\"{}\" --minimized", current_exe.to_string_lossy());
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
fn set_autostart_registry(_enable: bool) -> Result<(), String> {
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
        query_res == 0 && data_len > 0
    }
}

#[cfg(not(windows))]
fn get_autostart_registry() -> bool {
    false
}

// ── Tray status helpers ───────────────────────────────────────────────────────

fn tray_status_text(count: usize) -> String {
    if count == 0 {
        "No apps shielded".to_string()
    } else {
        format!("🛡️  {} app{} hidden from capture", count, if count == 1 { "" } else { "s" })
    }
}

fn create_tray_menu(app: &tauri::AppHandle) -> Result<Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let state = match app.try_state::<AppState>() {
        Some(s) => s,
        None => return Err("AppState not found".into()),
    };
    let (shielded, mode, auto_reapply) = state.config.lock().map(|c| {
        (c.shielded_exes.clone(), c.settings.shield_mode.clone(), c.settings.auto_reapply)
    }).unwrap_or_default();

    let wins = window_manager::enumerate_windows(&shielded, &mode, auto_reapply);
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
    for w in &wins {
        let exe_lower = w.exe_name.to_lowercase();
        if !seen_exes.insert(exe_lower.clone()) {
            continue;
        }
        let is_sh = shielded.contains(&exe_lower);
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

    let item_refs: Vec<&dyn IsMenuItem<tauri::Wry>> = app_items.iter().map(|i| i as &dyn IsMenuItem<tauri::Wry>).collect();
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
    let mode_exclude_label = if mode == "exclude" { "✓  Transparent (OBS / Discord)" } else { "○  Transparent (OBS / Discord)" };
    let mode_monitor_label = if mode == "monitor" { "✓  Black Screen Mask (ShadowPlay / Medal)" } else { "○  Black Screen Mask (ShadowPlay / Medal)" };
    let m_exclude = MenuItem::with_id(app, "tray_mode_exclude", mode_exclude_label, true, None::<&str>)?;
    let m_monitor = MenuItem::with_id(app, "tray_mode_monitor", mode_monitor_label, true, None::<&str>)?;
    let mode_submenu = Submenu::with_items(app, "🎯  Shield Mode", true, &[&m_exclude, &m_monitor])?;

    // ── Compact Mode Toggle ──
    let is_compact = state.config.lock().map(|c| c.settings.compact_mode).unwrap_or(false);
    let compact_label = if is_compact { "✓  Compact Mode" } else { "○  Compact Mode" };
    let m_compact = MenuItem::with_id(app, "tray_toggle_compact", compact_label, true, None::<&str>)?;

    let sep3 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit StreamShield", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[
        &show,
        &sep1,
        &status,
        &sep2,
        &apps_submenu,
        &quick_actions,
        &mode_submenu,
        &m_compact,
        &sep3,
        &quit,
    ])?;

    Ok(menu)
}

fn rebuild_tray_menu(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(tray) = app.tray_by_id("main") {
        let menu = create_tray_menu(app)?;
        tray.set_menu(Some(menu))?;
    }
    Ok(())
}

fn update_tray_status(state: &AppState) {
    let (shielded, mode, auto_reapply) = state.config.lock().map(|c| {
        (c.shielded_exes.clone(), c.settings.shield_mode.clone(), c.settings.auto_reapply)
    }).unwrap_or_default();
    
    let wins = window_manager::enumerate_windows(&shielded, &mode, auto_reapply);
    let active_count = wins.iter().filter(|w| w.is_shielded).count();
    if let Ok(guard) = state.tray_status.lock() {
        if let Some(mi) = guard.as_ref() {
            let _ = mi.set_text(tray_status_text(active_count));
        }
    }
}

// ── Tauri commands ───────────────────────────────────────────────────────────

#[tauri::command]
fn get_windows(state: State<AppState>) -> Vec<WindowInfo> {
    let (shielded, mode, auto_reapply) = state.config.lock().map(|c| {
        (c.shielded_exes.clone(), c.settings.shield_mode.clone(), c.settings.auto_reapply)
    }).unwrap_or_default();
    
    let wins = window_manager::enumerate_windows(&shielded, &mode, auto_reapply);
    let active_count = wins.iter().filter(|w| w.is_shielded).count();
    if let Ok(guard) = state.tray_status.lock() {
        if let Some(mi) = guard.as_ref() {
            let _ = mi.set_text(tray_status_text(active_count));
        }
    }
    wins
}

#[tauri::command]
fn toggle_shield(exe_name: String, hwnd: usize, _pid: u32, enable: bool, state: State<AppState>, app: tauri::AppHandle) -> Result<bool, String> {
    let shield_mode = state.config.lock().map(|c| c.settings.shield_mode.clone()).unwrap_or_else(|_| "exclude".to_string());
    injector::set_window_affinity(hwnd, enable, Some(&shield_mode))?;
    {
        let mut config = state.config.lock().map_err(|e| e.to_string())?;
        let normalized = exe_name.to_lowercase();
        // Robust case-insensitive removal of matching and variant executables (e.g. WhatsApp.exe <-> WhatsApp.Root.exe)
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
    }
    update_tray_status(&state);
    let _ = rebuild_tray_menu(&app);
    Ok(true)
}

#[tauri::command]
fn get_shielded_exes(state: State<AppState>) -> Vec<String> {
    state.config.lock().map(|c| c.shielded_exes.iter().cloned().collect()).unwrap_or_default()
}

#[tauri::command]
fn reapply_shields(state: State<AppState>, app: tauri::AppHandle) -> Vec<String> {
    let (exes, mode, auto_reapply) = state.config.lock().map(|c| {
        (c.shielded_exes.clone(), c.settings.shield_mode.clone(), c.settings.auto_reapply)
    }).unwrap_or_default();

    let res = window_manager::enumerate_windows(&exes, &mode, auto_reapply).into_iter()
        .filter(|w| exes.contains(&w.exe_name.to_lowercase()))
        .filter(|w| injector::set_window_affinity(w.hwnd, true, Some(&mode)).is_ok())
        .map(|w| w.exe_name).collect();
    update_tray_status(&state);
    let _ = rebuild_tray_menu(&app);
    res
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
    let mut settings = state.config.lock().map(|c| c.settings.clone()).unwrap_or_default();
    // Synchronize autostart and MPO fix with true Windows Registry state
    #[cfg(windows)]
    {
        settings.autostart = get_autostart_registry();
        settings.mpo_fix = get_mpo_fix_registry();
    }
    settings
}

#[tauri::command]
fn update_settings(settings: AppSettings, state: State<AppState>, app: tauri::AppHandle) -> Result<AppSettings, String> {
    #[cfg(windows)]
    {
        let current_autostart = get_autostart_registry();
        if settings.autostart != current_autostart {
            let _ = set_autostart_registry(settings.autostart);
        }

        let current_mpo = get_mpo_fix_registry();
        if settings.mpo_fix != current_mpo {
            let _ = set_mpo_fix_registry(settings.mpo_fix);
        }
    }

    {
        let mut config = state.config.lock().map_err(|e| e.to_string())?;
        config.settings = settings.clone();
        save_config(&config);
    }

    update_tray_status(&state);
    let _ = rebuild_tray_menu(&app);
    Ok(settings)
}

#[tauri::command]
fn reset_settings(state: State<AppState>, app: tauri::AppHandle) -> Result<AppSettings, String> {
    let default_settings = AppSettings::default();
    #[cfg(windows)]
    {
        let _ = set_autostart_registry(false);
        let _ = set_mpo_fix_registry(false);
    }

    {
        let mut config = state.config.lock().map_err(|e| e.to_string())?;
        config.settings = default_settings.clone();
        save_config(&config);
    }

    update_tray_status(&state);
    let _ = rebuild_tray_menu(&app);
    Ok(default_settings)
}

#[tauri::command]
fn clear_all_shields(state: State<AppState>, app: tauri::AppHandle) -> Result<bool, String> {
    let (exes, mode, _) = state.config.lock().map(|c| {
        (c.shielded_exes.clone(), c.settings.shield_mode.clone(), false)
    }).unwrap_or_default();

    let wins = window_manager::enumerate_windows(&exes, &mode, false);
    for w in wins {
        if w.is_shielded {
            let _ = injector::set_window_affinity(w.hwnd, false, None);
        }
    }

    {
        let mut config = state.config.lock().map_err(|e| e.to_string())?;
        config.shielded_exes.clear();
        save_config(&config);
    }

    update_tray_status(&state);
    let _ = rebuild_tray_menu(&app);
    Ok(true)
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
        Ok(false)
    } else {
        window.maximize().map_err(|e| e.to_string())?;
        Ok(true)
    }
}

#[tauri::command]
fn toggle_self_shield(enable: bool, window: WebviewWindow) -> Result<bool, String> {
    #[cfg(windows)]
    {
        use winapi::um::winuser::SetWindowDisplayAffinity;
        const WDA_NONE: u32 = 0x00000000;
        const WDA_EXCLUDEFROMCAPTURE: u32 = 0x00000011;

        let hwnd = window.hwnd().map_err(|e| e.to_string())?;
        let affinity = if enable { WDA_EXCLUDEFROMCAPTURE } else { WDA_NONE };
        let ok = unsafe { SetWindowDisplayAffinity(hwnd.0 as _, affinity) };
        if ok != 0 {
            Ok(enable)
        } else {
            Err("Failed to set StreamShield display affinity".to_string())
        }
    }
    #[cfg(not(windows))]
    Ok(false)
}

#[tauri::command]
fn is_self_shielded(window: WebviewWindow) -> bool {
    #[cfg(windows)]
    {
        use winapi::um::winuser::GetWindowDisplayAffinity;
        const WDA_EXCLUDEFROMCAPTURE: u32 = 0x00000011;
        if let Ok(hwnd) = window.hwnd() {
            let mut affinity: u32 = 0;
            let ok = unsafe { GetWindowDisplayAffinity(hwnd.0 as _, &mut affinity) };
            return ok != 0 && affinity == WDA_EXCLUDEFROMCAPTURE;
        }
        false
    }
    #[cfg(not(windows))]
    false
}



#[tauri::command]
fn open_external_url(url: String) -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use winapi::um::shellapi::ShellExecuteW;
        use winapi::um::winuser::SW_SHOWNORMAL;
        fn wide(s: &str) -> Vec<u16> { OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect() }
        let url_w = wide(&url);
        let verb_w = wide("open");
        unsafe {
            ShellExecuteW(std::ptr::null_mut(), verb_w.as_ptr(), url_w.as_ptr(), std::ptr::null(), std::ptr::null(), SW_SHOWNORMAL);
        }
    }
    Ok(())
}

#[tauri::command]
fn install_update_silent(download_url: String) -> Result<(), String> {
    #[cfg(windows)]
    {
        let ps_cmd = format!(
            "$temp = \"$env:TEMP\\StreamShield_Setup.exe\"; \
             [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; \
             Invoke-WebRequest -Uri \"{}\" -OutFile $temp; \
             Start-Process -FilePath $temp -ArgumentList \"/S\"",
            download_url
        );
        std::process::Command::new("powershell")
            .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &ps_cmd])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn get_capture_environment() -> nvidia_bypass::CaptureEnvironment {
    nvidia_bypass::detect_capture_environment()
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
    if let Some(state) = app.try_state::<AppState>() {
        if let Ok(mut cfg) = state.config.lock() {
            cfg.settings.compact_mode = compact;
            let _ = save_config(&cfg);
        }
        let _ = rebuild_tray_menu(&app);
    }
    if let Some(w) = app.get_webview_window("main") {
        let _ = apply_window_compact_mode(&w, compact);
        let _ = w.emit("compact_mode_changed", compact);
    }
    Ok(compact)
}

// ── Main ─────────────────────────────────────────────────────────────────────

/// Create the main webview window on demand.
/// Called immediately for manual launches, or lazily when the user clicks
/// the tray icon after an autostart boot (where no window is created initially
/// to avoid the white flash).
fn create_main_window(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // If it already exists, just show it
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
        return Ok(());
    }

    let is_compact = app.try_state::<AppState>()
        .and_then(|s| s.config.lock().ok().map(|c| c.settings.compact_mode))
        .unwrap_or(false);

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
        .visible(true)
        .additional_browser_args("--disable-background-timer-throttling --disable-renderer-backgrounding --disable-backgrounding-occluded-windows");

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone())?;
    }

    let win = builder.build()?;
    let _ = win.set_focus();
    Ok(())
}

fn main() {
    // Enforce strict single-instance guard before anything else
    if !ensure_single_instance() {
        std::process::exit(0);
    }

    let initial_config = load_config();

    #[cfg(windows)]
    if !is_admin() {
        // Only the --minimized CLI flag (injected by autostart registry) triggers tray-only mode.
        // The start_minimized *setting* only controls whether --minimized is written to the registry,
        // so manual double-clicks always open the full window.
        let args: Vec<String> = std::env::args().collect();
        let is_minimized = args.iter().any(|a| a == "--minimized" || a == "-m");
        relaunch_as_admin(is_minimized);
        std::process::exit(0);
    }

    disable_power_throttling();
    injector::cleanup_stale_dlls();

    tauri::Builder::default()
        .manage(AppState { config: Mutex::new(initial_config.clone()), tray_status: Mutex::new(None) })
        .invoke_handler(tauri::generate_handler![
            get_windows, toggle_shield, get_shielded_exes, reapply_shields,
            get_settings, update_settings, reset_settings, clear_all_shields,
            check_admin, hide_to_tray, minimize_window, toggle_maximize_window,
            toggle_self_shield, is_self_shielded, get_capture_environment,
            open_external_url, install_update_silent, set_compact_mode
        ])
        .setup(move |app| {
            let args: Vec<String> = std::env::args().collect();
            let launched_by_autostart = args.iter().any(|a| a == "--minimized" || a == "-m");

            if !launched_by_autostart {
                // Manual launch: create and show the window immediately.
                create_main_window(app.handle())?;
            }
            // Autostart: NO window created = NO white flash. Just tray icon.

            // ── Dynamic Tray menu with toggleable options ────────────────
            let menu = create_tray_menu(app.handle())?;

            // ── Tray icon ────────────────────────────────────────────────
            let icon = app.default_window_icon()
                .ok_or("no window icon")?
                .clone();

            TrayIconBuilder::with_id("main")
                .icon(icon)
                .tooltip("StreamShield")
                .menu(&menu)
                .show_menu_on_left_click(false)
                // Left click: toggle window (create on demand if needed)
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            if w.is_visible().unwrap_or(false) {
                                let _ = w.hide();
                            } else {
                                let _ = w.show();
                                let _ = w.unminimize();
                                let _ = w.set_focus();
                                let _ = w.emit("app_wake", ());
                            }
                        } else {
                            // Window doesn't exist yet (autostart mode) — create it now
                            let _ = create_main_window(app);
                        }
                    }
                })
                // Right-click menu actions & app toggles
                .on_menu_event(|app, event| {
                    let id = event.id.as_ref();
                    match id {
                        "show" => {
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.show();
                                let _ = w.unminimize();
                                let _ = w.set_focus();
                                let _ = w.emit("app_wake", ());
                            } else {
                                let _ = create_main_window(app);
                            }
                        }
                        "tray_shield_all" => {
                            if let Some(state) = app.try_state::<AppState>() {
                                let (mode, auto_reapply) = state.config.lock().map(|c| {
                                    (c.settings.shield_mode.clone(), c.settings.auto_reapply)
                                }).unwrap_or_default();
                                {
                                    let mut cfg = match state.config.lock() {
                                        Ok(c) => c,
                                        Err(_) => return,
                                    };
                                    let wins = window_manager::enumerate_windows(&cfg.shielded_exes, &mode, auto_reapply);
                                    for w in wins {
                                        let _ = injector::set_window_affinity(w.hwnd, true, Some(&mode));
                                        cfg.shielded_exes.insert(w.exe_name.to_lowercase());
                                    }
                                    save_config(&cfg);
                                }
                                let _ = rebuild_tray_menu(app);
                                if let Some(w) = app.get_webview_window("main") {
                                    let _ = w.emit("tray_action_completed", "shield_all");
                                }
                            }
                        }
                        "tray_unshield_all" => {
                            if let Some(state) = app.try_state::<AppState>() {
                                let auto_reapply = state.config.lock().map(|c| c.settings.auto_reapply).unwrap_or(true);
                                {
                                    let mut cfg = match state.config.lock() {
                                        Ok(c) => c,
                                        Err(_) => return,
                                    };
                                    let wins = window_manager::enumerate_windows(&cfg.shielded_exes, "exclude", auto_reapply);
                                    for w in wins {
                                        let _ = injector::set_window_affinity(w.hwnd, false, None);
                                    }
                                    cfg.shielded_exes.clear();
                                    save_config(&cfg);
                                }
                                let _ = rebuild_tray_menu(app);
                                if let Some(w) = app.get_webview_window("main") {
                                    let _ = w.emit("tray_action_completed", "unshield_all");
                                }
                            }
                        }
                        "tray_reapply" => {
                            if let Some(state) = app.try_state::<AppState>() {
                                let (exes, mode, auto_reapply) = state.config.lock().map(|c| {
                                    (c.shielded_exes.clone(), c.settings.shield_mode.clone(), c.settings.auto_reapply)
                                }).unwrap_or_default();
                                let wins = window_manager::enumerate_windows(&exes, &mode, auto_reapply);
                                for w in wins {
                                    if exes.contains(&w.exe_name.to_lowercase()) {
                                        let _ = injector::set_window_affinity(w.hwnd, true, Some(&mode));
                                    }
                                }
                                let _ = rebuild_tray_menu(app);
                            }
                        }
                        "tray_mode_exclude" => {
                            if let Some(state) = app.try_state::<AppState>() {
                                {
                                    let mut cfg = match state.config.lock() {
                                        Ok(c) => c,
                                        Err(_) => return,
                                    };
                                    cfg.settings.shield_mode = "exclude".to_string();
                                    let wins = window_manager::enumerate_windows(&cfg.shielded_exes, "exclude", cfg.settings.auto_reapply);
                                    for w in wins {
                                        if cfg.shielded_exes.contains(&w.exe_name.to_lowercase()) {
                                            let _ = injector::set_window_affinity(w.hwnd, true, Some("exclude"));
                                        }
                                    }
                                    save_config(&cfg);
                                }
                                let _ = rebuild_tray_menu(app);
                                if let Some(w) = app.get_webview_window("main") {
                                    let _ = w.emit("tray_action_completed", "mode_changed");
                                }
                            }
                        }
                        "tray_mode_monitor" => {
                            if let Some(state) = app.try_state::<AppState>() {
                                {
                                    let mut cfg = match state.config.lock() {
                                        Ok(c) => c,
                                        Err(_) => return,
                                    };
                                    cfg.settings.shield_mode = "monitor".to_string();
                                    let wins = window_manager::enumerate_windows(&cfg.shielded_exes, "monitor", cfg.settings.auto_reapply);
                                    for w in wins {
                                        if cfg.shielded_exes.contains(&w.exe_name.to_lowercase()) {
                                            let _ = injector::set_window_affinity(w.hwnd, true, Some("monitor"));
                                        }
                                    }
                                    save_config(&cfg);
                                }
                                let _ = rebuild_tray_menu(app);
                                if let Some(w) = app.get_webview_window("main") {
                                    let _ = w.emit("tray_action_completed", "mode_changed");
                                }
                            }
                        }
                        "tray_toggle_compact" => {
                            if let Some(state) = app.try_state::<AppState>() {
                                let new_compact = {
                                    let mut cfg = match state.config.lock() {
                                        Ok(c) => c,
                                        Err(_) => return,
                                    };
                                    cfg.settings.compact_mode = !cfg.settings.compact_mode;
                                    let val = cfg.settings.compact_mode;
                                    let _ = save_config(&cfg);
                                    val
                                };
                                let _ = rebuild_tray_menu(app);
                                if let Some(w) = app.get_webview_window("main") {
                                    let _ = apply_window_compact_mode(&w, new_compact);
                                    let _ = w.emit("compact_mode_changed", new_compact);
                                    let _ = w.emit("tray_action_completed", "compact_toggled");
                                }
                            }
                        }
                        "quit" => {
                            let _ = app.remove_tray_by_id("main");
                            app.exit(0);
                        }
                        other if other.starts_with("toggle_app:") => {
                            let target_exe = other.trim_start_matches("toggle_app:");
                            if let Some(state) = app.try_state::<AppState>() {
                                let (mode, auto_reapply) = state.config.lock().map(|c| {
                                    (c.settings.shield_mode.clone(), c.settings.auto_reapply)
                                }).unwrap_or_default();
                                {
                                    let mut cfg = match state.config.lock() {
                                        Ok(c) => c,
                                        Err(_) => return,
                                    };
                                    let norm = target_exe.to_lowercase();
                                    let currently_shielded = cfg.shielded_exes.contains(&norm);
                                    let new_state = !currently_shielded;
                                    let wins = window_manager::enumerate_windows(&cfg.shielded_exes, &mode, auto_reapply);
                                    for w in wins {
                                        if w.exe_name.eq_ignore_ascii_case(target_exe) {
                                            let _ = injector::set_window_affinity(w.hwnd, new_state, Some(&mode));
                                        }
                                    }
                                    if new_state {
                                        cfg.shielded_exes.insert(norm);
                                    } else {
                                        cfg.shielded_exes.remove(&norm);
                                    }
                                    save_config(&cfg);
                                }
                                let _ = rebuild_tray_menu(app);
                                if let Some(w) = app.get_webview_window("main") {
                                    let _ = w.emit("tray_action_completed", "app_toggled");
                                }
                            }
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
                    disable_power_throttling();
                    if let Some(state) = app_handle.try_state::<AppState>() {
                        let (shielded, mode, auto_reapply) = {
                            if let Ok(guard) = state.config.lock() {
                                (guard.shielded_exes.clone(), guard.settings.shield_mode.clone(), guard.settings.auto_reapply)
                            } else {
                                (std::collections::HashSet::new(), "exclude".to_string(), false)
                            }
                        };
                        
                        if auto_reapply && !shielded.is_empty() {
                            window_manager::auto_reapply_shields(&shielded, &mode);
                        }
                        update_tray_status(&state);
                    }
                }
            });

            Ok(())
        })
        // X button: hide to tray instead of close
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("StreamShield error");
}

