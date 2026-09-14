# Security Policy and Process Integrity Model

## Overview
StreamShield interacts with low-level Windows APIs (`SetWindowDisplayAffinity`) by injecting dynamic libraries into target processes. Maintaining process boundaries and user trust is central to its architecture.

## Process Integrity Architecture

* **UAC Elevation Model:** StreamShield requests standard User Account Control (UAC) elevation on launch (`requireAdministrator`). This ensures the process operates at High Mandatory Integrity Level to safely communicate with both elevated and standard desktop windows without security token mismatches.
* **Non-Destructive Hooking:** The injected DLL (`shield_dll.dll`) exclusively targets window compositing display affinity flags. It executes zero memory patching, zero code hooking (e.g. no Detours/MinHook byte splicing), and zero arbitrary code execution.
* **Process Boundary Isolation:** All process enumeration queries verify target window visibility prior to thread creation to eliminate visual artifacting or target instability.

## Reporting a Vulnerability

If you discover a potential vulnerability or security concern, please submit a responsible disclosure report directly via email to:

`goyalraghav1442@gmail.com`

Please include:
1. Target Windows build and architecture (e.g. Windows 11 Build 22631 x64).
2. Steps to reproduce the issue.
3. Observed process behavior.
