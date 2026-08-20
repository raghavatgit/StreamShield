import React from "react";

export interface WindowInfo {
  hwnd: number;
  pid: number;
  title: string;
  exe_name: string;
  is_shielded: boolean;
  is_audio_shielded: boolean;
  icon_base64?: string | null;
}

interface Props {
  window: WindowInfo;
  shielded: boolean;
  audioShielded: boolean;
  onToggle: (w: WindowInfo, e: boolean) => void;
  onToggleAudio: (w: WindowInfo, e: boolean) => void;
}

function getFallbackGradient(name: string): { bg: string; color: string } {
  const gradients = [
    { bg: "linear-gradient(135deg, rgba(88, 101, 242, 0.15), rgba(88, 101, 242, 0.3))", color: "#5865f2" },
    { bg: "linear-gradient(135deg, rgba(35, 165, 90, 0.15), rgba(35, 165, 90, 0.3))", color: "#23a55a" },
    { bg: "linear-gradient(135deg, rgba(0, 240, 255, 0.15), rgba(0, 240, 255, 0.3))", color: "#00f0ff" },
    { bg: "linear-gradient(135deg, rgba(250, 166, 26, 0.15), rgba(250, 166, 26, 0.3))", color: "#faa61a" },
    { bg: "linear-gradient(135deg, rgba(235, 69, 158, 0.15), rgba(235, 69, 158, 0.3))", color: "#eb459e" },
  ];
  let hash = 0;
  for (const c of name) hash = (hash * 31 + c.charCodeAt(0)) & 0xffffffff;
  return gradients[Math.abs(hash) % gradients.length];
}

export default function AppRow({
  window: win,
  shielded,
  audioShielded,
  onToggle,
  onToggleAudio,
}: Props) {
  const cleanName = win.exe_name.replace(/\.exe$/i, "");
  const initial = cleanName[0]?.toUpperCase() ?? "?";
  const { bg, color } = getFallbackGradient(win.exe_name);

  return (
    <div
      className={`app-card ${shielded ? "is-shielded" : ""}`}
      id={`app-row-${win.pid}-${win.hwnd}`}
      onClick={(e) => {
        const target = e.target as HTMLElement;
        if (target.tagName !== "INPUT" && !target.closest(".audio-privacy-btn")) {
          onToggle(win, !shielded);
        }
      }}
    >
      <div className="card-accent-strip" />

      {/* Real Process Icon or Fallback */}
      <div className="app-icon-slot">
        {win.icon_base64 ? (
          <img
            src={win.icon_base64}
            alt={cleanName}
            className="app-native-icon"
            loading="lazy"
            onError={(e) => {
              (e.target as HTMLElement).style.display = "none";
            }}
          />
        ) : (
          <div className="app-fallback-avatar" style={{ background: bg, borderColor: color }}>
            <span className="fallback-letter" style={{ color }}>{initial}</span>
          </div>
        )}
      </div>

      {/* Process Meta */}
      <div className="app-meta">
        <div className="app-header-line">
          <span className="app-name-label" title={win.exe_name}>
            {cleanName}
          </span>
          <span className="pid-tag">PID {win.pid}</span>
          {shielded && <span className="shield-tag">SHIELDED</span>}
          {audioShielded && <span className="shield-tag audio-tag">AUDIO ISOLATED</span>}
        </div>

        <div className="app-window-caption" title={win.title}>
          {win.title || "Background Window"}
        </div>
      </div>

      {/* Row Action Controls: Audio Privacy Toggle + Video Switch */}
      <div className="row-action-controls" onClick={(e) => e.stopPropagation()}>
        {/* Stream Audio Privacy Button */}
        <button
          className={`audio-privacy-btn ${audioShielded ? "is-active" : ""}`}
          onClick={() => onToggleAudio(win, !audioShielded)}
          title={
            audioShielded
              ? "Audio Privacy ON: Blocked from stream/recording (Audible in your headphones)"
              : "Audio Privacy OFF: Audio is audible to stream viewers"
          }
        >
          {audioShielded ? (
            <svg className="audio-icon" viewBox="0 0 24 24" fill="currentColor">
              <path d="M12 3a9 9 0 0 0-9 9v7c0 1.1.9 2 2 2h2a2 2 0 0 0 2-2v-4a2 2 0 0 0-2-2H5a7 7 0 1 1 14 0h-2a2 2 0 0 0-2 2v4a2 2 0 0 0 2 2h2c1.1 0 2-.9 2-2v-7a9 9 0 0 0-9-9zm-5 12v3H5v-3h2zm12 3h-2v-3h2v3z"/>
              <circle cx="12" cy="12" r="2" fill="currentColor" />
            </svg>
          ) : (
            <svg className="audio-icon" viewBox="0 0 24 24" fill="currentColor" opacity="0.6">
              <path d="M3 9v6h4l5 5V4L7 9H3zm13.5 3A4.5 4.5 0 0 0 14 8.5v7c1.48-.73 2.5-2.25 2.5-3.5zM14 3.23v2.06c2.89.86 5 3.54 5 6.71s-2.11 5.85-5 6.71v2.06c4.01-.91 7-4.49 7-8.77s-2.99-7.86-7-8.77z"/>
            </svg>
          )}
          <span className="audio-privacy-text">{audioShielded ? "Private" : "Stream"}</span>
        </button>

        {/* Squircle Display Capture Toggle */}
        <div className="switch-wrapper">
          <label className="toggle-control" htmlFor={`toggle-${win.hwnd}`}>
            <input
              id={`toggle-${win.hwnd}`}
              type="checkbox"
              checked={shielded}
              onChange={(e) => onToggle(win, e.target.checked)}
            />
            <span className="toggle-rail">
              <span className="toggle-knob" />
            </span>
          </label>
        </div>
      </div>
    </div>
  );
}
