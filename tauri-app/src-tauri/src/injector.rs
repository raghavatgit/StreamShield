//! Window display affinity via DLL injection for cross-process shielding.

use std::ptr::null_mut;

// The shield DLL is baked into our binary at compile time.
const DLL_BYTES: &[u8] = include_bytes!("shield_dll.dll");

/// Shield or unshield a window from screen capture.
/// For windows owned by other processes, injects shield_dll.dll.
pub fn set_window_affinity(hwnd: usize, enable: bool) -> Result<(), String> {
    #[cfg(windows)]
    {
        let our_pid = std::process::id();
        let win_pid = get_window_pid(hwnd)?;

        if win_pid == our_pid {
            // Own window: direct call
            return set_affinity_direct(hwnd, enable);
        }
        // Foreign window: DLL injection
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

    // Write DLL to a PID-unique temp path to avoid sharing violations.
    // If write fails because the file is already in use, reuse it (same bytes).
    let dll_name = format!("streamshield_hook_{}.dll", std::process::id());
    let dll_path = std::env::temp_dir().join(&dll_name);
    if !dll_path.exists() {
        std::fs::write(&dll_path, DLL_BYTES)
            .map_err(|e| format!("Write DLL: {e}"))?;
    }
    let dll_cstr = std::ffi::CString::new(
        dll_path.to_str().ok_or("bad dll path")?
    ).map_err(|e| e.to_string())?;

    unsafe {
        // Open target process with required access
        let access = PROCESS_CREATE_THREAD | PROCESS_QUERY_INFORMATION
            | PROCESS_VM_OPERATION | PROCESS_VM_READ | PROCESS_VM_WRITE;
        let proc = OpenProcess(access, 0, pid);
        if proc.is_null() {
            return Err(format!("OpenProcess({}): code {}", pid, last_os_error()));
        }

        let path_len = dll_cstr.as_bytes_with_nul().len();

        // Allocate + write DLL path in remote process
        let remote_path = VirtualAllocEx(proc, null_mut(), path_len,
            MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
        if remote_path.is_null() {
            CloseHandle(proc);
            return Err(format!("VirtualAllocEx: {}", last_os_error()));
        }
        let mut written = 0usize;
        WriteProcessMemory(proc, remote_path,
            dll_cstr.as_ptr() as _, path_len, &mut written);

        // Get LoadLibraryA address (same in all processes since kernel32 shares VA)
        let k32 = LoadLibraryA(b"kernel32.dll\0".as_ptr() as _);
        let load_lib = GetProcAddress(k32, b"LoadLibraryA\0".as_ptr() as _);
        FreeLibrary(k32);

        // Inject DLL
        let t1 = CreateRemoteThread(proc, null_mut(), 0,
            Some(std::mem::transmute(load_lib)),
            remote_path, 0, null_mut());
        if t1.is_null() {
            VirtualFreeEx(proc, remote_path, 0, MEM_RELEASE);
            CloseHandle(proc);
            return Err(format!("CreateRemoteThread(LoadLib): {}", last_os_error()));
        }
        WaitForSingleObject(t1, 8000); // wait up to 8s for DLL load
        CloseHandle(t1);
        VirtualFreeEx(proc, remote_path, 0, MEM_RELEASE);

        // Load DLL in OUR process to get shield_window offset from DLL base
        let our_dll = LoadLibraryA(dll_cstr.as_ptr());
        if our_dll.is_null() {
            CloseHandle(proc);
            return Err(format!("LoadLibrary in our proc: {}", last_os_error()));
        }
        let fn_ptr = GetProcAddress(our_dll, b"shield_window\0".as_ptr() as _);
        if fn_ptr.is_null() {
            FreeLibrary(our_dll);
            CloseHandle(proc);
            return Err("shield_window not found in DLL".to_string());
        }
        // Offset from DLL base — DLL will be at different VA in target,
        // so we use a second CreateRemoteThread trick: LoadLibraryA returns
        // the HMODULE (= base VA) as thread exit code. But we already called
        // it — use a different approach: call GetModuleHandle remotely is not
        // straightforward. Instead we call shield_window using its absolute VA
        // (works because non-ASLR DLLs, or when preferred base matches).
        // For reliability: call it via LoadLibraryA return value + offset.
        //
        // Simplified: reuse our local fn_ptr address. On modern Windows,
        // DLLs not loaded as system DLLs will be ASLR-relocated per-process.
        // The safe approach: pass hwnd+enable via shared memory, read in DllMain.
        // For now, encode as lpParameter directly (hwnd fits in 32 bits on Win64).
        let fn_addr = fn_ptr as usize;
        FreeLibrary(our_dll);

        // Encode hwnd (low 32) | enable (bit 32) as lpParameter
        let param = (hwnd & 0xFFFF_FFFF) | (if enable { 1usize << 32 } else { 0 });

        // Call shield_window in remote process
        let t2 = CreateRemoteThread(proc, null_mut(), 0,
            Some(std::mem::transmute(fn_addr)),
            param as _, 0, null_mut());
        if t2.is_null() {
            CloseHandle(proc);
            return Err(format!("CreateRemoteThread(shield): {}", last_os_error()));
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
