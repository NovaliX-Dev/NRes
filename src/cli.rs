use std::ffi::OsStr;

use ahash::AHashMap;
use clap::{
    builder::{TypedValueParser, ValueParserFactory},
    Parser,
};

use crate::device_utils::NewDisplayConfig;

mod utils {
    use std::fmt::Display;

    pub fn split_at_slip_mid_char(str: &str, mid: usize) -> Option<(&str, &str)> {
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

    pub fn create_error(
        cmd: &clap::Command,
        kind: clap::error::ErrorKind,
        message: impl Display,
    ) -> clap::Error {
        cmd.to_owned().error(kind, message)
    }
}

#[derive(Parser)]
struct CliRaw {
    refresh_rate: Vec<DisplaySettings>,
}

#[derive(Clone)]
struct DisplaySettings(u32, NewDisplayConfig);

impl ValueParserFactory for DisplaySettings {
    type Parser = RefreshRateParser;

    fn value_parser() -> Self::Parser {
        RefreshRateParser
    }
}

#[derive(derive_more::Display)]
enum DisplaySettingsParser {
    #[display(fmt = "Invalid value `{}` : Couldn't find any `:` in the input.", _0)]
    NoSemicolon(String),

    #[display(fmt = "Invalid value `{}` : No value after the `:`.", _0)]
    NoValueAfterSemicolon(String),

    #[display(fmt = "Invalid value `{}` : No value before the `:`.", _0)]
    NoValueBeforeSemicolon(String),

    #[display(
        fmt = "Invalid value `{}` : Found another `:` after the first `:`.",
        _0
    )]
    TooMuchSemicolon(String),
}

#[derive(Clone)]
struct RefreshRateParser;

impl TypedValueParser for RefreshRateParser {
    type Value = DisplaySettings;

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

        let equal_char_usize = memchr::memchr(b':', value.as_bytes());
        if equal_char_usize.is_none() {
            return Err(utils::create_error(
                cmd,
                clap::error::ErrorKind::InvalidValue,
                DisplaySettingsParser::NoSemicolon(value.to_owned()),
            ));
        }

        let str_pair = utils::split_at_slip_mid_char(value, equal_char_usize.unwrap());
        assert!(str_pair.is_some());

        let (display_index_str, refresh_rate_str) = str_pair.unwrap();

        if refresh_rate_str.is_empty() {
            return Err(utils::create_error(
                cmd,
                clap::error::ErrorKind::InvalidValue,
                DisplaySettingsParser::NoValueAfterSemicolon(value.to_owned()),
            ));
        }
        if display_index_str.is_empty() {
            return Err(utils::create_error(
                cmd,
                clap::error::ErrorKind::InvalidValue,
                DisplaySettingsParser::NoValueBeforeSemicolon(value.to_owned()),
            ));
        }
        if memchr::memchr(b':', refresh_rate_str.as_bytes()).is_some() {
            return Err(utils::create_error(
                cmd,
                clap::error::ErrorKind::InvalidValue,
                DisplaySettingsParser::TooMuchSemicolon(value.to_owned()),
            ));
        }

        let u32_parser = clap::value_parser!(u32);

        let display_index = u32_parser.parse_ref(cmd, arg, OsStr::new(display_index_str))?;
        let refresh_rate = u32_parser.parse_ref(cmd, arg, OsStr::new(refresh_rate_str))?;

        Ok(DisplaySettings(
            display_index,
            NewDisplayConfig {
                display_frequency: refresh_rate,
            },
        ))
    }
}

pub(crate) struct Cli {
    pub display_settings: AHashMap<u32, NewDisplayConfig>,
}

pub(crate) fn parse_cli() -> Option<Cli> {
    let cli_raw = CliRaw::parse();

    let mut validated = true;

    let mut refresh_rate_hash = AHashMap::new();
    for refresh_rate in cli_raw.refresh_rate {
        let DisplaySettings(id, infos) = refresh_rate;

        if refresh_rate_hash.contains_key(&id) {
            println!("Error : Found settings for display {}, but there is already a settings assignment for that display.", id);

            validated = false;
            continue;
        }

        refresh_rate_hash.insert(id, infos);
    }

    if validated {
        Some(Cli {
            display_settings: refresh_rate_hash,
        })
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
}
