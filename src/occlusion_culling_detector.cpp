#include "../include/occlusion_culling_detector.hpp"

namespace StreamShield {

bool OcclusionCullingDetector::IsWindowDrawable(HWND hwnd) {
    if (!IsWindow(hwnd)) return false;
    if (!IsWindowVisible(hwnd)) return false;
    if (IsIconic(hwnd)) return false;

    RECT rect;
    if (!GetWindowRect(hwnd, &rect)) return false;
    if (rect.right <= rect.left || rect.bottom <= rect.top) return false;

    return true;
}

bool OcclusionCullingDetector::RectsOverlap(const RECT& r1, const RECT& r2) {
    return (r1.left < r2.right && r1.right > r2.left &&
            r1.top < r2.bottom && r1.bottom > r2.top);
}

WindowVisibilityState OcclusionCullingDetector::EvaluateVisibility(HWND hwnd) {
    if (!IsWindowDrawable(hwnd)) {
        return WindowVisibilityState::MinimizedOrHidden;
    }

    RECT target_rect;
    GetWindowRect(hwnd, &target_rect);

    HWND sibling = GetWindow(hwnd, GW_HWNDPREV);
    while (sibling != nullptr) {
        if (IsWindowVisible(sibling) && !IsIconic(sibling)) {
            RECT sibling_rect;
            if (GetWindowRect(sibling, &sibling_rect)) {
                // If a sibling completely envelops target window
                if (sibling_rect.left <= target_rect.left &&
                    sibling_rect.top <= target_rect.top &&
                    sibling_rect.right >= target_rect.right &&
                    sibling_rect.bottom >= target_rect.bottom) {
                    return WindowVisibilityState::CompletelyCovered;
                }
            }
        }
        sibling = GetWindow(sibling, GW_HWNDPREV);
    }

    return WindowVisibilityState::FullyVisible;
}

} // namespace StreamShield
