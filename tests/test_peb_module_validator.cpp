#include "peb_module_validator.hpp"
#include <cassert>
#include <iostream>

void test_peb_enumeration() {
    std::cout << "[StreamShield] Traversing Process Environment Block (PEB)..." << std::endl;
    auto modules = StreamShield::PebModuleValidator::EnumeratePebModules();
    std::cout << "[StreamShield] Successfully discovered " << modules.size() << " loaded modules." << std::endl;

    assert(!modules.empty());
    assert(modules[0].baseAddress != 0);
    std::cout << "[PASS] PEB loader structures validated." << std::endl;
}

int main() {
    test_peb_enumeration();
    return 0;
}
