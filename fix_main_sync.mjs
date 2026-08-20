import fs from "fs";
const p = "C:\\Users\\GOYAL\\Documents\\work\\StreamShield\\tauri-app\\src-tauri\\src\\main.rs";
const c = `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod window_manager;
mod injector;
mod config;

use std::sync::Mutex;
use tauri::{Manager, State};
use window_manager::WindowInfo;
use config::{load_config, save_config, ShieldConfig};

struct AppState { config: Mutex<ShieldConfig> }

#[tauri::command]
fn get_windows() -> Vec<WindowInfo> { window_manager::enumerate_windows() }

#[tauri::command]
fn toggle_shield(exe_name: String, hwnd: usize, enable: bool, state: State<AppState>) -> Result<bool, String> {
    let result = injector::set_window_affinity(hwnd, enable);
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    if enable { config.shielded_exes.insert(exe_name); } else { config.shielded_exes.remove(&exe_name); }
    save_config(&config);
    result.map(|_| true)
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
        .map(|w| w.exe_name).collect()
}

fn main() {
    tauri::Builder::default()
        .manage(AppState { config: Mutex::new(load_config()) })
        .invoke_handler(tauri::generate_handler![
            get_windows, toggle_shield, get_shielded_exes, reapply_shields
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
`;
fs.writeFileSync(p, c, "utf8");
console.log("OK main.rs | starts:", JSON.stringify(c.slice(0,20)));