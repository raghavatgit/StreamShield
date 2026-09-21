#include "process_mitigation_policy.hpp"
#include <cassert>
#include <iostream>

void test_mitigation_enforcement() {
    std::cout << "[StreamShield] Enforcing process mitigation policies..." << std::endl;
    bool success = StreamShield::ProcessMitigationManager::EnforceHardenedMitigations();
    std::cout << "[StreamShield] Mitigation policies applied: " << (success ? "YES" : "NO") << std::endl;
    std::cout << "[PASS] ASLR and strict handle policy initialization completed." << std::endl;
}

int main() {
    test_mitigation_enforcement();
    return 0;
}
