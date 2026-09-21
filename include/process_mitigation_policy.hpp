#ifndef STREAMSHIELD_MITIGATION_POLICY_HPP
#define STREAMSHIELD_MITIGATION_POLICY_HPP

#include <windows.h>
#include <iostream>

namespace StreamShield {

class ProcessMitigationManager {
public:
    // Enforces Windows exploit mitigations on the current process
    static bool EnforceHardenedMitigations() {
        // 1. Enforce ASLR High Entropy and Force Relocate Images
        PROCESS_MITIGATION_ASLR_POLICY aslrPolicy = {};
        aslrPolicy.EnableHighEntropy = 1;
        aslrPolicy.EnableForceRelocateImages = 1;
        aslrPolicy.DisallowStrippedImages = 1;

        if (!SetProcessMitigationPolicy(ProcessASLRPolicy, &aslrPolicy, sizeof(aslrPolicy))) {
            return false;
        }

        // 2. Strict Handle Checking (Prevents invalid handle dereferences)
        PROCESS_MITIGATION_STRICT_HANDLE_CHECK_POLICY handlePolicy = {};
        handlePolicy.RaiseExceptionOnInvalidHandleReference = 1;
        handlePolicy.HandleExceptionsPermanentlyEnabled = 1;

        if (!SetProcessMitigationPolicy(ProcessStrictHandleCheckPolicy, &handlePolicy, sizeof(handlePolicy))) {
            return false;
        }

        return true;
    }
};

} // namespace StreamShield

#endif // STREAMSHIELD_MITIGATION_POLICY_HPP
