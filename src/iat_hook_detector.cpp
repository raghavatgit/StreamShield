// StreamShield IAT Hook Detector
// Validates that all imported function addresses in the IAT reside within expected module memory bounds.

#include <windows.h>
#include <iostream>

bool ValidateIAT(HMODULE hTargetModule, const char* expectedImportDll) {
    if (!hTargetModule) hTargetModule = GetModuleHandle(NULL);
    PIMAGE_DOS_HEADER dosHeader = (PIMAGE_DOS_HEADER)hTargetModule;
    if (dosHeader->e_magic != IMAGE_DOS_SIGNATURE) return false;

    PIMAGE_NT_HEADERS ntHeaders = (PIMAGE_NT_HEADERS)((BYTE*)hTargetModule + dosHeader->e_lfanew);
    IMAGE_DATA_DIRECTORY importDir = ntHeaders->OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_IMPORT];
    if (importDir.VirtualAddress == 0) return true;

    PIMAGE_IMPORT_DESCRIPTOR importDesc = (PIMAGE_IMPORT_DESCRIPTOR)((BYTE*)hTargetModule + importDir.VirtualAddress);

    HMODULE hImportMod = GetModuleHandleA(expectedImportDll);
    if (!hImportMod) return true;

    MODULEINFO modInfo;
    if (!GetModuleInformation(GetCurrentProcess(), hImportMod, &modInfo, sizeof(modInfo))) return false;

    uintptr_t modBase = (uintptr_t)modInfo.lpBaseOfDll;
    uintptr_t modEnd = modBase + modInfo.SizeOfImage;

    while (importDesc->Name) {
        const char* modName = (const char*)((BYTE*)hTargetModule + importDesc->Name);
        if (_stricmp(modName, expectedImportDll) == 0) {
            PIMAGE_THUNK_DATA thunk = (PIMAGE_THUNK_DATA)((BYTE*)hTargetModule + importDesc->FirstThunk);
            while (thunk->u1.Function) {
                uintptr_t funcAddr = (uintptr_t)thunk->u1.Function;
                // If the pointer lies outside the target DLL's legitimate mapped memory range:
                if (funcAddr < modBase || funcAddr >= modEnd) {
                    return false; // IAT Hook detected
                }
                thunk++;
            }
        }
        importDesc++;
    }

    return true; // Clean
}
