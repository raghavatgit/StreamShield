#pragma once
#include <windows.h>
#include <vector>
#include <string>

namespace StreamShield {

struct MonitorDescriptor {
    HMONITOR handle;
    RECT bounds;
    RECT work_area;
    bool is_primary;
    std::wstring device_name;
};

class DisplayTopologyMonitor {
public:
    DisplayTopologyMonitor() = default;
    ~DisplayTopologyMonitor() = default;

    // Enumerate all active connected physical displays
    std::vector<MonitorDescriptor> EnumerateMonitors();

    // Determine which monitor contains the largest area of target window
    HMONITOR GetDominantMonitorForWindow(HWND hwnd);

    // Compute virtual desktop bounding box across all active screens
    RECT GetVirtualDesktopBounds();

private:
    static BOOL CALLBACK MonitorEnumProc(HMONITOR hMonitor, HDC hdcMonitor, LPRECT lprcMonitor, LPARAM dwData);
};

} // namespace StreamShield
