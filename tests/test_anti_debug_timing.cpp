#include "anti_debug_timing.hpp"
#include <cassert>
#include <iostream>

int main() {
    std::cout << "[StreamShield] Measuring execution cycle latency..." << std::endl;
    bool isStepped = StreamShield::DebugTimingValidator::IsDebuggerSteppingDetected();
    std::cout << "[StreamShield] Interactive debugger stepping detected: " << (isStepped ? "YES" : "NO") << std::endl;

    assert(!isStepped);
    std::cout << "[PASS] Timing threshold verification completed cleanly." << std::endl;
    return 0;
}
