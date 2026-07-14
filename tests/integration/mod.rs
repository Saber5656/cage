#[test]
fn crate_metadata_is_available_to_integration_tests() {
    assert_eq!(env!("CARGO_PKG_NAME"), "cage");
}

mod cli {
    use assert_cmd::Command;
    use predicates::prelude::*;

    fn cage() -> Command {
        assert_cmd::cargo::cargo_bin_cmd!("cage")
    }

    #[test]
    fn root_help_lists_the_complete_command_surface() {
        cage()
            .arg("--help")
            .assert()
            .success()
            .stdout(include_str!("snapshots/root-help.txt"));
    }

    #[test]
    fn run_help_exposes_agent_passthrough_and_safety_options() {
        cage()
            .args(["run", "--help"])
            .assert()
            .success()
            .stdout(include_str!("snapshots/run-help.txt"));
    }

    #[test]
    fn sync_and_diff_help_expose_recovery_inputs() {
        cage()
            .args(["sync", "--help"])
            .assert()
            .success()
            .stdout(include_str!("snapshots/sync-help.txt"));
        cage()
            .args(["diff", "--help"])
            .assert()
            .success()
            .stdout(include_str!("snapshots/diff-help.txt"));
    }

    #[test]
    fn unimplemented_commands_fail_with_an_actionable_prefixed_error() {
        cage()
            .args(["--no-color", "update"])
            .assert()
            .code(1)
            .stderr(
                predicate::str::starts_with("[cage] ✗ ")
                    .and(predicate::str::contains("cause:"))
                    .and(predicate::str::contains("next:"))
                    .and(predicate::str::contains("not implemented"))
                    .and(predicate::str::contains('\u{1b}').not()),
            );
    }

    #[test]
    fn invalid_arguments_use_the_general_error_exit_code() {
        cage().arg("unknown-command").assert().code(1);
    }

    #[test]
    fn version_is_available_on_direct_and_nested_subcommands() {
        cage()
            .args(["run", "--version"])
            .assert()
            .success()
            .stdout(predicate::eq(format!(
                "cage-run {}\n",
                env!("CARGO_PKG_VERSION")
            )));
        cage()
            .args(["team", "up", "--version"])
            .assert()
            .success()
            .stdout(predicate::eq(format!(
                "cage-team-up {}\n",
                env!("CARGO_PKG_VERSION")
            )));
    }

    #[test]
    fn no_color_applies_to_clap_help_and_errors() {
        cage()
            .args(["--no-color", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains('\u{1b}').not());
        cage()
            .args(["--no-color", "unknown-command"])
            .assert()
            .code(1)
            .stderr(predicate::str::contains('\u{1b}').not());
    }

    #[test]
    fn every_top_level_command_uses_the_general_unimplemented_exit_code() {
        for args in [
            &["run"][..],
            &["sync"][..],
            &["diff"][..],
            &["team", "up"][..],
            &["config", "list"][..],
            &["images", "list"][..],
            &["update"][..],
        ] {
            cage()
                .args(args)
                .assert()
                .code(1)
                .stderr(predicate::str::starts_with("[cage] ✗ "));
        }
    }
}
