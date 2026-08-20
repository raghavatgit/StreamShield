interface WindowInfo { hwnd: number; pid: number; title: string; exe_name: string; }
interface Props { window: WindowInfo; shielded: boolean; onToggle: (w: WindowInfo, e: boolean) => void; }

function exeColor(name: string) {
  const colors = ["#63b3ed","#68d391","#f6ad55","#fc8181","#b794f4","#76e4f7","#fbb6ce","#9ae6b4"];
  let hash = 0;
  for (const c of name) hash = (hash * 31 + c.charCodeAt(0)) & 0xffffffff;
  return colors[Math.abs(hash) % colors.length];
}

export default function AppRow({ window: win, shielded, onToggle }: Props) {
  const color = exeColor(win.exe_name);
  const initial = win.exe_name.replace(/\.exe$/i,"")[0]?.toUpperCase() ?? "?";
  return (
    <div className={`app-row${shielded?" shielded":""}`} id={`app-row-${win.pid}`}>
      <div className="shield-indicator"/>
      <div className="app-icon">
        <span className="app-icon-letter" style={{color}}>{initial}</span>
      </div>
      <div className="app-info">
        <div className="app-name-row" title={win.title}>{win.exe_name.replace(/\.exe$/i,"")}</div>
        <div className="app-pid">PID {win.pid} � {win.title.slice(0,35)}{win.title.length>35?"�":""}</div>
      </div>
      <div className="toggle-wrap">
        <span className={`toggle-label${shielded?" on":""}`}>{shielded?"ON":"OFF"}</span>
        <label className="toggle" htmlFor={`t-${win.pid}`}>
          <input id={`t-${win.pid}`} type="checkbox" checked={shielded} onChange={e=>onToggle(win,e.target.checked)}/>
          <span className="toggle-slider"/>
        </label>
      </div>
    </div>
  );
}
