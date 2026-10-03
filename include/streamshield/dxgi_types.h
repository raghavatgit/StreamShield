#ifndef INCLUDE_STREAMSHIELD_DXGI_TYPES_H
#define INCLUDE_STREAMSHIELD_DXGI_TYPES_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    uint32_t header_flags;
    uint32_t payload_len;
    uint64_t timestamp_us;
} StreamShield_packet_t;

bool StreamShield_initialize_subsystem(void);
bool StreamShield_dispatch_packet(const StreamShield_packet_t* packet);

#ifdef __cplusplus
}
#endif

#endif // INCLUDE_STREAMSHIELD_DXGI_TYPES_H
