use std::process::ExitCode;

use windows::core::PCSTR;

mod cli;
mod device_info;
mod device_utils;
mod display_change;

fn update_device(config: device_utils::NewDisplayConfig, device_name: PCSTR) -> bool {
    let r = device_utils::change_display_settings(device_name, config);

    match r {
        Ok(state) => {
            match state {
                display_change::DisplayChangeOk::Ok => println!("{}", state),
                display_change::DisplayChangeOk::NeedRestart => println!("{}", state),
            }

            true
        }
        Err(error) => {
            println!("Change on display failed : {}", error);

            false
        }
    }
}

fn main() -> ExitCode {
    let cli = cli::parse_cli();
    if cli.is_none() {
        return ExitCode::FAILURE;
    }

    let cli = cli.unwrap();

    let display_devices = device_info::get_active_display_devices();
    let dd_names = device_info::dd_to_u32_pcstr_hashmap(&display_devices);

    // for the moment we don't need display_devices. Maybe in the future ?
    drop(display_devices);

    let mut validated = true;

    for (display_index, display_settings) in cli.display_settings {
        let display_name = dd_names.get(&display_index);
        if display_name.is_none() {
            println!("Error : Unknown display index.");
            validated = false;

            continue;
        }

        let r = update_device(display_settings, *display_name.unwrap());
        if !r {
            validated = false
        }
    }

    if validated {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
