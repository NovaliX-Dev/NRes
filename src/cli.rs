use ahash::AHashMap;
use clap::{Args, Parser};

use crate::device_utils::NewDisplayConfig;

mod utils {
    use std::fmt::Display;

    pub fn split_at_skip_mid_char(str: &str, mid: usize) -> Option<(&str, &str)> {
        // is_char_boundary checks that the index is in [0, .len()]
        if str.is_char_boundary(mid) && str.is_char_boundary(mid + 1) {
            // SAFETY: just checked that `mid` is on a char boundary.
            Some(unsafe {
                (
                    str.get_unchecked(0..mid),
                    str.get_unchecked(mid + 1..str.len()),
                )
            })
        } else {
            None
        }
    }

    pub fn create_error<D: Display>(
        cmd: &clap::Command,
        kind: clap::error::ErrorKind,
        message: D,
    ) -> clap::Error {
        cmd.to_owned().error(kind, message)
    }

    pub fn create_invalid_error<R: Display, V: Display>(
        cmd: &clap::Command,
        value: V,
        reason: R,
    ) -> clap::Error {
        let full_message = format!("Invalid value `{value}` : {reason}");

        create_error(cmd, clap::error::ErrorKind::InvalidValue, full_message)
    }
}

mod display_args {
    use std::ffi::OsStr;

    use clap::builder::{TypedValueParser, ValueParserFactory};

    use super::utils::{self, create_invalid_error};
    use crate::device_utils::NewDisplayConfig;

    #[derive(derive_more::Display)]
    pub(crate) enum ErrorKind {
        #[display(fmt = "Couldn't find any `:` in the input.")]
        NoSemicolon,

        #[display(fmt = "No refresh rate after the `:`.")]
        NoValueAfterSemicolon,

        #[display(fmt = "No display index before the `:`. See subcommands `list` for details.")]
        NoValueBeforeSemicolon,

        #[display(fmt = "Found another `:` after the first `:`")]
        TooMuchSemicolon,

        #[display(fmt = "The refresh rate given is null.")]
        RefreshRateNull,
    }

    #[derive(Clone)]
    pub struct DisplayArgs(pub u32, pub NewDisplayConfig);

    impl ValueParserFactory for DisplayArgs {
        type Parser = Parser;

        fn value_parser() -> Self::Parser {
            Parser
        }
    }

    #[derive(Clone)]
    pub struct Parser;

    impl TypedValueParser for Parser {
        type Value = DisplayArgs;

        fn parse_ref(
            &self,
            cmd: &clap::Command,
            arg: Option<&clap::Arg>,
            value: &std::ffi::OsStr,
        ) -> Result<Self::Value, clap::Error> {
            let value = match value.to_str() {
                Some(v) => v,
                None => return Err(clap::Error::new(clap::error::ErrorKind::InvalidUtf8)),
            };

            let mut equal_char_iter = value.bytes()
                .enumerate()
                .filter(|(_, b)| b == &b':')
                .map(|(i, _)| i);

            let first = match equal_char_iter.next() {
                Some(first) => first,
                None => {
                    return Err(utils::create_invalid_error(
                        cmd,
                        value,
                        ErrorKind::NoSemicolon,
                    ));
                }
            };
            if equal_char_iter.next().is_some() {
                return Err(utils::create_invalid_error(
                    cmd,
                    value,
                    ErrorKind::TooMuchSemicolon,
                ));
            }

            let str_pair = utils::split_at_skip_mid_char(value, first);
            debug_assert!(str_pair.is_some());

            let (display_index_str, refresh_rate_str) = str_pair.unwrap();

            if display_index_str.is_empty() {
                return Err(create_invalid_error(
                    cmd,
                    value,
                    ErrorKind::NoValueBeforeSemicolon,
                ));
            }
            if refresh_rate_str.is_empty() {
                return Err(create_invalid_error(
                    cmd,
                    value,
                    ErrorKind::NoValueAfterSemicolon,
                ));
            }

            let u32_parser = clap::value_parser!(u32);

            let display_index = u32_parser.parse_ref(cmd, arg, OsStr::new(display_index_str))?;
            let refresh_rate = u32_parser.parse_ref(cmd, arg, OsStr::new(refresh_rate_str))?;

            if refresh_rate == 0 {
                return Err(create_invalid_error(cmd, value, ErrorKind::RefreshRateNull));
            }

            Ok(DisplayArgs(
                display_index,
                NewDisplayConfig { refresh_rate },
            ))
        }
    }
}

#[derive(Parser)]
enum CliRaw {
    /// Change the current refresh rate settings.
    Set(SetCliRaw),

    /// List all the display monitors with their index and current refresh rate.
    List,
}

#[derive(Args)]
struct SetCliRaw {
    /// Specify the new refresh rates for the specifics monitors. The syntax is `<display_index>:<refresh_rate>`.
    #[arg(required = true)]
    refresh_rate: Vec<display_args::DisplayArgs>,
}

pub(crate) enum Cli {
    Set(SetCli),
    List,
}

pub(crate) struct SetCli {
    pub display_settings: AHashMap<u32, NewDisplayConfig>,
}

pub(crate) fn parse_cli() -> Option<Cli> {
    let cli_raw = CliRaw::parse();

    match cli_raw {
        CliRaw::Set(cli_raw) => validate_set_cli(cli_raw),
        CliRaw::List => Some(Cli::List),
    }
}

fn validate_set_cli(cli_raw: SetCliRaw) -> Option<Cli> {
    let mut validated = true;
    let mut refresh_rate_hash = AHashMap::new();
    for refresh_rate in cli_raw.refresh_rate {
        let display_args::DisplayArgs(id, infos) = refresh_rate;

        if refresh_rate_hash.contains_key(&id) {
            println!("Error : Found settings for display {}, but there is already a settings assignment for that display.", id);

            validated = false;
            continue;
        }

        refresh_rate_hash.insert(id, infos);
    }
    if validated {
        Some(Cli::Set(SetCli {
            display_settings: refresh_rate_hash,
        }))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn debug_assert() {
        use clap::CommandFactory;
        super::CliRaw::command().debug_assert()
    }

    // TODO : add tests
}
