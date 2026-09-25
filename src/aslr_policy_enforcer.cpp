// StreamShield ASLR Enforcement
// Enforces mandatory high-entropy 64-bit Address Space Layout Randomization.

#include <windows.h>

bool EnforceHighEntropyASLR() {
    PROCESS_MITIGATION_ASLR_POLICY policy;
    ZeroMemory(&policy, sizeof(policy));
    policy.EnableForceRelocateImages = 1;
    policy.EnableHighEntropy = 1;
    policy.DisallowStrippedImages = 1;

    BOOL result = SetProcessMitigationPolicy(
        ProcessASLRPolicy,
        &policy,
        sizeof(policy)
    );
    return result != 0;
}
