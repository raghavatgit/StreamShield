nterface WindowInfo {
  hwnd: number;
  pid: number;
  title: string;
  exe_name: string;
  icon_base64?: string;
}

interface Props {
  window: WindowInfo;
  shielded: boolean;
  onToggle: (win: WindowInfo, enable: boolean) => void;
}

// Get a color per exe for the icon letter badge
function exeColor(name: string): string {
  const colors = [
    "#63b3ed", "#68d391", "#f6ad55", "#fc8181",
    "#b794f4", "#76e4f7", "#fbb6ce", "#9ae6b4",
  ];
  let hash = 0;
  for (const c of name) hash = (hash * 31 + c.charCodeAt(0)) & 0xffffffff;
  return colors[Math.abs(hash) % colors.length];
}

// Clean up exe name for display
function displayName(exeName: string): string {
  return exeName.replace(/\.exe$/i, "");
}

export default function AppRow({ window: win, shielded, onToggle }: Props) {
  const color = exeColor(win.exe_name);
  const initial = displayName(win.exe_name)[0]?.toUpperCase() ?? "?";
  const id = `app-row-${win.pid}`;

  return (
    <div className={`app-row${shielded ? " shielded" : ""}`} id={id}>
      <div className="shield-indicator" />

      {/* Icon */}
      <div className="app-icon">
        {win.icon_base64 ? (
          <img src={`data:image/png;base64,${win.icon_base64}`} alt="" />
        ) : (
          <span
            className="app-icon-letter"
            style={{ color }}
          >
            {initial}
          </span>
        )}
      </div>

      {/* Info */}
      <div className="app-info">
        <div className="app-name-row" title={win.title}>
          {displayName(win.exe_name)}
        </div>
        <div className="app-pid">PID {win.pid} · {win.title.slice(0, 30)}{win.title.length > 30 ? "…" : ""}</div>
      </div>

      {/* Toggle */}
      <div className="toggle-wrap">
        <span className={`toggle-label${shielded ? " on" : ""}`}>
          {shielded ? "ON" : "OFF"}
        </span>
        <label
          className="toggle"
          htmlFor={`toggle-${win.pid}`}
          title={shielded ? "Click to unshield" : "Click to hide from streams"}
        >
          <input
            id={`toggle-${win.pid}`}
            type="checkbox"
            checked={shielded}
            onChange={e => onToggle(win, e.target.checked)}
          />
          <span className="toggle-slider" />
        </label>
      </div>
    </div>
  );
}
