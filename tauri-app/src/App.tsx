import { useState, useEffect, useCallback, useRef } from "react";
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
        <button className={`refresh-btn${refreshing ? " spinning" : ""}`} onClick={handleRefresh}>
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
