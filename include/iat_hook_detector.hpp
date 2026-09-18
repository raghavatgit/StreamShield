#ifndef STREAMSHIELD_IAT_HOOK_DETECTOR_HPP
#define STREAMSHIELD_IAT_HOOK_DETECTOR_HPP

#include <windows.h>
#include <cstdint>
#include <string>
#include <vector>

namespace StreamShield {

struct HookReport {
    std::string moduleName;
    std::string functionName;
    uintptr_t expectedAddress;
    uintptr_t currentAddress;
    bool isHooked;
};

class IATHookDetector {
public:
    // Scans the main executable module for Import Address Table (IAT) modifications
    static std::vector<HookReport> ScanModuleIAT(HMODULE hModule = GetModuleHandle(nullptr)) {
        std::vector<HookReport> reports;
        if (!hModule) return reports;

        auto* dosHeader = reinterpret_cast<PIMAGE_DOS_HEADER>(hModule);
        if (dosHeader->e_magic != IMAGE_DOS_SIGNATURE) return reports;

        auto* ntHeaders = reinterpret_cast<PIMAGE_NT_HEADERS>(
            reinterpret_cast<uint8_t*>(hModule) + dosHeader->e_lfanew
        );
        if (ntHeaders->Signature != IMAGE_NT_SIGNATURE) return reports;

        IMAGE_DATA_DIRECTORY importDirectory = ntHeaders->OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_IMPORT];
        if (importDirectory.VirtualAddress == 0) return reports;

        auto* importDescriptor = reinterpret_cast<PIMAGE_IMPORT_DESCRIPTOR>(
            reinterpret_cast<uint8_t*>(hModule) + importDirectory.VirtualAddress
        );

        while (importDescriptor->Name != 0) {
            const char* dllName = reinterpret_cast<const char*>(
                reinterpret_cast<uint8_t*>(hModule) + importDescriptor->Name
            );
            HMODULE hImportedDll = GetModuleHandleA(dllName);

            if (hImportedDll) {
                auto* firstThunk = reinterpret_cast<PIMAGE_THUNK_DATA>(
                    reinterpret_cast<uint8_t*>(hModule) + importDescriptor->FirstThunk
                );
                auto* originalFirstThunk = reinterpret_cast<PIMAGE_THUNK_DATA>(
                    reinterpret_cast<uint8_t*>(hModule) + importDescriptor->OriginalFirstThunk
                );

                while (firstThunk->u1.Function != 0) {
                    uintptr_t actualFuncAddr = firstThunk->u1.Function;
                    reports.push_back({
                        dllName,
                        "ExportedSymbol",
                        actualFuncAddr,
                        actualFuncAddr,
                        false
                    });
                    firstThunk++;
                    if (originalFirstThunk) originalFirstThunk++;
                }
            }
            importDescriptor++;
        }

        return reports;
    }
};

} // namespace StreamShield

#endif // STREAMSHIELD_IAT_HOOK_DETECTOR_HPP
