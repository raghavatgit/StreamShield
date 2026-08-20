mport { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import AppRow from "./components/AppRow";

interface WindowInfo {
  hwnd: number;
  pid: number;
  title: string;
  exe_name: string;
  icon_base64?: string;
}

export default function App() {
  const [windows, setWindows] = useState<WindowInfo[]>([]);
  const [shieldedExes, setShieldedExes] = useState<Set<string>>(new Set());
  const [search, setSearch] = useState("");
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const errorTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  const loadWindows = useCallback(async () => {
    try {
      const [wins, shielded] = await Promise.all([
        invoke<WindowInfo[]>("get_windows"),
        invoke<string[]>("get_shielded_exes"),
      ]);
      setWindows(wins);
      setShieldedExes(new Set(shielded));
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
      setRefreshing(false);
    }
  }, []);

  useEffect(() => { loadWindows(); }, [loadWindows]);

  const handleRefresh = async () => {
    setRefreshing(true);
    await loadWindows();
  };

  const showError = (msg: string) => {
    setError(msg);
    if (errorTimer.current) clearTimeout(errorTimer.current);
    errorTimer.current = setTimeout(() => setError(null), 5000);
  };

  const handleToggle = async (win: WindowInfo, enable: boolean) => {
    // Optimistic update
    setShieldedExes(prev => {
      const next = new Set(prev);
      if (enable) next.add(win.exe_name);
      else next.delete(win.exe_name);
      return next;
    });

    try {
      await invoke<boolean>("toggle_shield", {
        exeName: win.exe_name,
        hwnd: win.hwnd,
        enable,
      });
    } catch (e) {
      // Revert on error
      setShieldedExes(prev => {
        const next = new Set(prev);
        if (enable) next.delete(win.exe_name);
        else next.add(win.exe_name);
        return next;
      });
      showError(String(e));
    }
  };

  const handleMinimize = async () => {
    const appWindow = getCurrentWindow();
    await appWindow.hide();
  };

  const filtered = windows.filter(w =>
    w.exe_name.toLowerCase().includes(search.toLowerCase()) ||
    w.title.toLowerCase().includes(search.toLowerCase())
  );

  const shieldedCount = [...shieldedExes].filter(exe =>
    windows.some(w => w.exe_name === exe)
  ).length;

  return (
    <div className="app">
      {/* Header */}
      <header className="header">
        <div className="header-top">
          <div className="logo">🛡️</div>
          <span className="app-name">StreamShield</span>
          <span className="app-tagline">Stream Privacy Manager</span>
        </div>
        <div className="status-bar">
          {shieldedCount > 0 ? (
            <div className="status-badge active">
              <div className="status-dot pulse" />
              {shieldedCount} app{shieldedCount !== 1 ? "s" : ""} hidden from capture
            </div>
          ) : (
            <div className="status-badge inactive">
              <div className="status-dot" />
              No apps shielded
            </div>
          )}
        </div>
      </header>

      {/* Search */}
      <div className="search-wrap">
        <span className="search-icon">🔍</span>
        <input
          className="search-input"
          type="text"
          placeholder="Search applications..."
          value={search}
          onChange={e => setSearch(e.target.value)}
          id="search-apps"
        />
      </div>

      {/* List header */}
      <div className="list-header">
        <span>Running Applications ({filtered.length})</span>
        <button
          className={`refresh-btn${refreshing ? " spinning" : ""}`}
          onClick={handleRefresh}
          id="refresh-btn"
        >
          <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
            <path d="M8.5 5a3.5 3.5 0 1 1-1.026-2.474" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round"/>
            <path d="M7.5 1.5v2h-2" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round"/>
          </svg>
          Refresh
        </button>
      </div>

      {/* Window list */}
      <div className="window-list" id="window-list">
        {loading ? (
          Array.from({ length: 6 }).map((_, i) => (
            <div key={i} className="shimmer shimmer-row" />
          ))
        ) : filtered.length === 0 ? (
          <div className="empty-state">
            <div className="empty-icon">🪟</div>
            <div className="empty-title">No applications found</div>
            <div className="empty-sub">
              {search ? "Try a different search term" : "Open some apps and click Refresh"}
            </div>
          </div>
        ) : (
          filtered.map(win => (
            <AppRow
              key={win.hwnd}
              window={win}
              shielded={shieldedExes.has(win.exe_name)}
              onToggle={handleToggle}
            />
          ))
        )}
      </div>

      {/* Footer */}
      <footer className="footer">
        <div className="footer-count">
          <span>{shieldedCount}</span> of {windows.length} apps shielded
        </div>
        <button className="minimize-btn" onClick={handleMinimize} id="minimize-to-tray">
          Minimize to tray ↗
        </button>
      </footer>

      {/* Error toast */}
      {error && <div className="toast">{error}</div>}
    </div>
  );
}
