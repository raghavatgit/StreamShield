// StreamShield DLL Injection Monitor
// Subscribes to NT loader module events via LdrRegisterDllNotification.

#include <windows.h>
#include <iostream>

typedef struct _LDR_DLL_LOADED_NOTIFICATION_DATA {
    ULONG Flags;
    const UNICODE_STRING* FullDllName;
    const UNICODE_STRING* BaseDllName;
    PVOID DllBase;
    ULONG SizeOfImage;
} LDR_DLL_LOADED_NOTIFICATION_DATA, *PLDR_DLL_LOADED_NOTIFICATION_DATA;

typedef struct _LDR_DLL_UNLOADED_NOTIFICATION_DATA {
    ULONG Flags;
    const UNICODE_STRING* FullDllName;
    const UNICODE_STRING* BaseDllName;
    PVOID DllBase;
    ULONG SizeOfImage;
} LDR_DLL_UNLOADED_NOTIFICATION_DATA, *PLDR_DLL_UNLOADED_NOTIFICATION_DATA;

typedef union _LDR_DLL_NOTIFICATION_DATA {
    LDR_DLL_LOADED_NOTIFICATION_DATA Loaded;
    LDR_DLL_UNLOADED_NOTIFICATION_DATA Unloaded;
} LDR_DLL_NOTIFICATION_DATA, *PLDR_DLL_NOTIFICATION_DATA;

typedef VOID(CALLBACK* PLDR_DLL_NOTIFICATION_FUNCTION)(
    ULONG NotificationReason,
    PLDR_DLL_NOTIFICATION_DATA NotificationData,
    PVOID Context
);

typedef NTSTATUS(NTAPI* pfnLdrRegisterDllNotification)(
    ULONG Flags,
    PLDR_DLL_NOTIFICATION_FUNCTION NotificationFunction,
    PVOID Context,
    PVOID* Cookie
);

VOID CALLBACK DllNotificationCallback(
    ULONG NotificationReason,
    PLDR_DLL_NOTIFICATION_DATA NotificationData,
    PVOID Context
) {
    if (NotificationReason == 1) { // LDR_DLL_NOTIFICATION_REASON_LOADED
        // Inspect loaded library base address and name against allowlist
    }
}
