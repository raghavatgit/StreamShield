// StreamShield Child Process Policy Enforcer
// Blocks arbitrary child processes from spawning via SetProcessMitigationPolicy.

#include <windows.h>
#include <iostream>

bool EnforceDisallowChildProcesses() {
    PROCESS_MITIGATION_CHILD_PROCESS_POLICY policy;
    ZeroMemory(&policy, sizeof(policy));
    policy.NoChildProcessCreation = 1;

    BOOL result = SetProcessMitigationPolicy(
        ProcessChildProcessPolicy,
        &policy,
        sizeof(policy)
    );
    return result != 0;
}
