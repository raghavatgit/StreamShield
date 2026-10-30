import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import DesktopWidget from "./components/DesktopWidget";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import "./index.css";

function checkIsWidget(): boolean {
  try {
    const curWin = getCurrentWebviewWindow();
    if (curWin && curWin.label === "widget") {
      return true;
    }
  } catch {
    // fallback
  }

  if (typeof window !== "undefined") {
    const internalLabel =
      (window as any).__TAURI_INTERNALS__?.metadata?.currentWebview?.label ||
      (window as any).__TAURI_INTERNALS__?.metadata?.currentWindow?.label;
    if (internalLabel === "widget") return true;

    if (window.location.hash.includes("widget") || window.location.search.includes("widget")) {
      return true;
    }
  }
  return false;
}

const isWidget = checkIsWidget();
if (isWidget) {
  document.documentElement.classList.add("is-widget-window");
  document.body.classList.add("is-widget-window");
  document.documentElement.style.background = "transparent";
  document.body.style.background = "transparent";
}

class ErrorBoundary extends React.Component<{ children: React.ReactNode }, { hasError: boolean; error: Error | null }> {
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
        <div style={{
          backgroundColor: "#0a0e17",
          color: "#00f0ff",
          fontFamily: "sans-serif",
          padding: "24px",
          height: "100vh",
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
          textAlign: "center"
        }}>
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
              cursor: "pointer"
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
      {isWidget ? <DesktopWidget /> : <App />}
    </ErrorBoundary>
  </React.StrictMode>
);
