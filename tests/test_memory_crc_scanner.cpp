#include "memory_crc_scanner.hpp"
#include <cassert>
#include <cstring>
#include <iostream>

void test_crc32_computation() {
    const char* sample = "StreamShield_AntiTamper_TestVector";
    uint32_t crc1 = StreamShield::MemoryIntegrityScanner::ComputeCRC32(
        reinterpret_cast<const uint8_t*>(sample), std::strlen(sample)
    );
    uint32_t crc2 = StreamShield::MemoryIntegrityScanner::ComputeCRC32(
        reinterpret_cast<const uint8_t*>(sample), std::strlen(sample)
    );

    assert(crc1 != 0);
    assert(crc1 == crc2);
    std::cout << "[PASS] Deterministic CRC32 calculation: 0x" << std::hex << crc1 << std::endl;
}

int main() {
    test_crc32_computation();
    std::cout << "All memory CRC integrity tests passed successfully." << std::endl;
    return 0;
}
