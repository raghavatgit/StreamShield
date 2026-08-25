// Always hide console window — no terminal alongside the app
#![windows_subsystem = "windows"]

mod window_manager;
mod injector;
mod config;

use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, State, WebviewWindow,
};
use window_manager::WindowInfo;
use config::{load_config, save_config, ShieldConfig};

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
    use winapi::um::winuser::SW_SHOW;
    fn wide(s: &str) -> Vec<u16> { OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect() }
    let exe = std::env::current_exe().unwrap();
    let exe_w = wide(exe.to_str().unwrap());
    let verb_w = wide("runas");
    unsafe { ShellExecuteW(std::ptr::null_mut(), verb_w.as_ptr(), exe_w.as_ptr(), std::ptr::null(), std::ptr::null(), SW_SHOW as i32); }
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

// ── Tray status helpers ───────────────────────────────────────────────────────

fn tray_status_text(count: usize) -> String {
    if count == 0 {
        "No apps shielded".to_string()
    } else {
        format!("🛡️  {} app{} hidden from capture", count, if count == 1 { "" } else { "s" })
    }
}

fn update_tray_status(state: &AppState) {
    let shielded = state.config.lock().map(|c| c.shielded_exes.clone()).unwrap_or_default();
    let wins = window_manager::enumerate_windows(&shielded);
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
    let shielded = state.config.lock().map(|c| c.shielded_exes.clone()).unwrap_or_default();
    let wins = window_manager::enumerate_windows(&shielded);
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
    injector::set_window_affinity(hwnd, enable)?;
    {
        let mut config = state.config.lock().map_err(|e| e.to_string())?;
        if enable {
            config.shielded_exes.insert(exe_name);
        } else {
            config.shielded_exes.remove(&exe_name);
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
    let exes = state.config.lock().unwrap().shielded_exes.clone();
    let res = window_manager::enumerate_windows(&exes).into_iter()
        .filter(|w| exes.contains(&w.exe_name))
        .filter(|w| injector::set_window_affinity(w.hwnd, true).is_ok())
        .map(|w| w.exe_name).collect();
    update_tray_status(&state);
    res
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

// ── Main ─────────────────────────────────────────────────────────────────────

fn main() {
    #[cfg(windows)]
    if !is_admin() {
        relaunch_as_admin();
        std::process::exit(0);
    }

    disable_power_throttling();

    tauri::Builder::default()
        .manage(AppState { config: Mutex::new(load_config()), tray_status: Mutex::new(None) })
        .invoke_handler(tauri::generate_handler![
            get_windows, toggle_shield, get_shielded_exes, reapply_shields,
            check_admin, hide_to_tray, toggle_self_shield, is_self_shielded
        ])
        .setup(|app| {
            // ── Show main window ─────────────────────────────────────────
            let win = app.get_webview_window("main").ok_or("no main window")?;
            win.show()?;
            win.set_focus()?;

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
                .tooltip("StreamShield — Stream Privacy Manager")
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
                    std::thread::sleep(std::time::Duration::from_millis(2000));
                    disable_power_throttling();
                    if let Some(state) = app_handle.try_state::<AppState>() {
                        let shielded = state.config.lock().map(|c| c.shielded_exes.clone()).unwrap_or_default();
                        if !shielded.is_empty() {
                            window_manager::auto_reapply_shields(&shielded);
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
