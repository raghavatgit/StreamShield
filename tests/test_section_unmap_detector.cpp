#include "section_unmap_detector.hpp"
#include <cassert>
#include <iostream>

void test_module_mapping() {
    std::cout << "[StreamShield] Checking main module memory section mapping..." << std::endl;
    bool isMapped = StreamShield::SectionIntegrityValidator::ValidateMainModuleMapped();
    assert(isMapped);
    std::cout << "[PASS] Executable base confirmed mapped as MEM_IMAGE." << std::endl;
}

int main() {
    test_module_mapping();
    return 0;
}
