import React, { useState, useEffect, useCallback } from "react";
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
} from "./Icons";
import { ThemeType, applyDynamicTheme } from "../utils/themeEngine";

export default function DesktopWidget() {
  const [windows, setWindows] = useState<WindowInfo[]>([]);
  const [shieldedExes, setShieldedExes] = useState<Set<string>>(new Set());
  const [isSelfShielded, setIsSelfShielded] = useState(false);
  const [widgetStyle, setWidgetStyle] = useState<"pill" | "dock" | "badge">("pill");
  const [theme, setTheme] = useState<ThemeType>("violet");
  const [badgeExpanded, setBadgeExpanded] = useState(false);

  // Load state and listen for updates
  const refreshState = useCallback(async () => {
    try {
      const [wins, settings, selfShielded] = await Promise.all([
        invoke<WindowInfo[]>("get_windows"),
        invoke<AppSettings>("get_settings"),
        invoke<boolean>("is_self_shielded").catch(() => false),
      ]);

      setWindows(wins);
      setIsSelfShielded(selfShielded);
      if (settings.widget_style) {
        setWidgetStyle(settings.widget_style as any);
      }
      if (settings.theme) {
        setTheme(settings.theme as ThemeType);
        applyDynamicTheme(settings.theme as ThemeType);
      }

      const active = new Set<string>();
      for (const w of wins) {
        if (w.is_shielded) active.add(w.exe_name);
      }
      setShieldedExes(active);
    } catch (e) {
      console.error("Widget refresh error:", e);
    }
  }, []);

  useEffect(() => {
    refreshState();
    const interval = setInterval(refreshState, 3000);

    const unlistenStyle = listen<string>("widget_style_changed", (event) => {
      setWidgetStyle(event.payload as any);
    });

    const unlistenTray = listen("tray_action_completed", () => {
      refreshState();
    });

    return () => {
      clearInterval(interval);
      unlistenStyle.then((u) => u());
      unlistenTray.then((u) => u());
    };
  }, [refreshState]);

  // Window drag via native Win32 message
  const handleDrag = (e: React.MouseEvent) => {
    if (e.button === 0 && !(e.target as HTMLElement).closest("button, input, select, a, .no-drag")) {
      invoke("start_window_drag", { windowLabel: "widget" }).catch(() => {
        getCurrentWindow().startDragging();
      });
    }
  };

  const shieldedCount = windows.filter((w) => shieldedExes.has(w.exe_name)).length;
  const allShielded = windows.length > 0 && shieldedCount === windows.length;
  const anyShielded = shieldedCount > 0;

  const handleToggleApp = async (win: WindowInfo) => {
    const next = !shieldedExes.has(win.exe_name);
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
    } catch {
      refreshState();
    }
  };

  const handleMasterToggle = async () => {
    const enable = !allShielded;
    const targetWins = windows.filter((w) => (enable ? !shieldedExes.has(w.exe_name) : shieldedExes.has(w.exe_name)));
    for (const w of targetWins) {
      await handleToggleApp(w);
    }
  };

  const handleToggleSelfShield = async () => {
    const next = !isSelfShielded;
    try {
      await invoke("toggle_self_shield", { enable: next });
      setIsSelfShielded(next);
    } catch (e) {
      console.error("Self shield error:", e);
    }
  };

  const handleOpenMainWindow = async () => {
    try {
      await invoke("show_main_window");
    } catch {
      // If already exists, focus it
    }
  };

  const handleCloseWidget = async () => {
    try {
      await invoke("toggle_widget", { enable: false });
    } catch (e) {
      console.error(e);
    }
  };

  // ── 1. DYNAMIC PILL / ISLAND MODE ─────────────────────────
  if (widgetStyle === "pill") {
    return (
      <div className="desktop-widget-root pill-mode" onMouseDown={handleDrag}>
        <div className="widget-pill-container">
          {/* Logo & Shielded Count */}
          <div className="pill-brand-badge" onClick={handleOpenMainWindow} title="Click to open StreamShield">
            <img src="/logo.png" className="widget-pill-logo" alt="StreamShield" />
            <span className={`pill-counter-tag ${allShielded ? "all-safe" : anyShielded ? "partial" : ""}`}>
              {shieldedCount}/{windows.length}
            </span>
          </div>

          {/* Mini Avatars of Protected Apps */}
          <div className="pill-active-apps-deck">
            {windows.slice(0, 5).map((w) => {
              const isSh = shieldedExes.has(w.exe_name);
              const clean = w.exe_name.replace(/\.exe$/i, "");
              return (
                <button
                  key={`${w.pid}-${w.hwnd}`}
                  className={`pill-mini-app-tile ${isSh ? "is-shielded" : ""}`}
                  onClick={() => handleToggleApp(w)}
                  title={`${clean}: ${isSh ? "Protected (Tap to unshield)" : "Open (Tap to shield)"}`}
                  type="button"
                >
                  {w.icon_base64 ? (
                    <img src={w.icon_base64} alt={clean} className="pill-app-icon" />
                  ) : (
                    <span className="pill-app-initial">{clean[0]?.toUpperCase() || "?"}</span>
                  )}
                  {isSh && <span className="pill-mini-shield-dot" />}
                </button>
              );
            })}
          </div>

          {/* Action Suite */}
          <div className="pill-action-cluster">
            {/* Master Toggle */}
            <button
              className={`pill-master-btn ${allShielded ? "is-active" : anyShielded ? "is-partial" : ""}`}
              onClick={handleMasterToggle}
              title={allShielded ? "Unshield all applications" : "Shield all open applications"}
              type="button"
            >
              <IconShield size={12} />
            </button>

            {/* Stealth Eye */}
            <button
              className={`pill-eye-btn ${isSelfShielded ? "is-stealth" : ""}`}
              onClick={handleToggleSelfShield}
              title={isSelfShielded ? "App hidden from stream (Click to show)" : "App visible on stream (Click to hide)"}
              type="button"
            >
              {isSelfShielded ? <IconEyeOff size={13} /> : <IconEye size={13} />}
            </button>

            {/* Open Full Window */}
            <button
              className="pill-window-btn"
              onClick={handleOpenMainWindow}
              title="Open StreamShield Full Window"
              type="button"
            >
              <IconExternalLink size={12} />
            </button>

            {/* Hide Widget */}
            <button
              className="pill-close-btn"
              onClick={handleCloseWidget}
              title="Hide Desktop Widget"
              type="button"
            >
              <IconClose size={11} />
            </button>
          </div>
        </div>
      </div>
    );
  }

  // ── 2. FLOATING COMPACT DOCK MODE ─────────────────────────
  if (widgetStyle === "dock") {
    return (
      <div className="desktop-widget-root dock-mode" onMouseDown={handleDrag}>
        <div className="widget-dock-container">
          {/* Dock Top Strip */}
          <div className="widget-dock-header">
            <div className="dock-header-left" onClick={handleOpenMainWindow}>
              <img src="/logo.png" className="widget-dock-logo" alt="StreamShield" />
              <span className="dock-count-badge">
                {shieldedCount} protected
              </span>
            </div>

            <div className="dock-header-right">
              <button
                className={`dock-tool-btn ${allShielded ? "is-active" : anyShielded ? "is-partial" : ""}`}
                onClick={handleMasterToggle}
                title={allShielded ? "Unshield all" : "Shield all"}
                type="button"
              >
                <IconShield size={11} />
              </button>

              <button
                className={`dock-tool-btn ${isSelfShielded ? "is-stealth" : ""}`}
                onClick={handleToggleSelfShield}
                title={isSelfShielded ? "Stealth active" : "Stealth off"}
                type="button"
              >
                {isSelfShielded ? <IconEyeOff size={12} /> : <IconEye size={12} />}
              </button>

              <button
                className="dock-tool-btn"
                onClick={handleOpenMainWindow}
                title="Open Window"
                type="button"
              >
                <IconExternalLink size={11} />
              </button>

              <button
                className="dock-tool-btn close"
                onClick={handleCloseWidget}
                title="Close Widget"
                type="button"
              >
                <IconClose size={10} />
              </button>
            </div>
          </div>

          {/* Dock Icon Grid */}
          <div className="widget-dock-grid">
            {windows.slice(0, 8).map((w) => {
              const isSh = shieldedExes.has(w.exe_name);
              const clean = w.exe_name.replace(/\.exe$/i, "");
              return (
                <button
                  key={`${w.pid}-${w.hwnd}`}
                  className={`dock-app-tile ${isSh ? "is-shielded" : ""}`}
                  onClick={() => handleToggleApp(w)}
                  title={`${clean}: ${isSh ? "Shielded (Tap to unshield)" : "Open (Tap to shield)"}`}
                  type="button"
                >
                  {w.icon_base64 ? (
                    <img src={w.icon_base64} alt={clean} className="dock-app-icon" />
                  ) : (
                    <span className="dock-app-initial">{clean[0]?.toUpperCase() || "?"}</span>
                  )}
                  {isSh && (
                    <span className="dock-shield-badge">
                      <IconShieldCheck size={9} />
                    </span>
                  )}
                </button>
              );
            })}
          </div>
        </div>
      </div>
    );
  }

  // ── 3. CORNER STATUS BADGE MODE ───────────────────────────
  return (
    <div className="desktop-widget-root badge-mode" onMouseDown={handleDrag}>
      <div
        className={`widget-badge-circle ${allShielded ? "is-all" : anyShielded ? "is-partial" : ""}`}
        onClick={() => setBadgeExpanded(!badgeExpanded)}
        title="StreamShield Badge (Click for controls)"
      >
        <img src="/logo.png" className="widget-badge-logo" alt="StreamShield" />
        <span className="badge-count-pill">{shieldedCount}</span>
      </div>

      {badgeExpanded && (
        <div className="badge-quick-popover" onMouseDown={(e) => e.stopPropagation()}>
          <button
            className="badge-popover-item"
            onClick={handleMasterToggle}
            type="button"
          >
            <IconShield size={13} />
            <span>{allShielded ? "Clear All" : "Shield All"}</span>
          </button>

          <button
            className={`badge-popover-item ${isSelfShielded ? "stealth-active" : ""}`}
            onClick={handleToggleSelfShield}
            type="button"
          >
            {isSelfShielded ? <IconEyeOff size={13} /> : <IconEye size={13} />}
            <span>Stealth: {isSelfShielded ? "ON" : "OFF"}</span>
          </button>

          <button
            className="badge-popover-item"
            onClick={handleOpenMainWindow}
            type="button"
          >
            <IconExternalLink size={13} />
            <span>Open Window</span>
          </button>

          <button
            className="badge-popover-item close"
            onClick={handleCloseWidget}
            type="button"
          >
            <IconClose size={13} />
            <span>Hide Widget</span>
          </button>
        </div>
      )}
    </div>
  );
}
