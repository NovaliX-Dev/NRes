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
    let iter = dd.iter().filter_map(|(i, dd)| {
        let str = std::str::from_utf8(&dd.DeviceName)
            .unwrap()
            .trim_end_matches('\0');

        // there is an empty str so this check removes it
        if str.is_empty() {
            None
        } else {
            let pcstr = str.to_owned() + "\0";
            Some((*i, PCSTR::from_raw(pcstr.as_ptr())))
        }
    });

    ahash::AHashMap::from_iter(iter)
}
