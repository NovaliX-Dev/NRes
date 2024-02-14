use std::process::ExitCode;

use windows::core::PCSTR;

mod cli;
mod device_name_utils;
mod device_utils;
mod display_change_res;

macro_rules! print_display_msg {
    ($index: expr, $msg: literal) => {
        println!("[{}] {}", $index, $msg)
    };
    ($index: expr, $format_str: literal, $($format_args: expr),*) => {
        println!("[{}] {}", $index, format!($format_str, $($format_args),*))
    };
}

fn update_device(config: device_utils::NewDisplayConfig, device_id: (u32, PCSTR)) -> bool {
    let r = device_utils::change_display_settings(device_id.1, config);

    match r {
        Ok(state) => {
            match state {
                display_change_res::DisplayChangeOk::Ok => println!("[{}] {}", device_id.0, state),
                display_change_res::DisplayChangeOk::NeedRestart => {
                    print_display_msg!(device_id.0, "{}", state)
                }
            }

            true
        }
        Err(error) => {
            print_display_msg!(device_id.0, "Change on display failed : {}", error);

            false
        }
    }
}

fn main_set(
    display_devices: &[(u32, windows::Win32::Graphics::Gdi::DISPLAY_DEVICEA)],
    cli: cli::SetCli,
    validated: &mut bool,
) {
    let dd_names = device_name_utils::dd_to_u32_pcstr_hashmap(display_devices);
    for (d_index, display_settings) in cli.display_settings {
        let d_names = dd_names.get(&d_index);
        if d_names.is_none() {
            print_display_msg!(d_index, "Error: Unknown display index.");
            *validated = false;

            continue;
        }

        let r = update_device(display_settings, (d_index, *d_names.unwrap()));
        if !r {
            *validated = false
        }
    }
}

fn main_list(display_devices: Vec<(u32, windows::Win32::Graphics::Gdi::DISPLAY_DEVICEA)>) {
    let dd_names_str = device_name_utils::dd_to_str_vec(&display_devices);
    for ((i, _), name_str) in display_devices.iter().zip(dd_names_str) {
        let name = device_name_utils::str_to_pcstr(&name_str);
        let device_info = device_utils::get_display_device_settings(&name);

        if let Err(e) = device_info {
            print_display_msg!(i, "{}", e);
            continue;
        }

        let device_info = device_info.unwrap();
        println!(
            "Display {} : \n- name : {}\n- refresh_rate : {}",
            i, name_str, device_info.dmDisplayFrequency
        )
    }
}

fn main() -> ExitCode {
    let cli = cli::parse_cli();
    if cli.is_none() {
        return ExitCode::FAILURE;
    }

    let cli = cli.unwrap();

    let display_devices = device_utils::get_active_display_devices();

    let mut validated = true;
    match cli {
        cli::Cli::Set(cli) => main_set(&display_devices, cli, &mut validated),
        cli::Cli::List => main_list(display_devices),
    }

    if validated {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
