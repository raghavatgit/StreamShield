import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import AppRow, { WindowInfo } from "./components/AppRow";

export type ThemeType = "discord" | "cyberpunk" | "neumorphic-white";

interface ThemeOption {
  id: ThemeType;
  name: string;
  colors: string[];
}

const THEMES: ThemeOption[] = [
  { id: "discord", name: "Discord Dark", colors: ["#5865f2", "#23a55a"] },
  { id: "cyberpunk", name: "Cyberpunk", colors: ["#00f0ff", "#ff007f"] },
  { id: "neumorphic-white", name: "Clean White", colors: ["#ffffff", "#4f46e5"] },
];

export default function App() {
  const [windows, setWindows] = useState<WindowInfo[]>([]);
  const [shieldedExes, setShieldedExes] = useState<Set<string>>(new Set());
  const [isSelfShielded, setIsSelfShielded] = useState(false);
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
      const [wins, admin, selfShielded] = await Promise.all([
        invoke<WindowInfo[]>("get_windows"),
        invoke<boolean>("check_admin"),
        invoke<boolean>("is_self_shielded").catch(() => false),
      ]);

      setWindows(wins);
      setIsAdmin(admin);
      setIsSelfShielded(selfShielded);

      // Populate active shield state directly from Windows OS query
      const activeShielded = new Set<string>();
      for (const w of wins) {
        if (w.is_shielded) {
          activeShielded.add(w.exe_name);
        }
      }
      setShieldedExes(activeShielded);
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
      setRefreshing(false);
    }
  }, []);

  useEffect(() => {
    loadWindows();
    const interval = setInterval(() => {
      loadWindows();
    }, 2500);

    const onFocus = () => loadWindows();
    const onVisibilityChange = () => {
      if (document.visibilityState === "visible") {
        loadWindows();
      }
    };

    window.addEventListener("focus", onFocus);
    document.addEventListener("visibilitychange", onVisibilityChange);

    const unlistenPromise = listen("app_wake", () => {
      loadWindows();
    });

    return () => {
      clearInterval(interval);
      window.removeEventListener("focus", onFocus);
      document.removeEventListener("visibilitychange", onVisibilityChange);
      unlistenPromise.then((unlisten) => unlisten());
    };
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
        pid: win.pid,
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

  const handleToggleSelfShield = async () => {
    const next = !isSelfShielded;
    try {
      await invoke<boolean>("toggle_self_shield", { enable: next });
      setIsSelfShielded(next);
      showToast(
        next
          ? "StreamShield app is now hidden from streams"
          : "StreamShield app is now visible on streams",
        next ? "success" : "info"
      );
    } catch (e) {
      showToast(String(e), "error");
    }
  };

  const handleHideToTray = async () => {
    try {
      await invoke("hide_to_tray");
    } catch {
      try {
        getCurrentWindow().hide();
      } catch (err) {
        console.error("Hide error:", err);
      }
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
    <div
      className="streamshield-root"
      onClick={() => themeMenuOpen && setThemeMenuOpen(false)}
    >
      {/* Top Header */}
      <header className="app-top-header">
        <div className="header-main-row">
          <div className="header-logo-group">
            <img src="/logo.png" className="app-header-logo" alt="StreamShield Logo" />
            <div className="header-text-block">
              <div className="header-app-title">StreamShield</div>
              <div className="header-app-subtitle">Stream Privacy Manager</div>
            </div>
          </div>

          <div className="header-action-group">
            {/* Theme Selector Popover */}
            <div className="theme-picker-container" onClick={(e) => e.stopPropagation()}>
              <button
                className="theme-picker-trigger"
                onClick={() => setThemeMenuOpen(!themeMenuOpen)}
                title="Select Theme Preset"
              >
                <span className="picker-dot" />
                <span className="picker-text">{THEMES.find((t) => t.id === theme)?.name}</span>
                <span className="picker-caret">▾</span>
              </button>

              {themeMenuOpen && (
                <div className="theme-popover-menu">
                  <div className="theme-popover-title">Themes</div>
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
            <span>Administrator permissions recommended for full process capture exclusion</span>
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
              title="Shield all running applications"
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
              title="Refresh window list"
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
            placeholder="Search processes or windows..."
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
                ? "No matching windows found. Try another search query."
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

        <div className="footer-right-actions">
          {/* Stream Mode for StreamShield itself */}
          <button
            className={`self-stealth-toggle-btn ${isSelfShielded ? "is-active" : ""}`}
            onClick={handleToggleSelfShield}
            title={
              isSelfShielded
                ? "StreamShield is hidden from capture (Click to make visible)"
                : "Hide StreamShield window itself from screen capture/recording"
            }
          >
            <span className="stealth-indicator-dot" />
            <span className="stealth-label">App Stealth: {isSelfShielded ? "ON" : "OFF"}</span>
          </button>

          {/* Tray Minimize Button */}
          <button
            className="tray-dock-btn"
            onClick={handleHideToTray}
            title="Minimize StreamShield to system tray"
          >
            <span>Tray</span>
            <span className="dock-arrow">↗</span>
          </button>
        </div>
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
