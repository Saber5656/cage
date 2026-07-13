# Contributing to Cage

Cage is a pre-alpha security tool. Contributions are welcome, but changes must
not imply that incomplete controls are production ready.

## Before opening an issue

- Search existing issues and implementation approaches for related work.
- Describe the problem, expected behavior, environment, and reproduction steps.
- Keep vulnerability details out of public issues. Follow
  [SECURITY.md](SECURITY.md) instead.
- Use an issue for requirements or behavior changes before starting a large
  implementation.

## Pull requests

- Keep each pull request focused on one issue or independently reviewable
  concern.
- Explain the motivation, security implications, and verification performed.
- Add or update tests for behavior changes.
- Update documentation when user-visible behavior or security assumptions
  change.
- Do not add generated artifacts, credentials, private configuration, or
  unrelated refactors.
- Do not introduce scheduled GitHub Actions workflows during pre-alpha.

Once the Rust project skeleton is available, code changes are expected to pass:

```text
cargo fmt --all -- --check
cargo clippy --all-targets --all-features
cargo test --all-features
```

The exact required checks are defined by the repository's current workflows.

## Security-sensitive changes

Changes to sandbox boundaries, runtime arguments, host paths, credentials,
security policy, seccomp profiles, or GitHub workflows require explicit owner
review. Describe the threat being addressed, failure behavior, and tests that
demonstrate the control cannot be bypassed.

Do not weaken mandatory hardening to make a test pass. Design changes,
requirement additions, and changes to the permission model require maintainer
approval before implementation.

## Commit and review hygiene

Use small commits that each have one clear purpose and can be reviewed or
reverted independently. Address review findings with a separate focused commit
when practical.

By contributing, you agree that your contribution is licensed under either the
Apache License, Version 2.0 or the MIT license, at the recipient's option.
