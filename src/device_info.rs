use windows::{
    core::PCSTR,
    Win32::Graphics::Gdi::{EnumDisplayDevicesA, DISPLAY_DEVICEA, DISPLAY_DEVICE_ACTIVE},
};

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

pub(crate) fn dd_to_u32_pcstr_hashmap(
    dd: &[(u32, DISPLAY_DEVICEA)],
) -> ahash::AHashMap<u32, PCSTR> {
    let iter = dd.iter()
        .map(|(i, dd)| (i, dd_to_str(dd)))
        .map(|(i, dd)| (*i, str_to_pcstr(&dd)));

    ahash::AHashMap::from_iter(iter)
}

pub fn str_to_pcstr(str: &str) -> PCSTR {
    let pcstr = str.to_owned() + "\0";
    PCSTR::from_raw(pcstr.as_ptr())
}

pub(crate) fn dd_to_str_vec(dd: &[(u32, DISPLAY_DEVICEA)]) -> Vec<String> {
    dd.iter().map(|(_, dd)| dd_to_str(dd)).collect::<Vec<_>>()
}

pub(crate) fn dd_to_str(dd: &DISPLAY_DEVICEA) -> String {
    let str = std::str::from_utf8(&dd.DeviceName)
        .unwrap()
        .trim_end_matches('\0');

    str.to_owned()
}

