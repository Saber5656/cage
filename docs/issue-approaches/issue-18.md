# Issue #18 — Security regression tests in CI

| | |
|---|---|
| Issue | https://github.com/Saber5656/cage/issues/18 |
| Phase | 3 |
| Priority | Must |
| Requirements | NFR-3; FR-5.10; whole STRIDE model |
| Depends on | #7, #9, #3 |
| Blocks | — |
| Legacy reference | `cage-demo/tests/integration/security_test.rs`, `seccomp/default.json`, `.github/workflows/security.yml` |
| Status | Not started |

## Goal

Turn the threat model into CI gates on push/PR (no schedule), covering unit-level validation and
Docker live escape tests, with clean skips when Docker is absent.

## Approach

1. **Unit security tests**: path/volume validation, hardening args, sync validation, seccomp JSON validity.
2. **Live Docker tests** (`CAGE_INTEGRATION_DOCKER=1`, `#[ignore]` by default): socket
   inaccessibility, `no-new-privileges` blocks setuid, `cap-drop=ALL`.
3. **Seccomp validation**: parse `seccomp/default.json`, assert `defaultAction` + key restrictions.
4. **FR-5.10 no-shell lint gate (audit gap G-11)**: `clippy.toml` `disallowed-methods` (and/or a
   grep CI step) failing the build on `Command::new("sh"|"bash")`, `.arg("-c")` string-building, etc.
   This is the Rust equivalent of the PRD's gosec/bandit/shellcheck requirement.
5. **STRIDE traceability (audit gap)**: name each test after its THREAT-ID (socket → THREAT-SEC-E-01,
   no-new-privs → THREAT-SEC-E-02, traversal → THREAT-AS-T-01) or keep a doc table, so gaps are visible.
6. Triggers: push + pull_request only (#3); GitHub Actions Linux runners run live tests, macOS jobs
   skip cleanly (#28).

## Acceptance criteria → approach

- No scheduled trigger → push/PR only + guard.
- Security unit tests run on PRs → CI job.
- Live-test skip conditions explicit → env gate + logged skip.
- Seccomp syntax + key restrictions validated → JSON parse test.

## QA gate

- CI green with the security job; a deliberately-shell-invoking commit fails the lint gate.

## Risks & notes

- macOS CI has no Docker; don't let "skipped" masquerade as "passed" — log which cells ran (#28).
