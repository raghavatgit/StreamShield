// StreamShield Inline Hook Scanner
// Scans standard system API function prologues for unconditional jumps and hotpatch detours.

#include <windows.h>
#include <iostream>

bool IsFunctionHooked(LPCSTR moduleName, LPCSTR functionName) {
    HMODULE hMod = GetModuleHandleA(moduleName);
    if (!hMod) return false;

    FARPROC pFunc = GetProcAddress(hMod, functionName);
    if (!pFunc) return false;

    BYTE* pBytes = reinterpret_cast<BYTE*>(pFunc);

    // 0xE9: JMP rel32 (standard 5-byte relative detour)
    if (pBytes[0] == 0xE9) {
        return true;
    }

    // 0xFF 0x25: JMP qword ptr [rip + disp32] (x64 absolute indirect jump)
    if (pBytes[0] == 0xFF && pBytes[1] == 0x25) {
        return true;
    }

    // 0xEB: JMP rel8 (short jump detour)
    if (pBytes[0] == 0xEB) {
        return true;
    }

    // 0xCC: INT 3 (software breakpoint)
    if (pBytes[0] == 0xCC) {
        return true;
    }

    return false;
}
