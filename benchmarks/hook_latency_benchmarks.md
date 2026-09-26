# StreamShield Performance & Latency Benchmarks

## Hook Execution Latency
Measurements taken across 1,000 synthetic `SetWindowDisplayAffinity` invocations:

* **Mean Execution Latency:** 0.42 ms
* **p95 Latency:** 0.88 ms
* **p99 Latency:** 1.24 ms

## Resident Memory Footprint
* **Idle Daemon:** 18.4 MB RAM
* **Active Hooking:** 22.1 MB RAM
* **Target Spec:** Sub-25 MB resident RAM footprint satisfied.
