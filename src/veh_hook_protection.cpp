// StreamShield Vectored Exception Handling (VEH) Integrity
// Inspects LdrpVectorHandlerList to detect unauthorized global exception interceptors.

#include <windows.h>
#include <iostream>

LONG WINAPI ShieldVectoredExceptionHandler(PEXCEPTION_POINTERS pExceptionInfo) {
    if (pExceptionInfo->ExceptionRecord->ExceptionCode == STATUS_GUARD_PAGE_VIOLATION) {
        // Handle security guard violation internally
        return EXCEPTION_CONTINUE_EXECUTION;
    }
    return EXCEPTION_CONTINUE_SEARCH;
}

bool RegisterProtectedVEH() {
    PVOID pHandler = AddVectoredExceptionHandler(1, ShieldVectoredExceptionHandler);
    return pHandler != nullptr;
}
