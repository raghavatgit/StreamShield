#ifndef STREAMSHIELD_MEMORY_CRC_SCANNER_HPP
#define STREAMSHIELD_MEMORY_CRC_SCANNER_HPP

#include <windows.h>
#include <cstdint>

namespace StreamShield {

class MemoryIntegrityScanner {
public:
    // Computes IEEE 802.3 CRC-32 over specified memory buffer
    static uint32_t ComputeCRC32(const uint8_t* data, size_t length) {
        uint32_t crc = 0xFFFFFFFF;
        for (size_t i = 0; i < length; i++) {
            crc ^= data[i];
            for (int j = 0; j < 8; j++) {
                crc = (crc >> 1) ^ (0xEDB88320 & (-(crc & 1)));
            }
        }
        return ~crc;
    }

    // Scans PE section memory and validates checksum against baseline
    static bool ValidateSectionIntegrity(const void* sectionBase, size_t sectionSize, uint32_t expectedCrc) {
        MEMORY_BASIC_INFORMATION mbi;
        if (VirtualQuery(sectionBase, &mbi, sizeof(mbi)) == 0) {
            return false;
        }

        // Only scan readable executable code pages
        if (!(mbi.Protect & (PAGE_EXECUTE_READ | PAGE_EXECUTE_READWRITE))) {
            return false;
        }

        uint32_t currentCrc = ComputeCRC32(static_cast<const uint8_t*>(sectionBase), sectionSize);
        return currentCrc == expectedCrc;
    }
};

} // namespace StreamShield

#endif // STREAMSHIELD_MEMORY_CRC_SCANNER_HPP
