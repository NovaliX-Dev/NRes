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

mod display_change;

struct NewDisplayConfig {
    display_frequency: u32,
}

fn change_display_settings(
    device_name: PCSTR,
    new_config: NewDisplayConfig,
) -> Result<display_change::DisplayChangeOk, display_change::DisplayChangeErr> {
    let mut dm = DEVMODEA::default();
    dm.dmSize = std::mem::size_of_val(&dm).try_into().unwrap();

    let r = unsafe { EnumDisplaySettingsA(device_name, ENUM_CURRENT_SETTINGS, &mut dm).as_bool() };

    if r {
        dm.dmDisplayFrequency = new_config.display_frequency
    } else {
        return Err(display_change::DisplayChangeErr::CouldNotGetDisplaySettings);
    }

    let ret = unsafe {
        ChangeDisplaySettingsExA(
            device_name,
            Some(&dm),
            HWND::default(),
            CDS_TYPE::default(),
            None,
        )
    };

    display_change::disp_change_to_enum(ret)
}

fn main() {
    let config = NewDisplayConfig {
        display_frequency: 300,
    };
    let r = change_display_settings(PCSTR::null(), config);

    match r {
        Ok(state) => match state {
            display_change::DisplayChangeOk::Ok => println!("{}", state),
            display_change::DisplayChangeOk::NeedRestart => println!("{}", state),
        },
        Err(error) => println!("Change on display failed : {}", error),
    }
}
