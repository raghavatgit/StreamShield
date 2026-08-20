import React from "react";

export interface WindowInfo {
  hwnd: number;
  pid: number;
  title: string;
  exe_name: string;
  is_shielded: boolean;
  is_audio_muted: boolean;
  icon_base64?: string | null;
}

interface Props {
  window: WindowInfo;
  shielded: boolean;
  audioMuted: boolean;
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
  audioMuted,
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
        if (target.tagName !== "INPUT" && !target.closest(".audio-mute-btn")) {
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
        </div>

        <div className="app-window-caption" title={win.title}>
          {win.title || "Background Window"}
        </div>
      </div>

      {/* Controls: Audio Mute Button + Squircle Shield Switch */}
      <div className="row-action-controls">
        {/* Sound Privacy Toggle Button */}
        <button
          className={`audio-mute-btn ${audioMuted ? "is-muted" : "is-active-sound"}`}
          onClick={(e) => {
            e.stopPropagation();
            onToggleAudio(win, !audioMuted);
          }}
          title={
            audioMuted
              ? "Audio Shielded (Muted on stream — click to unmute)"
              : "Audio Live on stream (Click to mute from stream)"
          }
        >
          {audioMuted ? (
            <svg className="audio-icon" viewBox="0 0 24 24" fill="currentColor">
              <path d="M16.5 12c0-1.77-1.02-3.29-2.5-4.03v2.21l2.45 2.45c.03-.2.05-.41.05-.63zm2.5 0c0 .94-.2 1.82-.54 2.64l1.51 1.51C20.63 14.91 21 13.5 21 12c0-4.28-2.99-7.86-7-8.77v2.06c2.89.86 5 3.54 5 6.71zM4.27 3L3 4.27 7.73 9H3v6h4l5 5v-6.73l4.25 4.25c-.67.52-1.42.93-2.25 1.18v2.06c1.38-.31 2.63-.95 3.69-1.81L19.73 21 21 19.73l-9-9L4.27 3zM12 4L9.91 6.09 12 8.18V4z" />
            </svg>
          ) : (
            <svg className="audio-icon" viewBox="0 0 24 24" fill="currentColor">
              <path d="M3 9v6h4l5 5V4L7 9H3zm13.5 3c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02zM14 3.23v2.06c2.89.86 5 3.54 5 6.71s-2.11 5.85-5 6.71v2.06c4.01-.91 7-4.49 7-8.77s-2.99-7.86-7-8.77z" />
            </svg>
          )}
        </button>

        {/* Squircle Display Capture Toggle */}
        <div className="switch-wrapper" onClick={(e) => e.stopPropagation()}>
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
