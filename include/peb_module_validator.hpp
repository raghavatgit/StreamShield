#ifndef STREAMSHIELD_PEB_MODULE_VALIDATOR_HPP
#define STREAMSHIELD_PEB_MODULE_VALIDATOR_HPP

#include <windows.h>
#include <winternl.h>
#include <cstdint>
#include <vector>
#include <string>

namespace StreamShield {

struct ModuleEntryInfo {
    std::wstring modulePath;
    uintptr_t baseAddress;
    size_t sizeOfImage;
    bool isValid;
};

class PebModuleValidator {
public:
    // Traverses PEB InMemoryOrderModuleList to detect unlinked/hidden stealth modules
    static std::vector<ModuleEntryInfo> EnumeratePebModules() {
        std::vector<ModuleEntryInfo> modules;

#if defined(_M_X64) || defined(__x86_64__)
        auto* peb = reinterpret_cast<PPEB>(__readgsqword(0x60));
#else
        auto* peb = reinterpret_cast<PPEB>(__readfsdword(0x30));
#endif

        if (!peb || !peb->Ldr) return modules;

        PLIST_ENTRY head = &peb->Ldr->InMemoryOrderModuleList;
        PLIST_ENTRY curr = head->Flink;

        while (curr != head && curr != nullptr) {
            auto* tableEntry = CONTAINING_RECORD(curr, LDR_DATA_TABLE_ENTRY, InMemoryOrderLinks);

            if (tableEntry->DllBase != nullptr) {
                std::wstring name = tableEntry->FullDllName.Buffer ?
                    std::wstring(tableEntry->FullDllName.Buffer, tableEntry->FullDllName.Length / sizeof(wchar_t)) :
                    L"Unknown";

                modules.push_back({
                    name,
                    reinterpret_cast<uintptr_t>(tableEntry->DllBase),
                    0,
                    true
                });
            }

            curr = curr->Flink;
        }

        return modules;
    }
};

} // namespace StreamShield

#endif // STREAMSHIELD_PEB_MODULE_VALIDATOR_HPP
