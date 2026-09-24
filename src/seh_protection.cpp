// StreamShield Exception Handling Probe
// Validates SEH frame dispatch to detect debugger interception of deliberate faults.

#include <windows.h>
#include <iostream>

bool TestSEHDebuggerInterception() {
    bool exceptionHandled = false;

    __try {
        // Raise deliberate access violation or breakpoint
        RaiseException(STATUS_BREAKPOINT, 0, 0, NULL);
    }
    __except (GetExceptionCode() == STATUS_BREAKPOINT ? EXCEPTION_EXECUTE_HANDLER : EXCEPTION_CONTINUE_SEARCH) {
        exceptionHandled = true;
    }

    // If an external debugger swallowed or masked the breakpoint without routing to our __except handler:
    return !exceptionHandled;
}
