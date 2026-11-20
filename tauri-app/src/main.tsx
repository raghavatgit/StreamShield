import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import DesktopWidget from "./components/DesktopWidget";
import TrayMenu from "./components/TrayMenu";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { ThemeType, applyDynamicTheme } from "./utils/themeEngine";
import "./index.css";

interface WidgetInitPayload {
  style?: "pill" | "dock" | "badge";
  theme?: ThemeType;
  self_stealth?: boolean;
  widget_stealth?: boolean;
}

interface TrayInitPayload {
  theme?: ThemeType;
  shield_mode?: "exclude" | "monitor";
  compact_mode?: boolean;
  self_stealth?: boolean;
  widget_enabled?: boolean;
  widget_style?: "pill" | "dock" | "badge";
}

declare global {
  interface Window {
    __STREAMSHIELD_WIDGET_INIT__?: WidgetInitPayload;
    __STREAMSHIELD_TRAY_INIT__?: TrayInitPayload;
  }
}

type WindowMode = "main" | "widget" | "tray_menu";

function detectWindowMode(): WindowMode {
  if (typeof window !== "undefined") {
    if (window.__STREAMSHIELD_TRAY_INIT__) return "tray_menu";
    if (window.__STREAMSHIELD_WIDGET_INIT__) return "widget";
  }

  try {
    const curWin = getCurrentWebviewWindow();
    if (curWin?.label === "tray_menu") return "tray_menu";
    if (curWin?.label === "widget") return "widget";
  } catch {
    // fallback
  }

  if (typeof window !== "undefined") {
    const internalLabel =
      (window as any).__TAURI_INTERNALS__?.metadata?.currentWebview?.label ||
      (window as any).__TAURI_INTERNALS__?.metadata?.currentWindow?.label;
    if (internalLabel === "tray_menu") return "tray_menu";
    if (internalLabel === "widget") return "widget";

    if (window.location.hash.includes("tray_menu") || window.location.search.includes("tray_menu")) {
      return "tray_menu";
    }
    if (window.location.hash.includes("widget") || window.location.search.includes("widget")) {
      return "widget";
    }
  }
  return "main";
}

const windowMode = detectWindowMode();
if (windowMode === "widget") {
  document.documentElement.classList.add("is-widget-window");
  document.body.classList.add("is-widget-window");
  document.documentElement.style.background = "transparent";
  document.body.style.background = "transparent";
} else if (windowMode === "tray_menu") {
  document.documentElement.classList.add("is-tray-menu-window");
  document.body.classList.add("is-tray-menu-window");
  document.documentElement.style.background = "transparent";
  document.body.style.background = "transparent";
}

// Apply initial theme immediately before first React paint to eliminate FOUC
const initialTheme: ThemeType =
  (windowMode === "widget" ? window.__STREAMSHIELD_WIDGET_INIT__?.theme : undefined) ||
  (windowMode === "tray_menu" ? window.__STREAMSHIELD_TRAY_INIT__?.theme : undefined) ||
  (localStorage.getItem("streamshield_theme") as ThemeType) ||
  "violet";
applyDynamicTheme(initialTheme);

class ErrorBoundary extends React.Component<
  { children: React.ReactNode },
  { hasError: boolean; error: Error | null }
> {
  constructor(props: { children: React.ReactNode }) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error) {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error("StreamShield Render Error:", error, errorInfo);
  }

  render() {
    if (this.state.hasError) {
      return (
        <div
          style={{
            backgroundColor: "#0a0e17",
            color: "#00f0ff",
            fontFamily: "sans-serif",
            padding: "24px",
            height: "100vh",
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            justifyContent: "center",
            textAlign: "center",
          }}
        >
          <h2 style={{ color: "#ff007f", marginBottom: "8px" }}>StreamShield UI Error</h2>
          <p style={{ color: "#94a3b8", fontSize: "14px", maxWidth: "400px" }}>
            {this.state.error?.message || "An unexpected error occurred during rendering."}
          </p>
          <button
            onClick={() => window.location.reload()}
            style={{
              marginTop: "16px",
              padding: "8px 16px",
              backgroundColor: "#00f0ff",
              color: "#0a0e17",
              border: "none",
              borderRadius: "6px",
              fontWeight: "bold",
              cursor: "pointer",
            }}
          >
            Reload Interface
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <ErrorBoundary>
      {windowMode === "tray_menu" ? (
        <TrayMenu />
      ) : windowMode === "widget" ? (
        <DesktopWidget />
      ) : (
        <App />
      )}
    </ErrorBoundary>
  </React.StrictMode>
);
