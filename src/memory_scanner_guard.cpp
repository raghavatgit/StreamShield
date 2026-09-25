// StreamShield Memory Scanner Tripwire
// Traps external memory scanners attempting sequential memory reading.

#include <windows.h>

class ScannerTripwire {
    void* m_trapPage;

public:
    ScannerTripwire() : m_trapPage(nullptr) {}

    bool ArmTripwire() {
        m_trapPage = VirtualAlloc(NULL, 4096, MEM_COMMIT | MEM_RESERVE, PAGE_NOACCESS);
        return m_trapPage != nullptr;
    }

    ~ScannerTripwire() {
        if (m_trapPage) VirtualFree(m_trapPage, 0, MEM_RELEASE);
    }
};
