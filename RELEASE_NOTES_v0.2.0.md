# StreamShield v0.2.0 - Major Feature Release & NVIDIA ShadowPlay Bypass

**Publisher**: Raghav Goyal  
**Release Date**: August 31, 2026  
**Platforms**: Windows 10 / 11 (64-bit)

---

## 🚀 Headlining Feature: NVIDIA ShadowPlay & GeForce Experience Bypass

StreamShield v0.2.0 introduces native support and compatibility workarounds for **NVIDIA ShadowPlay, GeForce Experience, and Instant Replay**:

- **Bypasses Hardware MPO Plane Leakage**: Automatically configures Windows Desktop Window Manager (DWM) compositing so that hardware Multi-Plane Overlays (MPO) can no longer bypass display capture affinity.
- **Eliminates Capture Lockouts**: Prevents NVIDIA from tripping its DRM killswitch (*"An app is preventing screen recording"*) and locking up full-screen or desktop recording when StreamShield is running.
- **Flawless Clipping**: Gamers and content creators can now clip highlights or record their screens with NVIDIA ShadowPlay while keeping Discord, Telegram, banking tabs, and private windows 100% invisible.

---

## 🌟 What's New in v0.2.0

### 1. 🎮 NVIDIA ShadowPlay & Privacy Engine Tuning
- **NVIDIA ShadowPlay & Overlay Bypass (MPO Fix)**: One-click toggle in Settings → Engine to force full DWM compositing on hardware overlay planes.
- **Configurable Display Affinity Modes**:
  - **Exclude from Capture (0x11)**: Target windows vanish into transparent space on streams, recordings, and screen shares.
  - **Black Screen Mask (0x01)**: Obscures target windows with a solid black privacy rectangle.
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

---

## 📦 Download Assets

| File | Size | Publisher | Description |
|---|---|---|---|
| **`streamshield.exe`** | `3.16 MB` | Raghav Goyal | Standalone portable executable (No installation needed) |
| **`StreamShield_0.2.0_x64-setup.exe`** | `1.27 MB` | Raghav Goyal | Standard Windows NSIS Setup Installer |
| **`StreamShield_0.2.0_x64_en-US.msi`** | `1.94 MB` | Raghav Goyal | Microsoft Installer Package for Enterprise / System Deployment |
