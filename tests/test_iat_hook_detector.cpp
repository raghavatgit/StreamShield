#include "iat_hook_detector.hpp"
#include <cassert>
#include <iostream>

void test_iat_scanning() {
    std::cout << "[StreamShield] Initiating IAT integrity scan..." << std::endl;
    auto reports = StreamShield::IATHookDetector::ScanModuleIAT();
    std::cout << "[StreamShield] Verified " << reports.size() << " import entries." << std::endl;

    for (const auto& rep : reports) {
        assert(!rep.isHooked);
    }
    std::cout << "[PASS] All import table pointers validated." << std::endl;
}

int main() {
    test_iat_scanning();
    std::cout << "All IAT detector tests passed successfully." << std::endl;
    return 0;
}
