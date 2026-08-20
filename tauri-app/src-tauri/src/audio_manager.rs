#[cfg(windows)]
pub fn set_process_audio_mute(exe_name: &str, target_pid: u32, mute: bool) -> Result<(), String> {
    use std::ptr;
    use winapi::shared::guiddef::GUID;
    use winapi::shared::minwindef::BOOL;
    use winapi::shared::winerror::{HRESULT, S_OK};
    use winapi::um::combaseapi::{CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINITBASE_MULTITHREADED};
    use winapi::um::unknwnbase::{IUnknown, IUnknownVtbl};

    const CLSID_MM_DEVICE_ENUMERATOR: GUID = GUID {
        Data1: 0xBCDE0395, Data2: 0xE52F, Data3: 0x467C,
        Data4: [0x8E, 0x3D, 0xC4, 0x57, 0x92, 0x91, 0x69, 0x2E],
    };
    const IID_IMM_DEVICE_ENUMERATOR: GUID = GUID {
        Data1: 0xA95664D2, Data2: 0x9614, Data3: 0x4F35,
        Data4: [0xA7, 0x46, 0xDE, 0x8D, 0xB6, 0x36, 0x17, 0xE6],
    };
    const IID_IAUDIO_SESSION_MANAGER2: GUID = GUID {
        Data1: 0x77AA99A0, Data2: 0x1BD6, Data3: 0x484F,
        Data4: [0x8B, 0xC7, 0x2C, 0x65, 0x4C, 0x9A, 0x9B, 0x6F],
    };
    const IID_IAUDIO_SESSION_CONTROL2: GUID = GUID {
        Data1: 0xbfb7ff88, Data2: 0x7239, Data3: 0x4fc9,
        Data4: [0x8f, 0xa2, 0x07, 0xc9, 0x50, 0xbe, 0x9c, 0x6d],
    };
    const IID_ISIMPLE_AUDIO_VOLUME: GUID = GUID {
        Data1: 0x87CE5498, Data2: 0x68D6, Data3: 0x44E5,
        Data4: [0x92, 0x15, 0x6D, 0xA4, 0x7E, 0xF8, 0x83, 0xD8],
    };

    #[repr(C)]
    struct IMMDeviceEnumeratorVtbl {
        parent: IUnknownVtbl,
        enum_audio_endpoints: unsafe extern "system" fn(*mut IUnknown, i32, u32, *mut *mut IUnknown) -> HRESULT,
        get_default_audio_endpoint: unsafe extern "system" fn(*mut IUnknown, i32, i32, *mut *mut IUnknown) -> HRESULT,
    }

    #[repr(C)]
    struct IMMDeviceCollectionVtbl {
        parent: IUnknownVtbl,
        get_count: unsafe extern "system" fn(*mut IUnknown, *mut u32) -> HRESULT,
        item: unsafe extern "system" fn(*mut IUnknown, u32, *mut *mut IUnknown) -> HRESULT,
    }

    #[repr(C)]
    struct IMMDeviceVtbl {
        parent: IUnknownVtbl,
        activate: unsafe extern "system" fn(*mut IUnknown, *const GUID, u32, *mut (), *mut *mut IUnknown) -> HRESULT,
    }

    #[repr(C)]
    struct IAudioSessionManager2Vtbl {
        parent: IUnknownVtbl,
        get_audio_session_control: unsafe extern "system" fn(*mut IUnknown, *const GUID, u32, *mut *mut IUnknown) -> HRESULT,
        get_simple_audio_volume: unsafe extern "system" fn(*mut IUnknown, *const GUID, u32, *mut *mut IUnknown) -> HRESULT,
        get_session_enumerator: unsafe extern "system" fn(*mut IUnknown, *mut *mut IUnknown) -> HRESULT,
    }

    #[repr(C)]
    struct IAudioSessionEnumeratorVtbl {
        parent: IUnknownVtbl,
        get_count: unsafe extern "system" fn(*mut IUnknown, *mut i32) -> HRESULT,
        get_session: unsafe extern "system" fn(*mut IUnknown, i32, *mut *mut IUnknown) -> HRESULT,
    }

    #[repr(C)]
    struct IAudioSessionControl2Vtbl {
        parent: IUnknownVtbl,
        get_state: unsafe extern "system" fn(*mut IUnknown, *mut i32) -> HRESULT,
        get_display_name: unsafe extern "system" fn(*mut IUnknown, *mut *mut u16) -> HRESULT,
        set_display_name: unsafe extern "system" fn(*mut IUnknown, *const u16, *const GUID) -> HRESULT,
        get_icon_path: unsafe extern "system" fn(*mut IUnknown, *mut *mut u16) -> HRESULT,
        set_icon_path: unsafe extern "system" fn(*mut IUnknown, *const u16, *const GUID) -> HRESULT,
        get_grouping_param: unsafe extern "system" fn(*mut IUnknown, *mut GUID) -> HRESULT,
        set_grouping_param: unsafe extern "system" fn(*mut IUnknown, *const GUID, *const GUID) -> HRESULT,
        register_audio_session_notification: unsafe extern "system" fn(*mut IUnknown, *mut IUnknown) -> HRESULT,
        unregister_audio_session_notification: unsafe extern "system" fn(*mut IUnknown, *mut IUnknown) -> HRESULT,
        get_session_identifier: unsafe extern "system" fn(*mut IUnknown, *mut *mut u16) -> HRESULT,
        get_session_instance_identifier: unsafe extern "system" fn(*mut IUnknown, *mut *mut u16) -> HRESULT,
        get_process_id: unsafe extern "system" fn(*mut IUnknown, *mut u32) -> HRESULT,
    }

    #[repr(C)]
    struct ISimpleAudioVolumeVtbl {
        parent: IUnknownVtbl,
        set_master_volume: unsafe extern "system" fn(*mut IUnknown, f32, *const GUID) -> HRESULT,
        get_master_volume: unsafe extern "system" fn(*mut IUnknown, *mut f32) -> HRESULT,
        set_mute: unsafe extern "system" fn(*mut IUnknown, BOOL, *const GUID) -> HRESULT,
        get_mute: unsafe extern "system" fn(*mut IUnknown, *mut BOOL) -> HRESULT,
    }

    let mute_val: BOOL = if mute { 1 } else { 0 };
    let clean_exe = exe_name.to_lowercase();

    unsafe {
        CoInitializeEx(ptr::null_mut(), COINITBASE_MULTITHREADED);

        let mut device_enumerator: *mut IUnknown = ptr::null_mut();
        let hr = CoCreateInstance(
            &CLSID_MM_DEVICE_ENUMERATOR,
            ptr::null_mut(),
            CLSCTX_ALL,
            &IID_IMM_DEVICE_ENUMERATOR,
            &mut device_enumerator as *mut _ as *mut _,
        );
        if hr != S_OK || device_enumerator.is_null() {
            CoUninitialize();
            return Err("Failed to create MMDeviceEnumerator".to_string());
        }

        let enum_vtbl = &*((*device_enumerator).lpVtbl as *const IMMDeviceEnumeratorVtbl);
        let mut device_collection: *mut IUnknown = ptr::null_mut();
        // eRender = 0, DEVICE_STATE_ACTIVE = 1
        let hr = (enum_vtbl.enum_audio_endpoints)(device_enumerator, 0, 1, &mut device_collection);
        ((*device_enumerator).lpVtbl.as_ref().unwrap().Release)(device_enumerator);

        if hr != S_OK || device_collection.is_null() {
            CoUninitialize();
            return Err("Failed to enumerate active audio endpoints".to_string());
        }

        let col_vtbl = &*((*device_collection).lpVtbl as *const IMMDeviceCollectionVtbl);
        let mut dev_count: u32 = 0;
        (col_vtbl.get_count)(device_collection, &mut dev_count);

        for d in 0..dev_count {
            let mut device: *mut IUnknown = ptr::null_mut();
            if (col_vtbl.item)(device_collection, d, &mut device) != S_OK || device.is_null() {
                continue;
            }

            let dev_vtbl = &*((*device).lpVtbl as *const IMMDeviceVtbl);
            let mut session_manager: *mut IUnknown = ptr::null_mut();
            let hr = (dev_vtbl.activate)(
                device,
                &IID_IAUDIO_SESSION_MANAGER2,
                CLSCTX_ALL,
                ptr::null_mut(),
                &mut session_manager,
            );
            ((*device).lpVtbl.as_ref().unwrap().Release)(device);

            if hr != S_OK || session_manager.is_null() {
                continue;
            }

            let mgr_vtbl = &*((*session_manager).lpVtbl as *const IAudioSessionManager2Vtbl);
            let mut session_enum: *mut IUnknown = ptr::null_mut();
            let hr = (mgr_vtbl.get_session_enumerator)(session_manager, &mut session_enum);
            ((*session_manager).lpVtbl.as_ref().unwrap().Release)(session_manager);

            if hr != S_OK || session_enum.is_null() {
                continue;
            }

            let enum_session_vtbl = &*((*session_enum).lpVtbl as *const IAudioSessionEnumeratorVtbl);
            let mut session_count: i32 = 0;
            (enum_session_vtbl.get_count)(session_enum, &mut session_count);

            for i in 0..session_count {
                let mut session_control: *mut IUnknown = ptr::null_mut();
                if (enum_session_vtbl.get_session)(session_enum, i, &mut session_control) != S_OK || session_control.is_null() {
                    continue;
                }

                let mut session_control2: *mut IUnknown = ptr::null_mut();
                let hr = ((*session_control).lpVtbl.as_ref().unwrap().QueryInterface)(
                    session_control,
                    &IID_IAUDIO_SESSION_CONTROL2,
                    &mut session_control2 as *mut _ as *mut _,
                );
                if hr == S_OK && !session_control2.is_null() {
                    let ctrl2_vtbl = &*((*session_control2).lpVtbl as *const IAudioSessionControl2Vtbl);
                    let mut pid: u32 = 0;
                    if (ctrl2_vtbl.get_process_id)(session_control2, &mut pid) == S_OK && pid != 0 {
                        let is_match = (target_pid != 0 && pid == target_pid) || {
                            let session_exe = get_exe_name_from_pid(pid);
                            !session_exe.is_empty() && session_exe.to_lowercase() == clean_exe
                        };

                        if is_match {
                            let mut simple_volume: *mut IUnknown = ptr::null_mut();
                            let hr = ((*session_control).lpVtbl.as_ref().unwrap().QueryInterface)(
                                session_control,
                                &IID_ISIMPLE_AUDIO_VOLUME,
                                &mut simple_volume as *mut _ as *mut _,
                            );
                            if hr == S_OK && !simple_volume.is_null() {
                                let vol_vtbl = &*((*simple_volume).lpVtbl as *const ISimpleAudioVolumeVtbl);
                                (vol_vtbl.set_mute)(simple_volume, mute_val, ptr::null());
                                ((*simple_volume).lpVtbl.as_ref().unwrap().Release)(simple_volume);
                            }
                        }
                    }
                    ((*session_control2).lpVtbl.as_ref().unwrap().Release)(session_control2);
                }
                ((*session_control).lpVtbl.as_ref().unwrap().Release)(session_control);
            }

            ((*session_enum).lpVtbl.as_ref().unwrap().Release)(session_enum);
        }

        ((*device_collection).lpVtbl.as_ref().unwrap().Release)(device_collection);
        CoUninitialize();
    }

    Ok(())
}

#[cfg(windows)]
pub fn get_process_audio_mute(exe_name: &str, target_pid: u32) -> bool {
    use std::ptr;
    use winapi::shared::guiddef::GUID;
    use winapi::shared::minwindef::BOOL;
    use winapi::shared::winerror::S_OK;
    use winapi::um::combaseapi::{CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINITBASE_MULTITHREADED};
    use winapi::um::unknwnbase::{IUnknown, IUnknownVtbl};

    const CLSID_MM_DEVICE_ENUMERATOR: GUID = GUID {
        Data1: 0xBCDE0395, Data2: 0xE52F, Data3: 0x467C,
        Data4: [0x8E, 0x3D, 0xC4, 0x57, 0x92, 0x91, 0x69, 0x2E],
    };
    const IID_IMM_DEVICE_ENUMERATOR: GUID = GUID {
        Data1: 0xA95664D2, Data2: 0x9614, Data3: 0x4F35,
        Data4: [0xA7, 0x46, 0xDE, 0x8D, 0xB6, 0x36, 0x17, 0xE6],
    };
    const IID_IAUDIO_SESSION_MANAGER2: GUID = GUID {
        Data1: 0x77AA99A0, Data2: 0x1BD6, Data3: 0x484F,
        Data4: [0x8B, 0xC7, 0x2C, 0x65, 0x4C, 0x9A, 0x9B, 0x6F],
    };
    const IID_IAUDIO_SESSION_CONTROL2: GUID = GUID {
        Data1: 0xbfb7ff88, Data2: 0x7239, Data3: 0x4fc9,
        Data4: [0x8f, 0xa2, 0x07, 0xc9, 0x50, 0xbe, 0x9c, 0x6d],
    };
    const IID_ISIMPLE_AUDIO_VOLUME: GUID = GUID {
        Data1: 0x87CE5498, Data2: 0x68D6, Data3: 0x44E5,
        Data4: [0x92, 0x15, 0x6D, 0xA4, 0x7E, 0xF8, 0x83, 0xD8],
    };

    #[repr(C)]
    struct IMMDeviceEnumeratorVtbl {
        parent: IUnknownVtbl,
        enum_audio_endpoints: unsafe extern "system" fn(*mut IUnknown, i32, u32, *mut *mut IUnknown) -> winapi::shared::winerror::HRESULT,
        get_default_audio_endpoint: unsafe extern "system" fn(*mut IUnknown, i32, i32, *mut *mut IUnknown) -> winapi::shared::winerror::HRESULT,
    }

    #[repr(C)]
    struct IMMDeviceCollectionVtbl {
        parent: IUnknownVtbl,
        get_count: unsafe extern "system" fn(*mut IUnknown, *mut u32) -> winapi::shared::winerror::HRESULT,
        item: unsafe extern "system" fn(*mut IUnknown, u32, *mut *mut IUnknown) -> winapi::shared::winerror::HRESULT,
    }

    #[repr(C)]
    struct IMMDeviceVtbl {
        parent: IUnknownVtbl,
        activate: unsafe extern "system" fn(*mut IUnknown, *const GUID, u32, *mut (), *mut *mut IUnknown) -> winapi::shared::winerror::HRESULT,
    }

    #[repr(C)]
    struct IAudioSessionManager2Vtbl {
        parent: IUnknownVtbl,
        get_audio_session_control: unsafe extern "system" fn(*mut IUnknown, *const GUID, u32, *mut *mut IUnknown) -> winapi::shared::winerror::HRESULT,
        get_simple_audio_volume: unsafe extern "system" fn(*mut IUnknown, *const GUID, u32, *mut *mut IUnknown) -> winapi::shared::winerror::HRESULT,
        get_session_enumerator: unsafe extern "system" fn(*mut IUnknown, *mut *mut IUnknown) -> winapi::shared::winerror::HRESULT,
    }

    #[repr(C)]
    struct IAudioSessionEnumeratorVtbl {
        parent: IUnknownVtbl,
        get_count: unsafe extern "system" fn(*mut IUnknown, *mut i32) -> winapi::shared::winerror::HRESULT,
        get_session: unsafe extern "system" fn(*mut IUnknown, i32, *mut *mut IUnknown) -> winapi::shared::winerror::HRESULT,
    }

    #[repr(C)]
    struct IAudioSessionControl2Vtbl {
        parent: IUnknownVtbl,
        get_state: unsafe extern "system" fn(*mut IUnknown, *mut i32) -> winapi::shared::winerror::HRESULT,
        get_display_name: unsafe extern "system" fn(*mut IUnknown, *mut *mut u16) -> winapi::shared::winerror::HRESULT,
        set_display_name: unsafe extern "system" fn(*mut IUnknown, *const u16, *const GUID) -> winapi::shared::winerror::HRESULT,
        get_icon_path: unsafe extern "system" fn(*mut IUnknown, *mut *mut u16) -> winapi::shared::winerror::HRESULT,
        set_icon_path: unsafe extern "system" fn(*mut IUnknown, *const u16, *const GUID) -> winapi::shared::winerror::HRESULT,
        get_grouping_param: unsafe extern "system" fn(*mut IUnknown, *mut GUID) -> winapi::shared::winerror::HRESULT,
        set_grouping_param: unsafe extern "system" fn(*mut IUnknown, *const GUID, *const GUID) -> winapi::shared::winerror::HRESULT,
        register_audio_session_notification: unsafe extern "system" fn(*mut IUnknown, *mut IUnknown) -> winapi::shared::winerror::HRESULT,
        unregister_audio_session_notification: unsafe extern "system" fn(*mut IUnknown, *mut IUnknown) -> winapi::shared::winerror::HRESULT,
        get_session_identifier: unsafe extern "system" fn(*mut IUnknown, *mut *mut u16) -> winapi::shared::winerror::HRESULT,
        get_session_instance_identifier: unsafe extern "system" fn(*mut IUnknown, *mut *mut u16) -> winapi::shared::winerror::HRESULT,
        get_process_id: unsafe extern "system" fn(*mut IUnknown, *mut u32) -> winapi::shared::winerror::HRESULT,
    }

    #[repr(C)]
    struct ISimpleAudioVolumeVtbl {
        parent: IUnknownVtbl,
        set_master_volume: unsafe extern "system" fn(*mut IUnknown, f32, *const GUID) -> winapi::shared::winerror::HRESULT,
        get_master_volume: unsafe extern "system" fn(*mut IUnknown, *mut f32) -> winapi::shared::winerror::HRESULT,
        set_mute: unsafe extern "system" fn(*mut IUnknown, BOOL, *const GUID) -> winapi::shared::winerror::HRESULT,
        get_mute: unsafe extern "system" fn(*mut IUnknown, *mut BOOL) -> winapi::shared::winerror::HRESULT,
    }

    let mut is_muted = false;
    let clean_exe = exe_name.to_lowercase();

    unsafe {
        CoInitializeEx(ptr::null_mut(), COINITBASE_MULTITHREADED);

        let mut device_enumerator: *mut IUnknown = ptr::null_mut();
        if CoCreateInstance(&CLSID_MM_DEVICE_ENUMERATOR, ptr::null_mut(), CLSCTX_ALL, &IID_IMM_DEVICE_ENUMERATOR, &mut device_enumerator as *mut _ as *mut _) != S_OK
            || device_enumerator.is_null()
        {
            CoUninitialize();
            return false;
        }

        let enum_vtbl = &*((*device_enumerator).lpVtbl as *const IMMDeviceEnumeratorVtbl);
        let mut device_collection: *mut IUnknown = ptr::null_mut();
        if (enum_vtbl.enum_audio_endpoints)(device_enumerator, 0, 1, &mut device_collection) != S_OK || device_collection.is_null() {
            ((*device_enumerator).lpVtbl.as_ref().unwrap().Release)(device_enumerator);
            CoUninitialize();
            return false;
        }
        ((*device_enumerator).lpVtbl.as_ref().unwrap().Release)(device_enumerator);

        let col_vtbl = &*((*device_collection).lpVtbl as *const IMMDeviceCollectionVtbl);
        let mut dev_count: u32 = 0;
        (col_vtbl.get_count)(device_collection, &mut dev_count);

        'outer: for d in 0..dev_count {
            let mut device: *mut IUnknown = ptr::null_mut();
            if (col_vtbl.item)(device_collection, d, &mut device) != S_OK || device.is_null() {
                continue;
            }

            let dev_vtbl = &*((*device).lpVtbl as *const IMMDeviceVtbl);
            let mut session_manager: *mut IUnknown = ptr::null_mut();
            let hr = (dev_vtbl.activate)(device, &IID_IAUDIO_SESSION_MANAGER2, CLSCTX_ALL, ptr::null_mut(), &mut session_manager);
            ((*device).lpVtbl.as_ref().unwrap().Release)(device);

            if hr != S_OK || session_manager.is_null() {
                continue;
            }

            let mgr_vtbl = &*((*session_manager).lpVtbl as *const IAudioSessionManager2Vtbl);
            let mut session_enum: *mut IUnknown = ptr::null_mut();
            if (mgr_vtbl.get_session_enumerator)(session_manager, &mut session_enum) != S_OK || session_enum.is_null() {
                ((*session_manager).lpVtbl.as_ref().unwrap().Release)(session_manager);
                continue;
            }
            ((*session_manager).lpVtbl.as_ref().unwrap().Release)(session_manager);

            let enum_session_vtbl = &*((*session_enum).lpVtbl as *const IAudioSessionEnumeratorVtbl);
            let mut session_count: i32 = 0;
            (enum_session_vtbl.get_count)(session_enum, &mut session_count);

            for i in 0..session_count {
                let mut session_control: *mut IUnknown = ptr::null_mut();
                if (enum_session_vtbl.get_session)(session_enum, i, &mut session_control) != S_OK || session_control.is_null() {
                    continue;
                }

                let mut session_control2: *mut IUnknown = ptr::null_mut();
                let hr = ((*session_control).lpVtbl.as_ref().unwrap().QueryInterface)(
                    session_control,
                    &IID_IAUDIO_SESSION_CONTROL2,
                    &mut session_control2 as *mut _ as *mut _,
                );
                if hr == S_OK && !session_control2.is_null() {
                    let ctrl2_vtbl = &*((*session_control2).lpVtbl as *const IAudioSessionControl2Vtbl);
                    let mut pid: u32 = 0;
                    if (ctrl2_vtbl.get_process_id)(session_control2, &mut pid) == S_OK && pid != 0 {
                        let is_match = (target_pid != 0 && pid == target_pid) || {
                            let session_exe = get_exe_name_from_pid(pid);
                            !session_exe.is_empty() && session_exe.to_lowercase() == clean_exe
                        };

                        if is_match {
                            let mut simple_volume: *mut IUnknown = ptr::null_mut();
                            let hr = ((*session_control).lpVtbl.as_ref().unwrap().QueryInterface)(
                                session_control,
                                &IID_ISIMPLE_AUDIO_VOLUME,
                                &mut simple_volume as *mut _ as *mut _,
                            );
                            if hr == S_OK && !simple_volume.is_null() {
                                let vol_vtbl = &*((*simple_volume).lpVtbl as *const ISimpleAudioVolumeVtbl);
                                let mut mute_val: BOOL = 0;
                                if (vol_vtbl.get_mute)(simple_volume, &mut mute_val) == S_OK {
                                    if mute_val != 0 {
                                        is_muted = true;
                                        ((*simple_volume).lpVtbl.as_ref().unwrap().Release)(simple_volume);
                                        ((*session_control2).lpVtbl.as_ref().unwrap().Release)(session_control2);
                                        ((*session_control).lpVtbl.as_ref().unwrap().Release)(session_control);
                                        ((*session_enum).lpVtbl.as_ref().unwrap().Release)(session_enum);
                                        break 'outer;
                                    }
                                }
                                ((*simple_volume).lpVtbl.as_ref().unwrap().Release)(simple_volume);
                            }
                        }
                    }
                    ((*session_control2).lpVtbl.as_ref().unwrap().Release)(session_control2);
                }
                ((*session_control).lpVtbl.as_ref().unwrap().Release)(session_control);
            }

            ((*session_enum).lpVtbl.as_ref().unwrap().Release)(session_enum);
        }

        ((*device_collection).lpVtbl.as_ref().unwrap().Release)(device_collection);
        CoUninitialize();
    }

    is_muted
}

#[cfg(windows)]
fn get_exe_name_from_pid(pid: u32) -> String {
    use winapi::um::processthreadsapi::OpenProcess;
    use winapi::um::psapi::GetProcessImageFileNameW;
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::winnt::PROCESS_QUERY_LIMITED_INFORMATION;

    if pid == 0 { return String::new(); }
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() { return String::new(); }

        let mut buf = vec![0u16; 512];
        let len = GetProcessImageFileNameW(handle, buf.as_mut_ptr(), 512);
        CloseHandle(handle);

        if len > 0 {
            let path = String::from_utf16_lossy(&buf[..len as usize]);
            std::path::Path::new(&path)
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_default()
        } else {
            String::new()
        }
    }
}

#[cfg(not(windows))]
pub fn set_process_audio_mute(_exe_name: &str, _target_pid: u32, _mute: bool) -> Result<(), String> { Ok(()) }

#[cfg(not(windows))]
pub fn get_process_audio_mute(_exe_name: &str, _target_pid: u32) -> bool { false }
