use windows::{
    core::PCSTR,
    Win32::{
        Foundation::HWND,
        Graphics::Gdi::{
            ChangeDisplaySettingsExA, EnumDisplayDevicesA, EnumDisplaySettingsA, CDS_TYPE,
            DEVMODEA, DISPLAY_DEVICEA, DISPLAY_DEVICE_ACTIVE, ENUM_CURRENT_SETTINGS,
        },
    },
};

use crate::display_change_res;

#[derive(Clone)]
pub(crate) struct NewDisplayConfig {
    pub(crate) display_frequency: u32,
}

pub(crate) fn change_display_settings(
    device_name: PCSTR,
    new_config: NewDisplayConfig,
) -> Result<display_change_res::DisplayChangeOk, display_change_res::DisplayChangeErr> {
    let mut dm = get_display_device_settings(&device_name)?;

    dm.dmDisplayFrequency = new_config.display_frequency;

    apply_settings_to_display(device_name, dm)
}

pub(crate) fn apply_settings_to_display(
    device_name: PCSTR,
    dm: DEVMODEA,
) -> Result<display_change_res::DisplayChangeOk, display_change_res::DisplayChangeErr> {
    let ret = unsafe {
        ChangeDisplaySettingsExA(
            device_name,
            Some(&dm),
            HWND::default(),
            CDS_TYPE::default(),
            None,
        )
    };

    display_change_res::disp_change_to_result(ret)
}

pub(crate) fn get_display_device_settings(
    device_name: &PCSTR,
) -> Result<DEVMODEA, display_change_res::DisplayChangeErr> {
    let mut dm = DEVMODEA::default();
    dm.dmSize = std::mem::size_of_val(&dm).try_into().unwrap();

    let r = unsafe { EnumDisplaySettingsA(*device_name, ENUM_CURRENT_SETTINGS, &mut dm).as_bool() };

    if r {
        Ok(dm)
    } else {
        Err(display_change_res::DisplayChangeErr::CouldNotGetDisplaySettings)
    }
}

pub(crate) fn get_active_display_devices() -> Vec<(u32, DISPLAY_DEVICEA)> {
    let mut dd_list = Vec::new();

    let mut dd = DISPLAY_DEVICEA::default();
    dd.cb = std::mem::size_of_val(&dd).try_into().unwrap();

    let mut i = 0;
    let mut r = true;
    while r {
        i += 1;

        r = unsafe { EnumDisplayDevicesA(PCSTR::null(), i - 1, &mut dd, 0).as_bool() };
        if !r {
            break;
        }

        // Windows seems to keep track of all the displays ever connected, so we need to filter those currently connected
        // TODO: maybe add a trait to check flags ?
        if dd.StateFlags & DISPLAY_DEVICE_ACTIVE != DISPLAY_DEVICE_ACTIVE {
            continue;
        }

        // The devices seems to always be sorted to we can just set i as index
        dd_list.push((i, dd));
    }

    dd_list
}
