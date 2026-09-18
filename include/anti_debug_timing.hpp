#ifndef STREAMSHIELD_ANTI_DEBUG_TIMING_HPP
#define STREAMSHIELD_ANTI_DEBUG_TIMING_HPP

#include <windows.h>
#include <cstdint>
#include <intrin.h>

namespace StreamShield {

class DebugTimingValidator {
public:
    // Measures CPU cycle delta using RDTSC instruction to detect single-step debuggers
    static bool IsDebuggerSteppingDetected(uint64_t maxThresholdCycles = 0xFFFFF) {
        unsigned int aux;
        uint64_t startCycles = __rdtscp(&aux);

        // Execute small compute block
        volatile uint64_t accumulator = 0;
        for (int i = 0; i < 1000; i++) {
            accumulator += i;
        }

        uint64_t endCycles = __rdtscp(&aux);
        uint64_t delta = endCycles - startCycles;

        // If an interactive debugger (x64dbg, IDA Pro) paused on breakpoints,
        // delta exceeds millions of CPU clock cycles.
        return delta > maxThresholdCycles;
    }
};

} // namespace StreamShield

#endif // STREAMSHIELD_ANTI_DEBUG_TIMING_HPP
