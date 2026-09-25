// StreamShield Binary Signature Policy
// Blocks DLL injections signed by non-Microsoft or untrusted third-party certificates.

#include <windows.h>

bool EnforceMicrosoftSignedBinariesOnly() {
    PROCESS_MITIGATION_BINARY_SIGNATURE_POLICY policy;
    ZeroMemory(&policy, sizeof(policy));
    policy.MicrosoftSignedOnly = 1;

    BOOL result = SetProcessMitigationPolicy(
        ProcessSignaturePolicy,
        &policy,
        sizeof(policy)
    );
    return result != 0;
}
