#ifndef STREAMSHIELD_SECTION_UNMAP_HPP
#define STREAMSHIELD_SECTION_UNMAP_HPP

#include <windows.h>
#include <cstdint>

namespace StreamShield {

class SectionIntegrityValidator {
public:
    // Verifies that the main module image section has not been unmapped (process hollowing detection)
    static bool ValidateMainModuleMapped() {
        HMODULE hModule = GetModuleHandle(nullptr);
        if (!hModule) return false;

        MEMORY_BASIC_INFORMATION mbi;
        if (VirtualQuery(hModule, &mbi, sizeof(mbi)) == 0) {
            return false;
        }

        // Module base address must be committed MEM_IMAGE memory
        if (mbi.State != MEM_COMMIT || mbi.Type != MEM_IMAGE) {
            return false;
        }

        return true;
    }
};

} // namespace StreamShield

#endif // STREAMSHIELD_SECTION_UNMAP_HPP
