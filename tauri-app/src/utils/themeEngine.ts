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
  colors: [string, string];
}

export const THEMES: ThemeDefinition[] = [
  {
    id: "violet",
    name: "Neon Violet",
    mode: "dark",
    primary: "#a855f7",
    accent: "#ec4899",
    colors: ["#a855f7", "#ec4899"],
  },
  {
    id: "cyberpunk",
    name: "Cyberpunk Cyan",
    mode: "dark",
    primary: "#00f0ff",
    accent: "#ff007f",
    colors: ["#00f0ff", "#ff007f"],
  },
  {
    id: "emerald",
    name: "Matrix Emerald",
    mode: "dark",
    primary: "#10b981",
    accent: "#06b6d4",
    colors: ["#10b981", "#06b6d4"],
  },
  {
    id: "crimson",
    name: "Crimson Ruby",
    mode: "dark",
    primary: "#f43f5e",
    accent: "#fb923c",
    colors: ["#f43f5e", "#fb923c"],
  },
  {
    id: "amber",
    name: "Solar Amber",
    mode: "dark",
    primary: "#f59e0b",
    accent: "#ef4444",
    colors: ["#f59e0b", "#ef4444"],
  },
  {
    id: "sapphire",
    name: "Ocean Sapphire",
    mode: "dark",
    primary: "#3b82f6",
    accent: "#6366f1",
    colors: ["#3b82f6", "#6366f1"],
  },
  {
    id: "discord",
    name: "Discord Slate",
    mode: "dark",
    primary: "#5865f2",
    accent: "#23a55a",
    colors: ["#5865f2", "#23a55a"],
  },
  {
    id: "obsidian",
    name: "Midnight Obsidian",
    mode: "dark",
    primary: "#94a3b8",
    accent: "#cbd5e1",
    colors: ["#94a3b8", "#cbd5e1"],
  },
  {
    id: "neumorphic-white",
    name: "Clean Light",
    mode: "light",
    primary: "#4f46e5",
    accent: "#0284c7",
    colors: ["#ffffff", "#4f46e5"],
  },
];

function hexToRgb(hex: string): [number, number, number] {
  const clean = hex.replace("#", "");
  const num = parseInt(clean, 16);
  return [(num >> 16) & 255, (num >> 8) & 255, num & 255];
}

export function applyDynamicTheme(themeId: ThemeType) {
  const theme = THEMES.find((t) => t.id === themeId) || THEMES[0];
  const root = document.documentElement;

  root.setAttribute("data-theme", theme.id);

  const [pr, pg, pb] = hexToRgb(theme.primary);
  const [ar, ag, ab] = hexToRgb(theme.accent);

  if (theme.mode === "light") {
    root.style.setProperty("--bg-base", "#f1f5f9");
    root.style.setProperty("--bg-surface", "#ffffff");
    root.style.setProperty("--bg-card", "#ffffff");
    root.style.setProperty("--bg-card-hover", "#f8fafc");
    root.style.setProperty("--bg-card-shielded", `rgba(${pr}, ${pg}, ${pb}, 0.08)`);
    root.style.setProperty("--border-subtle", "#e2e8f0");
    root.style.setProperty("--border-card", "#cbd5e1");
    root.style.setProperty("--border-active", theme.primary);
    root.style.setProperty("--text-primary", "#0f172a");
    root.style.setProperty("--text-secondary", "#334155");
    root.style.setProperty("--text-muted", "#64748b");
    root.style.setProperty("--primary", theme.primary);
    root.style.setProperty("--accent", theme.accent);
    root.style.setProperty("--brand-gradient", `linear-gradient(135deg, ${theme.primary} 0%, ${theme.accent} 100%)`);
    root.style.setProperty("--toggle-rail-off", "#cbd5e1");
    root.style.setProperty("--toggle-rail-on", theme.primary);
    root.style.setProperty("--toggle-knob", "#ffffff");
    root.style.setProperty("--glow-accent", `rgba(${pr}, ${pg}, ${pb}, 0.22)`);
    root.style.setProperty("--ambient-glow-1", `rgba(${pr}, ${pg}, ${pb}, 0.10)`);
    root.style.setProperty("--ambient-glow-2", `rgba(${ar}, ${ag}, ${ab}, 0.08)`);
    root.style.setProperty("--card-shadow", "0 1px 3px rgba(15, 23, 42, 0.08), 0 1px 2px rgba(15, 23, 42, 0.04)");
    root.style.setProperty("--inset-shadow", "inset 0 1px 2px rgba(15, 23, 42, 0.06)");
  } else {
    // Dynamic Dark Mode Peripherals
    root.style.setProperty("--bg-base", "#07090e");
    root.style.setProperty("--bg-surface", "#0d121c");
    root.style.setProperty("--bg-card", "rgba(16, 23, 38, 0.75)");
    root.style.setProperty("--bg-card-hover", "rgba(24, 35, 58, 0.9)");
    root.style.setProperty("--bg-card-shielded", `rgba(${pr}, ${pg}, ${pb}, 0.12)`);
    root.style.setProperty("--border-subtle", `rgba(${pr}, ${pg}, ${pb}, 0.12)`);
    root.style.setProperty("--border-card", `rgba(${pr}, ${pg}, ${pb}, 0.20)`);
    root.style.setProperty("--border-active", `rgba(${pr}, ${pg}, ${pb}, 0.60)`);
    root.style.setProperty("--text-primary", "#f8fafc");
    root.style.setProperty("--text-secondary", "#94a3b8");
    root.style.setProperty("--text-muted", "#62728d");
    root.style.setProperty("--primary", theme.primary);
    root.style.setProperty("--accent", theme.accent);
    root.style.setProperty("--brand-gradient", `linear-gradient(135deg, ${theme.primary} 0%, ${theme.accent} 100%)`);
    root.style.setProperty("--toggle-rail-off", "#172236");
    root.style.setProperty("--toggle-rail-on", theme.primary);
    root.style.setProperty("--toggle-knob", "#ffffff");
    root.style.setProperty("--glow-accent", `rgba(${pr}, ${pg}, ${pb}, 0.35)`);
    root.style.setProperty("--ambient-glow-1", `rgba(${pr}, ${pg}, ${pb}, 0.22)`);
    root.style.setProperty("--ambient-glow-2", `rgba(${ar}, ${ag}, ${ab}, 0.16)`);
    root.style.setProperty("--card-shadow", "0 4px 16px rgba(0, 0, 0, 0.45)");
    root.style.setProperty("--inset-shadow", "inset 0 1px 3px rgba(0, 0, 0, 0.4)");
  }
}

