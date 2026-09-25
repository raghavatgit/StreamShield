#include "../include/display_topology_monitor.hpp"
#include <algorithm>

namespace StreamShield {

BOOL CALLBACK DisplayTopologyMonitor::MonitorEnumProc(HMONITOR hMonitor, HDC /*hdcMonitor*/, LPRECT /*lprcMonitor*/, LPARAM dwData) {
    auto* monitors = reinterpret_cast<std::vector<MonitorDescriptor>*>(dwData);

    MONITORINFOEXW minfo;
    minfo.cbSize = sizeof(MONITORINFOEXW);

    if (GetMonitorInfoW(hMonitor, &minfo)) {
        MonitorDescriptor desc;
        desc.handle = hMonitor;
        desc.bounds = minfo.rcMonitor;
        desc.work_area = minfo.rcWork;
        desc.is_primary = (minfo.dwFlags & MONITORINFOF_PRIMARY) != 0;
        desc.device_name = minfo.szDevice;

        monitors->push_back(desc);
    }

    return TRUE;
}

std::vector<MonitorDescriptor> DisplayTopologyMonitor::EnumerateMonitors() {
    std::vector<MonitorDescriptor> monitors;
    EnumDisplayMonitors(nullptr, nullptr, MonitorEnumProc, reinterpret_cast<LPARAM>(&monitors));
    return monitors;
}

HMONITOR DisplayTopologyMonitor::GetDominantMonitorForWindow(HWND hwnd) {
    if (!IsWindow(hwnd)) {
        return nullptr;
    }
    return MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
}

RECT DisplayTopologyMonitor::GetVirtualDesktopBounds() {
    RECT rect;
    rect.left = GetSystemMetrics(SM_XVIRTUALSCREEN);
    rect.top = GetSystemMetrics(SM_YVIRTUALSCREEN);
    rect.right = rect.left + GetSystemMetrics(SM_CXVIRTUALSCREEN);
    rect.bottom = rect.top + GetSystemMetrics(SM_CYVIRTUALSCREEN);
    return rect;
}

} // namespace StreamShield
