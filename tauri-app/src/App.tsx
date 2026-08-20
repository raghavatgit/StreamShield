import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import AppRow, { WindowInfo } from "./components/AppRow";

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
      const [wins, savedShielded, admin] = await Promise.all([
        invoke<WindowInfo[]>("get_windows"),
        invoke<string[]>("get_shielded_exes"),
        invoke<boolean>("check_admin"),
      ]);

      setWindows(wins);
      setIsAdmin(admin);

      // GROUND TRUTH: Populate active shield state directly from Windows OS query
      const activeShielded = new Set<string>();
      for (const w of wins) {
        if (w.is_shielded || savedShielded.includes(w.exe_name)) {
          activeShielded.add(w.exe_name);
        }
      }
      setShieldedExes(activeShielded);
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
    <div className="streamshield-root" onClick={() => themeMenuOpen && setThemeMenuOpen(false)}>
      {/* Background ambient lighting */}
      <div className="ambient-backdrop" />

      {/* Top Header */}
      <header className="app-top-header">
        <div className="header-main-row">
          <div className="header-logo-group">
            <img src="/logo.png" className="app-header-logo" alt="StreamShield Logo" />
            <div className="header-text-block">
              <div className="header-app-title">StreamShield</div>
              <div className="header-app-subtitle">Active Stream Privacy</div>
            </div>
          </div>

          <div className="header-action-group">
            {/* Theme Selector Popover */}
            <div className="theme-picker-container" onClick={(e) => e.stopPropagation()}>
              <button
                className="theme-picker-trigger"
                onClick={() => setThemeMenuOpen(!themeMenuOpen)}
                title="Change Design Preset"
              >
                <span className="picker-icon">🎨</span>
                <span className="picker-text">{THEMES.find((t) => t.id === theme)?.name}</span>
                <span className="picker-caret">▾</span>
              </button>

              {themeMenuOpen && (
                <div className="theme-popover-menu">
                  <div className="theme-popover-title">Visual Themes</div>
                  {THEMES.map((t) => (
                    <button
                      key={t.id}
                      className={`theme-menu-choice ${theme === t.id ? "is-selected" : ""}`}
                      onClick={() => {
                        setTheme(t.id);
                        setThemeMenuOpen(false);
                      }}
                    >
                      <div className="theme-color-dots">
                        <span style={{ background: t.colors[0] }} />
                        <span style={{ background: t.colors[1] }} />
                      </div>
                      <span className="theme-choice-name">{t.name}</span>
                      {theme === t.id && <span className="theme-selected-mark">✓</span>}
                    </button>
                  ))}
                </div>
              )}
            </div>
          </div>
        </div>

        {!isAdmin && (
          <div className="admin-status-banner">
            <span className="banner-icon">⚠️</span>
            <span>Run as Administrator to enable full multi-process shielding</span>
          </div>
        )}

        {/* Status bar & Batch controls */}
        <div className="header-status-strip">
          <div className={`active-counter-badge ${shieldedCount > 0 ? "is-active" : "is-idle"}`}>
            <span className="pulse-dot" />
            <span className="counter-text">
              {shieldedCount > 0
                ? `${shieldedCount} Application${shieldedCount !== 1 ? "s" : ""} Protected`
                : "No Applications Shielded"}
            </span>
          </div>

          <div className="batch-controls">
            <button
              className="batch-btn"
              onClick={handleShieldAll}
              disabled={windows.length === 0}
              title="Shield all visible applications"
            >
              Shield All
            </button>
            <button
              className="batch-btn"
              onClick={handleUnshieldAll}
              disabled={shieldedCount === 0}
              title="Unshield all applications"
            >
              Clear
            </button>
            <button
              className={`refresh-icon-btn ${refreshing ? "is-spinning" : ""}`}
              onClick={handleRefresh}
              title="Refresh running application list"
            >
              ↻
            </button>
          </div>
        </div>
      </header>

      {/* Search & Navigation Bar */}
      <section className="search-filter-section">
        <div className="search-input-shell">
          <span className="search-input-icon">🔍</span>
          <input
            className="search-text-input"
            type="text"
            placeholder="Search active applications or window titles..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
          {search && (
            <button className="search-clear-cross" onClick={() => setSearch("")}>
              ✕
            </button>
          )}
        </div>

        <nav className="tab-segmented-control">
          <button
            className={`tab-segment-btn ${filterMode === "all" ? "is-current" : ""}`}
            onClick={() => setFilterMode("all")}
          >
            All <span className="tab-count">{windows.length}</span>
          </button>
          <button
            className={`tab-segment-btn ${filterMode === "shielded" ? "is-current" : ""}`}
            onClick={() => setFilterMode("shielded")}
          >
            Shielded <span className="tab-count">{shieldedCount}</span>
          </button>
          <button
            className={`tab-segment-btn ${filterMode === "unshielded" ? "is-current" : ""}`}
            onClick={() => setFilterMode("unshielded")}
          >
            Open <span className="tab-count">{windows.length - shieldedCount}</span>
          </button>
        </nav>
      </section>

      {/* Main List Body */}
      <main className="app-card-viewport">
        {loading ? (
          <div className="card-skeleton-stack">
            {Array.from({ length: 6 }).map((_, i) => (
              <div key={i} className="card-skeleton" />
            ))}
          </div>
        ) : filtered.length === 0 ? (
          <div className="empty-results-view">
            <div className="empty-glyph">🛡️</div>
            <div className="empty-title-text">No Applications Found</div>
            <div className="empty-subtitle-text">
              {search
                ? "No matching windows found. Try another search keyword."
                : "No active application windows found. Open an application and click Refresh."}
            </div>
          </div>
        ) : (
          <div className="cards-layout-stack">
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

      {/* Bottom Footer */}
      <footer className="app-bottom-footer">
        <div className="footer-coverage-indicator">
          <div className="coverage-bar-track">
            <div className="coverage-bar-value" style={{ width: `${percentage}%` }} />
          </div>
          <span className="coverage-text">
            <strong>{shieldedCount}</strong> / {windows.length} Protected ({percentage}%)
          </span>
        </div>

        <button
          className="tray-dock-btn"
          onClick={() => getCurrentWindow().hide()}
          title="Minimize StreamShield to system tray"
        >
          <span>Tray</span>
          <span className="dock-arrow">↗</span>
        </button>
      </footer>

      {/* Floating Toast Notification */}
      {toast && (
        <div className={`floating-toast ${toast.type || "error"}`}>
          <span className="toast-symbol">
            {toast.type === "success" ? "✓" : toast.type === "info" ? "ℹ" : "⚠️"}
          </span>
          <span className="toast-message-body">{toast.message}</span>
        </div>
      )}
    </div>
  );
}
