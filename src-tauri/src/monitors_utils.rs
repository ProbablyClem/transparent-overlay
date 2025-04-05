use crate::config::MonitorPos;

// Source : https://gist.github.com/duckfromdiscord/3e66912adecd8702811dfba2de4bc2b9
use windows::Win32::{
    Devices::Display::{
        DisplayConfigGetDeviceInfo, GetDisplayConfigBufferSizes, QueryDisplayConfig,
        DISPLAYCONFIG_ADAPTER_NAME, DISPLAYCONFIG_DEVICE_INFO_GET_ADAPTER_NAME,
        DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME, DISPLAYCONFIG_MODE_INFO,
        DISPLAYCONFIG_MODE_INFO_TYPE, DISPLAYCONFIG_PATH_INFO, DISPLAYCONFIG_TARGET_DEVICE_NAME,
        QDC_ONLY_ACTIVE_PATHS, QDC_VIRTUAL_MODE_AWARE,
    },
    Foundation::{ERROR_INSUFFICIENT_BUFFER, ERROR_SUCCESS, WIN32_ERROR},
};

pub fn get_monitor_infos() -> Option<Vec<MonitorPos>> {
    unsafe {
        let mut monitors = vec![];

        let mut paths: Vec<DISPLAYCONFIG_PATH_INFO> = vec![];
        let mut modes: Vec<DISPLAYCONFIG_MODE_INFO> = vec![];
        let flags = QDC_ONLY_ACTIVE_PATHS | QDC_VIRTUAL_MODE_AWARE;
        let mut result;

        loop {
            let mut path_count: u32 = 0;
            let mut mode_count: u32 = 0;

            result = GetDisplayConfigBufferSizes(flags, &mut path_count, &mut mode_count);
            if result != ERROR_SUCCESS {
                return None;
            }

            paths.resize(
                path_count.try_into().unwrap(),
                DISPLAYCONFIG_PATH_INFO::default(),
            );
            modes.resize(
                mode_count.try_into().unwrap(),
                DISPLAYCONFIG_MODE_INFO::default(),
            );

            result = QueryDisplayConfig(
                flags,
                &mut path_count,
                paths.as_mut_ptr(),
                &mut mode_count,
                modes.as_mut_ptr(),
                None,
            );

            paths.resize(
                path_count.try_into().unwrap(),
                DISPLAYCONFIG_PATH_INFO::default(),
            );
            modes.resize(
                mode_count.try_into().unwrap(),
                DISPLAYCONFIG_MODE_INFO::default(),
            );

            if result != ERROR_INSUFFICIENT_BUFFER {
                break;
            }
        }

        if result != ERROR_SUCCESS {
            return None;
        }

        for path in paths {
            let mut target_name = DISPLAYCONFIG_TARGET_DEVICE_NAME::default();
            target_name.header.adapterId = path.targetInfo.adapterId;
            target_name.header.id = path.targetInfo.id;
            target_name.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME;
            target_name.header.size =
                core::mem::size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() as u32;
            result = WIN32_ERROR(
                DisplayConfigGetDeviceInfo(&mut target_name.header)
                    .try_into()
                    .unwrap(),
            );
            if result.is_err() {
                return None;
            }

            let mut adapter_name = DISPLAYCONFIG_ADAPTER_NAME::default();
            adapter_name.header.adapterId = path.targetInfo.adapterId;
            adapter_name.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_ADAPTER_NAME;
            adapter_name.header.size = core::mem::size_of::<DISPLAYCONFIG_ADAPTER_NAME>() as u32;
            result = WIN32_ERROR(
                DisplayConfigGetDeviceInfo(&mut adapter_name.header)
                    .try_into()
                    .unwrap(),
            );

            if result.is_err() {
                return None;
            }

            let dstr = widestring::U16String::from_vec(target_name.monitorFriendlyDeviceName)
                .to_string_lossy();

            if let Some(source_mode) = modes.iter().find(|mode| {
                mode.infoType == DISPLAYCONFIG_MODE_INFO_TYPE(1i32)
                    && mode.adapterId == path.sourceInfo.adapterId
                    && mode.id == path.sourceInfo.id
            }) {
                let position = source_mode.Anonymous.sourceMode.position;
                monitors.push(MonitorPos {
                    name: dstr.trim_end_matches("\0").to_string(),
                    pos_x: position.x,
                });
            }
        }

        Some(monitors)
    }
}

pub fn get_primary_monitor(monitor_vec: Vec<MonitorPos>) -> MonitorPos {
    monitor_vec
        .iter()
        .find(|m| m.pos_x == 0)
        .cloned()
        .unwrap_or_else(|| monitor_vec[0].clone())
}
