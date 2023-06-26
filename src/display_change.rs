use windows::Win32::Graphics::Gdi::{
    DISP_CHANGE, DISP_CHANGE_BADDUALVIEW, DISP_CHANGE_BADFLAGS, DISP_CHANGE_BADMODE,
    DISP_CHANGE_BADPARAM, DISP_CHANGE_FAILED, DISP_CHANGE_NOTUPDATED, DISP_CHANGE_RESTART,
    DISP_CHANGE_SUCCESSFUL,
};

#[derive(derive_more::Display)]
pub(crate) enum DisplayChangeOk {
    #[display(fmt = "Display successfully changed.")]
    Ok,

    #[display(fmt = "The computer must be restarted for the graphics mode to work.")]
    NeedRestart,
}

#[derive(derive_more::Display)]
pub(crate) enum DisplayChangeErr {
    #[display(fmt = "{}", _0)]
    Err(&'static str),

    #[display(fmt = "{}. This is likely to be a bug !", _0)]
    Bug(&'static str),

    #[display(fmt = "Couldn't get display settings.")]
    CouldNotGetDisplaySettings,
}

pub(crate) fn disp_change_to_enum(
    r: DISP_CHANGE,
) -> Result<DisplayChangeOk, DisplayChangeErr> {
    match r {
        DISP_CHANGE_SUCCESSFUL => {
            Ok(DisplayChangeOk::Ok)
        }
        DISP_CHANGE_RESTART => {
            Ok(DisplayChangeOk::NeedRestart)
        }

        DISP_CHANGE_BADDUALVIEW => {
            Err(DisplayChangeErr::Err("The settings change was unsuccessful because the system is DualView capable."))
        }
        DISP_CHANGE_BADMODE => {
            Err(DisplayChangeErr::Err("The config requested is not supported."))
        }
        DISP_CHANGE_FAILED => {
            Err(DisplayChangeErr::Err("The display driver failed to apply the specified graphics mode."))
        }
        DISP_CHANGE_NOTUPDATED => {
            Err(DisplayChangeErr::Err("Unable to write settings to the registry."))
        }

        DISP_CHANGE_BADPARAM => {
            Err(DisplayChangeErr::Bug("An invalid parameter was passed in. This can include an invalid flag or combination of flags."))
        }
        DISP_CHANGE_BADFLAGS => {
            Err(DisplayChangeErr::Bug("An invalid set of flags was passed in."))
        }

        _ => unreachable!(),
    }
}
