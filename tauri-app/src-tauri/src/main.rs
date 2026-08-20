#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod window_manager;
mod injector;
mod config;

use std::sync::Mutex;
use tauri::{Manager, State};
use window_manager::WindowInfo;
use config::{load_config, save_config, ShieldConfig};

struct AppState { config: Mutex<ShieldConfig> }

/// Check admin via token elevation level
#[cfg(windows)]
fn is_admin() -> bool {
    use winapi::um::processthreadsapi::{GetCurrentProcess, OpenProcessToken};
    use winapi::um::securitybaseapi::GetTokenInformation;
    use winapi::um::winnt::{TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY, HANDLE};
    use winapi::um::handleapi::CloseHandle;

    unsafe {
        let mut token: HANDLE = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return false;
        }
        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut size = std::mem::size_of::<TOKEN_ELEVATION>() as u32;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            &mut elevation as *mut _ as *mut _,
            size,
            &mut size,
        );
        CloseHandle(token);
        ok != 0 && elevation.TokenIsElevated != 0
    }
}

/// Relaunch with UAC elevation via ShellExecute "runas"
#[cfg(windows)]
fn relaunch_as_admin() {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::shellapi::ShellExecuteW;
    use winapi::um::winuser::SW_SHOW;

    fn wide(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    }
    let exe = std::env::current_exe().unwrap();
    let exe_w = wide(exe.to_str().unwrap());
    let verb_w = wide("runas");
    unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            verb_w.as_ptr(),
            exe_w.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOW as i32,
        );
    }
}

#[tauri::command]
fn get_windows() -> Vec<WindowInfo> { window_manager::enumerate_windows() }

#[tauri::command]
fn toggle_shield(exe_name: String, hwnd: usize, enable: bool, state: State<AppState>) -> Result<bool, String> {
    injector::set_window_affinity(hwnd, enable)?;
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    if enable { config.shielded_exes.insert(exe_name); }
    else { config.shielded_exes.remove(&exe_name); }
    save_config(&config);
    Ok(true)
}

#[tauri::command]
fn get_shielded_exes(state: State<AppState>) -> Vec<String> {
    state.config.lock().map(|c| c.shielded_exes.iter().cloned().collect()).unwrap_or_default()
}

#[tauri::command]
fn reapply_shields(state: State<AppState>) -> Vec<String> {
    let exes = state.config.lock().unwrap().shielded_exes.clone();
    window_manager::enumerate_windows().into_iter()
        .filter(|w| exes.contains(&w.exe_name))
        .filter(|w| injector::set_window_affinity(w.hwnd, true).is_ok())
        .map(|w| w.exe_name)
        .collect()
}

#[tauri::command]
fn check_admin() -> bool {
    #[cfg(windows)] { is_admin() }
    #[cfg(not(windows))] { true }
}

fn main() {
    // Auto-elevate: if not admin, relaunch with UAC prompt and exit
    #[cfg(windows)]
    if !is_admin() {
        relaunch_as_admin();
        std::process::exit(0);
    }

    tauri::Builder::default()
        .manage(AppState { config: Mutex::new(load_config()) })
        .invoke_handler(tauri::generate_handler![
            get_windows, toggle_shield, get_shielded_exes, reapply_shields, check_admin
        ])
        .setup(|app| {
            let win = app.get_webview_window("main").ok_or("no main window")?;
            let _ = win.show();
            let _ = win.set_focus();
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("StreamShield error");
}
