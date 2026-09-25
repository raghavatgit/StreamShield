// StreamShield High-Resolution Execution Timer Probe
// Detects interactive debugger stepping by measuring cycle latency deltas across instruction blocks.

#include <windows.h>
#include <intrin.h>
#include <iostream>

bool DetectDebuggerTimingAnomaly() {
    LARGE_INTEGER freq, start, end;
    QueryPerformanceFrequency(&freq);
    QueryPerformanceCounter(&start);

    unsigned __int64 tscStart = __rdtsc();

    // Sensitive cryptographic or integrity block
    volatile int dummy = 0;
    for (int i = 0; i < 500; ++i) {
        dummy += i * 3;
    }

    unsigned __int64 tscEnd = __rdtsc();
    QueryPerformanceCounter(&end);

    // Delta in microseconds
    double elapsedUs = (double)(end.QuadPart - start.QuadPart) * 1000000.0 / freq.QuadPart;

    // Normal execution takes < 50 microseconds. An attached debugger or breakpoint stall exceeds 5000us.
    if (elapsedUs > 15000.0 || (tscEnd - tscStart) > 1000000) {
        return true; // Debugger delay detected
    }

    return false;
}
