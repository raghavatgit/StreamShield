import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ThemeType } from "../App";
import {
  IconGear,
  IconStartup,
  IconShield,
  IconScanning,
  IconDisplay,
  IconTools,
  IconInfo,
  IconAlert,
  IconClose,
  IconCheck,
} from "./Icons";

export interface AppSettings {
  autostart: boolean;
  start_minimized: boolean;
  auto_reapply: boolean;
  shield_mode: "exclude" | "monitor";
  poll_interval_ms: number;
  theme: ThemeType;
  compact_mode: boolean;
  show_pid: boolean;
  confirm_batch: boolean;
  mpo_fix: boolean;
  self_stealth: boolean;
}

interface ThemeChoice {
  id: ThemeType;
  name: string;
  colors: string[];
}

const THEMES: ThemeChoice[] = [
  { id: "discord", name: "Discord Dark", colors: ["#5865f2", "#23a55a"] },
  { id: "cyberpunk", name: "Cyberpunk", colors: ["#00f0ff", "#ff007f"] },
  { id: "neumorphic-white", name: "Clean White", colors: ["#ffffff", "#4f46e5"] },
];

const SCAN_INTERVALS = [
  { value: 2000, label: "2 seconds (Fast & Responsive)" },
  { value: 3000, label: "3 seconds (Recommended)" },
  { value: 5000, label: "5 seconds (Balanced / Low CPU)" },
  { value: 10000, label: "10 seconds (Battery Saver)" },
  { value: 0, label: "Manual Refresh Only" },
];

interface Props {
  isOpen: boolean;
  onClose: () => void;
  settings: AppSettings;
  onUpdateSettings: (newSettings: AppSettings) => Promise<void>;
  onResetSettings: () => Promise<void>;
  onClearAllShields: () => Promise<void>;
  shieldedCount: number;
  totalWindows: number;
  isAdmin: boolean;
}

export default function SettingsModal({
  isOpen,
  onClose,
  settings,
  onUpdateSettings,
  onResetSettings,
  onClearAllShields,
  shieldedCount,
  totalWindows,
  isAdmin,
}: Props) {
  const [activeTab, setActiveTab] = useState<"system" | "shield" | "performance" | "ui" | "maintenance">("system");
  const [confirmClearOpen, setConfirmClearOpen] = useState(false);
  const [confirmResetOpen, setConfirmResetOpen] = useState(false);
  const [saving, setSaving] = useState(false);
  const [patchingNvidia, setPatchingNvidia] = useState(false);
  const [nvidiaPatchResult, setNvidiaPatchResult] = useState<string | null>(null);

  // Close on Escape
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        if (confirmClearOpen) setConfirmClearOpen(false);
        else if (confirmResetOpen) setConfirmResetOpen(false);
        else onClose();
      }
    };
    if (isOpen) {
      window.addEventListener("keydown", handleKeyDown);
    }
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, confirmClearOpen, confirmResetOpen, onClose]);

  if (!isOpen) return null;

  const handleToggle = async <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => {
    setSaving(true);
    try {
      const updated = { ...settings, [key]: value };
      await onUpdateSettings(updated);
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="settings-modal-backdrop" onClick={onClose}>
      <div className="settings-modal-card" onClick={(e) => e.stopPropagation()}>
        {/* Header */}
        <div className="settings-modal-header">
          <div className="settings-header-title-group">
            <div className="settings-header-glyph-box">
              <IconGear size={18} className="settings-header-glyph" />
            </div>
            <div>
              <h2 className="settings-title-text">StreamShield Preferences</h2>
              <p className="settings-subtitle-text">Engine configuration, startup behavior & aesthetics</p>
            </div>
          </div>
          <button className="settings-close-cross" onClick={onClose} title="Close (Esc)">
            <IconClose size={16} />
          </button>
        </div>

        {/* Navigation Tabs */}
        <div className="settings-nav-strip">
          <button
            className={`settings-nav-btn ${activeTab === "system" ? "is-active" : ""}`}
            onClick={() => setActiveTab("system")}
          >
            <IconStartup size={14} className="nav-btn-icon" />
            <span>Startup</span>
          </button>
          <button
            className={`settings-nav-btn ${activeTab === "shield" ? "is-active" : ""}`}
            onClick={() => setActiveTab("shield")}
          >
            <IconShield size={14} className="nav-btn-icon" />
            <span>Engine</span>
          </button>
          <button
            className={`settings-nav-btn ${activeTab === "performance" ? "is-active" : ""}`}
            onClick={() => setActiveTab("performance")}
          >
            <IconScanning size={14} className="nav-btn-icon" />
            <span>Scanning</span>
          </button>
          <button
            className={`settings-nav-btn ${activeTab === "ui" ? "is-active" : ""}`}
            onClick={() => setActiveTab("ui")}
          >
            <IconDisplay size={14} className="nav-btn-icon" />
            <span>Display</span>
          </button>
          <button
            className={`settings-nav-btn ${activeTab === "maintenance" ? "is-active" : ""}`}
            onClick={() => setActiveTab("maintenance")}
          >
            <IconTools size={14} className="nav-btn-icon" />
            <span>Tools</span>
          </button>
        </div>

        {/* Body Container */}
        <div className="settings-modal-body">
          {/* TAB 1: SYSTEM & STARTUP */}
          {activeTab === "system" && (
            <div className="settings-section-pane">
              <div className="setting-row-item">
                <div className="setting-meta">
                  <span className="setting-label">Launch on Windows Startup</span>
                  <span className="setting-subtext">
                    Automatically starts StreamShield when you log in to Windows
                  </span>
                </div>
                <label className="toggle-control">
                  <input
                    type="checkbox"
                    checked={settings.autostart}
                    onChange={(e) => handleToggle("autostart", e.target.checked)}
                    disabled={saving}
                  />
                  <span className="toggle-rail">
                    <span className="toggle-knob" />
                  </span>
                </label>
              </div>

              <div className="setting-row-item">
                <div className="setting-meta">
                  <span className="setting-label">Start Minimized to System Tray</span>
                  <span className="setting-subtext">
                    Keep the main window hidden when StreamShield launches in the background
                  </span>
                </div>
                <label className="toggle-control">
                  <input
                    type="checkbox"
                    checked={settings.start_minimized}
                    onChange={(e) => handleToggle("start_minimized", e.target.checked)}
                    disabled={saving}
                  />
                  <span className="toggle-rail">
                    <span className="toggle-knob" />
                  </span>
                </label>
              </div>

              <div className="setting-row-item">
                <div className="setting-meta">
                  <span className="setting-label">Shield StreamShield on Launch (App Stealth)</span>
                  <span className="setting-subtext">
                    Hides the StreamShield application window from all screen captures on startup
                  </span>
                </div>
                <label className="toggle-control">
                  <input
                    type="checkbox"
                    checked={settings.self_stealth}
                    onChange={(e) => handleToggle("self_stealth", e.target.checked)}
                    disabled={saving}
                  />
                  <span className="toggle-rail">
                    <span className="toggle-knob" />
                  </span>
                </label>
              </div>

              <div className="settings-callout-box info">
                <IconInfo size={16} className="callout-svg" />
                <span className="callout-text">
                  StreamShield runs with elevated Administrator privileges for hardware capture hook stability.
                </span>
              </div>
            </div>
          )}

          {/* TAB 2: PRIVACY ENGINE */}
          {activeTab === "shield" && (
            <div className="settings-section-pane">
              <div className="setting-stacked-block">
                <div className="setting-meta">
                  <span className="setting-label">Capture Exclusion Affinity Mode</span>
                  <span className="setting-subtext">
                    Select how OS-level display affinity masks shielded application windows
                  </span>
                </div>

                <div className="shield-mode-options-grid">
                  <div
                    className={`shield-mode-card ${settings.shield_mode === "exclude" ? "is-selected" : ""}`}
                    onClick={() => handleToggle("shield_mode", "exclude")}
                  >
                    <div className="shield-mode-card-header">
                      <span className="mode-tag">Invisible / Transparent</span>
                      {settings.shield_mode === "exclude" && <IconCheck size={14} className="mode-check" />}
                    </div>
                    <div className="shield-mode-title">Exclude from Capture</div>
                    <div className="shield-mode-desc">
                      Protected windows vanish entirely from stream and OBS captures. Transparent to viewers.
                    </div>
                  </div>

                  <div
                    className={`shield-mode-card ${settings.shield_mode === "monitor" ? "is-selected" : ""}`}
                    onClick={() => handleToggle("shield_mode", "monitor")}
                  >
                    <div className="shield-mode-card-header">
                      <span className="mode-tag">Black Out Mask</span>
                      {settings.shield_mode === "monitor" && <IconCheck size={14} className="mode-check" />}
                    </div>
                    <div className="shield-mode-title">Black Screen Mask</div>
                    <div className="shield-mode-desc">
                      Protected windows appear as solid black rectangles in capture output.
                    </div>
                  </div>
                </div>
              </div>

              <div className="setting-row-item">
                <div className="setting-meta">
                  <span className="setting-label">Auto-Reapply Shields in Background</span>
                  <span className="setting-subtext">
                    Automatically injects protection when you open new windows or restart protected apps
                  </span>
                </div>
                <label className="toggle-control">
                  <input
                    type="checkbox"
                    checked={settings.auto_reapply}
                    onChange={(e) => handleToggle("auto_reapply", e.target.checked)}
                    disabled={saving}
                  />
                  <span className="toggle-rail">
                    <span className="toggle-knob" />
                  </span>
                </label>
              </div>

              <div className="setting-row-item">
                <div className="setting-meta">
                  <span className="setting-label">NVIDIA ShadowPlay & Overlay Bypass (MPO Fix)</span>
                  <span className="setting-subtext">
                    Bypasses NVIDIA ShadowPlay capture restrictions and prevents hardware MPO overlay leakage so Instant Replay and Desktop Capture cleanly exclude shielded apps without blocking recording
                  </span>
                </div>
                <label className="toggle-control">
                  <input
                    type="checkbox"
                    checked={settings.mpo_fix}
                    onChange={(e) => handleToggle("mpo_fix", e.target.checked)}
                    disabled={saving}
                  />
                  <span className="toggle-rail">
                    <span className="toggle-knob" />
                  </span>
                </label>
              </div>

              <div className="setting-row-item">
                <div className="setting-meta">
                  <span className="setting-label">Live NVIDIA DRM Scanner Bypass</span>
                  <span className="setting-subtext">
                    Live-patches NVIDIA capture processes (nvcontainer & NVIDIA Share) to disable DRM killswitches and allow screen recording with private apps hidden
                  </span>
                  {nvidiaPatchResult && (
                    <span className="setting-subtext" style={{ color: "var(--accent-cyan, #00f0ff)", marginTop: "4px", fontWeight: 600 }}>
                      {nvidiaPatchResult}
                    </span>
                  )}
                </div>
                <button
                  className="settings-action-btn secondary"
                  onClick={async () => {
                    setPatchingNvidia(true);
                    try {
                      const count = await invoke<number>("apply_nvidia_bypass");
                      setNvidiaPatchResult(`Successfully patched ${count} NVIDIA capture process${count === 1 ? "" : "es"}`);
                    } catch (e) {
                      setNvidiaPatchResult(`Error: ${e}`);
                    } finally {
                      setPatchingNvidia(false);
                    }
                  }}
                  disabled={patchingNvidia}
                  style={{ minWidth: "150px" }}
                >
                  {patchingNvidia ? "Patching..." : "Bypass NVIDIA Now"}
                </button>
              </div>
            </div>
          )}

          {/* TAB 3: PERFORMANCE */}
          {activeTab === "performance" && (
            <div className="settings-section-pane">
              <div className="setting-row-item">
                <div className="setting-meta">
                  <span className="setting-label">Window Scan Interval</span>
                  <span className="setting-subtext">
                    Controls how frequently StreamShield checks for newly opened and closed windows
                  </span>
                </div>

                <div className="setting-control-right">
                  <select
                    className="settings-select-input"
                    value={settings.poll_interval_ms}
                    onChange={(e) => handleToggle("poll_interval_ms", Number(e.target.value))}
                    disabled={saving}
                  >
                    {SCAN_INTERVALS.map((opt) => (
                      <option key={opt.value} value={opt.value}>
                        {opt.label}
                      </option>
                    ))}
                  </select>
                </div>
              </div>

              <div className="settings-callout-box info">
                <IconInfo size={16} className="callout-svg" />
                <span className="callout-text">
                  StreamShield uses native Win32 window event debouncing to minimize CPU and power consumption.
                </span>
              </div>
            </div>
          )}

          {/* TAB 4: DISPLAY & UI */}
          {activeTab === "ui" && (
            <div className="settings-section-pane">
              {/* Theme Grid */}
              <div className="setting-stacked-block">
                <div className="setting-meta">
                  <span className="setting-label">Theme Preset</span>
                  <span className="setting-subtext">Choose your visual color palette and ambient glow</span>
                </div>

                <div className="theme-selection-cards">
                  {THEMES.map((t) => (
                    <div
                      key={t.id}
                      className={`theme-palette-card ${settings.theme === t.id ? "is-selected" : ""}`}
                      onClick={() => handleToggle("theme", t.id)}
                    >
                      <div className="theme-preview-dots">
                        <span style={{ background: t.colors[0] }} />
                        <span style={{ background: t.colors[1] }} />
                      </div>
                      <span className="theme-card-name">{t.name}</span>
                      {settings.theme === t.id && <span className="theme-card-badge">Active</span>}
                    </div>
                  ))}
                </div>
              </div>

              <div className="setting-row-item">
                <div className="setting-meta">
                  <span className="setting-label">Compact Card Mode</span>
                  <span className="setting-subtext">
                    Condensed list layout for managing dozens of applications easily
                  </span>
                </div>
                <label className="toggle-control">
                  <input
                    type="checkbox"
                    checked={settings.compact_mode}
                    onChange={(e) => handleToggle("compact_mode", e.target.checked)}
                    disabled={saving}
                  />
                  <span className="toggle-rail">
                    <span className="toggle-knob" />
                  </span>
                </label>
              </div>

              <div className="setting-row-item">
                <div className="setting-meta">
                  <span className="setting-label">Show Process PID Badges</span>
                  <span className="setting-subtext">
                    Display technical Windows Process ID tag next to executable names
                  </span>
                </div>
                <label className="toggle-control">
                  <input
                    type="checkbox"
                    checked={settings.show_pid}
                    onChange={(e) => handleToggle("show_pid", e.target.checked)}
                    disabled={saving}
                  />
                  <span className="toggle-rail">
                    <span className="toggle-knob" />
                  </span>
                </label>
              </div>

              <div className="setting-row-item">
                <div className="setting-meta">
                  <span className="setting-label">Confirm Batch Actions</span>
                  <span className="setting-subtext">
                    Prompt confirmation before executing 'Shield All' or 'Clear'
                  </span>
                </div>
                <label className="toggle-control">
                  <input
                    type="checkbox"
                    checked={settings.confirm_batch}
                    onChange={(e) => handleToggle("confirm_batch", e.target.checked)}
                    disabled={saving}
                  />
                  <span className="toggle-rail">
                    <span className="toggle-knob" />
                  </span>
                </label>
              </div>
            </div>
          )}

          {/* TAB 5: MAINTENANCE & RESET */}
          {activeTab === "maintenance" && (
            <div className="settings-section-pane">
              <div className="maintenance-action-card">
                <div className="maintenance-meta">
                  <div className="maintenance-title">Clear All Shielded Applications</div>
                  <div className="maintenance-desc">
                    Immediately unshields all {shieldedCount} protected app(s) and restores standard capture visibility.
                  </div>
                </div>
                <button
                  className="danger-outline-btn"
                  onClick={() => setConfirmClearOpen(true)}
                  disabled={shieldedCount === 0}
                >
                  Clear All ({shieldedCount})
                </button>
              </div>

              <div className="maintenance-action-card">
                <div className="maintenance-meta">
                  <div className="maintenance-title">Restore Factory Default Settings</div>
                  <div className="maintenance-desc">
                    Resets startup, engine parameters, scan interval, and theme preferences to defaults.
                  </div>
                </div>
                <button className="warning-outline-btn" onClick={() => setConfirmResetOpen(true)}>
                  Reset Defaults
                </button>
              </div>

              <div className="app-diagnostics-footer">
                <div className="diag-row">
                  <span className="diag-key">Publisher:</span>
                  <span className="diag-val">Raghav Goyal</span>
                </div>
                <div className="diag-row">
                  <span className="diag-key">Version:</span>
                  <span className="diag-val">v0.2.0</span>
                </div>
                <div className="diag-row">
                  <span className="diag-key">Privileges:</span>
                  <span className="diag-val">{isAdmin ? "Administrator" : "Standard"}</span>
                </div>
                <div className="diag-row">
                  <span className="diag-key">Windows:</span>
                  <span className="diag-val">{totalWindows} detected</span>
                </div>
              </div>
            </div>
          )}
        </div>

        {/* Confirmation Modal Overlays */}
        {confirmClearOpen && (
          <div className="settings-confirm-overlay" onClick={() => setConfirmClearOpen(false)}>
            <div className="settings-confirm-card" onClick={(e) => e.stopPropagation()}>
              <div className="confirm-icon-row">
                <IconAlert size={22} className="confirm-alert-icon" />
                <h3 className="confirm-title">Clear all shielded apps?</h3>
              </div>
              <p className="confirm-text">
                This will unshield all {shieldedCount} protected application windows, making them visible to all screen captures.
              </p>
              <div className="confirm-btn-row">
                <button className="confirm-cancel-btn" onClick={() => setConfirmClearOpen(false)}>
                  Cancel
                </button>
                <button
                  className="confirm-danger-btn"
                  onClick={async () => {
                    await onClearAllShields();
                    setConfirmClearOpen(false);
                  }}
                >
                  Yes, Unshield All
                </button>
              </div>
            </div>
          </div>
        )}

        {confirmResetOpen && (
          <div className="settings-confirm-overlay" onClick={() => setConfirmResetOpen(false)}>
            <div className="settings-confirm-card" onClick={(e) => e.stopPropagation()}>
              <div className="confirm-icon-row">
                <IconAlert size={22} className="confirm-alert-icon" />
                <h3 className="confirm-title">Reset all settings to default?</h3>
              </div>
              <p className="confirm-text">
                This will restore all preferences (startup, theme, scan interval, shield mode) to their initial values.
              </p>
              <div className="confirm-btn-row">
                <button className="confirm-cancel-btn" onClick={() => setConfirmResetOpen(false)}>
                  Cancel
                </button>
                <button
                  className="confirm-danger-btn"
                  onClick={async () => {
                    await onResetSettings();
                    setConfirmResetOpen(false);
                  }}
                >
                  Yes, Reset Defaults
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
