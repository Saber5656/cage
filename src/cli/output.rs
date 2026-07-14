//! Consistent user-facing output formatting.

use std::io::{self, IsTerminal as _, Write as _};

const PREFIX: &str = "[cage]";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MessageKind {
    Info,
    Success,
    Warning,
    Error,
    Action,
}

/// Formats all Cage-owned output with the stable prefix and color policy.
#[derive(Clone, Copy, Debug)]
pub struct Output {
    color: bool,
}

impl Output {
    #[must_use]
    pub fn for_stderr(no_color: bool) -> Self {
        Self {
            color: !no_color
                && std::env::var_os("NO_COLOR").is_none()
                && io::stderr().is_terminal(),
        }
    }

    #[must_use]
    pub fn for_stdout(no_color: bool) -> Self {
        Self {
            color: !no_color
                && std::env::var_os("NO_COLOR").is_none()
                && io::stdout().is_terminal(),
        }
    }

    #[must_use]
    pub const fn with_color(color: bool) -> Self {
        Self { color }
    }

    #[must_use]
    pub fn format(self, kind: MessageKind, message: &str) -> String {
        let (marker, ansi) = match kind {
            MessageKind::Info => ("", "37"),
            MessageKind::Success => ("✓ ", "32"),
            MessageKind::Warning => ("⚠ ", "33"),
            MessageKind::Error => ("✗ ", "31"),
            MessageKind::Action => ("→ ", "36"),
        };
        let rendered = message
            .split('\n')
            .enumerate()
            .map(|(index, line)| {
                if index == 0 {
                    format!("{PREFIX} {marker}{line}")
                } else {
                    format!("{PREFIX} {line}")
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        if self.color {
            format!("\u{1b}[{ansi}m{rendered}\u{1b}[0m")
        } else {
            rendered
        }
    }

    pub fn error(self, message: &str) -> io::Result<()> {
        writeln!(
            io::stderr().lock(),
            "{}",
            self.format(MessageKind::Error, message)
        )
    }

    pub fn info(self, message: &str) -> io::Result<()> {
        self.write_stdout(MessageKind::Info, message)
    }

    pub fn success(self, message: &str) -> io::Result<()> {
        self.write_stdout(MessageKind::Success, message)
    }

    pub fn warn(self, message: &str) -> io::Result<()> {
        self.write_stdout(MessageKind::Warning, message)
    }

    pub fn action(self, message: &str) -> io::Result<()> {
        self.write_stdout(MessageKind::Action, message)
    }

    fn write_stdout(self, kind: MessageKind, message: &str) -> io::Result<()> {
        writeln!(io::stdout().lock(), "{}", self.format(kind, message))
    }
}

#[cfg(test)]
mod tests {
    use super::{MessageKind, Output};

    #[test]
    fn every_message_has_the_cage_prefix() {
        for kind in [
            MessageKind::Info,
            MessageKind::Success,
            MessageKind::Warning,
            MessageKind::Error,
            MessageKind::Action,
        ] {
            assert!(
                Output::with_color(false)
                    .format(kind, "message")
                    .starts_with("[cage] ")
            );
        }
    }

    #[test]
    fn every_line_of_a_multiline_message_has_the_cage_prefix() {
        let rendered = Output::with_color(false).format(
            MessageKind::Error,
            "summary\n\n  cause: invalid input\n  next: fix it",
        );

        assert!(rendered.lines().all(|line| line.starts_with("[cage] ")));
    }

    #[test]
    fn no_color_output_has_no_ansi_escape() {
        let rendered = Output::with_color(false).format(MessageKind::Error, "failed");

        assert!(!rendered.contains('\u{1b}'));
    }

    #[test]
    fn color_output_wraps_the_complete_message() {
        let rendered = Output::with_color(true).format(MessageKind::Success, "done");

        assert_eq!(rendered, "\u{1b}[32m[cage] ✓ done\u{1b}[0m");
    }
}
