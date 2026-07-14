use std::process::ExitCode as ProcessExitCode;

use cage::cli::Cli;
use cage::cli::error::ExitCode;
use cage::cli::output::Output;
use clap::Parser as _;
use clap::error::ErrorKind;

fn main() -> ProcessExitCode {
    let cli = match Cli::try_parse() {
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

fn process_exit(exit_code: ExitCode) -> ProcessExitCode {
    u8::try_from(exit_code.as_i32()).map_or(ProcessExitCode::FAILURE, ProcessExitCode::from)
}
