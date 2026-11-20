# StreamShield Changelog

All notable changes to StreamShield are documented in this file.

## [0.2.5] - 2026-11-20
### Added & Improved
- **Theme-Compliant Custom Tray Right-Click Popover (`tray_menu`)**:
  - Replaced the native Windows `HMENU` tray context menu with a pre-warmed, frameless transparent WebView2 popover featuring top coverage telemetry, action trio (`Shield All` / `Stealth` / `Reapply`), scrollable 1-tap app list with live native icons, Capture Mode segmented selector (`Transparent` / `Black Mask`), Desktop Widget toggle + `Pill | Dock | Badge` switcher, `Compact Window` switch, `Preferences`, and `Quit`.
  - Protected with `WS_EX_TOOLWINDOW`, Win11 DWM border/corner suppression, `apply_own_window_stealth` capture exclusion, multi-monitor/all-taskbar-edge positioning (`compute_tray_menu_position`), right-click blur-bounce suppression, and `Esc`/arrow-key keyboard navigation.
- **Minimalistic, Futuristic Desktop Widget Redesign**:
  - **Dynamic Pill / Island**: Specular rim lighting, top active `--brand-gradient` shimmer strip, hover/focus-revealed drag grip, live status pulse dot with tabular counter, squircle app tiles with luminous bottom shield bars, and `IconShieldCheck` / `IconShield` state icons.
  - **Floating Compact Dock**: Top edge-to-edge `--brand-gradient` coverage progress rail, `ALL` / `ACTIVE` / `IDLE` telemetry tag, elevated 4-column squircle grid, and integrated `Pill | Dock | Badge` footer switcher.
  - **Corner Status Badge**: Precision SVG orbital progress ring (`ss-badge-ring`) around the StreamShield emblem with an expandable HUD containing telemetry ratio, quick action chips, 5 quick app toggles, mode switcher, and `Hide Widget`.
  - Updated Settings Preferences widget preview cards to match all three redesigned HUDs.
- **Per-Theme Atmospheric Glass & Accessibility Tokens**:
  - Added per-theme `surfaceRgb` base glass palettes, sRGB-contrast-computed `--on-primary`, semantic `--widget-*` / `--tray-*` tokens (removing hardcoded emerald/amber widget overrides), standard thin themed scrollbars with legacy WebKit fallbacks, and Windows Forced Colors (`forced-colors: active`) high-contrast support across all 9 themes.

## [0.2.4] - 2026-11-19
### Fixed
- **Desktop Screen Widget Overhaul**:
  - Unified WebView2 `additional_browser_args` across `main` and `widget` windows (fixing secondary WebView2 initialization failure `0x8007139F`).
  - Converted blocking IPC/Win32 operations (`toggle_widget`, `update_settings`, `get_windows`, `toggle_shield`, `clear_all_shields`) to non-blocking async tasks cleanly separated from mutex locks.
  - Preserved transparent DirectComposition alpha channel via dedicated self-stealth calls without `SWP_FRAMECHANGED`/`RDW_ERASE`.
  - Applied `WS_EX_TOOLWINDOW` (keeping widget out of `Alt+Tab`), Windows 11 border/corner DWM suppression (`DWMWA_WINDOW_CORNER_PREFERENCE`, `DWMWA_BORDER_COLOR`), persisted `widget_x`/`widget_y` coordinates, multi-monitor `work_area()` clamping, and clean tray sync on widget close.
  - Added 5px drag threshold and quadrant-aware inward expansion for the Corner Status Badge, scrollable multi-app grids and drag grips on Pill/Dock, and live inline error display.
- **Windows 10 & 11 Compatibility**:
  - Fixed `ProcessPowerThrottling` (`4`) enum value and removed `SetThreadExecutionState` sleep-blocking flag.
  - Upgraded DLL injection to Unicode `LoadLibraryW`, added `IsWow64Process` 32-bit target safety guard, and skipped redundant remote DLL loads.
  - Added cross-integrity (`Local\StreamShield_WakeEvent`) single-instance restore for elevated/tray-only instances.
  - Added dual Registry MPO status detection covering both Windows 10 (`Dwm\OverlayTestMode`) and Windows 11 24H2+ (`GraphicsDrivers\DisableOverlays`).
- **Capture Mode Switching & Multi-Process Shielding**:
  - Fixed `shield_dll.dll` skipping switches between `WDA_EXCLUDEFROMCAPTURE` and `WDA_MONITOR`.
  - Shielding/unshielding now covers all visible top-level processes of multi-process applications (browsers, Electron apps, etc.).
- **UI/UX, Themes & Security**:
  - Defined all missing semantic CSS variables and dedicated light/dark `--widget-*` tokens across all 9 themes, fixing Clean Light contrast, modal Diagnostics layout, and toast re-animations.
  - Added URL scheme and update origin validation plus MSI/NSIS installer asset selection.
- **Repository Cleanup & Tests**:
  - Removed unused placeholder root directories and added Rust unit tests covering settings compatibility, widget geometry/clamping, and URL validation.

## [0.2.3] - 2026-10-30
### Added
- Desktop Screen Widgets in three styles (`Dynamic Pill`, `Compact Dock`, `Corner Status Badge`) with settings previews and tray submenu.
- Smooth native DWM window dragging and click isolation on widget buttons.
- Bidirectional real-time stealth and shield state synchronization.

## [0.2.2] - 2026-10-30
### Added
- Ultra-Compact icon grid mode with one-tap shielding, master toggle switch, and stealth eye button.

## [0.2.1] - 2026-10-30
### Added
- Dynamic color theme engine with 9 theme presets and generated UI peripherals.
- Updated StreamShield brand logo and header wordmark, plus dynamic tray app toggles and in-app updater.

## [0.2.0] - 2026-08-30
### Added
- Comprehensive 5-tab Settings subsystem, Windows startup registry integration, NVIDIA ShadowPlay / Medal Black Screen Mask mode, and MPO registry optimizer.
