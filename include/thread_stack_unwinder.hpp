#ifndef STREAMSHIELD_STACK_UNWINDER_HPP
#define STREAMSHIELD_STACK_UNWINDER_HPP

#include <windows.h>
#include <cstdint>
#include <vector>

namespace StreamShield {

class StackFrameValidator {
public:
    // Captures stack backtrace and validates that return addresses point within valid module memory
    static bool ValidateExecutionStack(uint32_t framesToCapture = 16) {
        std::vector<void*> backtrace(framesToCapture);
        USHORT captured = CaptureStackBackTrace(0, framesToCapture, backtrace.data(), nullptr);

        if (captured == 0) return false;

        for (USHORT i = 0; i < captured; i++) {
            MEMORY_BASIC_INFORMATION mbi;
            if (VirtualQuery(backtrace[i], &mbi, sizeof(mbi)) == 0) {
                return false;
            }

            // Return address must reside in committed executable memory
            if (!(mbi.State & MEM_COMMIT) || !(mbi.Protect & (PAGE_EXECUTE_READ | PAGE_EXECUTE_READWRITE))) {
                return false;
            }
        }

        return true;
    }
};

} // namespace StreamShield

#endif // STREAMSHIELD_STACK_UNWINDER_HPP
