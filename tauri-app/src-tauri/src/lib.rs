od window_manager;
mod injector;
mod config;

use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};
use window_manager::WindowInfo;
use config::{load_config, save_config, ShieldConfig};

pub struct AppState {
    pub config: Mutex<ShieldConfig>,
}

#[tauri::command]
fn get_windows() -> Vec<WindowInfo> {
    window_manager::enumerate_windows()
}

#[tauri::command]
fn toggle_shield(
    exe_name: String,
    hwnd: usize,
    enable: bool,
    state: State<AppState>,
) -> Result<bool, String> {
    // Try direct API call first (works for same-session windows)
    let result = injector::set_window_affinity(hwnd, enable);

    // Update persistent config regardless (we track by exe name)
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    if enable {
        config.shielded_exes.insert(exe_name.clone());
    } else {
        config.shielded_exes.remove(&exe_name);
    }
    save_config(&config);

    match result {
        Ok(_) => Ok(true),
        Err(e) => {
            // Try injection fallback
            // For now return the error (user may need to run as admin)
            Err(format!("Could not shield window: {}. Try running StreamShield as Administrator.", e))
        }
    }
}

#[tauri::command]
fn get_shielded_exes(state: State<AppState>) -> Vec<String> {
    state.config.lock()
        .map(|c| c.shielded_exes.iter().cloned().collect())
        .unwrap_or_default()
}

#[tauri::command]
fn reapply_shields(state: State<AppState>) -> Vec<String> {
    let shielded_exes = {
        let config = state.config.lock().unwrap();
        config.shielded_exes.clone()
    };

    let windows = window_manager::enumerate_windows();
    let mut reapplied = Vec::new();

    for window in windows {
        if shielded_exes.contains(&window.exe_name) {
            if injector::set_window_affinity(window.hwnd, true).is_ok() {
                reapplied.push(window.exe_name.clone());
            }
        }
    }
    reapplied
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = load_config();

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(AppState {
            config: Mutex::new(config),
        })
        .invoke_handler(tauri::generate_handler![
            get_windows,
            toggle_shield,
            get_shielded_exes,
            reapply_shields,
        ])
        .setup(|app| {
            // System tray setup
            use tauri::tray::{TrayIconBuilder, MenuId, menu::MenuEvent};
            use tauri::menu::{Menu, MenuItem};

            let quit = MenuItem::with_id(app, "quit", "Quit StreamShield", true, None::<&str>)?;
            let show = MenuItem::with_id(app, "show", "Open StreamShield", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            let _tray = TrayIconBuilder::new()
                .tooltip("StreamShield — Stream Privacy Manager")
                .menu(&menu)
                .menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    use tauri::tray::TrayIconEvent;
                    if let TrayIconEvent::Click { .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // Reapply shields on startup in background
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_secs(2));
                let state: State<AppState> = app_handle.state();
                let _ = reapply_shields(state);
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Minimize to tray instead of closing
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running StreamShield");
}
