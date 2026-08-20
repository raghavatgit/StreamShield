 StreamShield 🛡️

> **Per-app stream privacy manager for Windows**

Block specific applications from appearing in screen recordings, Discord streams, Medal clips, OBS, and any other screen capture tool — all with a simple toggle.

## Features
- 📋 Lists all running applications with process name + PID
- 🔒 Toggle capture-exempt per app (they appear black on stream, visible to you)
- 🖥️ System tray — runs silently in the background
- 💾 Settings persist across restarts and auto-reapply
- 🔍 Search/filter running apps

## How it works
Uses Windows `SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)` API — the same mechanism used by banking apps and password managers to protect sensitive content from screen capture.

## Requirements
- Windows 10 version 2004+ (build 19041+)
- Run as **Administrator** for full per-app shielding capability

## Building
```bash
# Install dependencies
cd tauri-app && npm install

# Dev mode
npm run tauri dev

# Production build
npm run tauri build
```

## Tech Stack
- **Frontend**: React + Vite + TypeScript
- **Backend**: Rust + Tauri v2
- **Win32 API**: `windows` crate v0.58
