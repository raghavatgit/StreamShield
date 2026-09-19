#ifndef STREAMSHIELD_CALLBACK_SANITIZER_HPP
#define STREAMSHIELD_CALLBACK_SANITIZER_HPP

#include <windows.h>
#include <cstdint>

namespace StreamShield {

class KernelCallbackValidator {
public:
    // Validates that registered process and thread notification callbacks
    // point within valid, signed kernel driver memory ranges
    static bool ValidateDriverPointerBounds(uintptr_t callbackAddress, uintptr_t driverBase, size_t driverSize) {
        if (callbackAddress < driverBase) return false;
        if (callbackAddress >= driverBase + driverSize) return false;
        return true;
    }
};

} // namespace StreamShield

#endif // STREAMSHIELD_CALLBACK_SANITIZER_HPP
