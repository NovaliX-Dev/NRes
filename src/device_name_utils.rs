use windows::{core::PCSTR, Win32::Graphics::Gdi::DISPLAY_DEVICEA};

pub(crate) fn dd_to_u32_pcstr_hashmap(
    dd: &[(u32, DISPLAY_DEVICEA)],
) -> ahash::AHashMap<u32, PCSTR> {
    let iter = dd
        .iter()
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
