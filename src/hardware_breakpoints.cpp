// StreamShield Hardware Breakpoint Detector
// Inspects processor debug registers (DR0 - DR3, DR7) using GetThreadContext.

#include <windows.h>
#include <iostream>

bool DetectHardwareBreakpoints() {
    CONTEXT ctx;
    ZeroMemory(&ctx, sizeof(CONTEXT));
    ctx.ContextFlags = CONTEXT_DEBUG_REGISTERS;

    HANDLE hThread = GetCurrentThread();
    if (!GetThreadContext(hThread, &ctx)) {
        return false;
    }

    if (ctx.Dr0 != 0 || ctx.Dr1 != 0 || ctx.Dr2 != 0 || ctx.Dr3 != 0) {
        return true;
    }

    // Check if any local or global breakpoint is enabled in DR7
    if ((ctx.Dr7 & 0x000000FF) != 0) {
        return true;
    }

    return false;
}
