#ifndef STREAMSHIELD_DRIVER_FILTER_HPP
#define STREAMSHIELD_DRIVER_FILTER_HPP

#include <windows.h>
#include <cstdint>

namespace StreamShield {

#pragma pack(push, 1)
struct IpcPacketHeader {
    uint32_t magic;      // 0x53534844 ("SSHD")
    uint16_t commandId;
    uint16_t payloadLen;
    uint32_t checksum;   // Header + payload CRC32
};
#pragma pack(pop)

class DriverCommunicationFilter {
public:
    static constexpr uint32_t SSHD_MAGIC = 0x53534844;

    // Verifies packet integrity before passing to kernel dispatch
    static bool ValidatePacket(const IpcPacketHeader* header, const uint8_t* payload) {
        if (!header) return false;
        if (header->magic != SSHD_MAGIC) return false;
        if (header->payloadLen > 4096) return false; // Enforce bounded IPC buffer

        // Check for integer overflow in size calculation
        if (reinterpret_cast<uintptr_t>(payload) + header->payloadLen < reinterpret_cast<uintptr_t>(payload)) {
            return false;
        }

        return true;
    }
};

} // namespace StreamShield

#endif // STREAMSHIELD_DRIVER_FILTER_HPP
