export type ThemeType =
  | "violet"
  | "cyberpunk"
  | "emerald"
  | "crimson"
  | "amber"
  | "sapphire"
  | "discord"
  | "obsidian"
  | "neumorphic-white";

export interface ThemeDefinition {
  id: ThemeType;
  name: string;
  mode: "dark" | "light";
  primary: string;
  accent: string;
  surfaceRgb: [number, number, number];
  colors: [string, string];
}

export const THEMES: ThemeDefinition[] = [
  {
    id: "violet",
    name: "Neon Violet",
    mode: "dark",
    primary: "#a855f7",
    accent: "#ec4899",
    surfaceRgb: [12, 11, 22],
    colors: ["#a855f7", "#ec4899"],
  },
  {
    id: "cyberpunk",
    name: "Cyberpunk Cyan",
    mode: "dark",
    primary: "#00f0ff",
    accent: "#ff007f",
    surfaceRgb: [7, 13, 22],
    colors: ["#00f0ff", "#ff007f"],
  },
  {
    id: "emerald",
    name: "Matrix Emerald",
    mode: "dark",
    primary: "#10b981",
    accent: "#06b6d4",
    surfaceRgb: [8, 17, 17],
    colors: ["#10b981", "#06b6d4"],
  },
  {
    id: "crimson",
    name: "Crimson Ruby",
    mode: "dark",
    primary: "#f43f5e",
    accent: "#fb923c",
    surfaceRgb: [18, 10, 15],
    colors: ["#f43f5e", "#fb923c"],
  },
  {
    id: "amber",
    name: "Solar Amber",
    mode: "dark",
    primary: "#f59e0b",
    accent: "#ef4444",
    surfaceRgb: [18, 13, 10],
    colors: ["#f59e0b", "#ef4444"],
  },
  {
    id: "sapphire",
    name: "Ocean Sapphire",
    mode: "dark",
    primary: "#3b82f6",
    accent: "#6366f1",
    surfaceRgb: [9, 15, 28],
    colors: ["#3b82f6", "#6366f1"],
  },
  {
    id: "discord",
    name: "Discord Slate",
    mode: "dark",
    primary: "#5865f2",
    accent: "#23a55a",
    surfaceRgb: [26, 27, 30],
    colors: ["#5865f2", "#23a55a"],
  },
  {
    id: "obsidian",
    name: "Midnight Obsidian",
    mode: "dark",
    primary: "#94a3b8",
    accent: "#cbd5e1",
    surfaceRgb: [10, 12, 16],
    colors: ["#94a3b8", "#cbd5e1"],
  },
  {
    id: "neumorphic-white",
    name: "Clean Light",
    mode: "light",
    primary: "#4f46e5",
    accent: "#0284c7",
    surfaceRgb: [255, 255, 255],
    colors: ["#ffffff", "#4f46e5"],
  },
];

function hexToRgb(hex: string): [number, number, number] {
  const clean = hex.replace("#", "");
  const num = parseInt(clean, 16);
  return [(num >> 16) & 255, (num >> 8) & 255, num & 255];
}

function relativeLuminance(r: number, g: number, b: number): number {
  const linearize = (c: number) => {
    const s = c / 255;
    return s <= 0.03928 ? s / 12.92 : Math.pow((s + 0.055) / 1.055, 2.4);
  };
  return 0.2126 * linearize(r) + 0.7152 * linearize(g) + 0.0722 * linearize(b);
}

export function applyDynamicTheme(themeId: ThemeType) {
  const theme = THEMES.find((t) => t.id === themeId) || THEMES[0];
  const root = document.documentElement;

  root.setAttribute("data-theme", theme.id);
  root.style.colorScheme = theme.mode;

  const [pr, pg, pb] = hexToRgb(theme.primary);
  const [ar, ag, ab] = hexToRgb(theme.accent);
  const [sr, sg, sb] = theme.surfaceRgb;
  const lum = relativeLuminance(pr, pg, pb);
  const contrastWhite = 1.05 / (lum + 0.05);
  const contrastDark = (lum + 0.05) / (0.0044 + 0.05);
  const onPrimary = contrastDark > contrastWhite ? "#061018" : "#ffffff";

  root.style.setProperty("--primary-rgb", `${pr}, ${pg}, ${pb}`);
  root.style.setProperty("--accent-rgb", `${ar}, ${ag}, ${ab}`);

  if (theme.mode === "light") {
    root.style.setProperty("--bg-base", "#f1f5f9");
    root.style.setProperty("--bg-app", "#f1f5f9");
    root.style.setProperty("--bg-surface", "#ffffff");
    root.style.setProperty("--bg-secondary", "#f8fafc");
    root.style.setProperty("--bg-card", "#ffffff");
    root.style.setProperty("--bg-card-hover", "#f8fafc");
    root.style.setProperty("--bg-card-shielded", `rgba(${pr}, ${pg}, ${pb}, 0.08)`);
    root.style.setProperty("--bg-card-active", `rgba(${pr}, ${pg}, ${pb}, 0.12)`);
    root.style.setProperty("--bg-accent-subtle", `rgba(${pr}, ${pg}, ${pb}, 0.12)`);
    root.style.setProperty("--surface-subtle", "rgba(15, 23, 42, 0.04)");
    root.style.setProperty("--surface-hover", "rgba(15, 23, 42, 0.08)");
    root.style.setProperty("--border-subtle", "#e2e8f0");
    root.style.setProperty("--border-card", "#cbd5e1");
    root.style.setProperty("--border-active", theme.primary);
    root.style.setProperty("--border-accent", `rgba(${pr}, ${pg}, ${pb}, 0.55)`);
    root.style.setProperty("--border-hover", "#94a3b8");
    root.style.setProperty("--text-primary", "#0f172a");
    root.style.setProperty("--text-secondary", "#334155");
    root.style.setProperty("--text-muted", "#64748b");
    root.style.setProperty("--primary", theme.primary);
    root.style.setProperty("--accent", theme.accent);
    root.style.setProperty("--accent-color", theme.primary);
    root.style.setProperty("--on-primary", onPrimary);
    root.style.setProperty("--brand-gradient", `linear-gradient(135deg, ${theme.primary} 0%, ${theme.accent} 100%)`);
    root.style.setProperty("--toggle-rail-off", "#cbd5e1");
    root.style.setProperty("--toggle-rail-on", theme.primary);
    root.style.setProperty("--toggle-knob", "#ffffff");
    root.style.setProperty("--glow-accent", `rgba(${pr}, ${pg}, ${pb}, 0.22)`);
    root.style.setProperty("--ambient-glow-1", `rgba(${pr}, ${pg}, ${pb}, 0.10)`);
    root.style.setProperty("--ambient-glow-2", `rgba(${ar}, ${ag}, ${ab}, 0.08)`);
    root.style.setProperty("--card-shadow", "0 1px 3px rgba(15, 23, 42, 0.08), 0 1px 2px rgba(15, 23, 42, 0.04)");
    root.style.setProperty("--inset-shadow", "inset 0 1px 2px rgba(15, 23, 42, 0.06)");

    // Widget & Tray Tokens (Light Mode)
    root.style.setProperty("--widget-bg", "rgba(255, 255, 255, 0.94)");
    root.style.setProperty("--widget-popover-bg", "rgba(255, 255, 255, 0.98)");
    root.style.setProperty("--widget-border", `rgba(${pr}, ${pg}, ${pb}, 0.28)`);
    root.style.setProperty("--widget-border-strong", `rgba(${pr}, ${pg}, ${pb}, 0.62)`);
    root.style.setProperty("--widget-specular", "rgba(255, 255, 255, 0.95)");
    root.style.setProperty(
      "--widget-shadow",
      `0 6px 14px rgba(15, 23, 42, 0.14), 0 2px 4px rgba(15, 23, 42, 0.08)`
    );
    root.style.setProperty("--widget-tile-bg", "rgba(15, 23, 42, 0.045)");
    root.style.setProperty("--widget-tile-hover", "rgba(15, 23, 42, 0.09)");
    root.style.setProperty("--widget-tile-shielded", `rgba(${pr}, ${pg}, ${pb}, 0.12)`);
    root.style.setProperty("--widget-divider", "rgba(15, 23, 42, 0.10)");
    root.style.setProperty("--widget-ring-track", "rgba(15, 23, 42, 0.13)");
    root.style.setProperty("--widget-active-bg", `rgba(${pr}, ${pg}, ${pb}, 0.13)`);
    root.style.setProperty("--widget-active-fg", theme.primary);
    root.style.setProperty("--widget-partial-bg", `rgba(${ar}, ${ag}, ${ab}, 0.13)`);
    root.style.setProperty("--widget-partial-fg", theme.accent);
    root.style.setProperty("--tray-bg", "rgba(255, 255, 255, 0.97)");
    root.style.setProperty("--tray-surface", "rgba(15, 23, 42, 0.04)");
    root.style.setProperty("--tray-row-hover", "rgba(15, 23, 42, 0.075)");
    root.style.setProperty("--tray-segment-bg", "rgba(15, 23, 42, 0.06)");
    root.style.setProperty("--tray-segment-active", `rgba(${pr}, ${pg}, ${pb}, 0.14)`);
    root.style.setProperty("--scrollbar-thumb", "rgba(15, 23, 42, 0.34)");
    root.style.setProperty("--scrollbar-track", "rgba(15, 23, 42, 0.04)");
  } else {
    root.style.setProperty("--bg-base", "#07090e");
    root.style.setProperty("--bg-app", "#07090e");
    root.style.setProperty("--bg-surface", "#0d121c");
    root.style.setProperty("--bg-secondary", "rgba(13, 18, 28, 0.92)");
    root.style.setProperty("--bg-card", "rgba(16, 23, 38, 0.75)");
    root.style.setProperty("--bg-card-hover", "rgba(24, 35, 58, 0.9)");
    root.style.setProperty("--bg-card-shielded", `rgba(${pr}, ${pg}, ${pb}, 0.12)`);
    root.style.setProperty("--bg-card-active", `rgba(${pr}, ${pg}, ${pb}, 0.14)`);
    root.style.setProperty("--bg-accent-subtle", `rgba(${pr}, ${pg}, ${pb}, 0.16)`);
    root.style.setProperty("--surface-subtle", "rgba(255, 255, 255, 0.05)");
    root.style.setProperty("--surface-hover", "rgba(255, 255, 255, 0.10)");
    root.style.setProperty("--border-subtle", `rgba(${pr}, ${pg}, ${pb}, 0.12)`);
    root.style.setProperty("--border-card", `rgba(${pr}, ${pg}, ${pb}, 0.20)`);
    root.style.setProperty("--border-active", `rgba(${pr}, ${pg}, ${pb}, 0.60)`);
    root.style.setProperty("--border-accent", `rgba(${pr}, ${pg}, ${pb}, 0.55)`);
    root.style.setProperty("--border-hover", `rgba(${pr}, ${pg}, ${pb}, 0.42)`);
    root.style.setProperty("--text-primary", "#f8fafc");
    root.style.setProperty("--text-secondary", "#94a3b8");
    root.style.setProperty("--text-muted", "#62728d");
    root.style.setProperty("--primary", theme.primary);
    root.style.setProperty("--accent", theme.accent);
    root.style.setProperty("--accent-color", theme.primary);
    root.style.setProperty("--on-primary", onPrimary);
    root.style.setProperty("--brand-gradient", `linear-gradient(135deg, ${theme.primary} 0%, ${theme.accent} 100%)`);
    root.style.setProperty("--toggle-rail-off", "#172236");
    root.style.setProperty("--toggle-rail-on", theme.primary);
    root.style.setProperty("--toggle-knob", "#ffffff");
    root.style.setProperty("--glow-accent", `rgba(${pr}, ${pg}, ${pb}, 0.35)`);
    root.style.setProperty("--ambient-glow-1", `rgba(${pr}, ${pg}, ${pb}, 0.22)`);
    root.style.setProperty("--ambient-glow-2", `rgba(${ar}, ${ag}, ${ab}, 0.16)`);
    root.style.setProperty("--card-shadow", "0 4px 16px rgba(0, 0, 0, 0.45)");
    root.style.setProperty("--inset-shadow", "inset 0 1px 3px rgba(0, 0, 0, 0.4)");

    // Widget & Tray Tokens (Bespoke Per-Theme Atmospheric Glass)
    root.style.setProperty("--widget-bg", `rgba(${sr}, ${sg}, ${sb}, 0.91)`);
    root.style.setProperty(
      "--widget-popover-bg",
      `rgba(${Math.min(255, sr + 5)}, ${Math.min(255, sg + 5)}, ${Math.min(255, sb + 7)}, 0.97)`
    );
    root.style.setProperty("--widget-border", `rgba(${pr}, ${pg}, ${pb}, 0.34)`);
    root.style.setProperty("--widget-border-strong", `rgba(${pr}, ${pg}, ${pb}, 0.72)`);
    root.style.setProperty("--widget-specular", "rgba(255, 255, 255, 0.14)");
    root.style.setProperty(
      "--widget-shadow",
      `0 6px 15px rgba(0, 0, 0, 0.52), 0 2px 5px rgba(0, 0, 0, 0.36), 0 0 10px rgba(${pr}, ${pg}, ${pb}, 0.14)`
    );
    root.style.setProperty("--widget-tile-bg", "rgba(255, 255, 255, 0.048)");
    root.style.setProperty("--widget-tile-hover", "rgba(255, 255, 255, 0.11)");
    root.style.setProperty("--widget-tile-shielded", `rgba(${pr}, ${pg}, ${pb}, 0.17)`);
    root.style.setProperty("--widget-divider", "rgba(255, 255, 255, 0.09)");
    root.style.setProperty("--widget-ring-track", "rgba(255, 255, 255, 0.13)");
    root.style.setProperty("--widget-active-bg", `rgba(${pr}, ${pg}, ${pb}, 0.18)`);
    root.style.setProperty("--widget-active-fg", theme.primary);
    root.style.setProperty("--widget-partial-bg", `rgba(${ar}, ${ag}, ${ab}, 0.18)`);
    root.style.setProperty("--widget-partial-fg", theme.accent);
    root.style.setProperty("--tray-bg", `rgba(${sr}, ${sg}, ${sb}, 0.95)`);
    root.style.setProperty("--tray-surface", "rgba(255, 255, 255, 0.045)");
    root.style.setProperty("--tray-row-hover", "rgba(255, 255, 255, 0.085)");
    root.style.setProperty("--tray-segment-bg", "rgba(255, 255, 255, 0.055)");
    root.style.setProperty("--tray-segment-active", `rgba(${pr}, ${pg}, ${pb}, 0.22)`);
    root.style.setProperty("--scrollbar-thumb", `rgba(${pr}, ${pg}, ${pb}, 0.45)`);
    root.style.setProperty("--scrollbar-track", "rgba(255, 255, 255, 0.03)");
  }
}
