use std::process::ExitCode;

use windows::core::PCSTR;

mod cli;
mod device_info;
mod device_utils;
mod display_change;

fn update_device(config: device_utils::NewDisplayConfig, device_id: (u32, PCSTR)) -> bool {
    let r = device_utils::change_display_settings(device_id.1, config);

    match r {
        Ok(state) => {
            match state {
                display_change::DisplayChangeOk::Ok => println!("[{}] {}", device_id.0, state),
                display_change::DisplayChangeOk::NeedRestart => {
                    println!("[{}] {}", device_id.0, state)
                }
            }

            true
        }
        Err(error) => {
            println!("[{}] Change on display failed : {}", device_id.0, error);

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

    let mut validated = true;
    match cli {
        cli::Cli::Set(cli) => {
            let dd_names = device_info::dd_to_u32_pcstr_hashmap(&display_devices);

            for (d_index, display_settings) in cli.display_settings {
                let d_names = dd_names.get(&d_index);
                if d_names.is_none() {
                    println!("Error : Unknown display index.");
                    validated = false;

                    continue;
                }

                let r = update_device(display_settings, (d_index, *d_names.unwrap()));
                if !r {
                    validated = false
                }
            }
        }
        cli::Cli::List => {
            let dd_names_str = device_info::dd_to_str_vec(&display_devices);

            for ((i, _), name_str) in display_devices.iter().zip(dd_names_str) {
                let name = device_info::str_to_pcstr(&name_str);
                let device_info = device_utils::get_display_device_settings(&name);

                if let Err(e) = device_info {
                    println!("[{}] {}", i, e);
                    continue;
                }

                let device_info = device_info.unwrap();
                println!("Display {} : \n- name : {}\n- refresh_rate : {}", i, name_str, device_info.dmDisplayFrequency)
            }
        },
    }

    if validated {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
