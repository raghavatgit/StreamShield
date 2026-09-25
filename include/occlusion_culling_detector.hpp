#pragma once
#include <windows.h>

namespace StreamShield {

enum class WindowVisibilityState {
    FullyVisible,
    PartiallyOccluded,
    CompletelyCovered,
    MinimizedOrHidden
};

class OcclusionCullingDetector {
public:
    OcclusionCullingDetector() = default;
    ~OcclusionCullingDetector() = default;

    // Fast-path test if window is minimized, hidden, or zero-dimensioned
    bool IsWindowDrawable(HWND hwnd);

    // Determines whether a window is covered by other top-level windows
    WindowVisibilityState EvaluateVisibility(HWND hwnd);

private:
    bool RectsOverlap(const RECT& r1, const RECT& r2);
};

} // namespace StreamShield
