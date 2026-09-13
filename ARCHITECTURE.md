# StreamShield Architecture Specification

## Overview
StreamShield enforces per-application screen capture privacy by injecting a lightweight dynamic library (`shield_dll.dll`) directly into target processes and calling the native Windows API:

```c
SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE);
```

## System Topology

```text
[ React Frontend UI ] <--> [ Tauri Rust Core ] <--> [ Windows Win32 API ]
                                    |
                 CreateRemoteThread / LoadLibraryW
                                    v
                     [ Target Process (e.g. Discord) ]
                                    |
                     [ shield_dll.dll Injected Hook ]
                                    |
                     SetWindowDisplayAffinity(HWND)
                                    v
               [ Windows Desktop Window Manager (DWM) ]
             /                                          \
    [ Physical Display ]                     [ Capture Feeds (OBS/Discord) ]
       Visible (100%)                              Masked / Excluded (0%)
```

## Fault Tolerance
* **ASLR Address Resolution:** Computes kernel entry points in uniform session memory.
* **Non-Blocking Watchdog:** Polls process lifecycles asynchronously without UI thread stalls.
