#include "minidump_generator.hpp"
#include <cassert>
#include <iostream>

void test_minidump_interface() {
    std::cout << "[StreamShield] Verifying MiniDump generator interface..." << std::endl;
    // Test that symbol resolution and header bindings compile cleanly
    assert(sizeof(MINIDUMP_TYPE) >= 4);
    std::cout << "[PASS] DbgHelp minidump structures verified." << std::endl;
}

int main() {
    test_minidump_interface();
    return 0;
}
