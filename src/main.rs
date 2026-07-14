use std::ffi::OsString;
use std::process::ExitCode as ProcessExitCode;

use cage::cli::Cli;
use cage::cli::error::ExitCode;
use cage::cli::output::Output;
use clap::error::ErrorKind;
use clap::{ColorChoice, CommandFactory as _, FromArgMatches as _};

fn main() -> ProcessExitCode {
    let arguments = std::env::args_os().collect::<Vec<_>>();
    let no_color = std::env::var_os("NO_COLOR").is_some() || requests_no_color(&arguments);
    let cli = match parse_cli(arguments, no_color) {
        Ok(cli) => cli,
        Err(error) => {
            let exit_code = if matches!(
                error.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            ) {
                ExitCode::Success
            } else {
                ExitCode::General
            };
            let _ = error.print();
            return process_exit(exit_code);
        }
    };

    match cage::cli::execute(&cli) {
        Ok(()) => process_exit(ExitCode::Success),
        Err(error) => {
            let _write_result = Output::for_stderr(cli.no_color).error(&error.to_string());
            process_exit(error.exit_code())
        }
    }
}

fn parse_cli(arguments: Vec<OsString>, no_color: bool) -> Result<Cli, clap::Error> {
    let matches = cli_command(no_color).try_get_matches_from(arguments)?;
    Cli::from_arg_matches(&matches)
}

fn cli_command(no_color: bool) -> clap::Command {
    let command = Cli::command();
    if no_color {
        command.color(ColorChoice::Never)
    } else {
        command
    }
}

fn requests_no_color(arguments: &[OsString]) -> bool {
    arguments
        .iter()
        .skip(1)
        .take_while(|argument| argument.as_os_str() != "--")
        .any(|argument| argument == "--no-color")
}

fn process_exit(exit_code: ExitCode) -> ProcessExitCode {
    u8::try_from(exit_code.as_i32()).map_or(ProcessExitCode::FAILURE, ProcessExitCode::from)
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use clap::ColorChoice;

    use super::{cli_command, requests_no_color};

    fn arguments(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn no_color_is_detected_before_agent_passthrough() {
        assert!(requests_no_color(&arguments(&[
            "cage",
            "run",
            "--no-color",
            "claude",
        ])));
    }

    #[test]
    fn agent_no_color_argument_is_not_treated_as_a_cage_flag() {
        assert!(!requests_no_color(&arguments(&[
            "cage",
            "run",
            "claude",
            "--",
            "--no-color",
        ])));
    }

    #[test]
    fn no_color_configures_clap_before_rendering() {
        assert_eq!(cli_command(true).get_color(), ColorChoice::Never);
    }
}
