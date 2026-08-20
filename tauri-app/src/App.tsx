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
    } catch (e) { console.error(e); }
    finally { setLoading(false); setRefreshing(false); }
  }, []);

  useEffect(() => { loadWindows(); }, [loadWindows]);

  const handleRefresh = async () => { setRefreshing(true); await loadWindows(); };

  const showError = (msg: string) => {
    setError(msg);
    if (errorTimer.current) clearTimeout(errorTimer.current);
    errorTimer.current = setTimeout(() => setError(null), 5000);
  };

  const handleToggle = async (win: WindowInfo, enable: boolean) => {
    setShieldedExes(prev => { const n = new Set(prev); enable ? n.add(win.exe_name) : n.delete(win.exe_name); return n; });
    try {
      await invoke<boolean>("toggle_shield", { exeName: win.exe_name, hwnd: win.hwnd, enable });
    } catch (e) {
      setShieldedExes(prev => { const n = new Set(prev); enable ? n.delete(win.exe_name) : n.add(win.exe_name); return n; });
      showError(String(e));
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
          <div className="logo">???</div>
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
              <div className="status-dot" /> No apps shielded
            </div>
          )}
        </div>
      </header>
      <div className="search-wrap">
        <span className="search-icon">??</span>
        <input className="search-input" type="text" placeholder="Search applications..."
          value={search} onChange={e => setSearch(e.target.value)} />
      </div>
      <div className="list-header">
        <span>Running Applications ({filtered.length})</span>
        <button className={`refresh-btn${refreshing ? " spinning" : ""}`} onClick={handleRefresh}>
          ? Refresh
        </button>
      </div>
      <div className="window-list">
        {loading ? Array.from({length:6}).map((_,i) => <div key={i} className="shimmer shimmer-row"/>) :
         filtered.length === 0 ? (
          <div className="empty-state">
            <div className="empty-icon">??</div>
            <div className="empty-title">No applications found</div>
            <div>{search ? "Try a different search" : "Open apps and click Refresh"}</div>
          </div>
        ) : filtered.map(win => (
          <AppRow key={win.hwnd} window={win} shielded={shieldedExes.has(win.exe_name)} onToggle={handleToggle}/>
        ))}
      </div>
      <footer className="footer">
        <div className="footer-count"><span>{shieldedCount}</span> of {windows.length} shielded</div>
        <button className="minimize-btn" onClick={() => getCurrentWindow().hide()}>Minimize to tray ?</button>
      </footer>
      {error && <div className="toast">{error}</div>}
    </div>
  );
}
