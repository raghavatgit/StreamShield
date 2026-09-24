// StreamShield PAGE_GUARD Canary Traps
// Marks sentinel memory regions with PAGE_GUARD to catch out-of-bounds scanners and cheats.

#include <windows.h>
#include <iostream>

class MemoryCanaryGuard {
    void* m_canaryPage;
    size_t m_pageSize;

public:
    MemoryCanaryGuard() : m_canaryPage(nullptr), m_pageSize(4096) {}

    bool ArmCanary() {
        m_canaryPage = VirtualAlloc(NULL, m_pageSize, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
        if (!m_canaryPage) return false;

        DWORD oldProtect;
        if (!VirtualProtect(m_canaryPage, m_pageSize, PAGE_READWRITE | PAGE_GUARD, &oldProtect)) {
            VirtualFree(m_canaryPage, 0, MEM_RELEASE);
            m_canaryPage = nullptr;
            return false;
        }
        return true;
    }

    ~MemoryCanaryGuard() {
        if (m_canaryPage) {
            VirtualFree(m_canaryPage, 0, MEM_RELEASE);
        }
    }
};
