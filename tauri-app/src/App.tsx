import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import AppRow from "./components/AppRow";

interface WindowInfo {
  hwnd: number;
  pid: number;
  title: string;
  exe_name: string;
}

export type ThemeType = "cyberpunk" | "aurora" | "synthwave" | "oled";

interface ThemeOption {
  id: ThemeType;
  name: string;
  colors: string[];
}

const THEMES: ThemeOption[] = [
  { id: "cyberpunk", name: "Cyberpunk", colors: ["#00f0ff", "#ff007f"] },
  { id: "aurora", name: "Aurora", colors: ["#10b981", "#38bdf8"] },
  { id: "synthwave", name: "Synthwave", colors: ["#ff5e62", "#b5179e"] },
  { id: "oled", name: "OLED Stealth", colors: ["#3b82f6", "#8b5cf6"] },
];

export default function App() {
  const [windows, setWindows] = useState<WindowInfo[]>([]);
  const [shieldedExes, setShieldedExes] = useState<Set<string>>(new Set());
  const [search, setSearch] = useState("");
  const [filterMode, setFilterMode] = useState<"all" | "shielded" | "unshielded">("all");
  const [theme, setTheme] = useState<ThemeType>(() => {
    return (localStorage.getItem("streamshield_theme") as ThemeType) || "cyberpunk";
  });
  const [themeMenuOpen, setThemeMenuOpen] = useState(false);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [isAdmin, setIsAdmin] = useState(true);
  const [toast, setToast] = useState<{ message: string; type?: "error" | "info" | "success" } | null>(null);
  const toastTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Apply theme to document root
  useEffect(() => {
    document.documentElement.setAttribute("data-theme", theme);
    localStorage.setItem("streamshield_theme", theme);
  }, [theme]);

  const showToast = (message: string, type: "error" | "info" | "success" = "error") => {
    setToast({ message, type });
    if (toastTimer.current) clearTimeout(toastTimer.current);
    toastTimer.current = setTimeout(() => setToast(null), 3500);
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
    } catch (e) {
      console.error(e);
      showToast(String(e).replace("Error: ", ""), "error");
    } finally {
      setLoading(false);
      setRefreshing(false);
    }
  }, []);

  useEffect(() => {
    loadWindows();
  }, [loadWindows]);

  const handleRefresh = async () => {
    setRefreshing(true);
    await loadWindows();
    showToast("Application list refreshed", "info");
  };

  const handleToggle = async (win: WindowInfo, enable: boolean) => {
    // Optimistic update
    setShieldedExes((prev) => {
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
      showToast(
        enable ? `Shielded ${win.exe_name}` : `Unshielded ${win.exe_name}`,
        enable ? "success" : "info"
      );
    } catch (e) {
      // Revert on failure
      setShieldedExes((prev) => {
        const next = new Set(prev);
        if (enable) next.delete(win.exe_name);
        else next.add(win.exe_name);
        return next;
      });
      showToast(String(e).replace("Error: ", ""), "error");
    }
  };

  const handleShieldAll = async () => {
    const toShield = windows.filter((w) => !shieldedExes.has(w.exe_name));
    if (toShield.length === 0) return;

    for (const win of toShield) {
      handleToggle(win, true);
    }
  };

  const handleUnshieldAll = async () => {
    const toUnshield = windows.filter((w) => shieldedExes.has(w.exe_name));
    if (toUnshield.length === 0) return;

    for (const win of toUnshield) {
      handleToggle(win, false);
    }
  };

  const filtered = windows.filter((w) => {
    const matchesSearch =
      w.exe_name.toLowerCase().includes(search.toLowerCase()) ||
      w.title.toLowerCase().includes(search.toLowerCase());

    const isShielded = shieldedExes.has(w.exe_name);
    if (filterMode === "shielded") return matchesSearch && isShielded;
    if (filterMode === "unshielded") return matchesSearch && !isShielded;
    return matchesSearch;
  });

  const shieldedCount = windows.filter((w) => shieldedExes.has(w.exe_name)).length;
  const percentage = windows.length > 0 ? Math.round((shieldedCount / windows.length) * 100) : 0;

  return (
    <div className="app-container" onClick={() => themeMenuOpen && setThemeMenuOpen(false)}>
      {/* Background ambient glow effect */}
      <div className="ambient-spotlight" />

      {/* Header */}
      <header className="main-header">
        <div className="header-brand-row">
          <div className="brand-badge">
            <img src="/logo.png" className="app-brand-logo" alt="StreamShield Logo" />
            <div className="brand-text">
              <span className="brand-title">StreamShield</span>
              <span className="brand-version">v0.1.0 • PRIVACY</span>
            </div>
          </div>

          <div className="header-actions">
            {/* Theme Selector Popover */}
            <div className="theme-selector-wrap" onClick={(e) => e.stopPropagation()}>
              <button
                className="theme-button"
                onClick={() => setThemeMenuOpen(!themeMenuOpen)}
                title="Change Theme Preset"
              >
                <span className="theme-icon">🎨</span>
                <span className="theme-name">{THEMES.find((t) => t.id === theme)?.name}</span>
                <span className="theme-arrow">▾</span>
              </button>

              {themeMenuOpen && (
                <div className="theme-dropdown">
                  <div className="theme-dropdown-header">Select Theme Preset</div>
                  {THEMES.map((t) => (
                    <button
                      key={t.id}
                      className={`theme-dropdown-item ${theme === t.id ? "selected" : ""}`}
                      onClick={() => {
                        setTheme(t.id);
                        setThemeMenuOpen(false);
                      }}
                    >
                      <div className="theme-swatch">
                        <span style={{ background: t.colors[0] }} />
                        <span style={{ background: t.colors[1] }} />
                      </div>
                      <span className="theme-item-label">{t.name}</span>
                      {theme === t.id && <span className="theme-check">✓</span>}
                    </button>
                  ))}
                </div>
              )}
            </div>
          </div>
        </div>

        {!isAdmin && (
          <div className="admin-alert">
            <span className="alert-icon">⚠️</span>
            <span>Administrator privileges recommended for full app shielding</span>
          </div>
        )}

        {/* Status Badge & Metrics */}
        <div className="status-overview">
          <div className={`status-pill ${shieldedCount > 0 ? "active" : "idle"}`}>
            <span className="status-orb" />
            <span className="status-text">
              {shieldedCount > 0
                ? `${shieldedCount} App${shieldedCount !== 1 ? "s" : ""} Hidden from Capture`
                : "No Apps Shielded"}
            </span>
          </div>

          <div className="quick-actions">
            <button
              className="action-btn"
              onClick={handleShieldAll}
              disabled={windows.length === 0}
              title="Shield all visible apps"
            >
              Shield All
            </button>
            <button
              className="action-btn"
              onClick={handleUnshieldAll}
              disabled={shieldedCount === 0}
              title="Unshield all apps"
            >
              Clear
            </button>
            <button
              className={`refresh-action-btn ${refreshing ? "spinning" : ""}`}
              onClick={handleRefresh}
              title="Refresh window list"
            >
              ↻
            </button>
          </div>
        </div>
      </header>

      {/* Search and Filter Controls */}
      <div className="control-bar">
        <div className="search-box">
          <span className="search-glyph">🔍</span>
          <input
            className="search-field"
            type="text"
            placeholder="Search processes or window titles..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
          {search && (
            <button className="search-clear-btn" onClick={() => setSearch("")}>
              ✕
            </button>
          )}
        </div>

        <div className="filter-tabs">
          <button
            className={`filter-tab ${filterMode === "all" ? "active" : ""}`}
            onClick={() => setFilterMode("all")}
          >
            All ({windows.length})
          </button>
          <button
            className={`filter-tab ${filterMode === "shielded" ? "active" : ""}`}
            onClick={() => setFilterMode("shielded")}
          >
            Shielded ({shieldedCount})
          </button>
          <button
            className={`filter-tab ${filterMode === "unshielded" ? "active" : ""}`}
            onClick={() => setFilterMode("unshielded")}
          >
            Open ({windows.length - shieldedCount})
          </button>
        </div>
      </div>

      {/* Main Window List */}
      <main className="window-list-scroll">
        {loading ? (
          <div className="loading-container">
            {Array.from({ length: 5 }).map((_, i) => (
              <div key={i} className="skeleton-card" />
            ))}
          </div>
        ) : filtered.length === 0 ? (
          <div className="empty-container">
            <div className="empty-artwork">🛡️</div>
            <div className="empty-heading">No Applications Found</div>
            <div className="empty-desc">
              {search
                ? "No matching windows found. Try another search keyword."
                : "No active desktop windows detected. Open an app and click Refresh."}
            </div>
          </div>
        ) : (
          <div className="cards-wrapper">
            {filtered.map((win) => (
              <AppRow
                key={`${win.pid}-${win.hwnd}`}
                window={win}
                shielded={shieldedExes.has(win.exe_name)}
                onToggle={handleToggle}
              />
            ))}
          </div>
        )}
      </main>

      {/* Footer */}
      <footer className="main-footer">
        <div className="footer-stats">
          <div className="progress-bar-bg">
            <div className="progress-bar-fill" style={{ width: `${percentage}%` }} />
          </div>
          <span className="stats-label">
            <strong>{shieldedCount}</strong> of {windows.length} protected ({percentage}%)
          </span>
        </div>

        <button
          className="tray-minimize-button"
          onClick={() => getCurrentWindow().hide()}
          title="Minimize StreamShield to system tray"
        >
          <span>Tray</span>
          <span className="btn-glyph">↗</span>
        </button>
      </footer>

      {/* Toast Notification */}
      {toast && (
        <div className={`toast-notification ${toast.type || "error"}`}>
          <span className="toast-icon">
            {toast.type === "success" ? "✓" : toast.type === "info" ? "ℹ" : "⚠️"}
          </span>
          <span className="toast-text">{toast.message}</span>
        </div>
      )}
    </div>
  );
}
