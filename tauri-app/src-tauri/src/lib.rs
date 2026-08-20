mod window_manager;
mod injector;
mod config;

use std::sync::Mutex;
use tauri::{Manager, State};
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
    let result = injector::set_window_affinity(hwnd, enable);

    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    if enable {
        config.shielded_exes.insert(exe_name.clone());
    } else {
        config.shielded_exes.remove(&exe_name);
    }
    save_config(&config);

    match result {
        Ok(_) => Ok(true),
        Err(e) => Err(format!(
            "Could not shield window: {}. Try running StreamShield as Administrator.",
            e
        )),
    }
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
            use tauri::menu::{Menu, MenuItem};
            use tauri::tray::TrayIconBuilder;

            let quit =
                MenuItem::with_id(app, "quit", "Quit StreamShield", true, None::<&str>)?;
            let show =
                MenuItem::with_id(app, "show", "Open StreamShield", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            let _tray = TrayIconBuilder::new()
                .tooltip("StreamShield")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    use tauri::tray::TrayIconEvent;
                    if let TrayIconEvent::Click { .. } = event {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running StreamShield");
}
