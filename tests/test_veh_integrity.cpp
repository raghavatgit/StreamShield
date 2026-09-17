#include "veh_integrity.hpp"
#include <iostream>
#include <cassert>

int main() {
    std::cout << "[StreamShield] Initializing hardware breakpoint validation..." << std::endl;

    bool bp_detected = StreamShield::HardwareBreakpointDetector::HasActiveHardwareBreakpoints();
    std::cout << "[StreamShield] Active hardware breakpoints detected: " << (bp_detected ? "YES" : "NO") << std::endl;

    bool neutralized = StreamShield::HardwareBreakpointDetector::NeutralizeHardwareBreakpoints();
    assert(neutralized);

    std::cout << "[StreamShield] Debug registers cleared and verified." << std::endl;
    return 0;
}
