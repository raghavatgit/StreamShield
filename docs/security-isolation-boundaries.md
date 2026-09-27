# Security Boundaries & UIPI in Windows

## User Interface Privilege Isolation (UIPI)
Non-elevated processes cannot send messages or manipulate windows belonging to elevated processes (running as Administrator).

StreamShield recommends running with standard user rights for standard applications, and provides an optional elevated companion worker service when protecting administrative windows.
