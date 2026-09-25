// StreamShield Win32k System Call Filter
// Disallows calling legacy win32k graphics/windowing syscalls from untrusted background threads.

#include <windows.h>

bool EnforceSystemCallDisablePolicy() {
    PROCESS_MITIGATION_SYSTEM_CALL_DISABLE_POLICY policy;
    ZeroMemory(&policy, sizeof(policy));
    policy.DisallowWin32kSystemCalls = 1;

    BOOL result = SetProcessMitigationPolicy(
        ProcessSystemCallDisablePolicy,
        &policy,
        sizeof(policy)
    );
    return result != 0;
}
