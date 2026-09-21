#ifndef STREAMSHIELD_THREAD_INJECTION_HPP
#define STREAMSHIELD_THREAD_INJECTION_HPP

#include <windows.h>
#include <cstdint>

namespace StreamShield {

class ThreadCreationFilter {
public:
    // Inspects thread creation parameters to detect remote code execution attempts
    static bool IsThreadCreationLegitimate(HANDLE hProcess, void* startAddress) {
        // Disallow remote thread injection if target process is not current process
        if (hProcess != GetCurrentProcess() && GetProcessId(hProcess) != GetCurrentProcessId()) {
            return false;
        }

        MEMORY_BASIC_INFORMATION mbi;
        if (VirtualQuery(startAddress, &mbi, sizeof(mbi)) == 0) {
            return false;
        }

        // Execution entry point must reside in committed executable image memory
        if (!(mbi.State & MEM_COMMIT) || !(mbi.Protect & (PAGE_EXECUTE_READ | PAGE_EXECUTE_READWRITE))) {
            return false;
        }

        return true;
    }
};

} // namespace StreamShield

#endif // STREAMSHIELD_THREAD_INJECTION_HPP
