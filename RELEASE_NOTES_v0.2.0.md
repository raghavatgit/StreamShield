# StreamShield v0.2.0 - Major Feature Release & Capture Compatibility Engine

**Publisher**: Raghav Goyal  
**Release Date**: August 31, 2026  
**Platforms**: Windows 10 / 11 (64-bit)

---

## 🚀 Headlining Feature: Smart Capture Compatibility & NVIDIA ShadowPlay Support

StreamShield v0.2.0 introduces native support and compatibility enhancements for **NVIDIA ShadowPlay, GeForce Experience, NVIDIA App, and Instant Replay**:

- **Dual Display Affinity Modes**:
  - **Invisible / Transparent (`WDA_EXCLUDEFROMCAPTURE`)**: Best for OBS Studio, Discord Screen Share, Zoom, Microsoft Teams, and Google Meet. Windows completely disappear from the stream; viewers see whatever is behind the window.
  - **Black Screen Mask (`WDA_MONITOR`)**: Best for **NVIDIA ShadowPlay, GeForce Experience, and NVIDIA App**. Renders private windows as solid black rectangles, completely eliminating *"A protected app is preventing screen recording"* DRM recording pauses.
- **Hardware Multiplane Overlay (MPO) Optimization**: Automatically configures Windows Desktop Window Manager (DWM) compositing so that hardware Multiplane Overlays (MPO) can no longer leak private window frames into GPU-direct screen captures.
- **Real-Time Capture Tool Detection**: Automatically detects active streaming and recording software (NVIDIA ShadowPlay, OBS Studio, Discord) and suggests the optimal affinity mode.
- **Flawless Clipping**: Gamers and content creators can now clip highlights or record their screens with NVIDIA ShadowPlay while keeping Discord, Telegram, banking tabs, and private windows 100% hidden.

---

## 🌟 What's New in v0.2.0

### 1. 🎮 Privacy Engine & Capture Compatibility Tuning
- **Dual Affinity Modes**: Instant switching between Transparent (0x11) and Black Screen Mask (0x01) directly inside Settings → Engine.
- **Hardware MPO Optimization Toggle**: One-click toggle in Settings → Engine to force full DWM compositing on hardware overlay planes, with real-time registry status display.
- **Background Watchdog Auto-Reapply**: Automatically re-applies capture protection when shielded applications restart or open secondary windows.

### 2. 🚀 Windows Startup & System Integration
- **Native Registry Autostart**: Enable/disable launching StreamShield on Windows boot (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run\StreamShield`) with zero third-party background services.
- **Start Minimized to System Tray**: Silently launch in the notification tray on boot without flashing the main window.
- **App Stealth Launch Controls**: Option to configure whether StreamShield shields its own UI on launch.

### 3. ⚡ Dynamic Scanning & Performance Tuning
- **Configurable Polling Rates**: Choose from 2s (Fast), 3s (Balanced), 5s (Low CPU), 10s (Battery Saver), or Manual Refresh only.

### 4. 🎨 Aesthetic & UI Overhaul
- **Custom Vector Symbols (No Raw Emojis)**: Replaced all raw unicode emojis with crisp, high-resolution SVG glyphs across navigation tabs, search bars, header actions, and modal callouts.
- **Seamless Slide/Nav Strip**: Redesigned tab strip into a clean 5-column segmented grid with no native Windows scrollbars.
- **Compact Card Mode**: Condensed application layout designed for power users with 20+ active windows.
- **PID Badges Toggle**: Option to show or hide Windows Process ID tags.
- **Batch Action Confirmation**: Confirmation dialogs for *Shield All* and *Clear* to prevent accidental clicks.

### 5. ⚙️ Maintenance & Diagnostics
- **One-Click Clear All**: Immediately unshields all protected application windows and restores capture visibility.
- **Factory Reset**: Restores default settings while keeping the protected application list intact.
- **System Diagnostics**: Real-time privilege elevation, process count, version, and publisher status.

### 6. 🧹 Codebase Quality
- Completely eliminated em-dashes across comments, tooltips, documentation, and source code.
- Added publisher metadata ("Raghav Goyal") to Windows binaries, MSI installer, and NSIS setup.
- Cleaned up obsolete directories and redundant build artifacts.

---

## 📦 Download Assets

| File | Size | Publisher | Description |
|---|---|---|---|
| **`streamshield.exe`** | `3.17 MB` | Raghav Goyal | Standalone portable executable (No installation needed) |
| **`StreamShield_0.2.0_x64-setup.exe`** | `1.28 MB` | Raghav Goyal | Standard Windows NSIS Setup Installer |
| **`StreamShield_0.2.0_x64_en-US.msi`** | `1.95 MB` | Raghav Goyal | Microsoft Installer Package for Enterprise / System Deployment |
