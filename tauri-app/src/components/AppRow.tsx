interface WindowInfo {
  hwnd: number;
  pid: number;
  title: string;
  exe_name: string;
}

interface Props {
  window: WindowInfo;
  shielded: boolean;
  onToggle: (w: WindowInfo, e: boolean) => void;
}

// Generate consistent vibrant color gradients based on executable name
function getExeGradient(name: string): { bg: string; color: string } {
  const gradients = [
    { bg: "linear-gradient(135deg, rgba(0, 240, 255, 0.2), rgba(0, 114, 255, 0.2))", color: "#00f0ff" },
    { bg: "linear-gradient(135deg, rgba(255, 0, 128, 0.2), rgba(181, 23, 158, 0.2))", color: "#ff007f" },
    { bg: "linear-gradient(135deg, rgba(16, 185, 129, 0.2), rgba(5, 150, 105, 0.2))", color: "#10b981" },
    { bg: "linear-gradient(135deg, rgba(245, 158, 11, 0.2), rgba(217, 119, 6, 0.2))", color: "#f59e0b" },
    { bg: "linear-gradient(135deg, rgba(168, 85, 247, 0.2), rgba(126, 34, 206, 0.2))", color: "#a855f7" },
    { bg: "linear-gradient(135deg, rgba(236, 72, 153, 0.2), rgba(219, 39, 119, 0.2))", color: "#ec4899" },
    { bg: "linear-gradient(135deg, rgba(56, 189, 248, 0.2), rgba(14, 165, 233, 0.2))", color: "#38bdf8" },
  ];
  let hash = 0;
  for (const c of name) hash = (hash * 31 + c.charCodeAt(0)) & 0xffffffff;
  return gradients[Math.abs(hash) % gradients.length];
}

export default function AppRow({ window: win, shielded, onToggle }: Props) {
  const { bg, color } = getExeGradient(win.exe_name);
  const cleanName = win.exe_name.replace(/\.exe$/i, "");
  const initial = cleanName[0]?.toUpperCase() ?? "?";

  return (
    <div
      className={`app-row ${shielded ? "shielded" : ""}`}
      id={`app-row-${win.pid}`}
      onClick={(e) => {
        // Prevent toggle triggering twice if clicked directly on input
        if ((e.target as HTMLElement).tagName !== "INPUT") {
          onToggle(win, !shielded);
        }
      }}
    >
      <div className="shield-bar-indicator" />

      <div className="app-avatar" style={{ background: bg, borderColor: color }}>
        <span className="app-avatar-initial" style={{ color }}>
          {initial}
        </span>
      </div>

      <div className="app-details">
        <div className="app-title-row">
          <span className="app-clean-name" title={win.exe_name}>
            {cleanName}
          </span>
          <span className="pid-badge">PID {win.pid}</span>
          {shielded && <span className="shielded-badge">HIDDEN</span>}
        </div>
        <div className="app-sub-title" title={win.title}>
          {win.title || "Background Window"}
        </div>
      </div>

      <div className="toggle-container" onClick={(e) => e.stopPropagation()}>
        <label className="switch-toggle" htmlFor={`t-${win.hwnd}`}>
          <input
            id={`t-${win.hwnd}`}
            type="checkbox"
            checked={shielded}
            onChange={(e) => onToggle(win, e.target.checked)}
          />
          <span className="slider-track">
            <span className="slider-thumb" />
          </span>
        </label>
      </div>
    </div>
  );
}
