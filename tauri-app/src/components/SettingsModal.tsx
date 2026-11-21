import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ThemeType, THEMES, applyDynamicTheme } from "../utils/themeEngine";
import { checkForAppUpdate, CURRENT_VERSION, UpdateInfo } from "../utils/updater";
import {
  IconGear,
  IconShield,
  IconDisplay,
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
  widget_enabled: boolean;
  widget_style: "pill" | "dock" | "badge";
  widget_always_on_top: boolean;
  widget_stealth: boolean;
  widget_x?: number | null;
  widget_y?: number | null;
  audio_isolation_enabled?: boolean;
  audio_stream_device_id?: string | null;
  audio_monitor_device_id?: string | null;
}

export interface AudioDeviceInfo {
  id: string;
  name: string;
  is_default: boolean;
  is_virtual: boolean;
}

export interface OsInfo {
  build: number;
  is_win11: boolean;
  supports_exclude_from_capture: boolean;
  display_name: string;
}

export interface CaptureEnvironment {
  nvidia_detected: boolean;
  obs_detected: boolean;
  discord_detected: boolean;
  medal_detected: boolean;
  mpo_active: boolean;
  recommended_mode: string;
  summary: string;
}

const SCAN_INTERVALS = [
  { value: 2000, label: "2s (Fast)" },
  { value: 3000, label: "3s (Default)" },
  { value: 5000, label: "5s (Balanced)" },
  { value: 10000, label: "10s (Eco)" },
  { value: 0, label: "Manual" },
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
  osInfo?: OsInfo | null;
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
  osInfo: propOsInfo,
}: Props) {
  const [activeTab, setActiveTab] = useState<"general" | "protection" | "appearance">("general");
  const [confirmClearOpen, setConfirmClearOpen] = useState(false);
  const [confirmResetOpen, setConfirmResetOpen] = useState(false);
  const [saving, setSaving] = useState(false);
  const [captureEnv, setCaptureEnv] = useState<CaptureEnvironment | null>(null);
  const [audioDevices, setAudioDevices] = useState<AudioDeviceInfo[]>([]);
  const [localOsInfo, setLocalOsInfo] = useState<OsInfo | null>(propOsInfo ?? null);
  const osInfo = propOsInfo ?? localOsInfo;

  const [updateChecking, setUpdateChecking] = useState(false);
  const [updateResult, setUpdateResult] = useState<string | null>(null);
  const [availableUpdate, setAvailableUpdate] = useState<UpdateInfo | null>(null);

  const handleCheckForUpdates = async () => {
    setUpdateChecking(true);
    setUpdateResult(null);
    try {
      const info = await checkForAppUpdate();
      if (info.hasUpdate) {
        setAvailableUpdate(info);
        setUpdateResult(`v${info.latestVersion} available`);
      } else {
        setAvailableUpdate(null);
        setUpdateResult("Up to date");
      }
    } catch {
      setUpdateResult("Offline");
    } finally {
      setUpdateChecking(false);
    }
  };

  useEffect(() => {
    if (isOpen) {
      invoke<CaptureEnvironment>("get_capture_environment")
        .then((env) => setCaptureEnv(env))
        .catch(() => {});
      invoke<AudioDeviceInfo[]>("get_audio_devices")
        .then((devs) => setAudioDevices(devs))
        .catch(() => {});
      if (!propOsInfo) {
        invoke<OsInfo>("get_os_info")
          .then((info) => setLocalOsInfo(info))
          .catch(() => {});
      }
    }
  }, [isOpen, propOsInfo]);

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
      const env = await invoke<CaptureEnvironment>("get_capture_environment").catch(() => null);
      if (env) setCaptureEnv(env);
    } finally {
      setSaving(false);
    }
  };

  const handleSelectWidgetStyle = async (style: "pill" | "dock" | "badge") => {
    setSaving(true);
    try {
      const updated: AppSettings = { ...settings, widget_style: style, widget_enabled: true };
      await onUpdateSettings(updated);
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="settings-modal-backdrop" onClick={onClose}>
      <div className="settings-modal-card" onClick={(e) => e.stopPropagation()}>
        {/* Compact Header */}
        <div className="settings-modal-header">
          <div className="settings-header-title-group">
            <div className="settings-header-glyph-box">
              <IconGear size={16} />
            </div>
            <h2 className="settings-title-text">Preferences</h2>
          </div>
          <button type="button" className="settings-close-cross" onClick={onClose} title="Close (Esc)" aria-label="Close preferences">
            <IconClose size={15} />
          </button>
        </div>

        {/* Clean 3-Tab Navigation */}
        <div className="settings-nav-strip three-tabs" role="tablist">
          <button
            type="button"
            role="tab"
            aria-selected={activeTab === "general"}
            className={`settings-nav-btn ${activeTab === "general" ? "is-active" : ""}`}
            onClick={() => setActiveTab("general")}
          >
            <IconGear size={13} className="nav-btn-icon" />
            <span>General</span>
          </button>
          <button
            type="button"
            role="tab"
            aria-selected={activeTab === "protection"}
            className={`settings-nav-btn ${activeTab === "protection" ? "is-active" : ""}`}
            onClick={() => setActiveTab("protection")}
          >
            <IconShield size={13} className="nav-btn-icon" />
            <span>Protection</span>
          </button>
          <button
            type="button"
            role="tab"
            aria-selected={activeTab === "appearance"}
            className={`settings-nav-btn ${activeTab === "appearance" ? "is-active" : ""}`}
            onClick={() => setActiveTab("appearance")}
          >
            <IconDisplay size={13} className="nav-btn-icon" />
            <span>Appearance</span>
          </button>
        </div>

        {/* Body */}
        <div className="settings-modal-body">
          {/* TAB 1: GENERAL */}
          {activeTab === "general" && (
            <div className="settings-section-pane">
              <div className="settings-section-heading">Behavior</div>
              <div className="settings-group-list">
                <div className="settings-group-row">
                  <div className="setting-meta">
                    <span className="setting-label">Launch at login</span>
                    <span className="setting-subtext">Start StreamShield with Windows</span>
                  </div>
                  <label className="toggle-control">
                    <input
                      type="checkbox"
                      checked={settings.autostart}
                      onChange={(e) => handleToggle("autostart", e.target.checked)}
                      disabled={saving}
                    />
                    <span className="toggle-rail"><span className="toggle-knob" /></span>
                  </label>
                </div>

                <div className="settings-group-row">
                  <div className="setting-meta">
                    <span className="setting-label">Start in tray</span>
                    <span className="setting-subtext">Keep window hidden on launch</span>
                  </div>
                  <label className="toggle-control">
                    <input
                      type="checkbox"
                      checked={settings.start_minimized}
                      onChange={(e) => handleToggle("start_minimized", e.target.checked)}
                      disabled={saving}
                    />
                    <span className="toggle-rail"><span className="toggle-knob" /></span>
                  </label>
                </div>

                <div className="settings-group-row">
                  <div className="setting-meta">
                    <span className="setting-label">App stealth on launch</span>
                    <span className="setting-subtext">Hide StreamShield itself from captures</span>
                  </div>
                  <label className="toggle-control">
                    <input
                      type="checkbox"
                      checked={settings.self_stealth}
                      onChange={(e) => handleToggle("self_stealth", e.target.checked)}
                      disabled={saving}
                    />
                    <span className="toggle-rail"><span className="toggle-knob" /></span>
                  </label>
                </div>

                <div className="settings-group-row">
                  <div className="setting-meta">
                    <span className="setting-label">Scan interval</span>
                    <span className="setting-subtext">Window detection frequency</span>
                  </div>
                  <select
                    className="settings-select-input"
                    value={settings.poll_interval_ms}
                    onChange={(e) => handleToggle("poll_interval_ms", Number(e.target.value))}
                    disabled={saving}
                  >
                    {SCAN_INTERVALS.map((item) => (
                      <option key={item.value} value={item.value}>{item.label}</option>
                    ))}
                  </select>
                </div>
              </div>

              <div className="settings-section-heading">Maintenance</div>
              <div className="settings-action-strip">
                <button
                  type="button"
                  className="danger-outline-btn"
                  onClick={() => setConfirmClearOpen(true)}
                  disabled={shieldedCount === 0}
                >
                  Clear Shields ({shieldedCount})
                </button>
                <button
                  type="button"
                  className="warning-outline-btn"
                  onClick={() => setConfirmResetOpen(true)}
                >
                  Reset Defaults
                </button>
                <button
                  type="button"
                  className="warning-outline-btn"
                  onClick={handleCheckForUpdates}
                  disabled={updateChecking}
                >
                  {updateChecking ? "Checking..." : updateResult || "Check Updates"}
                </button>
                {availableUpdate?.releaseUrl && (
                  <button
                    type="button"
                    className="diag-download-btn"
                    onClick={() => invoke("open_external_url", { url: availableUpdate.releaseUrl })}
                  >
                    Get v{availableUpdate.latestVersion}
                  </button>
                )}
              </div>

              <div className="settings-meta-footer">
                <span>v{CURRENT_VERSION}</span>
                <span>•</span>
                <span>{osInfo ? osInfo.display_name : "Windows"}</span>
                <span>•</span>
                <span>{isAdmin ? "Admin" : "Standard"}</span>
                <span>•</span>
                <span>{totalWindows} windows</span>
              </div>
            </div>
          )}

          {/* TAB 2: PROTECTION */}
          {activeTab === "protection" && (
            <div className="settings-section-pane">
              {captureEnv && (captureEnv.nvidia_detected || captureEnv.medal_detected || captureEnv.obs_detected || captureEnv.discord_detected) && (
                <div className={`capture-env-pill ${captureEnv.nvidia_detected || captureEnv.medal_detected ? "nvidia-active" : ""}`}>
                  <IconShield size={13} />
                  <span>
                    {captureEnv.nvidia_detected
                      ? "NVIDIA Capture active - Black Mask recommended"
                      : captureEnv.medal_detected
                      ? "Medal.tv active - Black Mask recommended"
                      : "OBS / Discord active - Transparent recommended"}
                  </span>
                </div>
              )}

              <div className="settings-section-heading">Capture Mask Mode</div>
              <div className="shield-mode-options-grid">
                <button
                  type="button"
                  className={`shield-mode-card ${settings.shield_mode === "exclude" ? "is-selected" : ""}`}
                  onClick={() => handleToggle("shield_mode", "exclude")}
                >
                  <div className="shield-mode-card-header">
                    <span className="mode-tag">OBS & Discord</span>
                    {settings.shield_mode === "exclude" && <IconCheck size={13} className="mode-check" />}
                  </div>
                  <div className="shield-mode-title">Transparent</div>
                  <div className="shield-mode-desc">Windows vanish completely from capture.</div>
                </button>

                <button
                  type="button"
                  className={`shield-mode-card ${settings.shield_mode === "monitor" ? "is-selected" : ""}`}
                  onClick={() => handleToggle("shield_mode", "monitor")}
                >
                  <div className="shield-mode-card-header">
                    <span className="mode-tag highlight">ShadowPlay / Medal</span>
                    {settings.shield_mode === "monitor" && <IconCheck size={13} className="mode-check" />}
                  </div>
                  <div className="shield-mode-title">Black Mask</div>
                  <div className="shield-mode-desc">Solid black box; prevents DRM pauses.</div>
                </button>
              </div>

              <div className="settings-section-heading">Engine</div>
              <div className="settings-group-list">
                <div className="settings-group-row">
                  <div className="setting-meta">
                    <span className="setting-label">Auto-reapply shields</span>
                    <span className="setting-subtext">Re-shield apps when restarted</span>
                  </div>
                  <label className="toggle-control">
                    <input
                      type="checkbox"
                      checked={settings.auto_reapply}
                      onChange={(e) => handleToggle("auto_reapply", e.target.checked)}
                      disabled={saving}
                    />
                    <span className="toggle-rail"><span className="toggle-knob" /></span>
                  </label>
                </div>

                <div className="settings-group-row">
                  <div className="setting-meta">
                    <div className="setting-label-row">
                      <span className="setting-label">Hardware MPO fix</span>
                      {settings.mpo_fix && <span className="status-pill-badge active">Active</span>}
                    </div>
                    <span className="setting-subtext">Prevents GPU overlay bypass (reboot required)</span>
                  </div>
                  <label className="toggle-control">
                    <input
                      type="checkbox"
                      checked={settings.mpo_fix}
                      onChange={(e) => handleToggle("mpo_fix", e.target.checked)}
                      disabled={saving}
                    />
                    <span className="toggle-rail"><span className="toggle-knob" /></span>
                  </label>
                </div>
              </div>

              <div className="settings-section-heading">Stream Audio Isolation</div>
              <div className="settings-group-list">
                <div className="settings-group-row">
                  <div className="setting-meta">
                    <span className="setting-label">Private headphone routing</span>
                    <span className="setting-subtext">You hear isolated apps; streams & screenshares don&apos;t</span>
                  </div>
                  <label className="toggle-control">
                    <input
                      type="checkbox"
                      checked={!!settings.audio_isolation_enabled}
                      onChange={(e) => handleToggle("audio_isolation_enabled", e.target.checked)}
                      disabled={saving}
                    />
                    <span className="toggle-rail"><span className="toggle-knob" /></span>
                  </label>
                </div>

                <div className="settings-group-row">
                  <div className="setting-meta">
                    <span className="setting-label">Stream Bus (Virtual)</span>
                    <span className="setting-subtext">Set as Windows Default for OBS / Discord</span>
                  </div>
                  <select
                    className="settings-select-input"
                    value={settings.audio_stream_device_id || ""}
                    onChange={(e) => handleToggle("audio_stream_device_id", e.target.value || null)}
                    disabled={saving}
                    style={{ maxWidth: "185px" }}
                  >
                    <option value="">Select virtual cable...</option>
                    {audioDevices.map((d) => (
                      <option key={d.id} value={d.id}>
                        {d.is_virtual ? "[Virtual] " : ""}{d.name}
                      </option>
                    ))}
                  </select>
                </div>

                <div className="settings-group-row">
                  <div className="setting-meta">
                    <span className="setting-label">Your Headphones</span>
                    <span className="setting-subtext">Isolated apps route directly here</span>
                  </div>
                  <select
                    className="settings-select-input"
                    value={settings.audio_monitor_device_id || ""}
                    onChange={(e) => handleToggle("audio_monitor_device_id", e.target.value || null)}
                    disabled={saving}
                    style={{ maxWidth: "185px" }}
                  >
                    <option value="">Select headphones...</option>
                    {audioDevices.map((d) => (
                      <option key={d.id} value={d.id}>
                        {d.is_default ? "[Default] " : ""}{d.name}
                      </option>
                    ))}
                  </select>
                </div>
              </div>

              {!audioDevices.some((d) => d.is_virtual) && (
                <button
                  type="button"
                  className="capture-env-pill"
                  style={{ cursor: "pointer", width: "fit-content" }}
                  onClick={() => invoke("open_external_url", { url: "https://vb-audio.com/Cable/" })}
                >
                  <span>No virtual cable detected - Get VB-Cable (free) ↗</span>
                </button>
              )}
            </div>
          )}

          {/* TAB 3: APPEARANCE */}
          {activeTab === "appearance" && (
            <div className="settings-section-pane">
              <div className="settings-section-heading">Theme</div>
              <div className="theme-selection-grid">
                {THEMES.map((t) => (
                  <button
                    type="button"
                    key={t.id}
                    className={`theme-card-preview ${settings.theme === t.id ? "is-active" : ""}`}
                    onClick={() => {
                      handleToggle("theme", t.id);
                      applyDynamicTheme(t.id);
                    }}
                  >
                    <div className="theme-swatch-strip">
                      {t.colors.map((c, i) => (
                        <span key={i} className="theme-color-dot" style={{ backgroundColor: c }} />
                      ))}
                    </div>
                    <span className="theme-card-name">{t.name}</span>
                    {settings.theme === t.id && <IconCheck size={12} className="theme-active-tick" />}
                  </button>
                ))}
              </div>

              <div className="settings-section-heading">Interface</div>
              <div className="settings-group-list">
                <div className="settings-group-row">
                  <div className="setting-meta">
                    <span className="setting-label">Compact mode</span>
                    <span className="setting-subtext">Minimal icon grid window</span>
                  </div>
                  <label className="toggle-control">
                    <input
                      type="checkbox"
                      checked={settings.compact_mode}
                      onChange={(e) => handleToggle("compact_mode", e.target.checked)}
                      disabled={saving}
                    />
                    <span className="toggle-rail"><span className="toggle-knob" /></span>
                  </label>
                </div>

                <div className="settings-group-row">
                  <div className="setting-meta">
                    <span className="setting-label">Show PID badges</span>
                    <span className="setting-subtext">Display process IDs next to names</span>
                  </div>
                  <label className="toggle-control">
                    <input
                      type="checkbox"
                      checked={settings.show_pid}
                      onChange={(e) => handleToggle("show_pid", e.target.checked)}
                      disabled={saving}
                    />
                    <span className="toggle-rail"><span className="toggle-knob" /></span>
                  </label>
                </div>

                <div className="settings-group-row">
                  <div className="setting-meta">
                    <span className="setting-label">Confirm batch actions</span>
                    <span className="setting-subtext">Prompt before Shield All / Clear</span>
                  </div>
                  <label className="toggle-control">
                    <input
                      type="checkbox"
                      checked={settings.confirm_batch}
                      onChange={(e) => handleToggle("confirm_batch", e.target.checked)}
                      disabled={saving}
                    />
                    <span className="toggle-rail"><span className="toggle-knob" /></span>
                  </label>
                </div>
              </div>

              <div className="settings-section-heading">Desktop Widget</div>
              <div className="widget-style-grid horizontal">
                <button
                  type="button"
                  className={`widget-style-card compact ${(!settings.widget_style || settings.widget_style === "pill") ? "is-selected" : ""}`}
                  onClick={() => handleSelectWidgetStyle("pill")}
                >
                  <div className="widget-style-preview pill-preview">
                    <div className="preview-pill-bar">
                      <span className="mini-pill-dot" />
                      <span className="mini-pill-tag">3/5</span>
                      <div className="mini-pill-icons">
                        <span className="mini-icon is-active" />
                        <span className="mini-icon" />
                      </div>
                    </div>
                  </div>
                  <span className="widget-style-name">Pill</span>
                </button>

                <button
                  type="button"
                  className={`widget-style-card compact ${settings.widget_style === "dock" ? "is-selected" : ""}`}
                  onClick={() => handleSelectWidgetStyle("dock")}
                >
                  <div className="widget-style-preview dock-preview">
                    <div className="preview-dock-shell">
                      <span className="preview-dock-bar" />
                      <div className="preview-dock-grid">
                        <span className="dock-dot active" />
                        <span className="dock-dot active" />
                        <span className="dock-dot" />
                        <span className="dock-dot" />
                      </div>
                    </div>
                  </div>
                  <span className="widget-style-name">Dock</span>
                </button>

                <button
                  type="button"
                  className={`widget-style-card compact ${settings.widget_style === "badge" ? "is-selected" : ""}`}
                  onClick={() => handleSelectWidgetStyle("badge")}
                >
                  <div className="widget-style-preview badge-preview">
                    <div className="preview-badge-circle">
                      <svg className="preview-badge-ring" viewBox="0 0 28 28" aria-hidden="true">
                        <circle cx="14" cy="14" r="11" className="preview-badge-track" />
                        <circle cx="14" cy="14" r="11" className="preview-badge-arc" />
                      </svg>
                      <span className="mini-badge-dot" />
                    </div>
                  </div>
                  <span className="widget-style-name">Badge</span>
                </button>
              </div>

              <div className="settings-group-list">
                <div className="settings-group-row">
                  <div className="setting-meta">
                    <span className="setting-label">Floating widget</span>
                    <span className="setting-subtext">Show quick-control desktop overlay</span>
                  </div>
                  <label className="toggle-control">
                    <input
                      type="checkbox"
                      checked={!!settings.widget_enabled}
                      onChange={(e) => handleToggle("widget_enabled", e.target.checked)}
                      disabled={saving}
                    />
                    <span className="toggle-rail"><span className="toggle-knob" /></span>
                  </label>
                </div>

                <div className="settings-group-row">
                  <div className="setting-meta">
                    <span className="setting-label">Always on top</span>
                    <span className="setting-subtext">Pin widget above other windows</span>
                  </div>
                  <label className="toggle-control">
                    <input
                      type="checkbox"
                      checked={settings.widget_always_on_top ?? true}
                      onChange={(e) => handleToggle("widget_always_on_top", e.target.checked)}
                      disabled={saving}
                    />
                    <span className="toggle-rail"><span className="toggle-knob" /></span>
                  </label>
                </div>

                <div className="settings-group-row">
                  <div className="setting-meta">
                    <span className="setting-label">Hide widget from capture</span>
                    <span className="setting-subtext">Invisible in OBS, Discord & recordings</span>
                  </div>
                  <label className="toggle-control">
                    <input
                      type="checkbox"
                      checked={settings.widget_stealth ?? true}
                      onChange={(e) => handleToggle("widget_stealth", e.target.checked)}
                      disabled={saving}
                    />
                    <span className="toggle-rail"><span className="toggle-knob" /></span>
                  </label>
                </div>
              </div>
            </div>
          )}
        </div>

        {/* Confirmation Dialogs */}
        {confirmClearOpen && (
          <div className="settings-confirm-overlay" onClick={() => setConfirmClearOpen(false)}>
            <div className="settings-confirm-card" onClick={(e) => e.stopPropagation()}>
              <div className="confirm-icon-row">
                <IconAlert size={20} className="confirm-alert-icon" />
                <h3 className="confirm-title">Clear all shielded apps?</h3>
              </div>
              <p className="confirm-text">
                Unshields all {shieldedCount} protected window(s) immediately.
              </p>
              <div className="confirm-btn-row">
                <button type="button" className="confirm-cancel-btn" onClick={() => setConfirmClearOpen(false)}>Cancel</button>
                <button
                  type="button"
                  className="confirm-danger-btn"
                  onClick={async () => {
                    await onClearAllShields();
                    setConfirmClearOpen(false);
                  }}
                >
                  Unshield All
                </button>
              </div>
            </div>
          </div>
        )}

        {confirmResetOpen && (
          <div className="settings-confirm-overlay" onClick={() => setConfirmResetOpen(false)}>
            <div className="settings-confirm-card" onClick={(e) => e.stopPropagation()}>
              <div className="confirm-icon-row">
                <IconAlert size={20} className="confirm-alert-icon" />
                <h3 className="confirm-title">Reset preferences?</h3>
              </div>
              <p className="confirm-text">
                Restores startup, engine, theme, and widget settings to defaults.
              </p>
              <div className="confirm-btn-row">
                <button type="button" className="confirm-cancel-btn" onClick={() => setConfirmResetOpen(false)}>Cancel</button>
                <button
                  type="button"
                  className="confirm-danger-btn"
                  onClick={async () => {
                    await onResetSettings();
                    setConfirmResetOpen(false);
                  }}
                >
                  Reset
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
