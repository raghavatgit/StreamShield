use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDeviceInfo {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub is_virtual: bool,
}

static BRIDGE_STOP_FLAG: Mutex<Option<Arc<AtomicBool>>> = Mutex::new(None);

#[cfg(windows)]
#[allow(non_snake_case, dead_code)]
mod wasapi_policy {
    use winapi::shared::guiddef::{GUID, LPCGUID, REFIID};
    use winapi::shared::minwindef::{BOOL, DWORD, UINT};
    use winapi::shared::ntdef::{HRESULT, LPCWSTR, LPWSTR};
    use winapi::um::unknwnbase::{IUnknown, IUnknownVtbl};
    use winapi::winrt::hstring::HSTRING;
    use winapi::RIDL;

    RIDL! {#[uuid(0xf4b1a599, 0x7266, 0x4319, 0xa8, 0xca, 0xe7, 0x0a, 0xcb, 0x11, 0xe8, 0xcd)]
    interface IAudioSessionControl(IAudioSessionControlVtbl): IUnknown(IUnknownVtbl) {
        fn GetState(pRetVal: *mut UINT,) -> HRESULT,
        fn GetDisplayName(pRetVal: *mut LPWSTR,) -> HRESULT,
        fn SetDisplayName(Value: LPCWSTR, EventContext: LPCGUID,) -> HRESULT,
        fn GetIconPath(pRetVal: *mut LPWSTR,) -> HRESULT,
        fn SetIconPath(Value: LPCWSTR, EventContext: LPCGUID,) -> HRESULT,
        fn GetGroupingParam(pRetVal: *mut GUID,) -> HRESULT,
        fn SetGroupingParam(Override: LPCGUID, EventContext: LPCGUID,) -> HRESULT,
        fn RegisterAudioSessionNotification(NewNotifications: *mut IUnknown,) -> HRESULT,
        fn UnregisterAudioSessionNotification(NewNotifications: *mut IUnknown,) -> HRESULT,
    }}

    RIDL! {#[uuid(0xbfb7ff88, 0x7239, 0x4fc9, 0x8f, 0xa2, 0x07, 0xc9, 0x50, 0xbe, 0x9c, 0x6d)]
    interface IAudioSessionControl2(IAudioSessionControl2Vtbl): IAudioSessionControl(IAudioSessionControlVtbl) {
        fn GetSessionIdentifier(pRetVal: *mut LPWSTR,) -> HRESULT,
        fn GetSessionInstanceIdentifier(pRetVal: *mut LPWSTR,) -> HRESULT,
        fn GetProcessId(pRetVal: *mut DWORD,) -> HRESULT,
        fn IsSystemSoundsSession() -> HRESULT,
        fn SetDuckingPreference(optOut: BOOL,) -> HRESULT,
    }}

    RIDL! {#[uuid(0xe2f5bb11, 0x0570, 0x40ca, 0xac, 0xdd, 0x3a, 0xa0, 0x12, 0x77, 0xdd, 0xe8)]
    interface IAudioSessionEnumerator(IAudioSessionEnumeratorVtbl): IUnknown(IUnknownVtbl) {
        fn GetCount(SessionCount: *mut i32,) -> HRESULT,
        fn GetSession(SessionCount: i32, Session: *mut *mut IAudioSessionControl,) -> HRESULT,
    }}

    RIDL! {#[uuid(0x77aa99a0, 0x1bd6, 0x484f, 0x8b, 0xc7, 0x2c, 0x65, 0x4c, 0x9a, 0x9b, 0x6f)]
    interface IAudioSessionManager2(IAudioSessionManager2Vtbl): IUnknown(IUnknownVtbl) {
        fn GetAudioSessionControl(AudioSessionGuid: LPCGUID, StreamFlags: DWORD, SessionControl: *mut *mut IAudioSessionControl,) -> HRESULT,
        fn GetSimpleAudioVolume(AudioSessionGuid: LPCGUID, StreamFlags: DWORD, AudioVolume: *mut *mut IUnknown,) -> HRESULT,
        fn GetSessionEnumerator(SessionEnum: *mut *mut IAudioSessionEnumerator,) -> HRESULT,
        fn RegisterSessionNotification(SessionNotification: *mut IUnknown,) -> HRESULT,
        fn UnregisterSessionNotification(SessionNotification: *mut IUnknown,) -> HRESULT,
        fn RegisterDuckNotification(sessionID: LPCWSTR, duckNotification: *mut IUnknown,) -> HRESULT,
        fn UnregisterDuckNotification(duckNotification: *mut IUnknown,) -> HRESULT,
    }}

    RIDL! {#[uuid(0x87ce5498, 0x68d6, 0x44e5, 0x92, 0x15, 0x6d, 0xa4, 0x7e, 0xf8, 0x83, 0xd8)]
    interface ISimpleAudioVolume(ISimpleAudioVolumeVtbl): IUnknown(IUnknownVtbl) {
        fn SetMasterVolume(fLevel: f32, EventContext: LPCGUID,) -> HRESULT,
        fn GetMasterVolume(pfLevel: *mut f32,) -> HRESULT,
        fn SetMute(bMute: BOOL, EventContext: LPCGUID,) -> HRESULT,
        fn GetMute(pbMute: *mut BOOL,) -> HRESULT,
    }}

    // Windows 10 1803+ / Windows 11 per-process audio routing (same API used by Windows Volume Mixer)
    // IInspectable has 3 methods after IUnknown (GetIids, GetRuntimeClassName, GetTrustLevel),
    // followed by IAudioPolicyConfigFactory vtable methods.
    RIDL! {#[uuid(0xab3d4648, 0xe242, 0x459f, 0xb0, 0x2f, 0x54, 0x1c, 0x70, 0x30, 0x63, 0x24)]
    interface IAudioPolicyConfigFactory(IAudioPolicyConfigFactoryVtbl): IUnknown(IUnknownVtbl) {
        fn GetIids(iidCount: *mut u32, iids: *mut *mut GUID,) -> HRESULT,
        fn GetRuntimeClassName(className: *mut HSTRING,) -> HRESULT,
        fn GetTrustLevel(trustLevel: *mut i32,) -> HRESULT,
        fn __unused0() -> HRESULT,
        fn __unused1() -> HRESULT,
        fn __unused2() -> HRESULT,
        fn __unused3() -> HRESULT,
        fn __unused4() -> HRESULT,
        fn __unused5() -> HRESULT,
        fn __unused6() -> HRESULT,
        fn __unused7() -> HRESULT,
        fn __unused8() -> HRESULT,
        fn __unused9() -> HRESULT,
        fn __unused10() -> HRESULT,
        fn __unused11() -> HRESULT,
        fn __unused12() -> HRESULT,
        fn __unused13() -> HRESULT,
        fn __unused14() -> HRESULT,
        fn __unused15() -> HRESULT,
        fn __unused16() -> HRESULT,
        fn __unused17() -> HRESULT,
        fn __unused18() -> HRESULT,
        fn __unused19() -> HRESULT,
        fn SetPersistedDefaultAudioEndpoint(
            processId: DWORD,
            flow: i32,
            role: i32,
            deviceId: HSTRING,
        ) -> HRESULT,
    }}

    #[allow(dead_code)]
    pub type _UnusedRefiid = REFIID;
}

#[cfg(windows)]
fn get_exe_name_for_pid(pid: u32) -> Option<String> {
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::processthreadsapi::OpenProcess;
    use winapi::um::winbase::QueryFullProcessImageNameW;
    use winapi::um::winnt::PROCESS_QUERY_LIMITED_INFORMATION;

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return None;
        }
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(handle, 0, buf.as_mut_ptr(), &mut len);
        CloseHandle(handle);
        if ok != 0 && len > 0 {
            let full = String::from_utf16_lossy(&buf[..len as usize]);
            return full.rsplit(['\\', '/']).next().map(|s| s.to_lowercase());
        }
        None
    }
}

#[cfg(windows)]
fn is_virtual_device_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("vb-audio")
        || lower.contains("cable")
        || lower.contains("voicemeeter")
        || lower.contains("virtual")
        || lower.contains("sonar")
        || lower.contains("wave link")
        || lower.contains("elgato")
}

/// Enumerate all active Windows audio playback (render) devices.
#[cfg(windows)]
pub fn list_audio_output_devices() -> Vec<AudioDeviceInfo> {
    use winapi::Interface;
    use winapi::shared::guiddef::GUID;
    use winapi::shared::wtypes::{PROPERTYKEY, VT_LPWSTR};
    use winapi::shared::winerror::{RPC_E_CHANGED_MODE, S_FALSE, S_OK};
    use winapi::um::combaseapi::{
        CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_ALL,
    };
    use winapi::um::mmdeviceapi::{
        eConsole, eRender, CLSID_MMDeviceEnumerator, IMMDevice, IMMDeviceCollection,
        IMMDeviceEnumerator, DEVICE_STATE_ACTIVE,
    };
    use winapi::um::objbase::COINIT_MULTITHREADED;
    use winapi::um::propidl::{PropVariantClear, PROPVARIANT};
    use winapi::um::propsys::IPropertyStore;

    const STGM_READ: u32 = 0;
    const PKEY_DEVICE_FRIENDLY_NAME: PROPERTYKEY = PROPERTYKEY {
        fmtid: GUID {
            Data1: 0xa45c254e,
            Data2: 0xdf1c,
            Data3: 0x4efd,
            Data4: [0x80, 0x20, 0x67, 0xd1, 0x46, 0xa8, 0x50, 0xe0],
        },
        pid: 14,
    };

    let mut devices = Vec::new();
    unsafe {
        let hr_init = CoInitializeEx(std::ptr::null_mut(), COINIT_MULTITHREADED);
        let should_uninit = hr_init == S_OK || hr_init == S_FALSE;
        if hr_init < 0 && hr_init != RPC_E_CHANGED_MODE {
            return devices;
        }

        let mut enumerator: *mut IMMDeviceEnumerator = std::ptr::null_mut();
        if CoCreateInstance(
            &CLSID_MMDeviceEnumerator,
            std::ptr::null_mut(),
            CLSCTX_ALL,
            &IMMDeviceEnumerator::uuidof(),
            &mut enumerator as *mut _ as *mut _,
        ) >= 0
            && !enumerator.is_null()
        {
            // Get default render device ID
            let mut default_id = String::new();
            let mut def_dev: *mut IMMDevice = std::ptr::null_mut();
            if (*enumerator).GetDefaultAudioEndpoint(eRender, eConsole, &mut def_dev) >= 0
                && !def_dev.is_null()
            {
                let mut pwstr = std::ptr::null_mut();
                if (*def_dev).GetId(&mut pwstr) >= 0 && !pwstr.is_null() {
                    let mut len = 0;
                    while *pwstr.offset(len) != 0 {
                        len += 1;
                    }
                    default_id = String::from_utf16_lossy(std::slice::from_raw_parts(
                        pwstr,
                        len as usize,
                    ));
                    CoTaskMemFree(pwstr as _);
                }
                (*def_dev).Release();
            }

            let mut collection: *mut IMMDeviceCollection = std::ptr::null_mut();
            if (*enumerator).EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE, &mut collection) >= 0
                && !collection.is_null()
            {
                let mut count: u32 = 0;
                if (*collection).GetCount(&mut count) >= 0 {
                    for i in 0..count {
                        let mut dev: *mut IMMDevice = std::ptr::null_mut();
                        if (*collection).Item(i, &mut dev) < 0 || dev.is_null() {
                            continue;
                        }
                        let mut id = String::new();
                        let mut pwstr = std::ptr::null_mut();
                        if (*dev).GetId(&mut pwstr) >= 0 && !pwstr.is_null() {
                            let mut len = 0;
                            while *pwstr.offset(len) != 0 {
                                len += 1;
                            }
                            id = String::from_utf16_lossy(std::slice::from_raw_parts(
                                pwstr,
                                len as usize,
                            ));
                            CoTaskMemFree(pwstr as _);
                        }

                        let mut name = id.clone();
                        let mut store: *mut IPropertyStore = std::ptr::null_mut();
                        if (*dev).OpenPropertyStore(STGM_READ, &mut store) >= 0 && !store.is_null() {
                            let mut var: PROPVARIANT = std::mem::zeroed();
                            if (*store).GetValue(&PKEY_DEVICE_FRIENDLY_NAME, &mut var) >= 0 {
                                if var.vt as u32 == VT_LPWSTR {
                                    let p = *var.data.pwszVal();
                                    if !p.is_null() {
                                        let mut len = 0;
                                        while *p.offset(len) != 0 {
                                            len += 1;
                                        }
                                        name = String::from_utf16_lossy(
                                            std::slice::from_raw_parts(p, len as usize),
                                        );
                                    }
                                }
                                let _ = PropVariantClear(&mut var);
                            }
                            (*store).Release();
                        }

                        if !id.is_empty() {
                            let is_default = id == default_id;
                            let is_virtual = is_virtual_device_name(&name);
                            devices.push(AudioDeviceInfo {
                                id,
                                name,
                                is_default,
                                is_virtual,
                            });
                        }
                        (*dev).Release();
                    }
                }
                (*collection).Release();
            }
            (*enumerator).Release();
        }

        if should_uninit {
            CoUninitialize();
        }
    }
    devices
}

#[cfg(not(windows))]
pub fn list_audio_output_devices() -> Vec<AudioDeviceInfo> {
    Vec::new()
}

/// Route all processes of `exe_name` (and `target_pid`) to a specific Windows audio endpoint ID,
/// or pass `None` to restore them to the System Default output device.
#[cfg(windows)]
pub fn route_app_audio_endpoint(
    exe_name: &str,
    target_pid: u32,
    device_id: Option<&str>,
) -> Result<usize, String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::Interface;
    use winapi::winrt::hstring::HSTRING;
    use winapi::winrt::roapi::{RoGetActivationFactory, RoInitialize, RoUninitialize, RO_INIT_MULTITHREADED};
    use winapi::winrt::winstring::{WindowsCreateString, WindowsDeleteString};
    use wasapi_policy::IAudioPolicyConfigFactory;

    // Collect all PIDs matching exe_name
    let mut pids: HashSet<u32> = HashSet::new();
    if target_pid != 0 {
        pids.insert(target_pid);
    }
    for (_, p) in crate::window_manager::find_all_windows_for_exe(exe_name) {
        if p != 0 {
            pids.insert(p);
        }
    }
    let target_lower = exe_name.to_lowercase();
    let _ = unsafe {
        with_audio_sessions(|pid, _| {
            if get_exe_name_for_pid(pid)
                .map(|n| n == target_lower)
                .unwrap_or(false)
            {
                pids.insert(pid);
            }
        })
    };

    if pids.is_empty() {
        return Err("No active process found for audio routing".to_string());
    }

    unsafe {
        let hr_ro = RoInitialize(RO_INIT_MULTITHREADED);
        let should_uninit = hr_ro >= 0;

        let class_name: Vec<u16> = OsStr::new("Windows.Media.Internal.AudioPolicyConfig")
            .encode_wide()
            .collect();
        let mut h_class: HSTRING = std::ptr::null_mut();
        if WindowsCreateString(class_name.as_ptr(), class_name.len() as u32, &mut h_class) < 0 {
            if should_uninit {
                RoUninitialize();
            }
            return Err("Failed to create AudioPolicyConfig class string".to_string());
        }

        let mut factory: *mut IAudioPolicyConfigFactory = std::ptr::null_mut();
        let hr_fac = RoGetActivationFactory(
            h_class,
            &IAudioPolicyConfigFactory::uuidof(),
            &mut factory as *mut _ as *mut _,
        );
        WindowsDeleteString(h_class);

        if hr_fac < 0 || factory.is_null() {
            if should_uninit {
                RoUninitialize();
            }
            return Err(format!(
                "AudioPolicyConfig unavailable on this Windows build (0x{:08X})",
                hr_fac as u32
            ));
        }

        let mut h_dev: HSTRING = std::ptr::null_mut();
        if let Some(dev_id) = device_id {
            let formatted = format!(
                "\\\\?\\SWD#MMDEVAPI#{}#{{e6327cad-dcec-4949-ae8a-991e976a79d2}}",
                dev_id
            );
            let wide: Vec<u16> = OsStr::new(&formatted).encode_wide().collect();
            let _ = WindowsCreateString(wide.as_ptr(), wide.len() as u32, &mut h_dev);
        }

        let mut routed = 0usize;
        for pid in pids {
            // eRender = 0; eConsole = 0, eMultimedia = 1
            let hr0 = (*factory).SetPersistedDefaultAudioEndpoint(pid, 0, 0, h_dev);
            let hr1 = (*factory).SetPersistedDefaultAudioEndpoint(pid, 0, 1, h_dev);
            if hr0 >= 0 || hr1 >= 0 {
                routed += 1;
            }
        }

        if !h_dev.is_null() {
            WindowsDeleteString(h_dev);
        }
        (*factory).Release();
        if should_uninit {
            RoUninitialize();
        }

        Ok(routed)
    }
}

#[cfg(not(windows))]
pub fn route_app_audio_endpoint(
    _exe_name: &str,
    _target_pid: u32,
    _device_id: Option<&str>,
) -> Result<usize, String> {
    Ok(0)
}

/// Start or stop the built-in low-latency WASAPI loopback bridge
/// forwarding `stream_device_id` -> `monitor_device_id` (your Headphones).
pub fn sync_audio_bridge(
    enabled: bool,
    stream_device_id: Option<String>,
    monitor_device_id: Option<String>,
) {
    // Stop any running bridge first
    if let Ok(mut guard) = BRIDGE_STOP_FLAG.lock() {
        if let Some(flag) = guard.take() {
            flag.store(true, Ordering::SeqCst);
        }
    }

    if !enabled {
        return;
    }

    let (src_id, dst_id) = match (stream_device_id, monitor_device_id) {
        (Some(s), Some(d)) if !s.is_empty() && !d.is_empty() && s != d => (s, d),
        _ => return,
    };

    let stop_flag = Arc::new(AtomicBool::new(false));
    if let Ok(mut guard) = BRIDGE_STOP_FLAG.lock() {
        *guard = Some(stop_flag.clone());
    }

    #[cfg(windows)]
    std::thread::spawn(move || {
        run_wasapi_loopback_bridge(&src_id, &dst_id, &stop_flag);
    });
    #[cfg(not(windows))]
    let _ = (src_id, dst_id, stop_flag);
}

#[cfg(windows)]
fn run_wasapi_loopback_bridge(src_id: &str, dst_id: &str, stop_flag: &AtomicBool) {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::Interface;
    use winapi::shared::mmreg::WAVEFORMATEX;
    use winapi::shared::winerror::{RPC_E_CHANGED_MODE, S_FALSE, S_OK};
    use winapi::um::audioclient::{
        IAudioCaptureClient, IAudioClient, IAudioRenderClient,
    };
    const AUDCLNT_SHAREMODE_SHARED: u32 = 0;
    use winapi::um::combaseapi::{
        CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_ALL,
    };
    use winapi::um::mmdeviceapi::{CLSID_MMDeviceEnumerator, IMMDevice, IMMDeviceEnumerator};
    use winapi::um::objbase::COINIT_MULTITHREADED;

    const AUDCLNT_STREAMFLAGS_LOOPBACK: u32 = 0x0002_0000;
    const AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM: u32 = 0x8000_0000;
    const AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY: u32 = 0x0800_0000;
    const BUFFER_DURATION_100NS: i64 = 200_000; // 20ms shared-mode buffer

    fn wide(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    }

    unsafe {
        let hr_init = CoInitializeEx(std::ptr::null_mut(), COINIT_MULTITHREADED);
        let should_uninit = hr_init == S_OK || hr_init == S_FALSE;
        if hr_init < 0 && hr_init != RPC_E_CHANGED_MODE {
            return;
        }

        let mut enumerator: *mut IMMDeviceEnumerator = std::ptr::null_mut();
        if CoCreateInstance(
            &CLSID_MMDeviceEnumerator,
            std::ptr::null_mut(),
            CLSCTX_ALL,
            &IMMDeviceEnumerator::uuidof(),
            &mut enumerator as *mut _ as *mut _,
        ) < 0
            || enumerator.is_null()
        {
            if should_uninit {
                CoUninitialize();
            }
            return;
        }

        let src_w = wide(src_id);
        let dst_w = wide(dst_id);
        let mut src_dev: *mut IMMDevice = std::ptr::null_mut();
        let mut dst_dev: *mut IMMDevice = std::ptr::null_mut();

        let ok_src = (*enumerator).GetDevice(src_w.as_ptr(), &mut src_dev) >= 0 && !src_dev.is_null();
        let ok_dst = (*enumerator).GetDevice(dst_w.as_ptr(), &mut dst_dev) >= 0 && !dst_dev.is_null();
        (*enumerator).Release();

        if !ok_src || !ok_dst {
            if !src_dev.is_null() { (*src_dev).Release(); }
            if !dst_dev.is_null() { (*dst_dev).Release(); }
            if should_uninit { CoUninitialize(); }
            return;
        }

        let mut cap_client: *mut IAudioClient = std::ptr::null_mut();
        let mut ren_client: *mut IAudioClient = std::ptr::null_mut();
        let ok_cap = (*src_dev).Activate(
            &IAudioClient::uuidof(),
            CLSCTX_ALL,
            std::ptr::null_mut(),
            &mut cap_client as *mut _ as *mut _,
        ) >= 0
            && !cap_client.is_null();
        let ok_ren = (*dst_dev).Activate(
            &IAudioClient::uuidof(),
            CLSCTX_ALL,
            std::ptr::null_mut(),
            &mut ren_client as *mut _ as *mut _,
        ) >= 0
            && !ren_client.is_null();
        (*src_dev).Release();
        (*dst_dev).Release();

        if !ok_cap || !ok_ren {
            if !cap_client.is_null() { (*cap_client).Release(); }
            if !ren_client.is_null() { (*ren_client).Release(); }
            if should_uninit { CoUninitialize(); }
            return;
        }

        let mut mix_fmt: *mut WAVEFORMATEX = std::ptr::null_mut();
        if (*cap_client).GetMixFormat(&mut mix_fmt) < 0 || mix_fmt.is_null() {
            (*cap_client).Release();
            (*ren_client).Release();
            if should_uninit { CoUninitialize(); }
            return;
        }

        let block_align = (*mix_fmt).nBlockAlign as usize;
        let hr_cinit = (*cap_client).Initialize(
            AUDCLNT_SHAREMODE_SHARED,
            AUDCLNT_STREAMFLAGS_LOOPBACK,
            BUFFER_DURATION_100NS,
            0,
            mix_fmt,
            std::ptr::null(),
        );
        let hr_rinit = (*ren_client).Initialize(
            AUDCLNT_SHAREMODE_SHARED,
            AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM | AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY,
            BUFFER_DURATION_100NS,
            0,
            mix_fmt,
            std::ptr::null(),
        );
        CoTaskMemFree(mix_fmt as _);

        if hr_cinit < 0 || hr_rinit < 0 || block_align == 0 {
            (*cap_client).Release();
            (*ren_client).Release();
            if should_uninit { CoUninitialize(); }
            return;
        }

        let mut cap_svc: *mut IAudioCaptureClient = std::ptr::null_mut();
        let mut ren_svc: *mut IAudioRenderClient = std::ptr::null_mut();
        let mut ren_buf_frames: u32 = 0;
        let _ = (*ren_client).GetBufferSize(&mut ren_buf_frames);

        if (*cap_client).GetService(&IAudioCaptureClient::uuidof(), &mut cap_svc as *mut _ as *mut _) < 0
            || (*ren_client).GetService(&IAudioRenderClient::uuidof(), &mut ren_svc as *mut _ as *mut _) < 0
            || cap_svc.is_null()
            || ren_svc.is_null()
        {
            if !cap_svc.is_null() { (*cap_svc).Release(); }
            if !ren_svc.is_null() { (*ren_svc).Release(); }
            (*cap_client).Release();
            (*ren_client).Release();
            if should_uninit { CoUninitialize(); }
            return;
        }

        let _ = (*cap_client).Start();
        let _ = (*ren_client).Start();

        while !stop_flag.load(Ordering::Relaxed) {
            let mut packet_frames: u32 = 0;
            while (*cap_svc).GetNextPacketSize(&mut packet_frames) >= 0 && packet_frames > 0 {
                let mut data_ptr: *mut u8 = std::ptr::null_mut();
                let mut num_frames: u32 = 0;
                let mut flags: u32 = 0;
                if (*cap_svc).GetBuffer(
                    &mut data_ptr,
                    &mut num_frames,
                    &mut flags,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                ) >= 0
                {
                    let mut padding: u32 = 0;
                    if (*ren_client).GetCurrentPadding(&mut padding) >= 0 {
                        let avail = ren_buf_frames.saturating_sub(padding);
                        let to_write = num_frames.min(avail);
                        if to_write > 0 {
                            let mut out_ptr: *mut u8 = std::ptr::null_mut();
                            if (*ren_svc).GetBuffer(to_write, &mut out_ptr) >= 0 && !out_ptr.is_null() {
                                let bytes = (to_write as usize) * block_align;
                                if (flags & 0x2) != 0 || data_ptr.is_null() {
                                    std::ptr::write_bytes(out_ptr, 0, bytes);
                                } else {
                                    std::ptr::copy_nonoverlapping(data_ptr, out_ptr, bytes);
                                }
                                let _ = (*ren_svc).ReleaseBuffer(to_write, 0);
                            }
                        }
                    }
                    let _ = (*cap_svc).ReleaseBuffer(num_frames);
                } else {
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }

        let _ = (*cap_client).Stop();
        let _ = (*ren_client).Stop();
        (*cap_svc).Release();
        (*ren_svc).Release();
        (*cap_client).Release();
        (*ren_client).Release();
        if should_uninit {
            CoUninitialize();
        }
    }
}

#[cfg(windows)]
unsafe fn with_audio_sessions<F>(mut callback: F) -> Result<(), String>
where
    F: FnMut(u32, *mut wasapi_policy::ISimpleAudioVolume),
{
    use winapi::Interface;
    use winapi::shared::winerror::{RPC_E_CHANGED_MODE, S_FALSE, S_OK};
    use wasapi_policy::{
        IAudioSessionControl, IAudioSessionControl2, IAudioSessionEnumerator,
        IAudioSessionManager2, ISimpleAudioVolume,
    };
    use winapi::um::combaseapi::{CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL};
    use winapi::um::mmdeviceapi::{
        eConsole, eRender, CLSID_MMDeviceEnumerator, IMMDevice, IMMDeviceEnumerator,
    };
    use winapi::um::objbase::COINIT_MULTITHREADED;

    let hr_init = CoInitializeEx(std::ptr::null_mut(), COINIT_MULTITHREADED);
    let should_uninit = hr_init == S_OK || hr_init == S_FALSE;
    if hr_init < 0 && hr_init != RPC_E_CHANGED_MODE {
        return Err(format!("CoInitializeEx failed: 0x{:08X}", hr_init as u32));
    }

    let result = (|| -> Result<(), String> {
        let mut enumerator: *mut IMMDeviceEnumerator = std::ptr::null_mut();
        let hr = CoCreateInstance(
            &CLSID_MMDeviceEnumerator,
            std::ptr::null_mut(),
            CLSCTX_ALL,
            &IMMDeviceEnumerator::uuidof(),
            &mut enumerator as *mut _ as *mut _,
        );
        if hr < 0 || enumerator.is_null() {
            return Err(format!("Failed to create IMMDeviceEnumerator: 0x{:08X}", hr as u32));
        }

        let mut device: *mut IMMDevice = std::ptr::null_mut();
        let hr_dev = (*enumerator).GetDefaultAudioEndpoint(eRender, eConsole, &mut device);
        (*enumerator).Release();
        if hr_dev < 0 || device.is_null() {
            return Err("No active audio output device found".to_string());
        }

        let mut mgr: *mut IAudioSessionManager2 = std::ptr::null_mut();
        let hr_mgr = (*device).Activate(
            &IAudioSessionManager2::uuidof(),
            CLSCTX_ALL,
            std::ptr::null_mut(),
            &mut mgr as *mut _ as *mut _,
        );
        (*device).Release();
        if hr_mgr < 0 || mgr.is_null() {
            return Err(format!("Failed to activate IAudioSessionManager2: 0x{:08X}", hr_mgr as u32));
        }

        let mut session_enum: *mut IAudioSessionEnumerator = std::ptr::null_mut();
        let hr_enum = (*mgr).GetSessionEnumerator(&mut session_enum);
        (*mgr).Release();
        if hr_enum < 0 || session_enum.is_null() {
            return Err("Failed to enumerate audio sessions".to_string());
        }

        let mut count: i32 = 0;
        if (*session_enum).GetCount(&mut count) >= 0 {
            for i in 0..count {
                let mut ctrl: *mut IAudioSessionControl = std::ptr::null_mut();
                if (*session_enum).GetSession(i, &mut ctrl) < 0 || ctrl.is_null() {
                    continue;
                }

                let mut ctrl2: *mut IAudioSessionControl2 = std::ptr::null_mut();
                let hr_q2 = (*ctrl).QueryInterface(
                    &IAudioSessionControl2::uuidof(),
                    &mut ctrl2 as *mut _ as *mut _,
                );
                let mut vol: *mut ISimpleAudioVolume = std::ptr::null_mut();
                let hr_qv = (*ctrl).QueryInterface(
                    &ISimpleAudioVolume::uuidof(),
                    &mut vol as *mut _ as *mut _,
                );

                if hr_q2 >= 0 && !ctrl2.is_null() && hr_qv >= 0 && !vol.is_null() {
                    let mut pid: u32 = 0;
                    if (*ctrl2).GetProcessId(&mut pid) >= 0 && pid != 0 {
                        callback(pid, vol);
                    }
                }

                if !vol.is_null() { (*vol).Release(); }
                if !ctrl2.is_null() { (*ctrl2).Release(); }
                (*ctrl).Release();
            }
        }

        (*session_enum).Release();
        Ok(())
    })();

    if should_uninit {
        CoUninitialize();
    }

    result
}

/// Set mute state for all active audio sessions belonging to `exe_name` (Panic Mute fallback).
#[cfg(windows)]
pub fn set_app_audio_mute(exe_name: &str, target_pid: u32, mute: bool) -> Result<usize, String> {
    let target_lower = exe_name.to_lowercase();
    let mut updated = 0usize;
    let mute_val: i32 = if mute { 1 } else { 0 };

    unsafe {
        with_audio_sessions(|pid, vol| {
            let matches_pid = target_pid != 0 && pid == target_pid;
            let matches_exe = if matches_pid {
                true
            } else {
                get_exe_name_for_pid(pid)
                    .map(|name| name == target_lower)
                    .unwrap_or(false)
            };
            if matches_exe {
                if (*vol).SetMute(mute_val, std::ptr::null()) >= 0 {
                    updated += 1;
                }
            }
        })?;
    }

    Ok(updated)
}

#[cfg(not(windows))]
pub fn set_app_audio_mute(_exe_name: &str, _target_pid: u32, _mute: bool) -> Result<usize, String> {
    Ok(0)
}

#[cfg(windows)]
pub fn get_muted_exes() -> HashSet<String> {
    let mut muted = HashSet::new();
    unsafe {
        let _ = with_audio_sessions(|pid, vol| {
            let mut is_muted: i32 = 0;
            if (*vol).GetMute(&mut is_muted) >= 0 && is_muted != 0 {
                if let Some(name) = get_exe_name_for_pid(pid) {
                    muted.insert(name);
                }
            }
        });
    }
    muted
}

#[cfg(not(windows))]
pub fn get_muted_exes() -> HashSet<String> {
    HashSet::new()
}

#[cfg(windows)]
pub fn unmute_all_exes(exes: &HashSet<String>) -> Result<usize, String> {
    let mut count = 0usize;
    unsafe {
        with_audio_sessions(|pid, vol| {
            let mut is_muted: i32 = 0;
            if (*vol).GetMute(&mut is_muted) >= 0 && is_muted != 0 {
                let should_unmute = exes.is_empty()
                    || get_exe_name_for_pid(pid)
                        .map(|n| exes.contains(&n))
                        .unwrap_or(false);
                if should_unmute && (*vol).SetMute(0, std::ptr::null()) >= 0 {
                    count += 1;
                }
            }
        })?;
    }
    Ok(count)
}

#[cfg(not(windows))]
pub fn unmute_all_exes(_exes: &HashSet<String>) -> Result<usize, String> {
    Ok(0)
}
