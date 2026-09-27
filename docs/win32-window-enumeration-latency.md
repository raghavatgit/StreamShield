# Win32 EnumWindows Traversal Latency Benchmarks

## Benchmark Methodology
Evaluated across 1,000 synthetic top-level window hierarchies on Windows 11 23H2:

| Window Count | Mean Latency | p95 Latency | p99 Latency |
| :--- | :--- | :--- | :--- |
| 50 Windows | 0.18 ms | 0.32 ms | 0.45 ms |
| 150 Windows | 0.44 ms | 0.81 ms | 1.12 ms |
| 500 Windows | 1.35 ms | 2.10 ms | 2.85 ms |

Polling interval tuned to 150ms delivers smooth protection with <0.5% CPU utilization.
