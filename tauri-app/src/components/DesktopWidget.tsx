import React, { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { WindowInfo } from "./AppRow";
import { AppSettings } from "./SettingsModal";
import {
  IconShield,
  IconShieldCheck,
  IconEye,
  IconEyeOff,
  IconExternalLink,
  IconClose,
  IconGrip,
} from "./Icons";
import { ThemeType, applyDynamicTheme } from "../utils/themeEngine";
import { formatBackendError } from "../utils/win32ErrorCodes";

interface BadgeExpandDir {
  expand_up: boolean;
  expand_left: boolean;
}

export default function DesktopWidget() {
  const initPayload = typeof window !== "undefined" ? window.__STREAMSHIELD_WIDGET_INIT__ : undefined;

  const [windows, setWindows] = useState<WindowInfo[]>([]);
  const [shieldedExes, setShieldedExes] = useState<Set<string>>(new Set());
  const [isSelfShielded, setIsSelfShielded] = useState<boolean>(Boolean(initPayload?.self_stealth));
  const [widgetStyle, setWidgetStyle] = useState<"pill" | "dock" | "badge">(initPayload?.style || "pill");
  const [badgeExpanded, setBadgeExpanded] = useState(false);
  const [expandDir, setExpandDir] = useState<{ expandUp: boolean; expandLeft: boolean }>({
    expandUp: true,
    expandLeft: true,
  });
  const [pendingExes, setPendingExes] = useState<Set<string>>(new Set());
  const [masterBusy, setMasterBusy] = useState(false);
  const [widgetError, setWidgetError] = useState<string | null>(null);

  const lastThemeRef = useRef<ThemeType>(initPayload?.theme || "violet");
  const pollIntervalRef = useRef<number>(3000);
  const errorTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const showWidgetError = useCallback((err: unknown) => {
    const msg = formatBackendError(err);
    setWidgetError(msg);
    if (errorTimerRef.current) clearTimeout(errorTimerRef.current);
    errorTimerRef.current = setTimeout(() => setWidgetError(null), 3200);
  }, []);

  const syncThemeIfNeeded = useCallback((nextTheme?: string) => {
    if (!nextTheme) return;
    const t = nextTheme as ThemeType;
    if (t !== lastThemeRef.current) {
      lastThemeRef.current = t;
      applyDynamicTheme(t);
    }
  }, []);

  const collapseBadge = useCallback(async () => {
    setBadgeExpanded(false);
    try {
      await invoke("set_widget_badge_expanded", { expanded: false });
    } catch {
      // fallback ignore if window hidden
    }
  }, []);

  const refreshWindows = useCallback(async () => {
    try {
      const wins = await invoke<WindowInfo[]>("get_windows");
      setWindows(wins);
      const active = new Set<string>();
      for (const w of wins) {
        if (w.is_shielded) active.add(w.exe_name);
      }
      setShieldedExes(active);
    } catch (e) {
      console.error("Widget window scan error:", e);
    }
  }, []);

  const loadInitialSettings = useCallback(async () => {
    try {
      const s = await invoke<AppSettings>("get_settings");
      if (s.widget_style) setWidgetStyle(s.widget_style);
      if (s.self_stealth !== undefined) setIsSelfShielded(s.self_stealth);
      if (s.poll_interval_ms && s.poll_interval_ms >= 1000) {
        pollIntervalRef.current = s.poll_interval_ms;
      }
      syncThemeIfNeeded(s.theme);
    } catch (e) {
      console.error("Widget initial settings error:", e);
    }
  }, [syncThemeIfNeeded]);

  useEffect(() => {
    loadInitialSettings();
    refreshWindows();

    const interval = setInterval(() => {
      if (document.visibilityState === "visible") {
        refreshWindows();
      }
    }, Math.max(2000, pollIntervalRef.current));

    const onVisibilityChange = () => {
      if (document.visibilityState === "visible") {
        refreshWindows();
      }
    };
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        collapseBadge();
      }
    };

    document.addEventListener("visibilitychange", onVisibilityChange);
    window.addEventListener("keydown", onKeyDown);

    const unlistenStyle = listen<string>("widget_style_changed", (event) => {
      setWidgetStyle(event.payload as "pill" | "dock" | "badge");
      setBadgeExpanded(false);
    });

    const unlistenSettings = listen<AppSettings>("settings_changed", (event) => {
      const s = event.payload;
      if (s.widget_style) {
        setWidgetStyle((prev) => {
          if (prev !== s.widget_style) setBadgeExpanded(false);
          return s.widget_style;
        });
      }
      if (s.self_stealth !== undefined) setIsSelfShielded(s.self_stealth);
      syncThemeIfNeeded(s.theme);
    });

    const unlistenTray = listen("tray_action_completed", () => {
      refreshWindows();
    });

    const unlistenShield = listen("shield_state_changed", () => {
      refreshWindows();
    });

    const unlistenSelfShield = listen<boolean>("self_shield_changed", (event) => {
      setIsSelfShielded(event.payload);
    });

    const unlistenBlur = listen("widget_blur", () => {
      setBadgeExpanded((prev) => {
        if (prev) {
          invoke("set_widget_badge_expanded", { expanded: false }).catch(() => {});
        }
        return false;
      });
    });

    return () => {
      clearInterval(interval);
      if (errorTimerRef.current) clearTimeout(errorTimerRef.current);
      document.removeEventListener("visibilitychange", onVisibilityChange);
      window.removeEventListener("keydown", onKeyDown);
      unlistenStyle.then((u) => u());
      unlistenSettings.then((u) => u());
      unlistenTray.then((u) => u());
      unlistenShield.then((u) => u());
      unlistenSelfShield.then((u) => u());
      unlistenBlur.then((u) => u());
    };
  }, [loadInitialSettings, refreshWindows, syncThemeIfNeeded, collapseBadge]);

  // Native smooth dragging only when pressing non-interactive drag regions
  const handleDrag = (e: React.MouseEvent) => {
    if (e.button !== 0 || e.detail > 1) return;
    const target = e.target as HTMLElement;
    if (target.closest("button, input, select, a, .no-drag, [data-no-drag]")) {
      return;
    }
    invoke("start_window_drag", { windowLabel: "widget" }).catch(() => {
      getCurrentWindow().startDragging().catch(() => {});
    });
  };

  const shieldedCount = windows.filter((w) => shieldedExes.has(w.exe_name)).length;
  const allShielded = windows.length > 0 && shieldedCount === windows.length;
  const anyShielded = shieldedCount > 0;
  const ratio = windows.length > 0 ? shieldedCount / windows.length : 0;
  const percentage = Math.round(ratio * 100);

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
      showWidgetError(e);
      await refreshWindows();
    } finally {
      setPendingExes((prev) => {
        const s = new Set(prev);
        s.delete(win.exe_name);
        return s;
      });
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
      showWidgetError(e);
      await refreshWindows();
    } finally {
      setMasterBusy(false);
    }
  };

  const handleToggleSelfShield = async () => {
    const next = !isSelfShielded;
    try {
      await invoke("toggle_self_shield", { enable: next });
      setIsSelfShielded(next);
    } catch (e) {
      showWidgetError(e);
    }
  };

  const handleOpenMainWindow = async () => {
    try {
      await invoke("show_main_window");
    } catch (e) {
      showWidgetError(e);
    }
  };

  const handleCloseWidget = async () => {
    try {
      setBadgeExpanded(false);
      await invoke("toggle_widget", { enable: false });
    } catch (e) {
      showWidgetError(e);
    }
  };

  const handleSwitchStyle = async (style: "pill" | "dock" | "badge") => {
    try {
      setBadgeExpanded(false);
      await invoke("toggle_widget", { enable: true, style });
    } catch (e) {
      showWidgetError(e);
    }
  };

  const toggleBadgeExpanded = useCallback(async () => {
    const next = !badgeExpanded;
    try {
      const dir = await invoke<BadgeExpandDir>("set_widget_badge_expanded", { expanded: next });
      setExpandDir({ expandUp: dir.expand_up, expandLeft: dir.expand_left });
      setBadgeExpanded(next);
    } catch (e) {
      setBadgeExpanded(next);
      showWidgetError(e);
    }
  }, [badgeExpanded, showWidgetError]);

  // Drag threshold on corner badge: <=5px toggles controls, >5px starts smooth DWM drag
  const handleBadgePointerDown = (e: React.PointerEvent) => {
    if (e.button !== 0) return;
    e.stopPropagation();

    const startX = e.screenX;
    const startY = e.screenY;
    let dragged = false;

    const cleanup = () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
      window.removeEventListener("pointercancel", onUp);
    };

    const onMove = (mv: PointerEvent) => {
      if (dragged) return;
      if (Math.hypot(mv.screenX - startX, mv.screenY - startY) > 5) {
        dragged = true;
        cleanup();
        invoke("start_window_drag", { windowLabel: "widget" }).catch(() => {
          getCurrentWindow().startDragging().catch(() => {});
        });
      }
    };

    const onUp = () => {
      cleanup();
      if (!dragged) {
        toggleBadgeExpanded();
      }
    };

    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
    window.addEventListener("pointercancel", onUp);
  };

  // ── 1. DYNAMIC PILL / ISLAND MODE ─────────────────────────
  if (widgetStyle === "pill") {
    const visibleWins = windows.slice(0, 5);
    const overflowCount = Math.max(0, windows.length - visibleWins.length);

    return (
      <div className="desktop-widget-root pill-mode" onMouseDown={handleDrag}>
        <div
          className={`widget-pill-container ${
            allShielded ? "all-active" : anyShielded ? "has-active" : ""
          }`}
        >
          <span className="widget-drag-grip" title="Drag widget">
            <IconGrip size={11} />
          </span>

          <button
            type="button"
            className="pill-brand-badge no-drag"
            onClick={handleOpenMainWindow}
            onMouseDown={(e) => e.stopPropagation()}
            title="Open StreamShield"
            aria-label={`Open StreamShield (${shieldedCount} of ${windows.length} shielded)`}
          >
            <img src="/logo.png" className="widget-pill-logo" alt="" />
            <span
              className={`pill-counter-tag ${
                allShielded ? "all-safe" : anyShielded ? "partial" : ""
              }`}
            >
              <span className="pill-status-dot" aria-hidden="true" />
              <span>
                {shieldedCount}/{windows.length}
              </span>
            </span>
          </button>

          <div className="pill-active-apps-deck" onMouseDown={(e) => e.stopPropagation()}>
            {windows.length === 0 ? (
              <span className="pill-empty-label">No apps</span>
            ) : (
              visibleWins.map((w) => {
                const isSh = shieldedExes.has(w.exe_name);
                const isBusy = pendingExes.has(w.exe_name);
                const clean = w.exe_name.replace(/\.exe$/i, "");
                return (
                  <button
                    key={`${w.pid}-${w.hwnd}`}
                    className={`pill-mini-app-tile ${isSh ? "is-shielded" : ""} ${
                      isBusy ? "is-busy" : ""
                    }`}
                    onClick={() => handleToggleApp(w)}
                    onMouseDown={(e) => e.stopPropagation()}
                    disabled={isBusy}
                    aria-pressed={isSh}
                    aria-label={`${clean}: ${isSh ? "Shielded" : "Open"}`}
                    title={`${clean}: ${isSh ? "Shielded (Tap to unshield)" : "Open (Tap to shield)"}`}
                    type="button"
                  >
                    {w.icon_base64 ? (
                      <img src={w.icon_base64} alt="" className="pill-app-icon" />
                    ) : (
                      <span className="pill-app-initial">{clean[0]?.toUpperCase() || "?"}</span>
                    )}
                    {isSh && <span className="pill-shield-bar" aria-hidden="true" />}
                  </button>
                );
              })
            )}
            {overflowCount > 0 && (
              <button
                type="button"
                className="pill-overflow-badge no-drag"
                onClick={handleOpenMainWindow}
                onMouseDown={(e) => e.stopPropagation()}
                title={`${overflowCount} more open app(s) — Open full window`}
              >
                +{overflowCount}
              </button>
            )}
          </div>

          <div className="pill-action-cluster" onMouseDown={(e) => e.stopPropagation()}>
            <button
              className={`pill-master-btn ${
                allShielded ? "is-active" : anyShielded ? "is-partial" : ""
              }`}
              onClick={handleMasterToggle}
              onMouseDown={(e) => e.stopPropagation()}
              disabled={masterBusy || windows.length === 0}
              aria-pressed={allShielded}
              title={allShielded ? "Unshield all applications" : "Shield all open applications"}
              type="button"
            >
              {allShielded ? <IconShieldCheck size={12} /> : <IconShield size={12} />}
            </button>

            <button
              className={`pill-eye-btn ${isSelfShielded ? "is-stealth" : ""}`}
              onClick={handleToggleSelfShield}
              onMouseDown={(e) => e.stopPropagation()}
              aria-pressed={isSelfShielded}
              title={
                isSelfShielded
                  ? "App hidden from stream (Click to show)"
                  : "App visible on stream (Click to hide)"
              }
              type="button"
            >
              {isSelfShielded ? <IconEyeOff size={13} /> : <IconEye size={13} />}
            </button>

            <button
              className="pill-window-btn"
              onClick={handleOpenMainWindow}
              onMouseDown={(e) => e.stopPropagation()}
              title="Open StreamShield Full Window"
              type="button"
            >
              <IconExternalLink size={12} />
            </button>

            <button
              className="pill-close-btn"
              onClick={handleCloseWidget}
              onMouseDown={(e) => e.stopPropagation()}
              title="Hide Desktop Widget"
              type="button"
            >
              <IconClose size={11} />
            </button>
          </div>
        </div>
        {widgetError && <div className="widget-inline-error">{widgetError}</div>}
      </div>
    );
  }

  // ── 2. FLOATING COMPACT DOCK MODE ─────────────────────────
  if (widgetStyle === "dock") {
    return (
      <div className="desktop-widget-root dock-mode" onMouseDown={handleDrag}>
        <div className="widget-dock-container">
          <div className="dock-coverage-rail" aria-hidden="true">
            <div className="dock-coverage-fill" style={{ width: `${percentage}%` }} />
          </div>

          <div className="widget-dock-header">
            <div className="dock-header-drag-group">
              <span className="widget-drag-grip" title="Drag widget">
                <IconGrip size={11} />
              </span>
              <button
                type="button"
                className="dock-header-left no-drag"
                onClick={handleOpenMainWindow}
                onMouseDown={(e) => e.stopPropagation()}
                title="Click to open StreamShield"
              >
                <img src="/logo.png" className="widget-dock-logo" alt="StreamShield" />
                <span className="dock-count-badge">
                  {shieldedCount}/{windows.length}
                </span>
                <span
                  className={`dock-state-tag ${
                    allShielded ? "is-all" : anyShielded ? "is-partial" : ""
                  }`}
                >
                  {allShielded ? "ALL" : anyShielded ? "ACTIVE" : "IDLE"}
                </span>
              </button>
            </div>

            <div className="dock-header-right" onMouseDown={(e) => e.stopPropagation()}>
              <button
                className={`dock-tool-btn ${
                  allShielded ? "is-active" : anyShielded ? "is-partial" : ""
                }`}
                onClick={handleMasterToggle}
                onMouseDown={(e) => e.stopPropagation()}
                disabled={masterBusy || windows.length === 0}
                aria-pressed={allShielded}
                title={allShielded ? "Unshield all" : "Shield all"}
                type="button"
              >
                {allShielded ? <IconShieldCheck size={11} /> : <IconShield size={11} />}
              </button>

              <button
                className={`dock-tool-btn ${isSelfShielded ? "is-stealth" : ""}`}
                onClick={handleToggleSelfShield}
                onMouseDown={(e) => e.stopPropagation()}
                aria-pressed={isSelfShielded}
                title={
                  isSelfShielded ? "Stealth active (Click to disable)" : "Stealth off (Click to enable)"
                }
                type="button"
              >
                {isSelfShielded ? <IconEyeOff size={12} /> : <IconEye size={12} />}
              </button>

              <button
                className="dock-tool-btn"
                onClick={handleOpenMainWindow}
                onMouseDown={(e) => e.stopPropagation()}
                title="Open Window"
                type="button"
              >
                <IconExternalLink size={11} />
              </button>

              <button
                className="dock-tool-btn close"
                onClick={handleCloseWidget}
                onMouseDown={(e) => e.stopPropagation()}
                title="Close Widget"
                type="button"
              >
                <IconClose size={10} />
              </button>
            </div>
          </div>

          {windows.length === 0 ? (
            <div className="dock-empty-state">No active application windows</div>
          ) : (
            <div className="widget-dock-grid" onMouseDown={(e) => e.stopPropagation()}>
              {windows.map((w) => {
                const isSh = shieldedExes.has(w.exe_name);
                const isBusy = pendingExes.has(w.exe_name);
                const clean = w.exe_name.replace(/\.exe$/i, "");
                return (
                  <button
                    key={`${w.pid}-${w.hwnd}`}
                    className={`dock-app-tile ${isSh ? "is-shielded" : ""} ${
                      isBusy ? "is-busy" : ""
                    }`}
                    onClick={() => handleToggleApp(w)}
                    onMouseDown={(e) => e.stopPropagation()}
                    disabled={isBusy}
                    aria-pressed={isSh}
                    aria-label={`${clean}: ${isSh ? "Shielded" : "Open"}`}
                    title={`${clean}: ${isSh ? "Shielded (Tap to unshield)" : "Open (Tap to shield)"}`}
                    type="button"
                  >
                    {w.icon_base64 ? (
                      <img src={w.icon_base64} alt="" className="dock-app-icon" />
                    ) : (
                      <span className="dock-app-initial">{clean[0]?.toUpperCase() || "?"}</span>
                    )}
                    {isSh && (
                      <span className="dock-shield-badge" aria-hidden="true">
                        <IconShieldCheck size={9} />
                      </span>
                    )}
                  </button>
                );
              })}
            </div>
          )}

          <div className="dock-footer-bar" onMouseDown={(e) => e.stopPropagation()}>
            <div className="widget-mode-switcher" role="group" aria-label="Widget display style">
              <button
                type="button"
                className="widget-mode-btn"
                onClick={() => handleSwitchStyle("pill")}
              >
                Pill
              </button>
              <button
                type="button"
                className="widget-mode-btn is-selected"
                onClick={() => handleSwitchStyle("dock")}
              >
                Dock
              </button>
              <button
                type="button"
                className="widget-mode-btn"
                onClick={() => handleSwitchStyle("badge")}
              >
                Badge
              </button>
            </div>
            <span className={`dock-stealth-status ${isSelfShielded ? "is-on" : ""}`}>
              {isSelfShielded ? "Stealth ON" : "Visible"}
            </span>
          </div>

          {widgetError && <div className="widget-inline-error dock-error">{widgetError}</div>}
        </div>
      </div>
    );
  }

  // ── 3. CORNER STATUS BADGE MODE (ORBITAL TELEMETRY RING) ─
  const ringRadius = 24.5;
  const ringCircumference = 2 * Math.PI * ringRadius;
  const ringDashOffset = ringCircumference * (1 - ratio);
  const badgeQuickWins = windows.slice(0, 5);

  const badgeClasses = [
    "desktop-widget-root",
    "badge-mode",
    expandDir.expandUp ? "expand-up" : "expand-down",
    expandDir.expandLeft ? "expand-left" : "expand-right",
    badgeExpanded ? "is-expanded" : "",
  ]
    .filter(Boolean)
    .join(" ");

  return (
    <div className={badgeClasses} onMouseDown={handleDrag}>
      <button
        type="button"
        className={`widget-badge-circle no-drag ${
          allShielded ? "is-all" : anyShielded ? "is-partial" : ""
        }`}
        onPointerDown={handleBadgePointerDown}
        onMouseDown={(e) => e.stopPropagation()}
        onKeyDown={(e) => {
          if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            toggleBadgeExpanded();
          }
        }}
        aria-expanded={badgeExpanded}
        aria-label={`StreamShield Badge: ${shieldedCount} of ${windows.length} protected`}
        title="StreamShield Badge (Click for controls • Drag to move)"
      >
        <svg className="badge-progress-ring" viewBox="0 0 56 56" aria-hidden="true">
          <defs>
            <linearGradient id="ss-badge-ring" x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" style={{ stopColor: "var(--primary)" }} />
              <stop offset="100%" style={{ stopColor: "var(--accent)" }} />
            </linearGradient>
          </defs>
          <circle cx="28" cy="28" r={ringRadius} className="badge-ring-track" />
          <circle
            cx="28"
            cy="28"
            r={ringRadius}
            className="badge-ring-fill"
            stroke="url(#ss-badge-ring)"
            strokeDasharray={ringCircumference}
            strokeDashoffset={ringDashOffset}
          />
        </svg>

        <img src="/logo.png" className="widget-badge-logo" alt="" />
        <span className={`badge-count-pill ${shieldedCount === 0 ? "is-idle" : ""}`}>
          {shieldedCount}
        </span>
      </button>

      {badgeExpanded && (
        <div
          className="badge-quick-popover"
          role="dialog"
          aria-label="StreamShield Badge Quick Controls"
          onMouseDown={(e) => e.stopPropagation()}
        >
          <div className="badge-popover-rail" aria-hidden="true">
            <div className="badge-popover-fill" style={{ width: `${percentage}%` }} />
          </div>

          <div className="badge-popover-header">
            <span className="badge-popover-ratio">
              {shieldedCount}/{windows.length} Protected
            </span>
            <span className={`badge-popover-stealth ${isSelfShielded ? "is-on" : ""}`}>
              {isSelfShielded ? "Stealth" : "Visible"}
            </span>
          </div>

          <div className="badge-action-row">
            <button
              className={`badge-action-chip ${allShielded ? "is-active" : ""}`}
              onClick={handleMasterToggle}
              onMouseDown={(e) => e.stopPropagation()}
              disabled={masterBusy || windows.length === 0}
              type="button"
              title={allShielded ? "Clear all shields" : "Shield all open apps"}
            >
              {allShielded ? <IconShieldCheck size={13} /> : <IconShield size={13} />}
              <span>{allShielded ? "Clear" : "Shield"}</span>
            </button>

            <button
              className={`badge-action-chip ${isSelfShielded ? "is-active" : ""}`}
              onClick={handleToggleSelfShield}
              onMouseDown={(e) => e.stopPropagation()}
              type="button"
              aria-pressed={isSelfShielded}
              title={isSelfShielded ? "Disable self stealth" : "Enable self stealth"}
            >
              {isSelfShielded ? <IconEyeOff size={13} /> : <IconEye size={13} />}
              <span>Stealth</span>
            </button>

            <button
              className="badge-action-chip"
              onClick={handleOpenMainWindow}
              onMouseDown={(e) => e.stopPropagation()}
              type="button"
              title="Open full StreamShield window"
            >
              <IconExternalLink size={13} />
              <span>Open</span>
            </button>
          </div>

          {badgeQuickWins.length > 0 && (
            <div className="badge-apps-strip">
              {badgeQuickWins.map((w) => {
                const isSh = shieldedExes.has(w.exe_name);
                const isBusy = pendingExes.has(w.exe_name);
                const clean = w.exe_name.replace(/\.exe$/i, "");
                return (
                  <button
                    key={`${w.pid}-${w.hwnd}`}
                    type="button"
                    className={`badge-mini-app ${isSh ? "is-shielded" : ""} ${
                      isBusy ? "is-busy" : ""
                    }`}
                    onClick={() => handleToggleApp(w)}
                    disabled={isBusy}
                    aria-pressed={isSh}
                    title={`${clean}: ${isSh ? "Shielded" : "Open"}`}
                  >
                    {w.icon_base64 ? (
                      <img src={w.icon_base64} alt="" className="badge-mini-app-icon" />
                    ) : (
                      <span className="badge-mini-app-initial">
                        {clean[0]?.toUpperCase() || "?"}
                      </span>
                    )}
                    {isSh && <span className="badge-mini-app-bar" aria-hidden="true" />}
                  </button>
                );
              })}
            </div>
          )}

          <div className="widget-mode-switcher badge-modes" role="group" aria-label="Widget style">
            <button
              type="button"
              className="widget-mode-btn"
              onClick={() => handleSwitchStyle("pill")}
            >
              Pill
            </button>
            <button
              type="button"
              className="widget-mode-btn"
              onClick={() => handleSwitchStyle("dock")}
            >
              Dock
            </button>
            <button
              type="button"
              className="widget-mode-btn is-selected"
              onClick={() => handleSwitchStyle("badge")}
            >
              Badge
            </button>
          </div>

          <hr className="widget-menu-sep" />

          <button
            className="badge-popover-item close"
            onClick={handleCloseWidget}
            onMouseDown={(e) => e.stopPropagation()}
            type="button"
          >
            <IconClose size={12} />
            <span>Hide Widget</span>
          </button>

          {widgetError && <div className="widget-inline-error badge-error">{widgetError}</div>}
        </div>
      )}
    </div>
  );
}
