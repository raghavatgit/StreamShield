//! NVIDIA ShadowPlay / GeForce Experience DRM Bypass Engine.
//! Live-patches user32!GetWindowDisplayAffinity inside NVIDIA capture processes
//! (nvcontainer.exe, nvsphelper64.exe, NVIDIA Share.exe, NVIDIA App.exe)
//! to return WDA_NONE (0), preventing NVIDIA from triggering its DRM killswitch
//! while Windows Desktop Window Manager (DWM) continues to visually exclude shielded windows.

#[cfg(windows)]
pub fn patch_nvidia_processes() -> usize {
    use std::ffi::CString;
    use winapi::um::libloaderapi::{GetModuleHandleA, GetProcAddress};
    use winapi::um::processthreadsapi::{OpenProcess, FlushInstructionCache};
    use winapi::um::memoryapi::{VirtualProtectEx, WriteProcessMemory};
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::tlhelp32::{CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS};
    use winapi::um::winnt::{PROCESS_VM_OPERATION, PROCESS_VM_WRITE, PROCESS_VM_READ, PROCESS_QUERY_INFORMATION, PAGE_EXECUTE_READWRITE};

    // 17-byte x86_64 machine code detour for GetWindowDisplayAffinity:
    // 48 85 D2                ; test rdx, rdx
    // 74 06                   ; jz +6
    // C7 02 00 00 00 00       ; mov dword ptr [rdx], 0  (*pdwAffinity = WDA_NONE)
    // B8 01 00 00 00          ; mov eax, 1             (return TRUE)
    // C3                      ; ret
    const PATCH: [u8; 17] = [
        0x48, 0x85, 0xD2,
        0x74, 0x06,
        0xC7, 0x02, 0x00, 0x00, 0x00, 0x00,
        0xB8, 0x01, 0x00, 0x00, 0x00,
        0xC3,
    ];

    let user32_name = CString::new("user32.dll").unwrap();
    let proc_name = CString::new("GetWindowDisplayAffinity").unwrap();

    let target_func = unsafe {
        let h_mod = GetModuleHandleA(user32_name.as_ptr());
        if h_mod.is_null() {
            return 0;
        }
        GetProcAddress(h_mod, proc_name.as_ptr())
    };

    if target_func.is_null() {
        return 0;
    }

    let target_names = [
        "nvcontainer.exe",
        "nvsphelper64.exe",
        "nvidia share.exe",
        "nvidia app.exe",
        "nvidia overlay.exe",
    ];

    let mut patched_count = 0;

    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot.is_null() || snapshot == winapi::um::handleapi::INVALID_HANDLE_VALUE {
            return 0;
        }

        let mut pe: PROCESSENTRY32W = std::mem::zeroed();
        pe.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        if Process32FirstW(snapshot, &mut pe) != 0 {
            loop {
                let null_pos = pe.szExeFile.iter().position(|&c| c == 0).unwrap_or(pe.szExeFile.len());
                let exe_name = String::from_utf16_lossy(&pe.szExeFile[..null_pos]).to_lowercase();

                let is_target = target_names.iter().any(|&t| exe_name == t || exe_name.starts_with(t));

                if is_target {
                    let pid = pe.th32ProcessID;
                    let h_proc = OpenProcess(
                        PROCESS_VM_OPERATION | PROCESS_VM_WRITE | PROCESS_VM_READ | PROCESS_QUERY_INFORMATION,
                        0,
                        pid,
                    );

                    if !h_proc.is_null() {
                        let mut old_prot = 0u32;
                        if VirtualProtectEx(
                            h_proc,
                            target_func as *mut _,
                            PATCH.len(),
                            PAGE_EXECUTE_READWRITE,
                            &mut old_prot,
                        ) != 0 {
                            let mut written = 0usize;
                            let ok = WriteProcessMemory(
                                h_proc,
                                target_func as *mut _,
                                PATCH.as_ptr() as *const _,
                                PATCH.len(),
                                &mut written,
                            );
                            let mut dummy = 0u32;
                            VirtualProtectEx(
                                h_proc,
                                target_func as *mut _,
                                PATCH.len(),
                                old_prot,
                                &mut dummy,
                            );
                            FlushInstructionCache(h_proc, target_func as *const _, PATCH.len());

                            if ok != 0 && written == PATCH.len() {
                                patched_count += 1;
                            }
                        }
                        CloseHandle(h_proc);
                    }
                }

                if Process32NextW(snapshot, &mut pe) == 0 {
                    break;
                }
            }
        }

        CloseHandle(snapshot);
    }

    patched_count
}

#[cfg(not(windows))]
pub fn patch_nvidia_processes() -> usize {
    0
}
