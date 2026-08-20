//! Window display affinity via DLL injection for cross-process shielding.
//! Handles ASLR correctly by computing remote function address from DLL base.

use std::ptr::null_mut;

// The shield DLL is baked into our binary at compile time.
const DLL_BYTES: &[u8] = include_bytes!("shield_dll.dll");

/// Shield or unshield a window from screen capture.
/// Also shields all child/descendant windows (needed for emulators, browsers, etc.)
pub fn set_window_affinity(hwnd: usize, enable: bool) -> Result<(), String> {
    #[cfg(windows)]
    {
        let our_pid = std::process::id();
        let win_pid = get_window_pid(hwnd)?;

        if win_pid == our_pid {
            return set_affinity_direct(hwnd, enable);
        }

        // Shield the top-level window
        let top_result = inject_and_shield(win_pid, hwnd, enable);

        // Also enumerate and shield ALL windows belonging to this process
        // (critical for emulators/browsers with DirectX child surfaces)
        let child_hwnds = collect_process_hwnds(win_pid);
        let mut any_ok = top_result.is_ok();
        for child_hwnd in child_hwnds {
            if child_hwnd != hwnd {
                if inject_and_shield(win_pid, child_hwnd, enable).is_ok() {
                    any_ok = true;
                }
            }
        }

        if any_ok { Ok(()) }
        else { Err(format!("Could not shield any window for PID {win_pid}")) }
    }
    #[cfg(not(windows))]
    Err("Windows only".to_string())
}

/// Collect all visible HWNDs belonging to a given PID (top-level + children).
#[cfg(windows)]
fn collect_process_hwnds(target_pid: u32) -> Vec<usize> {
    use winapi::shared::minwindef::{BOOL, LPARAM};
    use winapi::shared::windef::HWND;
    use winapi::um::winuser::{EnumWindows, EnumChildWindows, GetWindowThreadProcessId, IsWindowVisible};

    struct CollectState { pid: u32, hwnds: Vec<usize> }

    unsafe extern "system" fn top_level_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let state = &mut *(lparam as *mut CollectState);
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == state.pid {
            state.hwnds.push(hwnd as usize);
            // Also collect children of this top-level window
            EnumChildWindows(hwnd, Some(child_proc), lparam);
        }
        1
    }

    unsafe extern "system" fn child_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let state = &mut *(lparam as *mut CollectState);
        if IsWindowVisible(hwnd) != 0 {
            state.hwnds.push(hwnd as usize);
        }
        1
    }

    let mut state = CollectState { pid: target_pid, hwnds: Vec::new() };
    unsafe {
        EnumWindows(Some(top_level_proc), &mut state as *mut _ as LPARAM);
    }
    state.hwnds
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
    use winapi::um::processthreadsapi::{OpenProcess, CreateRemoteThread, GetExitCodeThread};
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
        // Open target process
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

        // kernel32.dll maps at the same VA in all processes (shared section)
        let k32 = LoadLibraryA(b"kernel32.dll\0".as_ptr() as _);
        let load_lib = GetProcAddress(k32, b"LoadLibraryA\0".as_ptr() as _);
        FreeLibrary(k32);

        // Inject DLL — LoadLibraryA returns HMODULE (= remote base) as thread exit code
        let t1 = CreateRemoteThread(proc, null_mut(), 0,
            Some(std::mem::transmute(load_lib)),
            remote_path, 0, null_mut());
        if t1.is_null() {
            VirtualFreeEx(proc, remote_path, 0, MEM_RELEASE);
            CloseHandle(proc);
            return Err(format!("CreateRemoteThread(LoadLib): {}", last_os_error()));
        }
        WaitForSingleObject(t1, 8000);

        // ── ASLR fix: get the actual DLL base in the remote process ──────────
        // GetExitCodeThread gives us LoadLibraryA's return value = remote HMODULE
        let mut remote_base: u32 = 0;
        GetExitCodeThread(t1, &mut remote_base);
        CloseHandle(t1);
        VirtualFreeEx(proc, remote_path, 0, MEM_RELEASE);

        if remote_base == 0 {
            CloseHandle(proc);
            return Err(format!("DLL injection failed for PID {pid}: LoadLibrary returned NULL"));
        }

        // Load DLL locally to compute function offset from our base
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

        // ASLR-correct remote address:
        //   remote_fn = remote_base + (local_fn - local_base)
        let our_base = our_dll as usize;
        let fn_offset = fn_ptr as usize - our_base;
        let remote_fn = (remote_base as usize).wrapping_add(fn_offset);
        FreeLibrary(our_dll);

        // Encode hwnd (low 32 bits) | enable flag (bit 32) as lpParameter
        let param = (hwnd & 0xFFFF_FFFF) | (if enable { 1usize << 32 } else { 0 });

        // Call shield_window in remote process at the correct ASLR address
        let t2 = CreateRemoteThread(proc, null_mut(), 0,
            Some(std::mem::transmute(remote_fn)),
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
