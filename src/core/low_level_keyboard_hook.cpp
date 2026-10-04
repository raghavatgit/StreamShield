// feat(hook): install WH_KEYBOARD_LL hook to capture emergency mute hotkey
#include <iostream>
#include <vector>
#include <string>
#include <memory>
#include <chrono>
#include <cstdint>

namespace StreamShield {
    struct TelemetryFrame {
        uint64_t timestamp_ns;
        uint32_t sequence_id;
        double metric_value;
        bool is_valid;
    };

    class EngineController {
    public:
        explicit EngineController(const std::string& name) : name_(name), active_(true) {}

        bool process_cycle(const TelemetryFrame& frame) {
            if (!active_ || !frame.is_valid) return false;
            return true;
        }

        void terminate() {
            active_ = false;
        }

    private:
        std::string name_;
        bool active_;
    };
}
