// Always hide console window - no terminal alongside the app
#![windows_subsystem = "windows"]

mod window_manager;
mod injector;
mod config;
mod nvidia_bypass;

use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, State, WebviewWindow,
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
fn relaunch_as_admin() {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::shellapi::ShellExecuteW;
    use winapi::um::winuser::{SW_SHOW, SW_HIDE};
    fn wide(s: &str) -> Vec<u16> { OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect() }
    let exe = std::env::current_exe().unwrap();
    let exe_w = wide(exe.to_str().unwrap());
    let verb_w = wide("runas");

    let raw_args: Vec<String> = std::env::args().skip(1).collect();
    let is_minimized = raw_args.iter().any(|a| a == "--minimized" || a == "-m");
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
    unsafe {
        // Prevent Windows from suspending background watchdog thread and timer execution
        SetThreadExecutionState(ES_CONTINUOUS | ES_SYSTEM_REQUIRED);
    }
}

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
fn toggle_shield(exe_name: String, hwnd: usize, _pid: u32, enable: bool, state: State<AppState>) -> Result<bool, String> {
    let shield_mode = state.config.lock().map(|c| c.settings.shield_mode.clone()).unwrap_or_else(|_| "exclude".to_string());
    injector::set_window_affinity(hwnd, enable, Some(&shield_mode))?;
    {
        let mut config = state.config.lock().map_err(|e| e.to_string())?;
        let normalized = exe_name.to_lowercase();
        if enable {
            config.shielded_exes.insert(normalized);
        } else {
            config.shielded_exes.remove(&normalized);
        }
        save_config(&config);
    }
    update_tray_status(&state);
    Ok(true)
}

#[tauri::command]
fn get_shielded_exes(state: State<AppState>) -> Vec<String> {
    state.config.lock().map(|c| c.shielded_exes.iter().cloned().collect()).unwrap_or_default()
}

#[tauri::command]
fn reapply_shields(state: State<AppState>) -> Vec<String> {
    let (exes, mode, auto_reapply) = state.config.lock().map(|c| {
        (c.shielded_exes.clone(), c.settings.shield_mode.clone(), c.settings.auto_reapply)
    }).unwrap_or_default();

    let res = window_manager::enumerate_windows(&exes, &mode, auto_reapply).into_iter()
        .filter(|w| exes.contains(&w.exe_name.to_lowercase()))
        .filter(|w| injector::set_window_affinity(w.hwnd, true, Some(&mode)).is_ok())
        .map(|w| w.exe_name).collect();
    update_tray_status(&state);
    res
}

#[cfg(windows)]
fn set_mpo_fix_registry(enable: bool) -> Result<(), String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::winreg::{RegOpenKeyExW, RegSetValueExW, RegDeleteValueW, RegCloseKey, HKEY_LOCAL_MACHINE};
    use winapi::um::winnt::{KEY_SET_VALUE, REG_DWORD};
    
    fn wide(s: &str) -> Vec<u16> { OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect() }
    
    let subkey = wide(r"SOFTWARE\Microsoft\Windows\Dwm");
    let val_name = wide("OverlayTestMode");
    
    unsafe {
        let mut hkey = std::ptr::null_mut();
        let status = RegOpenKeyExW(HKEY_LOCAL_MACHINE, subkey.as_ptr(), 0, KEY_SET_VALUE, &mut hkey);
        if status != 0 {
            return Err(format!("RegOpenKeyExW (HKLM) failed (error code {}). Administrator privileges required to change MPO settings.", status));
        }
        
        let res = if enable {
            let data: u32 = 5; // 5 = disable hardware MPO overlay plane bypass
            let set_res = RegSetValueExW(
                hkey,
                val_name.as_ptr(),
                0,
                REG_DWORD,
                &data as *const u32 as *const _,
                std::mem::size_of::<u32>() as u32,
            );
            if set_res != 0 {
                Err(format!("RegSetValueExW failed with error: {}", set_res))
            } else {
                Ok(())
            }
        } else {
            let del_res = RegDeleteValueW(hkey, val_name.as_ptr());
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
fn set_mpo_fix_registry(_enable: bool) -> Result<(), String> {
    Ok(())
}

#[cfg(windows)]
fn get_mpo_fix_registry() -> bool {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::winreg::{RegOpenKeyExW, RegQueryValueExW, RegCloseKey, HKEY_LOCAL_MACHINE};
    use winapi::um::winnt::KEY_QUERY_VALUE;
    
    fn wide(s: &str) -> Vec<u16> { OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect() }
    
    let subkey = wide(r"SOFTWARE\Microsoft\Windows\Dwm");
    let val_name = wide("OverlayTestMode");
    
    unsafe {
        let mut hkey = std::ptr::null_mut();
        if RegOpenKeyExW(HKEY_LOCAL_MACHINE, subkey.as_ptr(), 0, KEY_QUERY_VALUE, &mut hkey) != 0 {
            return false;
        }
        let mut data_type = 0u32;
        let mut data: u32 = 0;
        let mut data_len = std::mem::size_of::<u32>() as u32;
        let query_res = RegQueryValueExW(
            hkey,
            val_name.as_ptr(),
            std::ptr::null_mut(),
            &mut data_type,
            &mut data as *mut u32 as *mut _,
            &mut data_len,
        );
        RegCloseKey(hkey);
        query_res == 0 && data == 5
    }
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
fn update_settings(settings: AppSettings, state: State<AppState>) -> Result<AppSettings, String> {
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
    Ok(settings)
}

#[tauri::command]
fn reset_settings(state: State<AppState>) -> Result<AppSettings, String> {
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
    Ok(default_settings)
}

#[tauri::command]
fn clear_all_shields(state: State<AppState>) -> Result<bool, String> {
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
fn apply_nvidia_bypass() -> Result<usize, String> {
    #[cfg(windows)]
    {
        Ok(nvidia_bypass::patch_nvidia_processes())
    }
    #[cfg(not(windows))]
    Ok(0)
}

// ── Main ─────────────────────────────────────────────────────────────────────

fn main() {
    #[cfg(windows)]
    if !is_admin() {
        relaunch_as_admin();
        std::process::exit(0);
    }

    disable_power_throttling();
    injector::cleanup_stale_dlls();

    let initial_config = load_config();

    tauri::Builder::default()
        .manage(AppState { config: Mutex::new(initial_config.clone()), tray_status: Mutex::new(None) })
        .invoke_handler(tauri::generate_handler![
            get_windows, toggle_shield, get_shielded_exes, reapply_shields,
            get_settings, update_settings, reset_settings, clear_all_shields,
            check_admin, hide_to_tray, toggle_self_shield, is_self_shielded,
            apply_nvidia_bypass
        ])
        .setup(move |app| {
            // ── Show main window (unless started with --minimized or start_minimized is on) ──
            let win = app.get_webview_window("main").ok_or("no main window")?;
            let args: Vec<String> = std::env::args().collect();
            let is_minimized_arg = args.iter().any(|a| a == "--minimized" || a == "-m");
            let should_start_minimized = is_minimized_arg || initial_config.settings.start_minimized;

            if !should_start_minimized {
                win.show()?;
                win.unminimize()?;
                win.set_focus()?;
            } else {
                let _ = win.hide();
            }

            // ── Tray menu ────────────────────────────────────────────────
            let show   = MenuItem::with_id(app, "show",   "🛡️  Open StreamShield", true, None::<&str>)?;
            let sep1   = PredefinedMenuItem::separator(app)?;
            let status = MenuItem::with_id(app, "status", "No apps shielded",      false, None::<&str>)?;
            let sep2   = PredefinedMenuItem::separator(app)?;
            let quit   = MenuItem::with_id(app, "quit",   "Quit",                  true, None::<&str>)?;
            let menu   = Menu::with_items(app, &[&show, &sep1, &status, &sep2, &quit])?;

            // Store status item handle so toggle_shield can update it
            if let Some(state) = app.try_state::<AppState>() {
                if let Ok(mut guard) = state.tray_status.lock() {
                    *guard = Some(status);
                }
                update_tray_status(&state);
            }

            // ── Tray icon ────────────────────────────────────────────────
            let icon = app.default_window_icon()
                .ok_or("no window icon")?
                .clone();

            TrayIconBuilder::with_id("main")
                .icon(icon)
                .tooltip("StreamShield - Stream Privacy Manager")
                .menu(&menu)
                .show_menu_on_left_click(false)
                // Left click: toggle window
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
                        }
                    }
                })
                // Right-click menu actions
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.unminimize();
                            let _ = w.set_focus();
                            let _ = w.emit("app_wake", ());
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
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
                            #[cfg(windows)]
                            {
                                nvidia_bypass::patch_nvidia_processes();
                            }
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

