#ifndef STREAMSHIELD_MINIDUMP_GENERATOR_HPP
#define STREAMSHIELD_MINIDUMP_GENERATOR_HPP

#include <windows.h>
#include <dbghelp.h>
#include <string>

#pragma comment(lib, "dbghelp.lib")

namespace StreamShield {

class CrashDumpGenerator {
public:
    // Captures user-mode minidump upon unhandled exception or critical tampering event
    static bool CreateMiniDump(EXCEPTION_POINTERS* pExceptionPointers, const std::wstring& dumpPath) {
        HANDLE hFile = CreateFileW(
            dumpPath.c_str(),
            GENERIC_WRITE,
            0,
            nullptr,
            CREATE_ALWAYS,
            FILE_ATTRIBUTE_NORMAL,
            nullptr
        );

        if (hFile == INVALID_HANDLE_VALUE) {
            return false;
        }

        MINIDUMP_EXCEPTION_INFORMATION exceptionInfo;
        exceptionInfo.ThreadId = GetCurrentThreadId();
        exceptionInfo.ExceptionPointers = pExceptionPointers;
        exceptionInfo.ClientPointers = FALSE;

        MINIDUMP_TYPE dumpType = static_cast<MINIDUMP_TYPE>(
            MiniDumpNormal |
            MiniDumpWithDataSegs |
            MiniDumpWithThreadInfo
        );

        BOOL success = MiniDumpWriteDump(
            GetCurrentProcess(),
            GetCurrentProcessId(),
            hFile,
            dumpType,
            pExceptionPointers ? &exceptionInfo : nullptr,
            nullptr,
            nullptr
        );

        CloseHandle(hFile);
        return success != FALSE;
    }
};

} // namespace StreamShield

#endif // STREAMSHIELD_MINIDUMP_GENERATOR_HPP
