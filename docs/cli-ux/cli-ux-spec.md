# Cage CLI baseline contract

Status: pre-alpha. This document describes the command surface introduced by Issue #5. Unless an
owning feature issue says otherwise, parsing a command ends with an explicit not-implemented error;
no command is a silent no-op.

## Global behavior

| Item | Contract |
|---|---|
| Prefix | Cage-owned messages begin with `[cage]`. |
| Color | Disabled by `--no-color`, `NO_COLOR`, or non-TTY output. |
| Verbose output | `-v` / `--verbose` is accepted globally; feature owners define added diagnostics. |
| Help and version | `--help` and `--version` print through clap and exit 0. |
| Invalid arguments | clap diagnostic, exit 1. |
| Unimplemented command | `[cage]` error with a cause and next action, exit 1. |

Stable feature exit codes are 0 for success, 1 for general or argument errors, 2 for security
errors, 3 for container errors, 4 for authentication errors, and 130 for user interruption.

## Command tree

```text
cage <COMMAND>
  run [AGENT_OR_PROFILE] [OPTIONS] -- [AGENT_ARGS]...
  sync [OPTIONS]
  diff [OPTIONS]
  team <up|down|status>
  config <get|set|list|edit|validate>
  images <list|pull|remove|prune>
  update [--check]
```

Agent arguments are accepted only after `--`, so agent-owned flags cannot be interpreted as Cage
flags. `run`, `sync`, and `diff` expose the reserved inputs shown by their `--help` output. The
owning feature issues must replace the common not-implemented dispatch error when they add real
behavior; they must not leave parse-only flags or successful no-ops.

## Message styles

| Kind | Plain-text form |
|---|---|
| Information | `[cage] message` |
| Success | `[cage] ✓ message` |
| Warning | `[cage] ⚠ message` |
| Error | `[cage] ✗ message` |
| Action | `[cage] → message` |

User-facing errors include a summary followed by `cause:` and `next:` lines.
