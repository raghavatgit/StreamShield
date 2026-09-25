// StreamShield KernelCallbackTable Verification
// Verifies KernelCallbackTable in the PEB points inside user32.dll image boundaries.

#include <windows.h>
#include <psapi.h>

bool VerifyKernelCallbackTable() {
#if defined(_M_X64) || defined(__x86_64__)
    unsigned char* ppeb = reinterpret_cast<unsigned char*>(__readgsqword(0x60));
    void* callbackTable = *reinterpret_cast<void**>(ppeb + 0x58);
#elif defined(_M_IX86) || defined(__i386__)
    unsigned char* ppeb = reinterpret_cast<unsigned char*>(__readfsdword(0x30));
    void* callbackTable = *reinterpret_cast<void**>(ppeb + 0x2C);
#else
    return true;
#endif

    if (!callbackTable) return true; // GUI not initialized

    HMODULE hUser32 = GetModuleHandleA("user32.dll");
    if (!hUser32) return true;

    MODULEINFO modInfo;
    if (!GetModuleInformation(GetCurrentProcess(), hUser32, &modInfo, sizeof(modInfo))) return false;

    uintptr_t base = (uintptr_t)modInfo.lpBaseOfDll;
    uintptr_t end = base + modInfo.SizeOfImage;
    uintptr_t tableAddr = (uintptr_t)callbackTable;

    // KernelCallbackTable must strictly reside within user32.dll
    return (tableAddr >= base && tableAddr < end);
}
