//! Command-line parsing and dispatch.

pub mod error;
pub mod output;

use std::ffi::OsString;
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use self::error::CageError;

/// A universal sandbox CLI tool for safely running any AI coding agent.
#[derive(Debug, Parser)]
#[command(name = "cage", version, about)]
pub struct Cli {
    /// Show detailed progress and diagnostic output.
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Disable ANSI color output.
    #[arg(long, global = true)]
    pub no_color: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Run an AI agent inside a sandboxed container.
    Run(RunArgs),
    /// Review and apply sandbox changes to the host.
    Sync(SyncArgs),
    /// Display sandbox changes without applying them.
    Diff(DiffArgs),
    /// Manage agent teams.
    Team(TeamArgs),
    /// Manage Cage configuration.
    Config(ConfigArgs),
    /// Manage agent container images.
    Images(ImagesArgs),
    /// Update Cage.
    Update(UpdateArgs),
}

#[derive(Debug, Args)]
#[allow(clippy::struct_excessive_bools)]
pub struct RunArgs {
    /// Agent name or profile. Uses the configured default when omitted.
    pub agent_or_profile: Option<String>,

    /// Project directory.
    #[arg(long, value_name = "PATH", default_value = ".")]
    pub dir: PathBuf,

    /// Project configuration file.
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,

    /// Rebuild the agent image before starting.
    #[arg(long)]
    pub rebuild: bool,

    /// Disable the image build cache. Requires --rebuild when implemented.
    #[arg(long)]
    pub no_cache: bool,

    /// Continue the most recent compatible session.
    #[arg(long = "continue")]
    pub continue_session: bool,

    /// Container memory limit, such as 4g or 512m.
    #[arg(long, value_name = "SIZE")]
    pub memory: Option<String>,

    /// GPU request, such as all or device=0.
    #[arg(long, value_name = "SPEC")]
    pub gpus: Option<String>,

    /// Pin the agent package version.
    #[arg(long, value_name = "VERSION")]
    pub agent_version: Option<String>,

    /// Use Podman instead of Docker.
    #[arg(long)]
    pub podman: bool,

    /// Enable the Docker-in-Docker sidecar.
    #[arg(long)]
    pub dind: bool,

    /// Forward the host SSH agent socket.
    #[arg(long)]
    pub with_ssh: bool,

    /// Copy approved host hooks into the sandbox.
    #[arg(long)]
    pub with_hooks: bool,

    /// Remove the container after the run finishes.
    #[arg(long)]
    pub cleanup: bool,

    /// Show the planned run without creating a container.
    #[arg(long)]
    pub dry_run: bool,

    /// Arguments passed unchanged to the agent after `--`.
    #[arg(last = true, value_name = "AGENT_ARGS", allow_hyphen_values = true)]
    pub agent_args: Vec<OsString>,
}

#[derive(Debug, Args)]
pub struct SyncArgs {
    /// Session identifier.
    #[arg(long, value_name = "ID")]
    pub session: Option<String>,

    /// Container identifier.
    #[arg(long, value_name = "ID")]
    pub container: Option<String>,

    /// Recover changes from a session workspace or volume.
    #[arg(long, value_name = "SOURCE")]
    pub from_volume: Option<String>,

    /// Host project directory.
    #[arg(long, value_name = "PATH", default_value = ".")]
    pub dir: PathBuf,

    /// Limit synchronization to a relative path.
    #[arg(long, value_name = "PATH")]
    pub file: Option<PathBuf>,

    /// Synchronize team output when team support is available.
    #[arg(long)]
    pub team: bool,

    /// Apply all accepted changes without interactive prompts.
    #[arg(long)]
    pub auto: bool,

    /// Proceed after an explicit destructive-operation opt-in.
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct DiffArgs {
    /// Session identifier.
    #[arg(long, value_name = "ID")]
    pub session: Option<String>,

    /// Container identifier.
    #[arg(long, value_name = "ID")]
    pub container: Option<String>,

    /// Recover changes from a session workspace or volume.
    #[arg(long, value_name = "SOURCE")]
    pub from_volume: Option<String>,

    /// Host project directory.
    #[arg(long, value_name = "PATH", default_value = ".")]
    pub dir: PathBuf,

    /// Number of context lines in unified output.
    #[arg(short = 'U', long, value_name = "LINES", default_value_t = 3)]
    pub unified: usize,
}

#[derive(Debug, Args)]
pub struct TeamArgs {
    #[command(subcommand)]
    pub command: TeamCommand,
}

#[derive(Debug, Subcommand)]
pub enum TeamCommand {
    /// Start an agent team.
    Up,
    /// Stop an agent team.
    Down,
    /// Show agent team status.
    Status,
}

#[derive(Debug, Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommand,
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    /// Read a configuration value.
    Get { key: String },
    /// Set a configuration value.
    Set { key: String, value: String },
    /// List effective configuration values.
    List,
    /// Open the configuration file in an editor.
    Edit,
    /// Validate configuration without applying it.
    Validate,
}

#[derive(Debug, Args)]
pub struct ImagesArgs {
    #[command(subcommand)]
    pub command: ImagesCommand,
}

#[derive(Debug, Subcommand)]
pub enum ImagesCommand {
    /// List Cage images.
    List,
    /// Pull an agent image.
    Pull { image: String },
    /// Remove an agent image.
    Remove { image: String },
    /// Remove unused Cage images.
    Prune,
}

#[derive(Debug, Args)]
pub struct UpdateArgs {
    /// Only check whether an update is available.
    #[arg(long)]
    pub check: bool,
}

/// Dispatch a parsed command.
///
/// Feature issues replace these explicit errors as implementations land. Keeping
/// the dispatch here ensures that a parsed command can never become a silent no-op.
pub fn execute(cli: &Cli) -> Result<(), CageError> {
    Err(CageError::not_implemented(command_name(&cli.command)))
}

fn command_name(command: &Command) -> &'static str {
    match command {
        Command::Run(_) => "run",
        Command::Sync(_) => "sync",
        Command::Diff(_) => "diff",
        Command::Team(_) => "team",
        Command::Config(_) => "config",
        Command::Images(_) => "images",
        Command::Update(_) => "update",
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser as _;

    use super::{Cli, Command, execute};
    use crate::cli::error::ExitCode;

    #[test]
    fn agent_arguments_are_only_accepted_after_separator() {
        let parsed = Cli::try_parse_from(["cage", "run", "claude", "--", "--model", "opus"]);
        assert!(parsed.is_ok());
        let Ok(cli) = parsed else {
            return;
        };
        assert!(matches!(&cli.command, Command::Run(_)));
        let Command::Run(args) = cli.command else {
            return;
        };

        assert_eq!(args.agent_args, ["--model", "opus"]);
    }

    #[test]
    fn parsed_commands_fail_explicitly_until_implemented() {
        let parsed = Cli::try_parse_from(["cage", "update"]);
        assert!(parsed.is_ok());
        let Ok(cli) = parsed else {
            return;
        };
        let result = execute(&cli);
        assert!(result.is_err());
        let Err(error) = result else {
            return;
        };

        assert_eq!(error.exit_code(), ExitCode::General);
        assert!(error.to_string().contains("cause:"));
        assert!(error.to_string().contains("next:"));
    }
}
