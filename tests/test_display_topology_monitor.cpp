#include "../include/display_topology_monitor.hpp"
#include <cassert>
#include <iostream>

void TestDisplayTopologyEnumeration() {
    StreamShield::DisplayTopologyMonitor monitor;
    auto monitors = monitor.EnumerateMonitors();

    // At least one physical or virtual monitor must be detected on active system
    assert(!monitors.empty());

    bool has_primary = false;
    for (const auto& m : monitors) {
        if (m.is_primary) {
            has_primary = true;
            break;
        }
    }
    assert(has_primary);

    RECT vbounds = monitor.GetVirtualDesktopBounds();
    assert(vbounds.right > vbounds.left);
    assert(vbounds.bottom > vbounds.top);

    std::cout << "[PASS] TestDisplayTopologyEnumeration passed. Detected " 
              << monitors.size() << " monitors." << std::endl;
}

int main() {
    TestDisplayTopologyEnumeration();
    return 0;
}
