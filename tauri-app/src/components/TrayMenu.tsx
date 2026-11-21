import React, { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { WindowInfo } from "./AppRow";
import { AppSettings } from "./SettingsModal";
import {
  IconShield,
  IconShieldCheck,
  IconEye,
  IconEyeOff,
  IconRefresh,
  IconExternalLink,
  IconGear,
  IconPower,
  IconCheck,
  IconVolume,
  IconVolumeOff,
} from "./Icons";
import { ThemeType, applyDynamicTheme } from "../utils/themeEngine";
import { formatBackendError } from "../utils/win32ErrorCodes";

interface TrayOpenedEventPayload {
  origin?: "bottom" | "top" | "left" | "right";
}

export default function TrayMenu() {
  const initPayload = typeof window !== "undefined" ? window.__STREAMSHIELD_TRAY_INIT__ : undefined;

  const [windows, setWindows] = useState<WindowInfo[]>([]);
  const [shieldedExes, setShieldedExes] = useState<Set<string>>(new Set());
  const [mutedExes, setMutedExes] = useState<Set<string>>(new Set());
  const [settings, setSettings] = useState<AppSettings>(() => ({
    autostart: false,
    start_minimized: false,
    auto_reapply: true,
    shield_mode: (initPayload?.shield_mode as "exclude" | "monitor") || "exclude",
    poll_interval_ms: 3000,
    theme: initPayload?.theme || "violet",
    compact_mode: Boolean(initPayload?.compact_mode),
    show_pid: true,
    confirm_batch: false,
    mpo_fix: false,
    self_stealth: Boolean(initPayload?.self_stealth),
    widget_enabled: Boolean(initPayload?.widget_enabled),
    widget_style: (initPayload?.widget_style as "pill" | "dock" | "badge") || "pill",
    widget_always_on_top: true,
    widget_stealth: true,
  }));

  const [pendingExes, setPendingExes] = useState<Set<string>>(new Set());
  const [masterBusy, setMasterBusy] = useState(false);
  const [reapplying, setReapplying] = useState(false);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const [origin, setOrigin] = useState<"bottom" | "top" | "left" | "right">("bottom");
  const [openKey, setOpenKey] = useState<number>(1);

  const lastThemeRef = useRef<ThemeType>(initPayload?.theme || "violet");
  const errorTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const shellRef = useRef<HTMLDivElement>(null);

  const showError = useCallback((err: unknown) => {
    setErrorMsg(formatBackendError(err));
    if (errorTimerRef.current) clearTimeout(errorTimerRef.current);
    errorTimerRef.current = setTimeout(() => setErrorMsg(null), 3200);
  }, []);

  const syncThemeIfNeeded = useCallback((nextTheme?: string) => {
    if (!nextTheme) return;
    const t = nextTheme as ThemeType;
    if (t !== lastThemeRef.current) {
      lastThemeRef.current = t;
      applyDynamicTheme(t);
    }
  }, []);

  const refreshWindows = useCallback(async () => {
    try {
      const wins = await invoke<WindowInfo[]>("get_windows");
      setWindows(wins);
      const active = new Set<string>();
      const muted = new Set<string>();
      for (const w of wins) {
        if (w.is_shielded) active.add(w.exe_name);
        if (w.is_audio_muted) muted.add(w.exe_name);
      }
      setShieldedExes(active);
      setMutedExes(muted);
    } catch (e) {
      console.error("TrayMenu window scan error:", e);
    }
  }, []);

  const loadSettings = useCallback(async () => {
    try {
      const s = await invoke<AppSettings>("get_settings");
      setSettings(s);
      syncThemeIfNeeded(s.theme);
    } catch (e) {
      console.error("TrayMenu settings load error:", e);
    }
  }, [syncThemeIfNeeded]);

  const handleDismiss = useCallback(() => {
    invoke("hide_tray_menu").catch(() => {});
  }, []);

  useEffect(() => {
    loadSettings();
    refreshWindows();

    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        handleDismiss();
        return;
      }
      if (["ArrowDown", "ArrowUp", "Home", "End"].includes(e.key)) {
        const container = shellRef.current;
        if (!container) return;
        const focusable = Array.from(
          container.querySelectorAll<HTMLButtonElement>("button:not(:disabled)")
        );
        if (focusable.length === 0) return;
        e.preventDefault();
        const curIdx = focusable.indexOf(document.activeElement as HTMLButtonElement);
        let nextIdx = 0;
        if (e.key === "Home") {
          nextIdx = 0;
        } else if (e.key === "End") {
          nextIdx = focusable.length - 1;
        } else if (e.key === "ArrowDown") {
          nextIdx = curIdx < 0 ? 0 : (curIdx + 1) % focusable.length;
        } else {
          nextIdx = curIdx <= 0 ? focusable.length - 1 : curIdx - 1;
        }
        focusable[nextIdx]?.focus();
      }
    };

    window.addEventListener("keydown", onKeyDown);

    const unlistenOpened = listen<TrayOpenedEventPayload>("tray_menu_opened", (event) => {
      if (event.payload?.origin) {
        setOrigin(event.payload.origin);
      }
      setOpenKey((k) => k + 1);
      loadSettings();
      refreshWindows();
    });

    const unlistenSettings = listen<AppSettings>("settings_changed", (event) => {
      const s = event.payload;
      setSettings(s);
      syncThemeIfNeeded(s.theme);
    });

    const unlistenShield = listen("shield_state_changed", () => {
      refreshWindows();
    });

    const unlistenAudio = listen("audio_state_changed", () => {
      refreshWindows();
    });

    const unlistenSelfShield = listen<boolean>("self_shield_changed", (event) => {
      setSettings((prev) => ({ ...prev, self_stealth: event.payload }));
    });

    const unlistenWidget = listen<boolean>("widget_state_changed", (event) => {
      setSettings((prev) => ({ ...prev, widget_enabled: event.payload }));
    });

    const unlistenCompact = listen<boolean>("compact_mode_changed", (event) => {
      setSettings((prev) => ({ ...prev, compact_mode: event.payload }));
    });

    return () => {
      if (errorTimerRef.current) clearTimeout(errorTimerRef.current);
      window.removeEventListener("keydown", onKeyDown);
      unlistenOpened.then((u) => u());
      unlistenSettings.then((u) => u());
      unlistenShield.then((u) => u());
      unlistenAudio.then((u) => u());
      unlistenSelfShield.then((u) => u());
      unlistenWidget.then((u) => u());
      unlistenCompact.then((u) => u());
    };
  }, [loadSettings, refreshWindows, syncThemeIfNeeded, handleDismiss]);

  const shieldedCount = windows.filter((w) => shieldedExes.has(w.exe_name)).length;
  const allShielded = windows.length > 0 && shieldedCount === windows.length;
  const anyShielded = shieldedCount > 0;
  const percentage = windows.length > 0 ? Math.round((shieldedCount / windows.length) * 100) : 0;

  const handleOpenMain = async () => {
    try {
      await invoke("show_main_window");
      await invoke("hide_tray_menu");
    } catch (e) {
      showError(e);
    }
  };

  const handleOpenPreferences = async () => {
    try {
      await invoke("open_main_settings");
    } catch (e) {
      showError(e);
    }
  };

  const handleQuit = async () => {
    try {
      await invoke("quit_app");
    } catch (e) {
      showError(e);
    }
  };

  const handleToggleApp = async (win: WindowInfo) => {
    if (pendingExes.has(win.exe_name)) return;
    const next = !shieldedExes.has(win.exe_name);
    setPendingExes((prev) => new Set(prev).add(win.exe_name));
    setShieldedExes((prev) => {
      const s = new Set(prev);
      if (next) s.add(win.exe_name);
      else s.delete(win.exe_name);
      return s;
    });

    try {
      await invoke("toggle_shield", {
        exeName: win.exe_name,
        hwnd: win.hwnd,
        pid: win.pid,
        enable: next,
      });
    } catch (e) {
      showError(e);
      await refreshWindows();
    } finally {
      setPendingExes((prev) => {
        const s = new Set(prev);
        s.delete(win.exe_name);
        return s;
      });
    }
  };

  const handleToggleAudio = async (e: React.MouseEvent, win: WindowInfo) => {
    e.stopPropagation();
    const next = !mutedExes.has(win.exe_name);
    setMutedExes((prev) => {
      const s = new Set(prev);
      if (next) s.add(win.exe_name);
      else s.delete(win.exe_name);
      return s;
    });
    try {
      await invoke("toggle_audio_mute", {
        exeName: win.exe_name,
        pid: win.pid,
        mute: next,
      });
    } catch (err) {
      showError(err);
      await refreshWindows();
    }
  };

  const handleMasterToggle = async () => {
    if (masterBusy || windows.length === 0) return;
    setMasterBusy(true);
    try {
      if (allShielded) {
        await invoke("clear_all_shields");
        setShieldedExes(new Set());
      } else {
        await invoke("shield_all_windows");
        setShieldedExes(new Set(windows.map((w) => w.exe_name)));
      }
      await refreshWindows();
    } catch (e) {
      showError(e);
      await refreshWindows();
    } finally {
      setMasterBusy(false);
    }
  };

  const handleToggleStealth = async () => {
    const next = !settings.self_stealth;
    setSettings((prev) => ({ ...prev, self_stealth: next }));
    try {
      await invoke("toggle_self_shield", { enable: next });
    } catch (e) {
      setSettings((prev) => ({ ...prev, self_stealth: !next }));
      showError(e);
    }
  };

  const handleReapply = async () => {
    if (reapplying) return;
    setReapplying(true);
    try {
      await invoke("reapply_shields");
      await refreshWindows();
    } catch (e) {
      showError(e);
    } finally {
      setTimeout(() => setReapplying(false), 450);
    }
  };

  const handleSwitchMode = async (mode: "exclude" | "monitor") => {
    if (settings.shield_mode === mode) return;
    const prevMode = settings.shield_mode;
    const nextSettings: AppSettings = { ...settings, shield_mode: mode };
    setSettings(nextSettings);
    try {
      await invoke("update_settings", { settings: nextSettings });
    } catch (e) {
      setSettings((prev) => ({ ...prev, shield_mode: prevMode }));
      showError(e);
    }
  };

  const handleToggleWidget = async () => {
    const next = !settings.widget_enabled;
    setSettings((prev) => ({ ...prev, widget_enabled: next }));
    try {
      await invoke("toggle_widget", { enable: next });
    } catch (e) {
      setSettings((prev) => ({ ...prev, widget_enabled: !next }));
      showError(e);
    }
  };

  const handleSwitchWidgetStyle = async (style: "pill" | "dock" | "badge") => {
    setSettings((prev) => ({ ...prev, widget_enabled: true, widget_style: style }));
    try {
      await invoke("toggle_widget", { enable: true, style });
    } catch (e) {
      showError(e);
      await loadSettings();
    }
  };

  const handleToggleCompact = async () => {
    const next = !settings.compact_mode;
    setSettings((prev) => ({ ...prev, compact_mode: next }));
    try {
      await invoke("set_compact_mode", { compact: next });
    } catch (e) {
      setSettings((prev) => ({ ...prev, compact_mode: !next }));
      showError(e);
    }
  };

  return (
    <div
      className={`tray-menu-root origin-${origin}`}
      onMouseDown={(e) => {
        if (e.target === e.currentTarget) handleDismiss();
      }}
    >
      <div
        key={openKey}
        ref={shellRef}
        className={`tray-menu-shell origin-${origin}`}
        role="menu"
        aria-label="StreamShield Quick Control"
      >
        {/* Top Luminous Telemetry Bar */}
        <div className="tray-coverage-rail" aria-hidden="true">
          <div className="tray-coverage-fill" style={{ width: `${percentage}%` }} />
        </div>

        {/* Brand + Telemetry Header */}
        <div className="tray-menu-header">
          <div className="tray-brand-left">
            <img src="/logo.png" className="tray-brand-logo" alt="StreamShield" />
            <span className="tray-brand-title">StreamShield</span>
            <span
              className={`tray-telemetry-chip ${
                allShielded ? "is-all" : anyShielded ? "is-partial" : ""
              }`}
            >
              <span className="tray-chip-dot" aria-hidden="true" />
              {shieldedCount}/{windows.length}
            </span>
          </div>

          <button
            type="button"
            className="tray-open-btn"
            onClick={handleOpenMain}
            title="Open StreamShield window"
          >
            <span>Open</span>
            <IconExternalLink size={11} />
          </button>
        </div>

        {/* Quick Action Trio */}
        <div className="tray-action-trio">
          <button
            type="button"
            className={`tray-action-chip ${
              allShielded ? "is-active" : anyShielded ? "is-partial" : ""
            }`}
            onClick={handleMasterToggle}
            disabled={masterBusy || windows.length === 0}
            title={allShielded ? "Clear all shields" : "Shield all open apps"}
          >
            {allShielded ? <IconShieldCheck size={14} /> : <IconShield size={14} />}
            <span>{allShielded ? "Clear All" : "Shield All"}</span>
          </button>

          <button
            type="button"
            className={`tray-action-chip ${settings.self_stealth ? "is-active" : ""}`}
            onClick={handleToggleStealth}
            aria-pressed={settings.self_stealth}
            title={
              settings.self_stealth
                ? "Window hidden from capture (Click to reveal)"
                : "Hide StreamShield UI from capture"
            }
          >
            {settings.self_stealth ? <IconEyeOff size={14} /> : <IconEye size={14} />}
            <span>Stealth</span>
          </button>

          <button
            type="button"
            className={`tray-action-chip ${reapplying ? "is-busy-spin" : ""}`}
            onClick={handleReapply}
            disabled={reapplying}
            title="Reapply capture masks to shielded windows"
          >
            <IconRefresh size={13} className={reapplying ? "tray-spin-icon" : ""} />
            <span>Reapply</span>
          </button>
        </div>

        {/* Open Applications Section */}
        <div className="tray-section-label-row">
          <span className="tray-section-label">Applications</span>
          <span className="tray-section-count">{windows.length}</span>
        </div>

        <div className="tray-apps-list">
          {windows.length === 0 ? (
            <div className="tray-apps-empty">No open application windows detected</div>
          ) : (
            windows.map((w) => {
              const isSh = shieldedExes.has(w.exe_name);
              const isMuted = mutedExes.has(w.exe_name);
              const isBusy = pendingExes.has(w.exe_name);
              const clean = w.exe_name.replace(/\.exe$/i, "");
              return (
                <div
                  key={`${w.pid}-${w.hwnd}`}
                  className={`tray-app-row-wrap ${isSh ? "is-shielded" : ""}`}
                >
                  <button
                    type="button"
                    role="menuitemcheckbox"
                    aria-checked={isSh}
                    className={`tray-app-row ${isSh ? "is-shielded" : ""} ${
                      isBusy ? "is-busy" : ""
                    }`}
                    onClick={() => handleToggleApp(w)}
                    disabled={isBusy}
                    title={w.title || clean}
                  >
                    <span className="tray-app-icon-wrap">
                      {w.icon_base64 ? (
                        <img src={w.icon_base64} alt="" className="tray-app-icon" />
                      ) : (
                        <span className="tray-app-initial">{clean[0]?.toUpperCase() || "?"}</span>
                      )}
                    </span>

                    <span className="tray-app-name">{clean}</span>

                    <span className={`tray-app-state ${isSh ? "is-shielded" : ""}`}>
                      {isSh ? <IconCheck size={11} /> : <span className="tray-app-ring" />}
                      <span>{isSh ? "Shielded" : "Open"}</span>
                    </span>
                  </button>

                  <button
                    type="button"
                    className={`tray-app-audio-btn ${isMuted ? "is-muted" : ""}`}
                    onClick={(e) => handleToggleAudio(e, w)}
                    aria-pressed={isMuted}
                    title={isMuted ? `Unmute ${clean}` : `Mute ${clean} audio`}
                  >
                    {isMuted ? <IconVolumeOff size={12} /> : <IconVolume size={12} />}
                  </button>
                </div>
              );
            })
          )}
        </div>

        <hr className="tray-divider" />

        {/* Theme-Synced Quick Settings Deck */}
        <div className="tray-controls-deck">
          {/* Shield Mode */}
          <div className="tray-control-row">
            <span className="tray-control-label">Shield Mode</span>
            <div className="tray-segmented-pill" role="group" aria-label="Shield capture mode">
              <button
                type="button"
                className={`tray-seg-btn ${settings.shield_mode === "exclude" ? "is-selected" : ""}`}
                onClick={() => handleSwitchMode("exclude")}
              >
                Transparent
              </button>
              <button
                type="button"
                className={`tray-seg-btn ${settings.shield_mode === "monitor" ? "is-selected" : ""}`}
                onClick={() => handleSwitchMode("monitor")}
              >
                Black Mask
              </button>
            </div>
          </div>

          {/* Desktop Widget Toggle + Layout Style */}
          <div className="tray-widget-block">
            <div className="tray-control-row">
              <span className="tray-control-label">Desktop Widget</span>
              <button
                type="button"
                role="switch"
                aria-checked={settings.widget_enabled}
                className={`tray-mini-switch ${settings.widget_enabled ? "is-on" : ""}`}
                onClick={handleToggleWidget}
                aria-label="Toggle Desktop Widget"
              >
                <span className="tray-mini-knob" />
              </button>
            </div>

            <div className="tray-segmented-pill full" role="group" aria-label="Widget display style">
              <button
                type="button"
                className={`tray-seg-btn ${
                  settings.widget_enabled && settings.widget_style === "pill" ? "is-selected" : ""
                }`}
                onClick={() => handleSwitchWidgetStyle("pill")}
              >
                Pill
              </button>
              <button
                type="button"
                className={`tray-seg-btn ${
                  settings.widget_enabled && settings.widget_style === "dock" ? "is-selected" : ""
                }`}
                onClick={() => handleSwitchWidgetStyle("dock")}
              >
                Dock
              </button>
              <button
                type="button"
                className={`tray-seg-btn ${
                  settings.widget_enabled && settings.widget_style === "badge" ? "is-selected" : ""
                }`}
                onClick={() => handleSwitchWidgetStyle("badge")}
              >
                Badge
              </button>
            </div>
          </div>

          {/* Compact View */}
          <div className="tray-control-row">
            <span className="tray-control-label">Compact Window</span>
            <button
              type="button"
              role="switch"
              aria-checked={settings.compact_mode}
              className={`tray-mini-switch ${settings.compact_mode ? "is-on" : ""}`}
              onClick={handleToggleCompact}
              aria-label="Toggle Compact Mode"
            >
              <span className="tray-mini-knob" />
            </button>
          </div>
        </div>

        <hr className="tray-divider" />

        {/* Footer Actions */}
        <div className="tray-menu-footer">
          <button type="button" className="tray-footer-btn" onClick={handleOpenPreferences}>
            <IconGear size={13} />
            <span>Preferences</span>
          </button>

          <button type="button" className="tray-footer-btn quit" onClick={handleQuit}>
            <IconPower size={13} />
            <span>Quit</span>
          </button>
        </div>

        {errorMsg && <div className="tray-inline-error">{errorMsg}</div>}
      </div>
    </div>
  );
}
