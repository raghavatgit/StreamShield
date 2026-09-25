#include "../include/occlusion_culling_detector.hpp"
#include <cassert>
#include <iostream>

void TestOcclusionDetector() {
    StreamShield::OcclusionCullingDetector detector;

    // NULL handle must return false
    assert(!detector.IsWindowDrawable(nullptr));

    // Desktop window handle check
    HWND desktop = GetDesktopWindow();
    assert(IsWindow(desktop));

    std::cout << "[PASS] TestOcclusionDetector completed successfully." << std::endl;
}

int main() {
    TestOcclusionDetector();
    return 0;
}
