//! Window display affinity via DLL injection for cross-process shielding.
//! Uses CreateToolhelp32Snapshot for correct 64-bit ASLR remote base address.

use std::ptr::null_mut;

const DLL_BYTES: &[u8] = include_bytes!("shield_dll.dll");

/// Shield or unshield a window from screen capture.
pub fn set_window_affinity(hwnd: usize, enable: bool) -> Result<(), String> {
    #[cfg(windows)]
    {
        let our_pid = std::process::id();
        let win_pid = get_window_pid(hwnd)?;

        if win_pid == our_pid {
            return set_affinity_direct(hwnd, enable);
        }
        inject_and_shield(win_pid, hwnd, enable)
    }
    #[cfg(not(windows))]
    Err("Windows only".to_string())
}

#[cfg(windows)]
fn get_window_pid(hwnd: usize) -> Result<u32, String> {
    use winapi::um::winuser::GetWindowThreadProcessId;
    let mut pid: u32 = 0;
    unsafe { GetWindowThreadProcessId(hwnd as _, &mut pid); }
    if pid == 0 { Err(format!("GetWindowThreadProcessId failed for hwnd {hwnd}")) }
    else { Ok(pid) }
}

#[cfg(windows)]
fn set_affinity_direct(hwnd: usize, enable: bool) -> Result<(), String> {
    use winapi::um::winuser::SetWindowDisplayAffinity;
    const WDA_NONE: u32 = 0x00000000;
    const WDA_EXCLUDEFROMCAPTURE: u32 = 0x00000011;
    let affinity = if enable { WDA_EXCLUDEFROMCAPTURE } else { WDA_NONE };
    let ok = unsafe { SetWindowDisplayAffinity(hwnd as _, affinity) };
    if ok != 0 { Ok(()) }
    else { Err(format!("SetWindowDisplayAffinity failed: {}", last_os_error())) }
}

/// Get the real 64-bit base address of a module in the remote process.
/// Uses CreateToolhelp32Snapshot — safe on both 32-bit and 64-bit Windows.
#[cfg(windows)]
fn get_remote_module_base(pid: u32, dll_filename: &str) -> Option<usize> {
    use winapi::um::tlhelp32::{
        CreateToolhelp32Snapshot, Module32FirstW, Module32NextW,
        MODULEENTRY32W, TH32CS_SNAPMODULE, TH32CS_SNAPMODULE32,
    };
    use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};

    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid);
        if snap == INVALID_HANDLE_VALUE {
            return None;
        }

        let mut me: MODULEENTRY32W = std::mem::zeroed();
        me.dwSize = std::mem::size_of::<MODULEENTRY32W>() as u32;

        let mut found: Option<usize> = None;
        if Module32FirstW(snap, &mut me) != 0 {
            loop {
                let name_len = me.szModule.iter().position(|&c| c == 0).unwrap_or(me.szModule.len());
                let name = String::from_utf16_lossy(&me.szModule[..name_len]).to_lowercase();
                if name.contains(dll_filename) {
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
}

#[cfg(windows)]
fn inject_and_shield(pid: u32, hwnd: usize, enable: bool) -> Result<(), String> {
    use winapi::um::processthreadsapi::{OpenProcess, CreateRemoteThread};
    use winapi::um::memoryapi::{VirtualAllocEx, WriteProcessMemory, VirtualFreeEx};
    use winapi::um::libloaderapi::{LoadLibraryA, GetProcAddress, FreeLibrary};
    use winapi::um::synchapi::WaitForSingleObject;
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::winnt::{
        PROCESS_CREATE_THREAD, PROCESS_QUERY_INFORMATION,
        PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE,
        MEM_COMMIT, MEM_RESERVE, MEM_RELEASE, PAGE_READWRITE,
    };

    // Write DLL to a stable temp path (PID-unique to avoid sharing violations)
    let dll_name = format!("streamshield_hook_{}.dll", std::process::id());
    let dll_path = std::env::temp_dir().join(&dll_name);
    if !dll_path.exists() {
        std::fs::write(&dll_path, DLL_BYTES)
            .map_err(|e| format!("Write DLL: {e}"))?;
    }
    let dll_path_str = dll_path.to_str().ok_or("bad dll path")?.to_lowercase();
    let dll_cstr = std::ffi::CString::new(dll_path.to_str().ok_or("bad dll path")?)
        .map_err(|e| e.to_string())?;

    // DLL filename for module snapshot lookup (lowercase, no path)
    let dll_basename = dll_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&dll_name)
        .to_lowercase();

    unsafe {
        let access = PROCESS_CREATE_THREAD | PROCESS_QUERY_INFORMATION
            | PROCESS_VM_OPERATION | PROCESS_VM_READ | PROCESS_VM_WRITE;
        let proc = OpenProcess(access, 0, pid);
        if proc.is_null() {
            return Err(format!("OpenProcess({}): {}", pid, last_os_error()));
        }

        let path_len = dll_cstr.as_bytes_with_nul().len();

        let remote_path = VirtualAllocEx(proc, null_mut(), path_len,
            MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
        if remote_path.is_null() {
            CloseHandle(proc);
            return Err(format!("VirtualAllocEx: {}", last_os_error()));
        }

        let mut written = 0usize;
        WriteProcessMemory(proc, remote_path, dll_cstr.as_ptr() as _, path_len, &mut written);

        // kernel32 maps at the same VA in every process (shared section, no ASLR between procs)
        let k32 = LoadLibraryA(b"kernel32.dll\0".as_ptr() as _);
        let load_lib = GetProcAddress(k32, b"LoadLibraryA\0".as_ptr() as _);
        FreeLibrary(k32);

        // Inject DLL into target process
        let t1 = CreateRemoteThread(proc, null_mut(), 0,
            Some(std::mem::transmute(load_lib)),
            remote_path, 0, null_mut());
        if t1.is_null() {
            VirtualFreeEx(proc, remote_path, 0, MEM_RELEASE);
            CloseHandle(proc);
            return Err(format!("CreateRemoteThread(LoadLib): {}", last_os_error()));
        }
        WaitForSingleObject(t1, 8000);
        CloseHandle(t1);
        VirtualFreeEx(proc, remote_path, 0, MEM_RELEASE);

        // ── ASLR fix: use module snapshot for real 64-bit remote base ────────
        // (GetExitCodeThread only gives 32-bit, wrong above 4GB)
        let remote_base = get_remote_module_base(pid, &dll_basename)
            .ok_or_else(|| format!("DLL not found in remote process modules (injection may have failed for PID {})", pid))?;

        // Load DLL locally to compute function offset
        let our_dll = LoadLibraryA(dll_cstr.as_ptr());
        if our_dll.is_null() {
            CloseHandle(proc);
            return Err(format!("LoadLibrary locally: {}", last_os_error()));
        }
        let fn_ptr = GetProcAddress(our_dll, b"shield_window\0".as_ptr() as _);
        if fn_ptr.is_null() {
            FreeLibrary(our_dll);
            CloseHandle(proc);
            return Err("shield_window not exported by DLL".to_string());
        }

        // offset = local fn address - local DLL base
        let fn_offset = fn_ptr as usize - our_dll as usize;
        // remote fn = remote DLL base + offset  →  correct ASLR address
        let remote_fn = remote_base.wrapping_add(fn_offset);
        FreeLibrary(our_dll);

        // Encode: low 32 bits = hwnd, bit 32 = enable flag
        let param = (hwnd & 0xFFFF_FFFF) | (if enable { 1usize << 32 } else { 0 });

        let t2 = CreateRemoteThread(proc, null_mut(), 0,
            Some(std::mem::transmute(remote_fn)),
            param as _, 0, null_mut());
        if t2.is_null() {
            CloseHandle(proc);
            return Err(format!("CreateRemoteThread(shield_window): {}", last_os_error()));
        }
        WaitForSingleObject(t2, 5000);
        CloseHandle(t2);
        CloseHandle(proc);
    }

    Ok(())
}

#[cfg(windows)]
fn last_os_error() -> u32 {
    unsafe { winapi::um::errhandlingapi::GetLastError() }
}
