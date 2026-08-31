//! NVIDIA ShadowPlay Bypass Engine (Three-Layer Architecture)
//!
//! Layer 1: ASLR-safe DRM killswitch patch
//!   - Patches GetWindowDisplayAffinity in NVIDIA capture processes to return WDA_NONE
//!   - Patches Module32FirstW to return FALSE (hides from process enumeration)
//!   - Uses CreateToolhelp32Snapshot(TH32CS_SNAPMODULE) for correct remote addresses
//!
//! Layer 2: MPO (Multiplane Overlay) disable via registry
//!   - Sets OverlayTestMode=5 in DWM registry (Win10/11)
//!   - Sets DisableOverlays=1 in GraphicsDrivers (Win11 25H2+)
//!   - Forces DWM software composition so SetWindowDisplayAffinity is respected
//!
//! Layer 3: NvFBC disable to force DXGI Desktop Duplication
//!   - Disables NvFBC so ShadowPlay falls back to DXGI DD
//!   - DXGI DD respects SetWindowDisplayAffinity natively

use serde::{Deserialize, Serialize};

/// Status report from the bypass engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NvidiaBypassStatus {
    pub drm_patch_count: usize,
    pub drm_patch_errors: Vec<String>,
    pub mpo_fix_applied: bool,
    pub mpo_fix_error: Option<String>,
    pub nvfbc_disabled: bool,
    pub nvfbc_error: Option<String>,
}

/// Run all three bypass layers and return a combined status report.
#[cfg(windows)]
pub fn apply_full_bypass() -> NvidiaBypassStatus {
    let (drm_count, drm_errors) = patch_nvidia_drm_check();
    let (mpo_ok, mpo_err) = apply_mpo_and_overlay_fix();
    let (nvfbc_ok, nvfbc_err) = disable_nvfbc_capture();

    NvidiaBypassStatus {
        drm_patch_count: drm_count,
        drm_patch_errors: drm_errors,
        mpo_fix_applied: mpo_ok,
        mpo_fix_error: mpo_err,
        nvfbc_disabled: nvfbc_ok,
        nvfbc_error: nvfbc_err,
    }
}

#[cfg(not(windows))]
pub fn apply_full_bypass() -> NvidiaBypassStatus {
    NvidiaBypassStatus {
        drm_patch_count: 0,
        drm_patch_errors: vec!["Not supported on this platform".to_string()],
        mpo_fix_applied: false,
        mpo_fix_error: Some("Not supported on this platform".to_string()),
        nvfbc_disabled: false,
        nvfbc_error: Some("Not supported on this platform".to_string()),
    }
}

/// Run only Layer 1 (DRM patch) for the background watchdog.
/// Lighter than `apply_full_bypass` since registry doesn't need re-applying.
#[cfg(windows)]
pub fn patch_nvidia_processes() -> usize {
    let (count, _) = patch_nvidia_drm_check();
    count
}

#[cfg(not(windows))]
pub fn patch_nvidia_processes() -> usize {
    0
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// LAYER 1: ASLR-safe DRM killswitch patch
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[cfg(windows)]
fn patch_nvidia_drm_check() -> (usize, Vec<String>) {
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::memoryapi::ReadProcessMemory;
    use winapi::um::processthreadsapi::OpenProcess;
    use winapi::um::tlhelp32::{
        CreateToolhelp32Snapshot, Process32FirstW,
        Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
    };
    use winapi::um::winnt::{
        PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION,
        PROCESS_VM_READ, PROCESS_VM_WRITE,
    };

    // 17-byte x86_64 detour for GetWindowDisplayAffinity:
    //   test rdx, rdx       ; null-check pdwAffinity pointer
    //   jz +6               ; skip store if null
    //   mov dword [rdx], 0  ; *pdwAffinity = WDA_NONE (0)
    //   mov eax, 1          ; return TRUE
    //   ret
    const GWDA_PATCH: [u8; 17] = [
        0x48, 0x85, 0xD2,                   // test rdx, rdx
        0x74, 0x06,                         // jz +6
        0xC7, 0x02, 0x00, 0x00, 0x00, 0x00, // mov dword ptr [rdx], 0
        0xB8, 0x01, 0x00, 0x00, 0x00,       // mov eax, 1
        0xC3,                               // ret
    ];

    // 6-byte x86_64 detour for Module32FirstW:
    //   xor eax, eax  ; return FALSE (0) - "no modules found"
    //   ret
    const M32F_PATCH: [u8; 3] = [
        0x31, 0xC0, // xor eax, eax
        0xC3,       // ret
    ];

    // NVIDIA processes that perform the DRM check
    let target_names: &[&str] = &[
        "nvcontainer.exe",
        "nvsphelper64.exe",
        "nvidia share.exe",
        "nvidia app.exe",
        "nvidia overlay.exe",
        "nvspcaps64.exe",
    ];

    let mut patched_count: usize = 0;
    let mut errors: Vec<String> = Vec::new();

    unsafe {
        // ── Step 1: Enumerate running processes ──────────────────────────
        let proc_snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if proc_snapshot.is_null()
            || proc_snapshot == winapi::um::handleapi::INVALID_HANDLE_VALUE
        {
            errors.push("Failed to create process snapshot".to_string());
            return (0, errors);
        }

        let mut pe: PROCESSENTRY32W = std::mem::zeroed();
        pe.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        let mut nvidia_pids: Vec<(u32, String)> = Vec::new();

        if Process32FirstW(proc_snapshot, &mut pe) != 0 {
            loop {
                let null_pos = pe
                    .szExeFile
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(pe.szExeFile.len());
                let exe_name =
                    String::from_utf16_lossy(&pe.szExeFile[..null_pos]).to_lowercase();

                if target_names.iter().any(|&t| exe_name == t) {
                    nvidia_pids.push((pe.th32ProcessID, exe_name));
                }

                if Process32NextW(proc_snapshot, &mut pe) == 0 {
                    break;
                }
            }
        }
        CloseHandle(proc_snapshot);

        if nvidia_pids.is_empty() {
            errors.push("No NVIDIA capture processes found running".to_string());
            return (0, errors);
        }

        // ── Step 2: For each NVIDIA process, find remote module bases and patch ──
        for (pid, exe_name) in &nvidia_pids {
            let h_proc = OpenProcess(
                PROCESS_VM_OPERATION
                    | PROCESS_VM_WRITE
                    | PROCESS_VM_READ
                    | PROCESS_QUERY_INFORMATION,
                0,
                *pid,
            );
            if h_proc.is_null() {
                errors.push(format!(
                    "{} (PID {}): OpenProcess failed (error {})",
                    exe_name,
                    pid,
                    winapi::um::errhandlingapi::GetLastError()
                ));
                continue;
            }

            let mut process_patched = false;

            // ── Patch GetWindowDisplayAffinity ────────────────────────
            match get_remote_func_address(
                *pid,
                h_proc,
                "user32.dll",
                "GetWindowDisplayAffinity",
            ) {
                Ok(remote_addr) => {
                    // Read the first few bytes to check if already patched
                    let mut current_bytes = [0u8; 3];
                    let mut bytes_read = 0usize;
                    ReadProcessMemory(
                        h_proc,
                        remote_addr as *const _,
                        current_bytes.as_mut_ptr() as *mut _,
                        3,
                        &mut bytes_read,
                    );

                    // Skip if already patched (first 3 bytes match our patch)
                    if bytes_read == 3
                        && current_bytes[0] == GWDA_PATCH[0]
                        && current_bytes[1] == GWDA_PATCH[1]
                        && current_bytes[2] == GWDA_PATCH[2]
                    {
                        process_patched = true;
                    } else {
                        if write_patch(h_proc, remote_addr, &GWDA_PATCH) {
                            process_patched = true;
                        } else {
                            errors.push(format!(
                                "{} (PID {}): WriteProcessMemory for GWDA failed (error {})",
                                exe_name,
                                pid,
                                winapi::um::errhandlingapi::GetLastError()
                            ));
                        }
                    }
                }
                Err(e) => {
                    errors.push(format!(
                        "{} (PID {}): GWDA address resolution failed: {}",
                        exe_name, pid, e
                    ));
                }
            }

            // ── Patch Module32FirstW ─────────────────────────────────
            match get_remote_func_address(
                *pid,
                h_proc,
                "kernel32.dll",
                "Module32FirstW",
            ) {
                Ok(remote_addr) => {
                    let mut current_bytes = [0u8; 2];
                    let mut bytes_read = 0usize;
                    ReadProcessMemory(
                        h_proc,
                        remote_addr as *const _,
                        current_bytes.as_mut_ptr() as *mut _,
                        2,
                        &mut bytes_read,
                    );

                    // Skip if already patched
                    if !(bytes_read == 2
                        && current_bytes[0] == M32F_PATCH[0]
                        && current_bytes[1] == M32F_PATCH[1])
                    {
                        if !write_patch(h_proc, remote_addr, &M32F_PATCH) {
                            errors.push(format!(
                                "{} (PID {}): WriteProcessMemory for M32F failed (error {})",
                                exe_name,
                                pid,
                                winapi::um::errhandlingapi::GetLastError()
                            ));
                        }
                    }
                }
                Err(e) => {
                    // Not critical - Module32FirstW hook is secondary
                    errors.push(format!(
                        "{} (PID {}): M32F address resolution: {}",
                        exe_name, pid, e
                    ));
                }
            }

            CloseHandle(h_proc);

            if process_patched {
                patched_count += 1;
            }
        }
    }

    (patched_count, errors)
}

/// Find the remote address of a function inside a target process.
/// Uses module snapshot to get the correct ASLR base address.
#[cfg(windows)]
unsafe fn get_remote_func_address(
    pid: u32,
    _h_proc: winapi::um::winnt::HANDLE,
    module_name: &str,
    func_name: &str,
) -> Result<usize, String> {
    use winapi::um::libloaderapi::{FreeLibrary, GetProcAddress, LoadLibraryA};
    use std::ffi::CString;

    // Step 1: Find the remote base address of the module in the target process
    let remote_base = get_remote_module_base(pid, module_name)
        .ok_or_else(|| format!("{} not found in PID {}", module_name, pid))?;

    // Step 2: Load the module locally to compute the function offset
    let mod_cstr =
        CString::new(module_name).map_err(|e| format!("CString error: {}", e))?;
    let local_mod = LoadLibraryA(mod_cstr.as_ptr());
    if local_mod.is_null() {
        return Err(format!("LoadLibraryA({}) failed locally", module_name));
    }

    let func_cstr =
        CString::new(func_name).map_err(|e| format!("CString error: {}", e))?;
    let local_func = GetProcAddress(local_mod, func_cstr.as_ptr());
    if local_func.is_null() {
        FreeLibrary(local_mod);
        return Err(format!(
            "GetProcAddress({}, {}) failed locally",
            module_name, func_name
        ));
    }

    // Step 3: offset = local_func - local_base
    let offset = local_func as usize - local_mod as usize;
    FreeLibrary(local_mod);

    // Step 4: remote_func = remote_base + offset
    Ok(remote_base.wrapping_add(offset))
}

/// Get the real base address of a module inside a remote process
/// using CreateToolhelp32Snapshot (ASLR-safe).
#[cfg(windows)]
unsafe fn get_remote_module_base(pid: u32, dll_name: &str) -> Option<usize> {
    use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
    use winapi::um::tlhelp32::{
        CreateToolhelp32Snapshot, Module32FirstW, Module32NextW, MODULEENTRY32W,
        TH32CS_SNAPMODULE, TH32CS_SNAPMODULE32,
    };

    let snap = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid);
    if snap.is_null() || snap == INVALID_HANDLE_VALUE {
        return None;
    }

    let mut me: MODULEENTRY32W = std::mem::zeroed();
    me.dwSize = std::mem::size_of::<MODULEENTRY32W>() as u32;

    let dll_lower = dll_name.to_lowercase();
    let mut found: Option<usize> = None;

    if Module32FirstW(snap, &mut me) != 0 {
        loop {
            let name_len = me
                .szModule
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(me.szModule.len());
            let name = String::from_utf16_lossy(&me.szModule[..name_len]).to_lowercase();
            if name == dll_lower || name.contains(&dll_lower) {
                found = Some(me.modBaseAddr as usize);
                break;
            }
            if Module32NextW(snap, &mut me) == 0 {
                break;
            }
        }
    }

    CloseHandle(snap);
    found
}

/// Write a patch to a remote process at the given address.
/// Handles VirtualProtectEx for execute-read pages.
#[cfg(windows)]
unsafe fn write_patch(
    h_proc: winapi::um::winnt::HANDLE,
    remote_addr: usize,
    patch: &[u8],
) -> bool {
    use winapi::um::memoryapi::{VirtualProtectEx, WriteProcessMemory};
    use winapi::um::processthreadsapi::FlushInstructionCache;
    use winapi::um::winnt::PAGE_EXECUTE_READWRITE;

    let mut old_prot = 0u32;
    if VirtualProtectEx(
        h_proc,
        remote_addr as *mut _,
        patch.len(),
        PAGE_EXECUTE_READWRITE,
        &mut old_prot,
    ) == 0
    {
        return false;
    }

    let mut written = 0usize;
    let ok = WriteProcessMemory(
        h_proc,
        remote_addr as *mut _,
        patch.as_ptr() as *const _,
        patch.len(),
        &mut written,
    );

    // Restore original protection
    let mut dummy = 0u32;
    VirtualProtectEx(
        h_proc,
        remote_addr as *mut _,
        patch.len(),
        old_prot,
        &mut dummy,
    );

    FlushInstructionCache(h_proc, remote_addr as *const _, patch.len());

    ok != 0 && written == patch.len()
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// LAYER 2: MPO (Multiplane Overlay) disable via registry
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[cfg(windows)]
fn apply_mpo_and_overlay_fix() -> (bool, Option<String>) {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::winnt::{KEY_SET_VALUE, REG_DWORD};
    use winapi::um::winreg::{
        RegCloseKey, RegCreateKeyExW, RegSetValueExW, HKEY_LOCAL_MACHINE,
    };

    fn wide(s: &str) -> Vec<u16> {
        OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    let mut any_success = false;
    let mut all_errors: Vec<String> = Vec::new();

    // ── Fix 1: DWM OverlayTestMode = 5 (standard MPO disable) ──────
    unsafe {
        let subkey = wide(r"SOFTWARE\Microsoft\Windows\Dwm");
        let val_name = wide("OverlayTestMode");
        let mut hkey = std::ptr::null_mut();
        let mut disposition = 0u32;

        let status = RegCreateKeyExW(
            HKEY_LOCAL_MACHINE,
            subkey.as_ptr(),
            0,
            std::ptr::null_mut(),
            0,
            KEY_SET_VALUE,
            std::ptr::null_mut(),
            &mut hkey,
            &mut disposition,
        );

        if status == 0 {
            let data: u32 = 5;
            let set_res = RegSetValueExW(
                hkey,
                val_name.as_ptr(),
                0,
                REG_DWORD,
                &data as *const u32 as *const _,
                std::mem::size_of::<u32>() as u32,
            );
            if set_res == 0 {
                any_success = true;
            } else {
                all_errors.push(format!("OverlayTestMode set failed: error {}", set_res));
            }
            RegCloseKey(hkey);
        } else {
            all_errors.push(format!(
                "DWM registry open failed: error {} (need admin)",
                status
            ));
        }
    }

    // ── Fix 2: GraphicsDrivers DisableOverlays = 1 (Win11 25H2+) ───
    unsafe {
        let subkey = wide(r"SYSTEM\CurrentControlSet\Control\GraphicsDrivers");
        let val_name = wide("DisableOverlays");
        let mut hkey = std::ptr::null_mut();
        let mut disposition = 0u32;

        let status = RegCreateKeyExW(
            HKEY_LOCAL_MACHINE,
            subkey.as_ptr(),
            0,
            std::ptr::null_mut(),
            0,
            KEY_SET_VALUE,
            std::ptr::null_mut(),
            &mut hkey,
            &mut disposition,
        );

        if status == 0 {
            let data: u32 = 1;
            let set_res = RegSetValueExW(
                hkey,
                val_name.as_ptr(),
                0,
                REG_DWORD,
                &data as *const u32 as *const _,
                std::mem::size_of::<u32>() as u32,
            );
            if set_res == 0 {
                any_success = true;
            } else {
                all_errors.push(format!("DisableOverlays set failed: error {}", set_res));
            }
            RegCloseKey(hkey);
        } else {
            all_errors.push(format!(
                "GraphicsDrivers registry open failed: error {}",
                status
            ));
        }
    }

    // ── Invalidate DWM composition to pick up changes without reboot ──
    invalidate_dwm_composition();

    let error = if all_errors.is_empty() {
        None
    } else {
        Some(all_errors.join("; "))
    };

    (any_success, error)
}

/// Signal DWM to re-compose all windows, picking up registry changes.
#[cfg(windows)]
fn invalidate_dwm_composition() {
    use winapi::um::winuser::{
        SystemParametersInfoW, SPI_SETDESKWALLPAPER, SPIF_SENDCHANGE,
    };

    // Toggling a harmless system parameter triggers DWM recomposition
    // without the visual disruption of killing dwm.exe
    unsafe {
        SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            std::ptr::null_mut(),
            SPIF_SENDCHANGE,
        );
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// LAYER 3: NvFBC disable to force DXGI Desktop Duplication
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[cfg(windows)]
fn disable_nvfbc_capture() -> (bool, Option<String>) {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::winnt::{KEY_SET_VALUE, REG_DWORD};
    use winapi::um::winreg::{
        RegCloseKey, RegCreateKeyExW, RegSetValueExW, HKEY_LOCAL_MACHINE,
    };

    fn wide(s: &str) -> Vec<u16> {
        OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    let mut any_success = false;
    let mut all_errors: Vec<String> = Vec::new();

    // ── Disable NvFBC via NVIDIA driver registry ────────────────────
    // When NvFBC is unavailable, ShadowPlay falls back to DXGI Desktop
    // Duplication, which respects SetWindowDisplayAffinity natively.
    let nvfbc_keys: &[(&str, &str, u32)] = &[
        // Primary NvFBC disable flag
        (
            r"SYSTEM\CurrentControlSet\Services\nvlddmkm\FTS",
            "EnableRID73779",
            0,
        ),
        // Disable NvFBC session creation for desktop capture
        (
            r"SOFTWARE\NVIDIA Corporation\Global\NvFBC",
            "EnableDesktopCapture",
            0,
        ),
    ];

    for (subkey_path, val_name_str, value) in nvfbc_keys {
        unsafe {
            let subkey = wide(subkey_path);
            let val_name = wide(val_name_str);
            let mut hkey = std::ptr::null_mut();
            let mut disposition = 0u32;

            let status = RegCreateKeyExW(
                HKEY_LOCAL_MACHINE,
                subkey.as_ptr(),
                0,
                std::ptr::null_mut(),
                0,
                KEY_SET_VALUE,
                std::ptr::null_mut(),
                &mut hkey,
                &mut disposition,
            );

            if status == 0 {
                let data: u32 = *value;
                let set_res = RegSetValueExW(
                    hkey,
                    val_name.as_ptr(),
                    0,
                    REG_DWORD,
                    &data as *const u32 as *const _,
                    std::mem::size_of::<u32>() as u32,
                );
                if set_res == 0 {
                    any_success = true;
                } else {
                    all_errors.push(format!(
                        "{}\\{} set failed: error {}",
                        subkey_path, val_name_str, set_res
                    ));
                }
                RegCloseKey(hkey);
            } else {
                // Not critical - these keys may not exist on all driver versions
                all_errors.push(format!(
                    "{} open failed: error {} (may not exist on this driver)",
                    subkey_path, status
                ));
            }
        }
    }

    let error = if all_errors.is_empty() {
        None
    } else {
        Some(all_errors.join("; "))
    };

    (any_success, error)
}

/// Check if the MPO fix is currently active in the registry.
#[cfg(windows)]
pub fn is_mpo_fix_active() -> bool {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::winnt::KEY_QUERY_VALUE;
    use winapi::um::winreg::{RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY_LOCAL_MACHINE};

    fn wide(s: &str) -> Vec<u16> {
        OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    let subkey = wide(r"SOFTWARE\Microsoft\Windows\Dwm");
    let val_name = wide("OverlayTestMode");

    unsafe {
        let mut hkey = std::ptr::null_mut();
        if RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            subkey.as_ptr(),
            0,
            KEY_QUERY_VALUE,
            &mut hkey,
        ) != 0
        {
            return false;
        }
        let mut data_type = 0u32;
        let mut data: u32 = 0;
        let mut data_len = std::mem::size_of::<u32>() as u32;
        let query_res = RegQueryValueExW(
            hkey,
            val_name.as_ptr(),
            std::ptr::null_mut(),
            &mut data_type,
            &mut data as *mut u32 as *mut _,
            &mut data_len,
        );
        RegCloseKey(hkey);
        query_res == 0 && data == 5
    }
}

#[cfg(not(windows))]
pub fn is_mpo_fix_active() -> bool {
    false
}
