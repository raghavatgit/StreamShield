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
  compactMode?: boolean;
  showPid?: boolean;
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
  onToggle,
  compactMode = false,
  showPid = true,
}: Props) {
  const cleanName = win.exe_name.replace(/\.exe$/i, "");
  const initial = cleanName[0]?.toUpperCase() ?? "?";
  const { bg, color } = getFallbackGradient(win.exe_name);

  return (
    <div
      className={`app-card ${shielded ? "is-shielded" : ""} ${compactMode ? "is-compact" : ""}`}
      id={`app-row-${win.pid}-${win.hwnd}`}
      onClick={(e) => {
        const target = e.target as HTMLElement;
        // Skip if click originated inside the toggle switch area (prevents double-fire)
        if (target.tagName === "INPUT" || target.closest(".switch-wrapper")) {
          return;
        }
        onToggle(win, !shielded);
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
          {showPid && <span className="pid-tag">PID {win.pid}</span>}
          {shielded && <span className="shield-tag">SHIELDED</span>}
        </div>

        <div className="app-window-caption" title={win.title}>
          {win.title || "Background Window"}
        </div>
      </div>

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
  );
}
