#ifndef STREAMSHIELD_VEH_INTEGRITY_HPP
#define STREAMSHIELD_VEH_INTEGRITY_HPP

#include <windows.h>
#include <cstdint>

namespace StreamShield {

class HardwareBreakpointDetector {
public:
    // Inspects thread context to detect hardware breakpoints installed on DR0-DR3
    static bool HasActiveHardwareBreakpoints(HANDLE hThread = GetCurrentThread()) {
        CONTEXT ctx;
        ctx.ContextFlags = CONTEXT_DEBUG_REGISTERS;

        if (!GetThreadContext(hThread, &ctx)) {
            return false;
        }

        // DR0-DR3 store breakpoint linear addresses; DR7 controls breakpoint activation
        if (ctx.Dr0 != 0 || ctx.Dr1 != 0 || ctx.Dr2 != 0 || ctx.Dr3 != 0) {
            return true;
        }

        if ((ctx.Dr7 & 0x000000FF) != 0) {
            return true;
        }

        return false;
    }

    // Clear active debug registers to neutralize external hardware breakpoint hooks
    static bool NeutralizeHardwareBreakpoints(HANDLE hThread = GetCurrentThread()) {
        CONTEXT ctx;
        ctx.ContextFlags = CONTEXT_DEBUG_REGISTERS;

        if (!GetThreadContext(hThread, &ctx)) {
            return false;
        }

        ctx.Dr0 = 0;
        ctx.Dr1 = 0;
        ctx.Dr2 = 0;
        ctx.Dr3 = 0;
        ctx.Dr6 = 0;
        ctx.Dr7 = 0;

        return SetThreadContext(hThread, &ctx) != 0;
    }
};

} // namespace StreamShield

#endif // STREAMSHIELD_VEH_INTEGRITY_HPP
