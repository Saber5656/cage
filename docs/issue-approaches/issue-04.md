# Issue #4 — Restore Rust project skeleton & quality gates

| | |
|---|---|
| Issue | https://github.com/Saber5656/cage/issues/4 |
| Phase | 0 |
| Priority | P0 |
| Requirements | §10 tech selection; FR-5.7/5.10 (lint posture) |
| Depends on | #1 |
| Blocks | #5–#13 (all implementation) |
| Legacy reference | `cage-demo/Cargo.toml`, `src/main.rs`, `src/lib.rs`, `clippy.toml`, `deny.toml` |
| Status | Not started |

## Goal

A buildable, testable, lint-clean crate skeleton so every later issue lands against green gates.

## Approach

1. Restore project files: `Cargo.toml`, `.gitignore`, `rustfmt.toml`, `clippy.toml`, `deny.toml`.
2. Carry lint posture from legacy `Cargo.toml`: `unsafe_code = "deny"`, security clippy lints
   (`unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented` = warn; `dbg_macro` = deny;
   `pedantic` = warn with the documented allow-list).
3. Minimal `src/main.rs` + `src/lib.rs` + empty module tree matching the target layout
   (`cli/ config/ security/ engine/ adapter/ sync/ session/ team/`).
4. Keep the `[[test]] name = "integration"` target and dev-deps (`assert_cmd`, `predicates`,
   `tempfile`, `serde_json`) so #18's security tests have a home from day one.
5. **Set MSRV**: edition 2024 requires Rust ≥ 1.85 — add `package.rust-version = "1.85"` and a CI
   job that checks the crate builds on that toolchain.
6. CI workflow (push/PR per #3): `cargo fmt --check`,
   `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-features`.

## Key decisions (defaults taken)

- Edition **2024** retained (matches legacy); MSRV pinned to 1.85 and enforced.
- Modules are created as stubs now so later issues only fill them in (stable import paths).

## Acceptance criteria → approach

- `cargo test --all-features` green → skeleton compiles with placeholder tests.
- `cargo fmt --all -- --check` green → `rustfmt.toml` + formatted skeleton.
- `cargo clippy --all-targets --all-features -- -D warnings` green → lint config + deny warnings in CI.
- Crate metadata has description/license/repository → carried from legacy `Cargo.toml`.

## QA gate

- CI (fmt + clippy + test + MSRV) green on the PR.

## Risks & notes

- If contributors run an older toolchain, edition-2024 parse errors look cryptic — the explicit
  `rust-version` and MSRV CI job turn that into a clear message.
