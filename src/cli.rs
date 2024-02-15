use std::ffi::OsString;
use clap::Parser;

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
    pub(super) enum ErrorKind {
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

    #[derive(Clone, Debug)]
    pub(super) struct DisplayArgs(pub u32, pub NewDisplayConfig);

    impl ValueParserFactory for DisplayArgs {
        type Parser = Parser;

        fn value_parser() -> Self::Parser {
            Parser
        }
    }

    #[derive(Clone)]
    pub(super) struct Parser;

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

#[derive(Parser, Debug)]
enum CliRaw {
    /// Change the current refresh rate settings.
    Set(set_cli::SetCliRaw),

    /// List all the display monitors with their index and current refresh rate.
    List,
}

#[derive(Debug)]
pub(crate) enum Cli {
    Set(set_cli::SetCli),
    List,
}

pub(crate) mod set_cli {
    use super::{display_args, Cli};

    use crate::device_utils::NewDisplayConfig;

    use ahash::AHashMap;
    use clap::Args;

    #[derive(Args, Debug)]
    pub(super) struct SetCliRaw {
        /// Specify the new refresh rates for the specifics monitors. The syntax is `<display_index>:<refresh_rate>`.
        #[arg(required = true)]
        pub(super) refresh_rate: Vec<display_args::DisplayArgs>,
    }

    #[derive(Debug)]
    pub(crate) struct SetCli {
        pub display_settings: AHashMap<u32, NewDisplayConfig>,
    }

    pub(super) fn validate_set_cli(cli_raw: SetCliRaw) -> Result<Cli, Vec<display_args::DisplayArgs>> {
        let mut errors = Vec::with_capacity(cli_raw.refresh_rate.len());
        let mut refresh_rate_hash = AHashMap::new();
    
        for refresh_rate in cli_raw.refresh_rate {
            if refresh_rate_hash.contains_key(&refresh_rate.0) {
                errors.push(refresh_rate);
                continue;
            }
            
            let display_args::DisplayArgs(id, infos) = refresh_rate;
            refresh_rate_hash.insert(id, infos);
        }
    
        if errors.is_empty() {
            Ok(Cli::Set(SetCli {
                display_settings: refresh_rate_hash,
            }))
        } else {
            Err(errors)
        }
    }
}

#[derive(derive_more::From, Debug)]
enum ParseError {
    Clap(clap::Error),
    SetCliValidationError(Vec<display_args::DisplayArgs>)
}

fn try_parse_cli_from<T, I>(iter: I) -> Result<Cli, ParseError>
where
    T: Into<OsString> + Clone,
    I: IntoIterator<Item = T>
{
    let cli_raw = CliRaw::try_parse_from(iter)?;

    match cli_raw {
        CliRaw::Set(set_cli) => Ok(set_cli::validate_set_cli(set_cli)?),
        CliRaw::List => Ok(Cli::List),
    }
}

pub(crate) fn parse_cli() -> Option<Cli> {
    match try_parse_cli_from(std::env::args_os()) {
        Ok(cli) => Some(cli),
        Err(ParseError::Clap(err)) => err.exit(),
        Err(ParseError::SetCliValidationError(duplicates)) => {
            for duplicate in duplicates {
                println!("Error : Found settings for display {}, but there is already a settings assignment for that display.", duplicate.0);
            }

            None
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::{CommandFactory, Parser};

    use crate::cli::{display_args, utils};

    use super::{try_parse_cli_from, CliRaw, ParseError};

    #[test]
    fn debug_assert() {
        use clap::CommandFactory;
        super::CliRaw::command().debug_assert()
    }

    // that's clearly not a good way to get the error message but i didn't find another
    // way to get it and that seems to work.
    fn ger_error_msg(err: &clap::Error) -> String {
        let string = err.to_string();

        string.split("\n").next().unwrap().to_owned()
    }

    macro_rules! assert_errors_eq {
        ($err: ident, $expected_err: ident) => {
            assert_eq!($err.kind(), $expected_err.kind());
            assert_eq!(ger_error_msg(&$err), ger_error_msg(&$expected_err));
        };

        ($err: ident ; $($tt: tt)*) => {
            let expected_err = create_error!($($tt)*);

            assert_errors_eq!($err, expected_err);
        }
    }

    macro_rules! create_error {
        ([invalid_value] $value: literal, $kind: expr) => {
            create_invalid_error_test!($value, $kind)
        };
    }

    macro_rules! create_invalid_error_test {
        ($value: literal, $kind: expr) => {
            utils::create_invalid_error(&CliRaw::command(), $value, $kind)
        };
    }

    // that's based on `get_error_msg` which is not the best way to get error msg.
    macro_rules! assert_parse_result_eq {
        ([err] $value_to_parse: expr ; $($tt: tt)*) => {{
            let r = CliRaw::try_parse_from($value_to_parse);
            assert!(r.is_err());

            let err = r.unwrap_err();        
            assert_errors_eq!(err ; $($tt)*);
        }};

        ([ok] $value_to_parse: expr) => {{
            let r = CliRaw::try_parse_from($value_to_parse);
            assert!(r.is_ok());
        }}
    }

    // TODO : add tests
    #[test]
    fn test_raw_parse_err() {
        assert_parse_result_eq!([err] ["a", "set", ":"] ; [invalid_value] ":", display_args::ErrorKind::NoValueBeforeSemicolon);      
        assert_parse_result_eq!([err] ["a", "set", "1:"] ; [invalid_value] "1:", display_args::ErrorKind::NoValueAfterSemicolon);      
        assert_parse_result_eq!([err] ["a", "set", "1"] ; [invalid_value] "1", display_args::ErrorKind::NoSemicolon);     
        assert_parse_result_eq!([err] ["a", "set", "1::"] ; [invalid_value] "1::", display_args::ErrorKind::TooMuchSemicolon);     
        
        assert_parse_result_eq!([err] ["a", "set", "1:0"] ; [invalid_value] "1:0", display_args::ErrorKind::RefreshRateNull);     
    }

    #[test]
    fn test_raw_parse_ok() {
        assert_parse_result_eq!([ok] ["a", "set", "1:300"]);
    }

    macro_rules! assert_let_pattern {
        (let $pattern: pat = $right: expr) => {
            assert_let_pattern!( => let $pattern = $right)
        };

        ($($inner_ident: ident),* => let $pattern: pat = $right: expr) => {
            match $right {
                $pattern => ($($inner_ident),*),
                _ => panic!(
                    "let pattern assert failed\n  right : {:?}\n{}", 
                    $right, 
                    concat!("   left : ", stringify!($pattern))
                )
            }
        };
    }

    #[test]
    fn test_set_cli_validation() {
        let r = try_parse_cli_from(["a", "set", "1:300", "1:300"]);

        let vec = assert_let_pattern!(vec => let Err(ParseError::SetCliValidationError(vec)) = r);
        assert_eq!(vec[0].0, 1);
        assert_eq!(vec[0].1.refresh_rate, 300);
    }
}
