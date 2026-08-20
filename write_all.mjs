import fs from "fs";

const base = "C:\\Users\\GOYAL\\Documents\\work\\StreamShield\\tauri-app";
const srcTauri = base + "\\src-tauri\\src";

// ── main.rs ─────────────────────────────────────────────────────────────────
fs.writeFileSync(srcTauri + "\\main.rs", `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod window_manager;
mod injector;
mod config;

use std::sync::Mutex;
use tauri::{Manager, State};
use window_manager::WindowInfo;
use config::{load_config, save_config, ShieldConfig};

struct AppState { config: Mutex<ShieldConfig> }

/// Check if the current process has admin privileges
#[cfg(windows)]
fn is_admin() -> bool {
    unsafe { winapi::um::securitybaseapi::IsUserAnAdmin() != 0 }
}

/// Relaunch the current exe with "runas" verb (triggers UAC prompt)
#[cfg(windows)]
fn relaunch_as_admin() {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::shellapi::ShellExecuteW;
    use winapi::um::winuser::SW_SHOW;

    fn wide(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    }

    let exe = std::env::current_exe().expect("can't get exe path");
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
fn toggle_shield(
    exe_name: String, hwnd: usize, enable: bool, state: State<AppState>
) -> Result<bool, String> {
    injector::set_window_affinity(hwnd, enable)?;
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    if enable { config.shielded_exes.insert(exe_name); }
    else       { config.shielded_exes.remove(&exe_name); }
    save_config(&config);
    Ok(true)
}

#[tauri::command]
fn get_shielded_exes(state: State<AppState>) -> Vec<String> {
    state.config.lock()
        .map(|c| c.shielded_exes.iter().cloned().collect())
        .unwrap_or_default()
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
    // Auto-elevate on Windows if not already admin
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
`, "utf8");
console.log("main.rs written");

// ── App.tsx ──────────────────────────────────────────────────────────────────
fs.writeFileSync(base + "\\src\\App.tsx", `import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import AppRow from "./components/AppRow";

interface WindowInfo { hwnd: number; pid: number; title: string; exe_name: string; }

export default function App() {
  const [windows, setWindows] = useState<WindowInfo[]>([]);
  const [shieldedExes, setShieldedExes] = useState<Set<string>>(new Set());
  const [search, setSearch] = useState("");
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [isAdmin, setIsAdmin] = useState(true);
  const [toast, setToast] = useState<string | null>(null);
  const toastTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  const showToast = (msg: string) => {
    setToast(msg);
    if (toastTimer.current) clearTimeout(toastTimer.current);
    toastTimer.current = setTimeout(() => setToast(null), 4000);
  };

  const loadWindows = useCallback(async () => {
    try {
      const [wins, shielded, admin] = await Promise.all([
        invoke<WindowInfo[]>("get_windows"),
        invoke<string[]>("get_shielded_exes"),
        invoke<boolean>("check_admin"),
      ]);
      setWindows(wins);
      setShieldedExes(new Set(shielded));
      setIsAdmin(admin);
    } catch (e) { console.error(e); }
    finally { setLoading(false); setRefreshing(false); }
  }, []);

  useEffect(() => { loadWindows(); }, [loadWindows]);

  const handleRefresh = async () => { setRefreshing(true); await loadWindows(); };

  const handleToggle = async (win: WindowInfo, enable: boolean) => {
    // Optimistic update — toggle immediately in UI
    setShieldedExes(prev => {
      const next = new Set(prev);
      enable ? next.add(win.exe_name) : next.delete(win.exe_name);
      return next;
    });
    try {
      await invoke<boolean>("toggle_shield", { exeName: win.exe_name, hwnd: win.hwnd, enable });
    } catch (e) {
      // Revert on failure
      setShieldedExes(prev => {
        const next = new Set(prev);
        enable ? next.delete(win.exe_name) : next.add(win.exe_name);
        return next;
      });
      showToast(String(e).replace("Error: ", ""));
    }
  };

  const filtered = windows.filter(w =>
    w.exe_name.toLowerCase().includes(search.toLowerCase()) ||
    w.title.toLowerCase().includes(search.toLowerCase())
  );
  const shieldedCount = [...shieldedExes].filter(e => windows.some(w => w.exe_name === e)).length;

  return (
    <div className="app">
      <header className="header">
        <div className="header-top">
          <div className="logo">🛡️</div>
          <span className="app-name">StreamShield</span>
          <span className="app-tagline">Stream Privacy Manager</span>
        </div>
        {!isAdmin && (
          <div className="admin-warn">
            ⚠️ Not running as Administrator — shields may not work
          </div>
        )}
        <div className="status-bar">
          {shieldedCount > 0 ? (
            <div className="status-badge active">
              <div className="status-dot pulse" />
              {shieldedCount} app{shieldedCount !== 1 ? "s" : ""} hidden from capture
            </div>
          ) : (
            <div className="status-badge inactive">
              <div className="status-dot" /> No apps shielded
            </div>
          )}
        </div>
      </header>

      <div className="search-wrap">
        <span className="search-icon">🔍</span>
        <input className="search-input" type="text" placeholder="Search applications..."
          value={search} onChange={e => setSearch(e.target.value)} />
      </div>

      <div className="list-header">
        <span>Running Applications ({filtered.length})</span>
        <button className={\`refresh-btn\${refreshing ? " spinning" : ""}\`} onClick={handleRefresh}>
          ↻ Refresh
        </button>
      </div>

      <div className="window-list">
        {loading
          ? Array.from({ length: 6 }).map((_, i) => <div key={i} className="shimmer shimmer-row" />)
          : filtered.length === 0
          ? (
            <div className="empty-state">
              <div className="empty-icon">🪟</div>
              <div className="empty-title">No applications found</div>
              <div>{search ? "Try a different search" : "Open apps and click Refresh"}</div>
            </div>
          )
          : filtered.map(win => (
            <AppRow
              key={win.hwnd}
              window={win}
              shielded={shieldedExes.has(win.exe_name)}
              onToggle={handleToggle}
            />
          ))
        }
      </div>

      <footer className="footer">
        <div className="footer-count"><span>{shieldedCount}</span> of {windows.length} shielded</div>
        <button className="minimize-btn" onClick={() => getCurrentWindow().hide()}>
          Minimize to tray ↗
        </button>
      </footer>

      {toast && <div className="toast">{toast}</div>}
    </div>
  );
}
`, "utf8");
console.log("App.tsx written");

// ── index.css - add admin-warn style ────────────────────────────────────────
const css = fs.readFileSync(base + "\\src\\index.css", "utf8");
if (!css.includes("admin-warn")) {
  fs.writeFileSync(base + "\\src\\index.css",
    css + `\n.admin-warn { background: rgba(246,173,85,0.12); border: 1px solid rgba(246,173,85,0.35);
  color: #f6ad55; padding: 6px 10px; border-radius: var(--radius-sm);
  font-size: 11px; margin-bottom: 8px; }\n`,
    "utf8"
  );
  console.log("index.css: admin-warn style added");
} else {
  console.log("index.css: admin-warn already present");
}

console.log("All files written!");