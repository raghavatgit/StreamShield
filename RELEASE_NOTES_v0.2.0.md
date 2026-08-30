# StreamShield v0.2.0 - Major Feature & Settings Update

**Publisher**: Raghav Goyal  
**Release Date**: August 31, 2026  
**Platforms**: Windows 10 / 11 (64-bit)

---

## What's New in v0.2.0

### 1. Windows Startup & Background Launch
- **Native Registry Autostart**: Enable/disable launching StreamShield on Windows boot (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run\StreamShield`) with zero third-party dependencies.
- **Start Minimized to System Tray**: Automatically start silently in the background without popping up the main window.

### 2. Privacy & Shield Engine Customization
- **Display Affinity Modes**:
  - **Exclude from Capture (0x11)**: Completely invisible on OBS, Discord, and screen share captures (transparent to viewers).
  - **Black Screen Mask (0x01)**: Obscures protected windows with a solid black rectangle.
- **Background Watchdog Auto-Reapply**: Automatically re-applies capture protection when shielded applications restart or open secondary windows.

### 3. Dynamic Scanning & Performance Tuning
- **Configurable Polling Rates**: Choose from 2s (Fast), 3s (Balanced), 5s (Low CPU), 10s (Battery Saver), or Manual Refresh only.

### 4. Aesthetic & UI Overhaul
- **Custom Vector Symbols**: Replaced all raw unicode emojis with crisp, high-resolution SVG glyphs across navigation tabs, search bars, header actions, and modal callouts.
- **Seamless Slide/Nav Strip**: Redesigned tab strip into a clean 5-column segmented grid with no native Windows scrollbars.
- **Compact Card Mode**: Condensed application layout designed for power users with 20+ active windows.
- **PID Badges Toggle**: Option to show or hide Windows Process ID tags.
- **Batch Action Confirmation**: Confirmation dialogs for *Shield All* and *Clear* to prevent accidental clicks.

### 5. Maintenance & Diagnostics
- **One-Click Clear All**: Immediately unshields all protected application windows and restores capture visibility.
- **Factory Reset**: Restores default settings while keeping the protected application list intact.
- **System Diagnostics**: Real-time privilege elevation, process count, version, and publisher status.

### 6. Codebase Quality
- Completely eliminated em-dashes across comments, tooltips, documentation, and source code.
- Added publisher metadata ("Raghav Goyal") to Windows binaries, MSI installer, and NSIS setup.

---

## Download Assets

| File | Size | Description |
|---|---|---|
| **`streamshield.exe`** | `3.16 MB` | Standalone portable executable (No installation needed) |
| **`StreamShield_0.2.0_x64-setup.exe`** | `1.27 MB` | Standard Windows NSIS Setup Installer |
| **`StreamShield_0.2.0_x64_en-US.msi`** | `1.94 MB` | Microsoft Installer Package for Enterprise / System Deployment |
