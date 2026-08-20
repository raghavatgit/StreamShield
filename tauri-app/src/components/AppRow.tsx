import React from "react";

export interface WindowInfo {
  hwnd: number;
  pid: number;
  title: string;
  exe_name: string;
  is_shielded: boolean;
  icon_base64?: string | null;
}

interface Props {
  window: WindowInfo;
  shielded: boolean;
  onToggle: (w: WindowInfo, e: boolean) => void;
}

// Generate consistent clean gradient background for apps without native icons
function getFallbackGradient(name: string): { bg: string; color: string } {
  const gradients = [
    { bg: "linear-gradient(135deg, rgba(0, 240, 255, 0.15), rgba(0, 114, 255, 0.15))", color: "#00f0ff" },
    { bg: "linear-gradient(135deg, rgba(255, 0, 128, 0.15), rgba(181, 23, 158, 0.15))", color: "#ff007f" },
    { bg: "linear-gradient(135deg, rgba(16, 185, 129, 0.15), rgba(5, 150, 105, 0.15))", color: "#10b981" },
    { bg: "linear-gradient(135deg, rgba(245, 158, 11, 0.15), rgba(217, 119, 6, 0.15))", color: "#f59e0b" },
    { bg: "linear-gradient(135deg, rgba(168, 85, 247, 0.15), rgba(126, 34, 206, 0.15))", color: "#a855f7" },
    { bg: "linear-gradient(135deg, rgba(56, 189, 248, 0.15), rgba(14, 165, 233, 0.15))", color: "#38bdf8" },
  ];
  let hash = 0;
  for (const c of name) hash = (hash * 31 + c.charCodeAt(0)) & 0xffffffff;
  return gradients[Math.abs(hash) % gradients.length];
}

export default function AppRow({ window: win, shielded, onToggle }: Props) {
  const cleanName = win.exe_name.replace(/\.exe$/i, "");
  const initial = cleanName[0]?.toUpperCase() ?? "?";
  const { bg, color } = getFallbackGradient(win.exe_name);

  return (
    <div
      className={`app-card ${shielded ? "is-shielded" : ""}`}
      id={`app-row-${win.pid}-${win.hwnd}`}
      onClick={(e) => {
        if ((e.target as HTMLElement).tagName !== "INPUT") {
          onToggle(win, !shielded);
        }
      }}
    >
      <div className="card-accent-line" />

      {/* Real Process Icon or Fallback */}
      <div className="app-icon-slot">
        {win.icon_base64 ? (
          <img
            src={win.icon_base64}
            alt={cleanName}
            className="app-native-icon"
            loading="lazy"
            onError={(e) => {
              // On broken image, hide img so fallback shows
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

      {/* Switch Toggle */}
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
  );
}
