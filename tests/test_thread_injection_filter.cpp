#include "thread_injection_filter.hpp"
#include <cassert>
#include <iostream>

void test_thread_filter() {
    std::cout << "[StreamShield] Evaluating thread creation filter..." << std::endl;
    bool legit = StreamShield::ThreadCreationFilter::IsThreadCreationLegitimate(
        GetCurrentProcess(), reinterpret_cast<void*>(&test_thread_filter)
    );
    assert(legit);
    std::cout << "[PASS] Local function pointer validated in committed executable space." << std::endl;
}

int main() {
    test_thread_filter();
    return 0;
}
