<div align="center">

# 🛡️ StreamShield

**Per-Application Stream Privacy Manager for Windows**

*Hide private applications (Discord, WhatsApp, Spotify, Chrome, Banking, IDEs) from your live streams, screen shares, and recordings with a single toggle - while keeping them 100% visible on your monitor.*

[![Platform](https://img.shields.io/badge/Platform-Windows%2010%20%7C%2011-0078D6?style=for-the-badge&logo=windows)](https://github.com/raghavatgit/StreamShield)
[![Rust](https://img.shields.io/badge/Backend-Rust%202021-DEA584?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/Framework-Tauri%20v2-FFC131?style=for-the-badge&logo=tauri)](https://tauri.app/)
[![License](https://img.shields.io/badge/License-MIT-22C55E?style=for-the-badge)](LICENSE)
[![Binary Size](https://img.shields.io/badge/Portable%20Binary-3.1%20MB-6366F1?style=for-the-badge)](#-download--installation)

---

</div>

## 💡 The Problem vs. The StreamShield Solution

| Traditional Screen Sharing | With StreamShield 🛡️ |
| :--- | :--- |
| ❌ Sharing your entire screen accidentally leaks private DMs, Discord servers, Spotify playlists, credentials, and browsing tabs. | ✅ Toggle any app in StreamShield. It **vanishes into transparent/black space on stream** while remaining completely visible to you. |
| ❌ Sharing individual windows prevents you from multitasking or showing other apps seamlessly. | ✅ Share your entire display with zero fear of accidental leakages. |
| ❌ Bulky OBS plugins require complex setup and affect broadcast performance. | ✅ **100% Standalone & Driverless**. Works natively across **Discord, OBS Studio, Zoom, Microsoft Teams, Google Meet, and GeForce Experience**. |

---

## ✨ Features

- 🎮 **NVIDIA ShadowPlay & GeForce Experience Bypass**: Bypasses hardware Multi-Plane Overlay (MPO) plane leakage and prevents NVIDIA capture lockouts (*"An app is preventing screen recording"*) so Instant Replay and Desktop Capture cleanly exclude shielded apps.
- 🔒 **Zero-Latency Hardware Exclusion**: Leverages the Windows Desktop Window Manager (`DWM`) compositing pipeline (`SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)` and `WDA_MONITOR`).
- 🚀 **100% Standalone & Portable**: Single ~3.1 MB executable. No installation required, zero background drivers, zero bloat.
- ⚙️ **Comprehensive Settings Subsystem**: Windows Startup autostart, minimize to tray, customizable scan polling intervals (2s to 10s), and batch confirmation dialogs.
- 🎨 **3 Curated Theme Presets**:
  - 🌌 **Cyberpunk Glow**: Electric cyan & hot pink neon with ambient backdrop glows.
  - 💬 **Discord Dark**: Discord-native dark palette with Emerald active accents.
  - ☀️ **Clean White**: Crisp, high-contrast modern slate & indigo light theme.
- 🔄 **Autonomous Background Watchdog**: Automatically detects and re-shields private applications when they restart or open new windows.
- ⚡ **Anti-Sleep & Tray Throttling Protection**: Native Windows power management prevents background thread suspension when docked to the system tray.
- 🎧 **Zero Audio Interference**: Plays crystal-clear uncompressed audio directly into your physical headphones at 0ms latency without altering Windows audio sessions.
- 🔍 **Live Process Search & Filtering**: Instant search across running processes, window titles, and PIDs.

---

## 🛠️ How It Works (Architecture)

```
┌─────────────────────────────────────────────────────────┐
│                    StreamShield GUI                     │
│           (React 18 + TypeScript + Vite)                │
└──────────────────────────┬──────────────────────────────┘
                           │ Tauri IPC Invocation
┌──────────────────────────▼──────────────────────────────┐
│                  Rust Core Engine                       │
│    • Enumerates visible windows & extracts native icons │
│    • Remote ASLR DLL Injector (shield_dll.dll)          │
│    • Background watchdog & Power state controller       │
└──────────────────────────┬──────────────────────────────┘
                           │ Injects into Target Process
┌──────────────────────────▼──────────────────────────────┐
│             Windows DWM Compositor                      │
│     SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)    │
└──────────────────────────┬──────────────────────────────┘
             ┌─────────────┴─────────────┐
             │                           │
  [Your Physical Monitor]     [Capture Buffer / Stream Feed]
    100% VISIBLE & LIVE         100% EXCLUDED / HIDDEN
```

1. StreamShield identifies the 64-bit target process and dynamically injects `shield_dll.dll` via remote thread execution with full 64-bit ASLR address resolution.
2. The injected hook executes `SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE)` directly inside the target window's process context.
3. It immediately invalidates the DWM visual surface cache (`SWP_FRAMECHANGED` & `RedrawWindow`), forcing the Windows compositor to exclude the window from all capture streams while keeping it live on your display.

---

## 📥 Download & Installation

### Option 1: Standalone Portable Binary (Recommended)
Download **`streamshield.exe`** from the [Latest Releases](https://github.com/raghavatgit/StreamShield/releases) page.
- **No installation needed** - simply double-click and run!

### Option 2: Windows Installer
Download **`StreamShield-setup.exe`** for a standard Windows installation with Desktop and Start Menu shortcuts.

---

## 💻 System Requirements

- **OS**: Windows 10 (64-bit, Version 2004 / Build 19041 or newer) or Windows 11 (All versions).
- **Architecture**: `x86_64` (64-bit).
- **Permissions**: Administrator privileges (StreamShield automatically requests standard UAC elevation on startup to interact with elevated target processes).

---

## 🔨 Building From Source

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (1.78+)
- [Node.js](https://nodejs.org/) (v18+)
- Visual Studio C++ Build Tools (with Windows 10/11 SDK)

### Clone & Build
```bash
# 1. Clone the repository
git clone https://github.com/raghavatgit/StreamShield.git
cd StreamShield

# 2. Install frontend dependencies
cd tauri-app
npm install

# 3. Build standalone production binary
npm run tauri build -- --no-bundle
```

The optimized standalone binary will be generated at:
```
target4/release/streamshield.exe
```

---

## 🤝 Contributing

Contributions, issues, and feature requests are welcome!
Feel free to check the [issues page](https://github.com/raghavatgit/StreamShield/issues).

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'feat: add some amazing feature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

---

## 📜 License

Distributed under the **MIT License**. See [`LICENSE`](LICENSE) for more information.

<div align="center">
  <sub>Built with ❤️ for streamers, developers, and privacy enthusiasts.</sub>
</div>
