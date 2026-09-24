// StreamShield Anti-Debugging: Direct PEB Validation
// Detects attached user-mode debuggers by inspecting the Process Environment Block.

#include <windows.h>
#include <winternl.h>
#include <iostream>

bool CheckPEBDebugFlags() {
#if defined(_M_X64) || defined(__x86_64__)
    unsigned char* ppeb = reinterpret_cast<unsigned char*>(__readgsqword(0x60));
#elif defined(_M_IX86) || defined(__i386__)
    unsigned char* ppeb = reinterpret_cast<unsigned char*>(__readfsdword(0x30));
#else
    return false;
#endif

    if (!ppeb) return false;

    // BeingDebugged flag is located at offset 0x02
    unsigned char beingDebugged = *(ppeb + 0x02);
    if (beingDebugged != 0) {
        return true;
    }

    // NtGlobalFlag offset: 0x68 on x86, 0xBC on x64
    // Flags FLG_HEAP_ENABLE_TAIL_CHECK (0x10), FLG_HEAP_ENABLE_FREE_CHECK (0x20), FLG_HEAP_VALIDATE_PARAMETERS (0x40)
#if defined(_M_X64) || defined(__x86_64__)
    DWORD ntGlobalFlag = *reinterpret_cast<DWORD*>(ppeb + 0xBC);
#else
    DWORD ntGlobalFlag = *reinterpret_cast<DWORD*>(ppeb + 0x68);
#endif

    if ((ntGlobalFlag & 0x70) == 0x70) {
        return true;
    }

    return false;
}
