# Windows Capture Affinity API Matrix

## Supported Affinity Modes
The Windows API function `SetWindowDisplayAffinity(HWND hWnd, DWORD dwAffinity)` governs whether a top-level window appears in capture pipelines:

| Constant | Value | Behavior |
| :--- | :--- | :--- |
| `WDA_NONE` | `0x00000000` | Window is visible in all screen captures and monitor feeds. |
| `WDA_MONITOR` | `0x00000001` | Window appears only on the primary display and is hidden from captures. |
| `WDA_EXCLUDEFROMCAPTURE` | `0x00000011` | Introduced in Windows 10 version 2004. Window is masked completely black in OBS, Discord, and DXGI Desktop Duplication without interfering with user interaction. |
