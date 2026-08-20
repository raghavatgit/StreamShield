import fs from "fs";
import path from "path";

const ta = "C:\\Users\\GOYAL\\Documents\\work\\StreamShield\\tauri-app";

const write = (rel, content) => {
  const full = path.join(ta, rel);
  fs.mkdirSync(path.dirname(full), { recursive: true });
  fs.writeFileSync(full, content, "utf8");
  console.log(`OK: ${rel} => ${JSON.stringify(content.slice(0,15))}`);
};

write("src/index.css", `@import url("https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&display=swap");
*, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
:root {
  --bg-base: #0a0c10; --bg-surface: #111318; --bg-card: #161a22; --bg-hover: #1c2030;
  --border: rgba(255,255,255,0.07); --border-active: rgba(99,179,237,0.35);
  --text-primary: #edf2f7; --text-secondary: #8892a4; --text-muted: #4a5568;
  --accent-blue: #63b3ed; --accent-green: #68d391; --accent-red: #fc8181;
  --shield-on: #48bb78; --shield-off: #4a5568;
  --glow-green: rgba(72,187,120,0.25); --radius-sm: 6px; --radius-md: 10px;
  --font: "Inter", system-ui, sans-serif;
}
html, body, #root { height: 100%; width: 100%; overflow: hidden; font-family: var(--font);
  background: var(--bg-base); color: var(--text-primary); -webkit-font-smoothing: antialiased; }
::-webkit-scrollbar { width: 4px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb { background: var(--border); border-radius: 2px; }
.app { display: flex; flex-direction: column; height: 100vh; }
.app::before { content: ""; position: fixed; top: -50%; left: -20%; width: 60%; height: 60%;
  background: radial-gradient(ellipse, rgba(99,179,237,0.06) 0%, transparent 70%);
  pointer-events: none; z-index: 0; }
.header { padding: 18px 20px 14px; border-bottom: 1px solid var(--border);
  background: rgba(17,19,24,0.9); backdrop-filter: blur(12px); position: relative; z-index: 10; }
.header-top { display: flex; align-items: center; gap: 10px; margin-bottom: 14px; }
.logo { width: 28px; height: 28px; background: linear-gradient(135deg,#48bb78,#63b3ed);
  border-radius: 8px; display: flex; align-items: center; justify-content: center;
  font-size: 14px; flex-shrink: 0; box-shadow: 0 0 16px rgba(72,187,120,0.3); }
.app-name { font-size: 15px; font-weight: 700; }
.app-tagline { font-size: 11px; color: var(--text-muted); margin-left: auto; }
.status-bar { display: flex; gap: 8px; align-items: center; }
.status-badge { display: flex; align-items: center; gap: 5px; padding: 4px 10px;
  border-radius: 20px; font-size: 11px; font-weight: 500; border: 1px solid; }
.status-badge.active { background: rgba(72,187,120,0.12); border-color: rgba(72,187,120,0.3); color: var(--accent-green); }
.status-badge.inactive { background: rgba(74,85,104,0.12); border-color: rgba(74,85,104,0.3); color: var(--text-muted); }
.status-dot { width: 6px; height: 6px; border-radius: 50%; background: currentColor; }
.status-dot.pulse { animation: pulse 2s infinite; }
@keyframes pulse { 0%,100% { opacity:1; } 50% { opacity:0.4; } }
.search-wrap { padding: 14px 16px 10px; position: relative; }
.search-icon { position: absolute; left: 28px; top: 50%; transform: translateY(-50%); color: var(--text-muted); font-size: 13px; pointer-events: none; }
.search-input { width: 100%; background: var(--bg-card); border: 1px solid var(--border);
  border-radius: var(--radius-md); padding: 9px 12px 9px 36px; font-size: 13px;
  font-family: var(--font); color: var(--text-primary); outline: none; transition: border-color 0.2s; }
.search-input::placeholder { color: var(--text-muted); }
.search-input:focus { border-color: var(--border-active); box-shadow: 0 0 0 3px rgba(99,179,237,0.08); }
.list-header { display: flex; justify-content: space-between; align-items: center;
  padding: 0 16px 8px; font-size: 10.5px; font-weight: 600; letter-spacing: 0.5px;
  text-transform: uppercase; color: var(--text-muted); }
.refresh-btn { background: none; border: 1px solid var(--border); border-radius: var(--radius-sm);
  padding: 3px 8px; font-size: 11px; color: var(--text-secondary); cursor: pointer; font-family: var(--font); transition: all 0.2s; }
.refresh-btn:hover { border-color: var(--border-active); color: var(--accent-blue); }
.window-list { flex: 1; overflow-y: auto; padding: 0 10px 10px; }
.app-row { display: flex; align-items: center; gap: 12px; padding: 10px; border-radius: var(--radius-md);
  border: 1px solid transparent; transition: all 0.18s; margin-bottom: 3px; position: relative; }
.app-row:hover { background: var(--bg-hover); border-color: var(--border); }
.app-row.shielded { background: rgba(72,187,120,0.05); border-color: rgba(72,187,120,0.15); }
.app-icon { width: 34px; height: 34px; border-radius: var(--radius-sm); background: var(--bg-surface);
  display: flex; align-items: center; justify-content: center; flex-shrink: 0; border: 1px solid var(--border); }
.app-icon-letter { font-size: 14px; font-weight: 700; text-transform: uppercase; }
.app-info { flex: 1; min-width: 0; }
.app-name-row { font-size: 13px; font-weight: 500; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.app-pid { font-size: 10.5px; color: var(--text-muted); margin-top: 1px; }
.toggle-wrap { display: flex; align-items: center; gap: 8px; flex-shrink: 0; }
.toggle-label { font-size: 10px; font-weight: 600; text-transform: uppercase; color: var(--text-muted); min-width: 32px; text-align: right; transition: color 0.2s; }
.toggle-label.on { color: var(--shield-on); }
.toggle { position: relative; width: 40px; height: 22px; cursor: pointer; }
.toggle input { opacity: 0; width: 0; height: 0; position: absolute; }
.toggle-slider { position: absolute; inset: 0; background: var(--shield-off); border-radius: 22px; transition: background 0.25s; }
.toggle-slider::before { content: ""; position: absolute; width: 16px; height: 16px; left: 2px; top: 2px;
  background: white; border-radius: 50%; transition: transform 0.25s cubic-bezier(0.34,1.56,0.64,1); }
.toggle input:checked + .toggle-slider { background: var(--shield-on); box-shadow: 0 0 10px var(--glow-green); }
.toggle input:checked + .toggle-slider::before { transform: translateX(18px); }
.shield-indicator { position: absolute; left: 0; top: 50%; transform: translateY(-50%);
  width: 2px; height: 60%; border-radius: 2px; background: var(--shield-on); opacity: 0; transition: opacity 0.3s; }
.app-row.shielded .shield-indicator { opacity: 1; }
.empty-state { display: flex; flex-direction: column; align-items: center; justify-content: center;
  gap: 10px; padding: 60px 20px; color: var(--text-muted); text-align: center; }
.empty-icon { font-size: 36px; opacity: 0.5; }
.empty-title { font-size: 14px; font-weight: 500; color: var(--text-secondary); }
.footer { padding: 10px 16px; border-top: 1px solid var(--border); display: flex;
  align-items: center; justify-content: space-between; }
.footer-count { font-size: 11px; color: var(--text-muted); }
.footer-count span { color: var(--accent-green); font-weight: 600; }
.minimize-btn { background: none; border: none; font-size: 11px; color: var(--text-muted);
  cursor: pointer; font-family: var(--font); padding: 3px 6px; border-radius: var(--radius-sm); transition: color 0.2s; }
.toast { position: fixed; bottom: 56px; left: 50%; transform: translateX(-50%);
  background: rgba(252,129,129,0.15); border: 1px solid rgba(252,129,129,0.35);
  color: var(--accent-red); padding: 8px 14px; border-radius: var(--radius-md);
  font-size: 11.5px; max-width: 90%; text-align: center; z-index: 100; animation: fadeInUp 0.3s; }
@keyframes fadeInUp { from { opacity:0; transform:translateX(-50%) translateY(10px); } to { opacity:1; transform:translateX(-50%) translateY(0); } }
.shimmer { background: linear-gradient(90deg, var(--bg-card) 25%, var(--bg-hover) 50%, var(--bg-card) 75%);
  background-size: 200% 100%; animation: shimmer 1.4s infinite; border-radius: var(--radius-md); }
@keyframes shimmer { from { background-position:200% 0; } to { background-position:-200% 0; } }
.shimmer-row { height: 54px; margin-bottom: 3px; }
`);

write("src/App.tsx", `import { useState, useEffect, useCallback, useRef } from "react";
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
        <button className={\`refresh-btn\${refreshing ? " spinning" : ""}\`} onClick={handleRefresh}>
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
`);

write("src/components/AppRow.tsx", `interface WindowInfo { hwnd: number; pid: number; title: string; exe_name: string; }
interface Props { window: WindowInfo; shielded: boolean; onToggle: (w: WindowInfo, e: boolean) => void; }

function exeColor(name: string) {
  const colors = ["#63b3ed","#68d391","#f6ad55","#fc8181","#b794f4","#76e4f7","#fbb6ce","#9ae6b4"];
  let hash = 0;
  for (const c of name) hash = (hash * 31 + c.charCodeAt(0)) & 0xffffffff;
  return colors[Math.abs(hash) % colors.length];
}

export default function AppRow({ window: win, shielded, onToggle }: Props) {
  const color = exeColor(win.exe_name);
  const initial = win.exe_name.replace(/\\.exe$/i,"")[0]?.toUpperCase() ?? "?";
  return (
    <div className={\`app-row\${shielded?" shielded":""}\`} id={\`app-row-\${win.pid}\`}>
      <div className="shield-indicator"/>
      <div className="app-icon">
        <span className="app-icon-letter" style={{color}}>{initial}</span>
      </div>
      <div className="app-info">
        <div className="app-name-row" title={win.title}>{win.exe_name.replace(/\\.exe$/i,"")}</div>
        <div className="app-pid">PID {win.pid} · {win.title.slice(0,35)}{win.title.length>35?"…":""}</div>
      </div>
      <div className="toggle-wrap">
        <span className={\`toggle-label\${shielded?" on":""}\`}>{shielded?"ON":"OFF"}</span>
        <label className="toggle" htmlFor={\`t-\${win.pid}\`}>
          <input id={\`t-\${win.pid}\`} type="checkbox" checked={shielded} onChange={e=>onToggle(win,e.target.checked)}/>
          <span className="toggle-slider"/>
        </label>
      </div>
    </div>
  );
}
`);

write("src/main.tsx", `import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./index.css";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode><App /></React.StrictMode>
);
`);

console.log("All files written successfully!");
