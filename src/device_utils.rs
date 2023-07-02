use windows::{
    core::PCSTR,
    Win32::{
        Foundation::HWND,
        Graphics::Gdi::{
            ChangeDisplaySettingsExA, EnumDisplaySettingsA, CDS_TYPE, DEVMODEA,
            ENUM_CURRENT_SETTINGS,
        },
    },
};

use crate::display_change;

#[derive(Clone)]
pub(crate) struct NewDisplayConfig {
    pub(crate) display_frequency: u32,
}

pub(crate) fn change_display_settings(
    device_name: PCSTR,
    new_config: NewDisplayConfig,
) -> Result<display_change::DisplayChangeOk, display_change::DisplayChangeErr> {
    let mut dm = match get_display_device_settings(&device_name) {
        Some(dm) => dm,
        None => return Err(display_change::DisplayChangeErr::CouldNotGetDisplaySettings),
    };

    dm.dmDisplayFrequency = new_config.display_frequency;

    apply_settings_to_display(device_name, dm)
}

pub(crate) fn apply_settings_to_display(
    device_name: PCSTR,
    dm: DEVMODEA,
) -> Result<display_change::DisplayChangeOk, display_change::DisplayChangeErr> {
    let ret = unsafe {
        ChangeDisplaySettingsExA(
            device_name,
            Some(&dm),
            HWND::default(),
            CDS_TYPE::default(),
            None,
        )
    };

    display_change::disp_change_to_result(ret)
}

pub(crate) fn get_display_device_settings(device_name: &PCSTR) -> Option<DEVMODEA> {
    let mut dm = DEVMODEA::default();
    dm.dmSize = std::mem::size_of_val(&dm).try_into().unwrap();

    let r = unsafe { EnumDisplaySettingsA(*device_name, ENUM_CURRENT_SETTINGS, &mut dm).as_bool() };

    if r {
        Some(dm)
    } else {
        None
    }
}
