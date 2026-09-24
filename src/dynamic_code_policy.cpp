// StreamShield Dynamic Code Policy
// Disallows creation of executable code pages or tampering with existing executable memory.

#include <windows.h>
#include <iostream>

bool EnforceDisallowDynamicCode() {
    PROCESS_MITIGATION_DYNAMIC_CODE_POLICY policy;
    ZeroMemory(&policy, sizeof(policy));
    policy.ProhibitDynamicCode = 1;
    policy.AllowThreadOptOut = 0;

    BOOL result = SetProcessMitigationPolicy(
        ProcessDynamicCodePolicy,
        &policy,
        sizeof(policy)
    );
    return result != 0;
}
