#pragma once
#include <windows.h>

namespace StreamShield {

class DpiAwarenessScaler {
public:
    DpiAwarenessScaler() = default;
    ~DpiAwarenessScaler() = default;

    // Return current DPI factor (e.g. 1.0 for 96 DPI, 1.25 for 120 DPI, 1.5 for 144 DPI)
    static double GetWindowScaleFactor(HWND hwnd);

    // Scales a physical pixel rect to logical points
    static RECT ScalePhysicalToLogical(HWND hwnd, const RECT& physical_rect);

    // Scales a logical point rect to physical pixels
    static RECT ScaleLogicalToPhysical(HWND hwnd, const RECT& logical_rect);
};

} // namespace StreamShield
