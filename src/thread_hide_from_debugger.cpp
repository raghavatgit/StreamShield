// StreamShield ThreadHideFromDebugger Enforcement
// Hides active security telemetry threads from kernel and user-mode debuggers.

#include <windows.h>
#include <winternl.h>

typedef NTSTATUS(NTAPI* pfnNtSetInformationThread)(
    HANDLE ThreadHandle,
    ULONG ThreadInformationClass,
    PVOID ThreadInformation,
    ULONG ThreadInformationLength
);

#define ThreadHideFromDebugger 0x11

bool HideCurrentThread() {
    HMODULE hNtdll = GetModuleHandleA("ntdll.dll");
    if (!hNtdll) return false;

    pfnNtSetInformationThread NtSetInfoThread = 
        (pfnNtSetInformationThread)GetProcAddress(hNtdll, "NtSetInformationThread");
    if (!NtSetInfoThread) return false;

    NTSTATUS status = NtSetInfoThread(GetCurrentThread(), ThreadHideFromDebugger, NULL, 0);
    return NT_SUCCESS(status);
}
