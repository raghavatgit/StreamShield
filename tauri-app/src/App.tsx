import { useState, useEffect, useCallback, useRef, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import AppRow, { WindowInfo } from "./components/AppRow";
import SettingsModal, { AppSettings, OsInfo } from "./components/SettingsModal";
import { checkForAppUpdate, CURRENT_VERSION, UpdateInfo } from "./utils/updater";
import { formatBackendError } from "./utils/win32ErrorCodes";
import {
  IconGear,
  IconRefresh,
  IconSearch,
  IconClose,
  IconCheck,
  IconInfo,
  IconAlert,
  IconShield,
  IconMinimize,
  IconMaximize,
  IconRestore,
  IconEye,
  IconEyeOff,
  IconCompact,
  IconExpand,
  IconWidget,
} from "./components/Icons";
import { ThemeType, THEMES, applyDynamicTheme } from "./utils/themeEngine";

export type { ThemeType };

export default function App() {
  const [windows, setWindows] = useState<WindowInfo[]>([]);
  const [shieldedExes, setShieldedExes] = useState<Set<string>>(new Set());
  const [isSelfShielded, setIsSelfShielded] = useState(false);
  const [search, setSearch] = useState("");
  const [filterMode, setFilterMode] = useState<"all" | "shielded" | "unshielded">("all");

  const [settings, setSettings] = useState<AppSettings>({
    autostart: false,
    start_minimized: false,
    auto_reapply: true,
    shield_mode: "exclude",
    poll_interval_ms: 3000,
    theme: (localStorage.getItem("streamshield_theme") as ThemeType) || "violet",
    compact_mode: false,
    show_pid: true,
    confirm_batch: false,
    mpo_fix: false,
    self_stealth: false,
    widget_enabled: false,
    widget_style: "pill",
    widget_always_on_top: true,
    widget_stealth: true,
  });

  const [settingsOpen, setSettingsOpen] = useState(false);
  const [batchConfirmState, setBatchConfirmState] = useState<"shield_all" | "clear_all" | null>(null);

  const [theme, setTheme] = useState<ThemeType>(() => {
    return (localStorage.getItem("streamshield_theme") as ThemeType) || "violet";
  });
  const [themeMenuOpen, setThemeMenuOpen] = useState(false);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [isAdmin, setIsAdmin] = useState(true);
  const [osInfo, setOsInfo] = useState<OsInfo | null>(null);
  const [toast, setToast] = useState<{ id: number; message: string; type?: "error" | "info" | "success" } | null>(null);
  const toastTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const toastIdRef = useRef<number>(0);
  const lastLoadTime = useRef<number>(0);

  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [isUpdating, setIsUpdating] = useState(false);
  const [checkingUpdate, setCheckingUpdate] = useState(false);
  const [isMaximized, setIsMaximized] = useState(false);

  const showToast = useCallback((message: string, type: "error" | "info" | "success" = "error") => {
    const id = ++toastIdRef.current;
    setToast({ id, message, type });
    if (toastTimer.current) clearTimeout(toastTimer.current);
    toastTimer.current = setTimeout(() => setToast(null), 3500);
  }, []);

  useEffect(() => {
    checkForAppUpdate()
      .then((info) => {
        if (info.hasUpdate) {
          setUpdateInfo(info);
        }
      })
      .catch(() => {});
  }, []);

  const handleApplyUpdate = async () => {
    if (!updateInfo?.setupDownloadUrl) {
      if (updateInfo?.releaseUrl) {
        await invoke("open_external_url", { url: updateInfo.releaseUrl }).catch(() => {});
      }
      return;
    }
    setIsUpdating(true);
    showToast("Downloading and applying update in background...", "info");
    try {
      await invoke("install_update_silent", { downloadUrl: updateInfo.setupDownloadUrl });
      showToast("Update installer started! App will update shortly.", "success");
    } catch {
      showToast("Opening release download page...", "info");
      if (updateInfo.releaseUrl) {
        await invoke("open_external_url", { url: updateInfo.releaseUrl }).catch(() => {});
      }
    } finally {
      setIsUpdating(false);
    }
  };

  const handleManualCheckUpdates = async () => {
    setCheckingUpdate(true);
    try {
      const info = await checkForAppUpdate();
      if (info.hasUpdate) {
        setUpdateInfo(info);
        showToast(`Update v${info.latestVersion} available!`, "success");
      } else {
        setUpdateInfo(null);
        showToast(`StreamShield is up to date (v${CURRENT_VERSION})`, "info");
      }
    } catch {
      showToast("Could not connect to update server", "error");
    } finally {
      setCheckingUpdate(false);
    }
  };

  // Apply dynamic theme and peripherals to document root
  useEffect(() => {
    applyDynamicTheme(theme);
    localStorage.setItem("streamshield_theme", theme);
  }, [theme]);

  const loadSettings = useCallback(async () => {
    try {
      const s = await invoke<AppSettings>("get_settings");
      setSettings(s);
      if (s.theme) {
        setTheme(s.theme as ThemeType);
      }
      if (s.self_stealth !== undefined) {
        setIsSelfShielded(s.self_stealth);
      }
    } catch (e) {
      console.error("Failed to load settings:", e);
    }
  }, []);

  const loadWindows = useCallback(async (force?: boolean) => {
    const now = Date.now();
    if (!force && now - lastLoadTime.current < 500) return;
    lastLoadTime.current = now;

    try {
      const wins = await invoke<WindowInfo[]>("get_windows");
      setWindows(wins);

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

  // One-time startup queries + permanent Tauri event listeners
  useEffect(() => {
    loadSettings();
    loadWindows(true);

    invoke<boolean>("check_admin").then(setIsAdmin).catch(() => {});
    invoke<OsInfo>("get_os_info")
      .then((info) => {
        setOsInfo(info);
        document.documentElement.setAttribute("data-os", info.is_win11 ? "win11" : "win10");
      })
      .catch(() => {});
    invoke<boolean>("is_self_shielded").then(setIsSelfShielded).catch(() => {});

    const curWin = getCurrentWindow();
    curWin.isMaximized().then(setIsMaximized).catch(() => {});
    const unlistenResize = curWin.onResized(() => {
      curWin.isMaximized().then(setIsMaximized).catch(() => {});
    });

    const onFocus = () => loadWindows();
    const onVisibilityChange = () => {
      if (document.visibilityState === "visible") {
        loadWindows();
      }
    };
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        setThemeMenuOpen(false);
        setBatchConfirmState(null);
      }
    };

    window.addEventListener("focus", onFocus);
    document.addEventListener("visibilitychange", onVisibilityChange);
    window.addEventListener("keydown", onKeyDown);

    const unlistenWake = listen("app_wake", () => {
      loadWindows(true);
    });

    const unlistenTray = listen("tray_action_completed", (event) => {
      loadWindows(true);
      loadSettings();
      if (event.payload === "shield_all") {
        showToast("All applications shielded from tray", "success");
      } else if (event.payload === "unshield_all") {
        showToast("All applications unshielded from tray", "info");
      } else if (event.payload === "mode_changed") {
        showToast("Shield mode updated from tray", "info");
      } else if (event.payload === "app_toggled") {
        showToast("Application privacy toggled from tray", "info");
      } else if (event.payload === "compact_toggled") {
        showToast("Compact Mode toggled from tray", "info");
      }
    });

    const unlistenCompact = listen<boolean>("compact_mode_changed", (event) => {
      setSettings((prev) => ({ ...prev, compact_mode: event.payload }));
      showToast(event.payload ? "Compact Mode active" : "Full View restored", "info");
    });

    const unlistenWidget = listen<boolean>("widget_state_changed", (event) => {
      setSettings((prev) => ({ ...prev, widget_enabled: event.payload }));
    });

    const unlistenShield = listen("shield_state_changed", () => {
      loadWindows(true);
    });

    const unlistenSelfShield = listen<boolean>("self_shield_changed", (event) => {
      setIsSelfShielded(event.payload);
    });

    const unlistenSettings = listen<AppSettings>("settings_changed", (event) => {
      const s = event.payload;
      setSettings(s);
      if (s.theme) setTheme(s.theme as ThemeType);
      if (s.self_stealth !== undefined) setIsSelfShielded(s.self_stealth);
    });

    const unlistenOpenSettings = listen("open_settings", () => {
      setSettingsOpen(true);
    });

    return () => {
      if (toastTimer.current) clearTimeout(toastTimer.current);
      window.removeEventListener("focus", onFocus);
      document.removeEventListener("visibilitychange", onVisibilityChange);
      window.removeEventListener("keydown", onKeyDown);
      unlistenResize.then((unlisten) => unlisten());
      unlistenWake.then((unlisten) => unlisten());
      unlistenTray.then((unlisten) => unlisten());
      unlistenCompact.then((unlisten) => unlisten());
      unlistenWidget.then((unlisten) => unlisten());
      unlistenShield.then((unlisten) => unlisten());
      unlistenSelfShield.then((unlisten) => unlisten());
      unlistenSettings.then((unlisten) => unlisten());
      unlistenOpenSettings.then((unlisten) => unlisten());
    };
  }, [loadWindows, loadSettings, showToast]);

  // Dedicated polling interval effect (avoids tearing down listeners when interval changes)
  useEffect(() => {
    if (settings.poll_interval_ms <= 0) return;
    const interval = setInterval(() => {
      if (document.visibilityState === "visible") {
        loadWindows();
      }
    }, settings.poll_interval_ms);
    return () => clearInterval(interval);
  }, [settings.poll_interval_ms, loadWindows]);

  const handleToggleWidget = async () => {
    const next = !settings.widget_enabled;
    try {
      const result = await invoke<boolean>("toggle_widget", { enable: next });
      setSettings((prev) => ({ ...prev, widget_enabled: result }));
      showToast(result ? "Desktop Widget enabled" : "Desktop Widget hidden", "info");
    } catch (e) {
      showToast(formatBackendError(e), "error");
    }
  };

  const handleStartDrag = (e: React.MouseEvent) => {
    if (
      e.button === 0 &&
      e.detail === 1 &&
      !(e.target as HTMLElement).closest("button, input, select, a, .theme-picker-container, [data-no-drag]")
    ) {
      invoke("start_window_drag", { windowLabel: "main" }).catch(() => {
        getCurrentWindow().startDragging().catch(() => {});
      });
    }
  };

  const handleToggleCompact = async (enable: boolean) => {
    try {
      await invoke("set_compact_mode", { compact: enable });
      setSettings((prev) => ({ ...prev, compact_mode: enable }));
    } catch (e) {
      showToast(formatBackendError(e), "error");
    }
  };

  const handleUpdateSettings = async (newSettings: AppSettings) => {
    try {
      const saved = await invoke<AppSettings>("update_settings", { settings: newSettings });
      setSettings(saved);
      if (saved.theme) {
        setTheme(saved.theme as ThemeType);
      }
      showToast("Settings saved", "success");
    } catch (e) {
      showToast(formatBackendError(e), "error");
    }
  };

  const handleResetSettings = async () => {
    try {
      const def = await invoke<AppSettings>("reset_settings");
      setSettings(def);
      if (def.theme) {
        setTheme(def.theme as ThemeType);
      }
      showToast("Settings reset to defaults", "info");
    } catch (e) {
      showToast(formatBackendError(e), "error");
    }
  };

  const handleClearAllShields = async () => {
    try {
      await invoke("clear_all_shields");
      setShieldedExes(new Set());
      setWindows((prev) => prev.map((w) => ({ ...w, is_shielded: false })));
      showToast("All applications unshielded", "info");
      await loadWindows(true);
    } catch (e) {
      showToast(formatBackendError(e), "error");
    }
  };

  const handleRefresh = async () => {
    setRefreshing(true);
    await loadWindows(true);
    showToast("Application list refreshed", "info");
  };

  const handleToggle = useCallback(
    async (win: WindowInfo, enable: boolean) => {
      setShieldedExes((prev) => {
        const next = new Set(prev);
        if (enable) next.add(win.exe_name);
        else next.delete(win.exe_name);
        return next;
      });

      setWindows((prev) =>
        prev.map((w) =>
          w.exe_name.toLowerCase() === win.exe_name.toLowerCase() ? { ...w, is_shielded: enable } : w
        )
      );

      try {
        await invoke<boolean>("toggle_shield", {
          exeName: win.exe_name,
          hwnd: win.hwnd,
          pid: win.pid,
          enable,
        });
        showToast(enable ? `Shielded ${win.exe_name}` : `Unshielded ${win.exe_name}`, enable ? "success" : "info");
      } catch (e) {
        setShieldedExes((prev) => {
          const next = new Set(prev);
          if (enable) next.delete(win.exe_name);
          else next.add(win.exe_name);
          return next;
        });
        setWindows((prev) =>
          prev.map((w) =>
            w.exe_name.toLowerCase() === win.exe_name.toLowerCase() ? { ...w, is_shielded: !enable } : w
          )
        );
        showToast(formatBackendError(e), "error");
      }
    },
    [showToast]
  );

  const handleToggleSelfShield = async () => {
    const next = !isSelfShielded;
    try {
      await invoke<boolean>("toggle_self_shield", { enable: next });
      setIsSelfShielded(next);
      showToast(
        next ? "StreamShield app is now hidden from streams" : "StreamShield app is now visible on streams",
        next ? "success" : "info"
      );
    } catch (e) {
      showToast(formatBackendError(e), "error");
    }
  };

  const handleMinimizeWindow = async () => {
    try {
      await invoke("minimize_window");
    } catch {
      try {
        await getCurrentWindow().minimize();
      } catch (err) {
        console.error("Minimize error:", err);
      }
    }
  };

  const handleToggleMaximize = async () => {
    try {
      const nextMax = await invoke<boolean>("toggle_maximize_window");
      setIsMaximized(nextMax);
    } catch {
      try {
        const win = getCurrentWindow();
        await win.toggleMaximize();
        const nextMax = await win.isMaximized();
        setIsMaximized(nextMax);
      } catch (err) {
        console.error("Maximize error:", err);
      }
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
    if (windows.length === 0) return;
    try {
      const count = await invoke<number>("shield_all_windows");
      await loadWindows(true);
      showToast(
        count > 0 ? `Shielded ${count} application(s)` : "All open applications are already shielded",
        "success"
      );
    } catch (e) {
      showToast(formatBackendError(e), "error");
      await loadWindows(true);
    }
  };

  const handleUnshieldAll = async () => {
    if (shieldedCount === 0) return;
    await handleClearAllShields();
  };

  const handleBatchShieldClick = () => {
    if (settings.confirm_batch) {
      setBatchConfirmState("shield_all");
    } else {
      handleShieldAll();
    }
  };

  const handleBatchClearClick = () => {
    if (settings.confirm_batch) {
      setBatchConfirmState("clear_all");
    } else {
      handleUnshieldAll();
    }
  };

  const filtered = useMemo(() => {
    const q = search.toLowerCase();
    return windows.filter((w) => {
      const matchesSearch =
        !q || w.exe_name.toLowerCase().includes(q) || w.title.toLowerCase().includes(q);

      const isShielded = shieldedExes.has(w.exe_name);
      if (filterMode === "shielded") return matchesSearch && isShielded;
      if (filterMode === "unshielded") return matchesSearch && !isShielded;
      return matchesSearch;
    });
  }, [windows, search, filterMode, shieldedExes]);

  const shieldedCount = windows.filter((w) => shieldedExes.has(w.exe_name)).length;
  const percentage = windows.length > 0 ? Math.round((shieldedCount / windows.length) * 100) : 0;
  const allShielded = windows.length > 0 && shieldedCount === windows.length;
  const anyShielded = shieldedCount > 0;

  const handleMasterToggle = async () => {
    if (allShielded) {
      await handleUnshieldAll();
    } else {
      await handleShieldAll();
    }
  };

  return (
    <div
      className={`streamshield-root ${isMaximized ? "is-maximized" : ""} ${settings.compact_mode ? "is-compact" : ""}`}
      onClick={() => themeMenuOpen && setThemeMenuOpen(false)}
    >
      {/* Top Header */}
      <header
        className={`app-top-header ${settings.compact_mode ? "is-compact" : ""}`}
        onMouseDown={handleStartDrag}
        onDoubleClick={settings.compact_mode ? undefined : handleToggleMaximize}
      >
        <div className="header-main-row">
          <div className="header-logo-group">
            <img src="/logo.png" className="app-header-logo" alt="StreamShield" />
            {!settings.compact_mode && (
              <img src="/text-logo.png" className="header-app-wordmark" alt="StreamShield" />
            )}
          </div>

          {settings.compact_mode ? (
            <div className="compact-header-actions" onClick={(e) => e.stopPropagation()}>
              {/* Single Master Switch that shields all or clears all itself */}
              <button
                className={`compact-master-toggle ${allShielded ? "is-all" : anyShielded ? "is-partial" : ""}`}
                onClick={handleMasterToggle}
                title={allShielded ? "Clear all shields (Unshield all)" : "Shield all open applications"}
                type="button"
                aria-label="Toggle all shields"
              >
                <span className="compact-master-rail">
                  <span className="compact-master-knob">
                    <IconShield size={10} className="compact-master-glyph" />
                  </span>
                </span>
              </button>

              {/* Stealth Display Eye */}
              <button
                className={`compact-eye-btn ${isSelfShielded ? "is-stealth" : ""}`}
                onClick={handleToggleSelfShield}
                title={
                  isSelfShielded
                    ? "StreamShield is hidden from capture (Click to make visible)"
                    : "StreamShield is visible on capture (Click to hide)"
                }
                type="button"
                aria-label="Toggle stealth mode"
              >
                {isSelfShielded ? <IconEyeOff size={15} /> : <IconEye size={15} />}
              </button>

              {/* Desktop Widget Shortcut */}
              <button
                className={`compact-mode-action-btn ${settings.widget_enabled ? "is-active" : ""}`}
                onClick={handleToggleWidget}
                title={settings.widget_enabled ? "Desktop Widget Active (Click to hide)" : "Show Desktop Screen Widget"}
                type="button"
                aria-label="Desktop Widget"
              >
                <IconWidget size={13} />
              </button>

              {/* Expand back to full view */}
              <button
                className="compact-mode-action-btn"
                onClick={() => handleToggleCompact(false)}
                title="Expand to Full View"
                type="button"
                aria-label="Full View"
              >
                <IconExpand size={13} />
              </button>

              {/* Window Controls */}
              <button
                className="compact-window-btn btn-min"
                onClick={handleMinimizeWindow}
                title="Minimize Window"
                aria-label="Minimize"
                type="button"
              >
                <IconMinimize size={11} />
              </button>
              <button
                className="compact-window-btn btn-close"
                onClick={handleHideToTray}
                title="Hide to System Tray"
                aria-label="Close"
                type="button"
              >
                <IconClose size={11} />
              </button>
            </div>
          ) : (
            <div className="header-action-group">
              {/* Theme Selector Popover */}
              <div className="theme-picker-container" onClick={(e) => e.stopPropagation()}>
                <button
                  type="button"
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
                        type="button"
                        key={t.id}
                        className={`theme-menu-choice ${theme === t.id ? "is-selected" : ""}`}
                        onClick={() => {
                          setTheme(t.id);
                          handleUpdateSettings({ ...settings, theme: t.id });
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

              {/* Compact Mode Toggle on regular page */}
              <button
                className="header-compact-mode-btn"
                onClick={() => handleToggleCompact(true)}
                title="Switch to Compact Mode"
                type="button"
                aria-label="Compact Mode"
              >
                <IconCompact size={15} />
              </button>

              {/* Desktop Widget Toggle */}
              <button
                className={`header-widget-btn ${settings.widget_enabled ? "is-active" : ""}`}
                onClick={handleToggleWidget}
                title={settings.widget_enabled ? "Desktop Widget Active (Click to hide)" : "Show Desktop Screen Widget"}
                type="button"
                aria-label="Desktop Widget"
              >
                <IconWidget size={15} />
              </button>

              {/* Settings Gear Button */}
              <button
                type="button"
                className="settings-gear-btn"
                onClick={() => setSettingsOpen(true)}
                title="Open StreamShield Preferences"
              >
                <IconGear size={16} />
              </button>

              {/* Window Controls Divider & Suite */}
              <div className="window-controls-divider" />

              <div className="window-controls-group" onClick={(e) => e.stopPropagation()}>
                <button
                  type="button"
                  className="window-control-btn btn-minimize"
                  onClick={handleMinimizeWindow}
                  title="Minimize Window"
                  aria-label="Minimize"
                >
                  <IconMinimize size={13} />
                </button>
                <button
                  type="button"
                  className="window-control-btn btn-maximize"
                  onClick={handleToggleMaximize}
                  title={isMaximized ? "Restore Window" : "Maximize Window"}
                  aria-label={isMaximized ? "Restore" : "Maximize"}
                >
                  {isMaximized ? <IconRestore size={13} /> : <IconMaximize size={13} />}
                </button>
                <button
                  type="button"
                  className="window-control-btn btn-close"
                  onClick={handleHideToTray}
                  title="Hide to System Tray"
                  aria-label="Close"
                >
                  <IconClose size={13} />
                </button>
              </div>
            </div>
          )}
        </div>

        {!settings.compact_mode && !isAdmin && (
          <div className="admin-status-banner">
            <IconAlert size={14} className="banner-icon" />
            <span>Administrator permissions recommended for full process capture exclusion</span>
          </div>
        )}

        {/* Automatic Release Update Alert Banner */}
        {!settings.compact_mode && updateInfo?.hasUpdate && (
          <div className="update-alert-banner">
            <div className="update-banner-info">
              <span className="update-indicator-dot" />
              <span>
                Update available: <strong>v{updateInfo.latestVersion}</strong>
              </span>
            </div>
            <div className="update-banner-btns">
              <button
                type="button"
                className="update-apply-btn"
                onClick={handleApplyUpdate}
                disabled={isUpdating}
              >
                {isUpdating ? "Installing..." : "Update Now"}
              </button>
              <button
                type="button"
                className="update-view-btn"
                onClick={() => invoke("open_external_url", { url: updateInfo.releaseUrl }).catch(() => {})}
              >
                Release Notes
              </button>
              <button
                type="button"
                className="update-dismiss-btn"
                onClick={() => setUpdateInfo(null)}
                title="Dismiss"
              >
                ✕
              </button>
            </div>
          </div>
        )}

        {/* Status bar & Batch controls */}
        {!settings.compact_mode && (
          <div className="header-status-strip">
            <div className="home-update-container">
              <button
                type="button"
                className={`home-check-update-btn ${updateInfo?.hasUpdate ? "has-update" : ""}`}
                onClick={updateInfo?.hasUpdate ? handleApplyUpdate : handleManualCheckUpdates}
                disabled={checkingUpdate || isUpdating}
                title={updateInfo?.hasUpdate ? "Click to install latest update" : "Check for latest StreamShield releases"}
              >
                <span className={`update-status-dot ${updateInfo?.hasUpdate ? "is-available" : ""}`} />
                <span className="update-btn-text">
                  {checkingUpdate
                    ? "Checking..."
                    : isUpdating
                    ? "Installing..."
                    : updateInfo?.hasUpdate
                    ? `Update to v${updateInfo.latestVersion}`
                    : "Check for Updates"}
                </span>
              </button>
            </div>

            <div className="batch-controls">
              <button
                type="button"
                className="batch-btn"
                onClick={handleBatchShieldClick}
                disabled={windows.length === 0}
                title="Shield all running applications"
              >
                Shield All
              </button>
              <button
                type="button"
                className="batch-btn"
                onClick={handleBatchClearClick}
                disabled={shieldedCount === 0}
                title="Unshield all applications"
              >
                Clear
              </button>
              <button
                type="button"
                className={`refresh-icon-btn ${refreshing ? "is-spinning" : ""}`}
                onClick={handleRefresh}
                title="Refresh window list"
              >
                <IconRefresh size={14} />
              </button>
            </div>
          </div>
        )}
      </header>

      {/* Search & Navigation Bar (Full Mode Only) */}
      {!settings.compact_mode && (
        <section className="search-filter-section">
          <div className="search-input-shell">
            <IconSearch size={14} className="search-input-icon" />
            <input
              className="search-text-input"
              type="text"
              placeholder="Search processes or windows..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
            />
            {search && (
              <button type="button" className="search-clear-cross" onClick={() => setSearch("")}>
                <IconClose size={12} />
              </button>
            )}
          </div>

          <nav className="tab-segmented-control">
            <button
              type="button"
              className={`tab-segment-btn ${filterMode === "all" ? "is-current" : ""}`}
              onClick={() => setFilterMode("all")}
            >
              All <span className="tab-count">{windows.length}</span>
            </button>
            <button
              type="button"
              className={`tab-segment-btn ${filterMode === "shielded" ? "is-current" : ""}`}
              onClick={() => setFilterMode("shielded")}
            >
              Shielded <span className="tab-count">{shieldedCount}</span>
            </button>
            <button
              type="button"
              className={`tab-segment-btn ${filterMode === "unshielded" ? "is-current" : ""}`}
              onClick={() => setFilterMode("unshielded")}
            >
              Open <span className="tab-count">{windows.length - shieldedCount}</span>
            </button>
          </nav>
        </section>
      )}

      {/* Main List Body */}
      <main className={`app-card-viewport ${settings.compact_mode ? "is-compact" : ""}`}>
        {loading ? (
          <div className={`card-skeleton-stack ${settings.compact_mode ? "is-compact" : ""}`}>
            {Array.from({ length: settings.compact_mode ? 8 : 6 }).map((_, i) => (
              <div key={i} className={settings.compact_mode ? "compact-skeleton-tile" : "card-skeleton"} />
            ))}
          </div>
        ) : (settings.compact_mode ? windows : filtered).length === 0 ? (
          <div className="empty-results-view">
            <div className="empty-glyph-box">
              <IconShield size={38} className="empty-glyph-svg" />
            </div>
            <div className="empty-title-text">No Applications Found</div>
            {!settings.compact_mode && (
              <div className="empty-subtitle-text">
                {search
                  ? "No matching windows found. Try another search query."
                  : "No active application windows found. Open an application and click Refresh."}
              </div>
            )}
          </div>
        ) : (
          <div className={`cards-layout-stack ${settings.compact_mode ? "is-compact" : ""}`}>
            {(settings.compact_mode ? windows : filtered).map((win) => (
              <AppRow
                key={`${win.pid}-${win.hwnd}`}
                window={win}
                shielded={shieldedExes.has(win.exe_name)}
                onToggle={handleToggle}
                compactMode={settings.compact_mode}
                showPid={settings.show_pid}
              />
            ))}
          </div>
        )}
      </main>

      {/* Bottom Footer (Full Mode Only) */}
      {!settings.compact_mode && (
        <footer className="app-bottom-footer">
          <div className="footer-coverage-indicator">
            <div className="coverage-bar-track">
              <div className="coverage-bar-value" style={{ width: `${percentage}%` }} />
            </div>
            <span className="coverage-text">
              <strong>{shieldedCount}</strong> of {windows.length} Protected
            </span>
          </div>

          <div className="footer-right-actions">
            <button
              type="button"
              className={`self-stealth-toggle-btn ${isSelfShielded ? "is-active" : ""}`}
              onClick={handleToggleSelfShield}
              title={
                isSelfShielded
                  ? "StreamShield is hidden from capture"
                  : "Hide StreamShield window itself from screen capture"
              }
            >
              <span className="stealth-indicator-dot" />
              <span className="stealth-label">Stealth: {isSelfShielded ? "ON" : "OFF"}</span>
            </button>
          </div>
        </footer>
      )}

      {/* Settings Modal */}
      <SettingsModal
        isOpen={settingsOpen}
        onClose={() => setSettingsOpen(false)}
        settings={settings}
        onUpdateSettings={handleUpdateSettings}
        onResetSettings={handleResetSettings}
        onClearAllShields={handleClearAllShields}
        shieldedCount={shieldedCount}
        totalWindows={windows.length}
        isAdmin={isAdmin}
        osInfo={osInfo}
      />

      {/* Batch Action Confirmation Dialog */}
      {batchConfirmState && (
        <div className="settings-confirm-overlay" onClick={() => setBatchConfirmState(null)}>
          <div className="settings-confirm-card" onClick={(e) => e.stopPropagation()}>
            <div className="confirm-icon-row">
              <IconAlert size={20} className="confirm-alert-icon" />
              <h3 className="confirm-title">
                {batchConfirmState === "shield_all" ? "Protect all applications?" : "Unshield all applications?"}
              </h3>
            </div>
            <p className="confirm-text">
              {batchConfirmState === "shield_all"
                ? `This will apply capture protection to all ${windows.length} active window(s).`
                : `This will remove capture protection from all ${shieldedCount} protected window(s).`}
            </p>
            <div className="confirm-btn-row">
              <button type="button" className="confirm-cancel-btn" onClick={() => setBatchConfirmState(null)}>
                Cancel
              </button>
              <button
                type="button"
                className="confirm-danger-btn"
                onClick={async () => {
                  if (batchConfirmState === "shield_all") {
                    await handleShieldAll();
                  } else {
                    await handleUnshieldAll();
                  }
                  setBatchConfirmState(null);
                }}
              >
                {batchConfirmState === "shield_all" ? "Yes, Shield All" : "Yes, Clear All"}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Floating Toast Notification */}
      {toast && (
        <div key={toast.id} className={`floating-toast ${toast.type || "error"}`}>
          <span className="toast-symbol">
            {toast.type === "success" ? (
              <IconCheck size={13} />
            ) : toast.type === "info" ? (
              <IconInfo size={13} />
            ) : (
              <IconAlert size={13} />
            )}
          </span>
          <span className="toast-message-body">{toast.message}</span>
        </div>
      )}
    </div>
  );
}
