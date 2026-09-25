#include "../include/dpi_awareness_scaler.hpp"

namespace StreamShield {

double DpiAwarenessScaler::GetWindowScaleFactor(HWND hwnd) {
    if (!IsWindow(hwnd)) {
        return 1.0;
    }

    // Windows 10 build 1607+ supports GetDpiForWindow
    HMODULE user32 = GetModuleHandleW(L"user32.dll");
    if (user32) {
        typedef UINT(WINAPI * PFN_GetDpiForWindow)(HWND);
        auto pfnGetDpiForWindow = reinterpret_cast<PFN_GetDpiForWindow>(GetProcAddress(user32, "GetDpiForWindow"));
        if (pfnGetDpiForWindow) {
            UINT dpi = pfnGetDpiForWindow(hwnd);
            if (dpi > 0) {
                return static_cast<double>(dpi) / 96.0;
            }
        }
    }

    // Fallback to primary screen DPI
    HDC screen_dc = GetDC(nullptr);
    int dpi_x = GetDeviceCaps(screen_dc, LOGPIXELSX);
    ReleaseDC(nullptr, screen_dc);
    return static_cast<double>(dpi_x) / 96.0;
}

RECT DpiAwarenessScaler::ScalePhysicalToLogical(HWND hwnd, const RECT& physical_rect) {
    double scale = GetWindowScaleFactor(hwnd);
    if (scale <= 0.0) scale = 1.0;

    RECT logical;
    logical.left = static_cast<LONG>(physical_rect.left / scale);
    logical.top = static_cast<LONG>(physical_rect.top / scale);
    logical.right = static_cast<LONG>(physical_rect.right / scale);
    logical.bottom = static_cast<LONG>(physical_rect.bottom / scale);
    return logical;
}

RECT DpiAwarenessScaler::ScaleLogicalToPhysical(HWND hwnd, const RECT& logical_rect) {
    double scale = GetWindowScaleFactor(hwnd);

    RECT physical;
    physical.left = static_cast<LONG>(logical_rect.left * scale);
    physical.top = static_cast<LONG>(logical_rect.top * scale);
    physical.right = static_cast<LONG>(logical_rect.right * scale);
    physical.bottom = static_cast<LONG>(logical_rect.bottom * scale);
    return physical;
}

} // namespace StreamShield
