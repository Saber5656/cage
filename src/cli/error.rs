//! User-facing CLI errors and stable process exit codes.

use std::fmt;

/// Stable process exit codes documented by the Cage CLI contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum ExitCode {
    Success = 0,
    General = 1,
    Security = 2,
    Container = 3,
    Auth = 4,
    Interrupted = 130,
}

impl ExitCode {
    #[must_use]
    pub const fn as_i32(self) -> i32 {
        self as i32
    }
}

/// An error that carries both an explanation and an actionable next step.
#[derive(Debug)]
pub struct CageError {
    summary: String,
    cause: String,
    next: String,
    exit_code: ExitCode,
}

impl CageError {
    #[must_use]
    pub fn new(
        summary: impl Into<String>,
        cause: impl Into<String>,
        next: impl Into<String>,
        exit_code: ExitCode,
    ) -> Self {
        Self {
            summary: summary.into(),
            cause: cause.into(),
            next: next.into(),
            exit_code,
        }
    }

    #[must_use]
    pub fn not_implemented(command: &str) -> Self {
        Self::new(
            format!("the `{command}` command is not implemented yet"),
            "this command surface is reserved for its owning implementation issue",
            format!("follow the owning issue before relying on `cage {command}`"),
            ExitCode::General,
        )
    }

    #[must_use]
    pub const fn exit_code(&self) -> ExitCode {
        self.exit_code
    }
}

impl fmt::Display for CageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}\n\n  cause: {}\n  next: {}",
            self.summary, self.cause, self.next
        )
    }
}

impl std::error::Error for CageError {}

#[cfg(test)]
mod tests {
    use super::{CageError, ExitCode};

    #[test]
    fn exit_codes_remain_stable() {
        assert_eq!(ExitCode::Success.as_i32(), 0);
        assert_eq!(ExitCode::General.as_i32(), 1);
        assert_eq!(ExitCode::Security.as_i32(), 2);
        assert_eq!(ExitCode::Container.as_i32(), 3);
        assert_eq!(ExitCode::Auth.as_i32(), 4);
        assert_eq!(ExitCode::Interrupted.as_i32(), 130);
    }

    #[test]
    fn display_includes_cause_and_next_action() {
        let error = CageError::new("failed", "bad input", "fix the input", ExitCode::General);
        let rendered = error.to_string();

        assert!(rendered.contains("cause: bad input"));
        assert!(rendered.contains("next: fix the input"));
    }
}
