# Process Detection Heuristics

## Identification Strategy
1. Retrieve window handle `HWND`.
2. Call `GetWindowThreadProcessId(hwnd, &pid)`.
3. Open process handle with `PROCESS_QUERY_LIMITED_INFORMATION`.
4. Call `QueryFullProcessImageNameW()` to obtain executable path.
5. Cache PID-to-process name mappings in a thread-safe LRU cache with 5-second TTL.
