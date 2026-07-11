# Issue Implementation Approaches

This directory is the canonical, in-repo home for the **implementation approach** of every
open GitHub issue. Each `issue-NN.md` records how the issue will be built, which requirements
it traces to, what it depends on, the defaults already chosen, and any decisions that still
need a human sign-off before coding starts.

These documents are written to be actionable by an implementer (human or agent) without
re-reading the whole design corpus: they carry the API contract, module structure, steps,
QA gate, and acceptance-criteria mapping for each issue.

> Sources of truth: requirements live in `cage-demo/docs/PRD.md` (v0.3.0); design lives in
> `cage-demo/docs/architecture/*`, `cage-demo/docs/security/*`, and the two `security-design-*`
> docs. The legacy implementation in `cage-demo/src` is a reference, not a drop-in.

## How to use

- Before implementing issue `#N`, read `issue-NN.md` here **and** the referenced PRD/design sections.
- If an approach doc has an **Open decisions** section, resolve those items with the owner first —
  do not silently pick a direction that changes scope.
- When an issue is completed, update its `Status` line and note the merge commit / PR.

## Phase overview

| Phase | Theme | Issues |
|---|---|---|
| 0 | Repository foundation | #1, #2, #3, #4 |
| 1 | Sandbox MVP core (`cage run`) | #5, #6, #7, #8, #9, #10, #11, #12, #13 |
| 2 | Artifact sync, diff, recovery, filters | #14, #15, #16, #17, #24, #25, #26 |
| 3 | Runtime breadth, teams, packaging, polish | #18, #19, #20, #21, #22, #23, #27, #28, #29 |

## Issue index

| # | Title | Phase | Priority | Depends on |
|---|---|---|---|---|
| [1](issue-01.md) | Canonicalize repo & migration scope | 0 | P0 | — |
| [2](issue-02.md) | OSS baseline documents | 0 | P0 | #1 |
| [3](issue-03.md) | CI limited to push/PR triggers | 0 | P0 | — |
| [4](issue-04.md) | Rust skeleton & quality gates | 0 | P0 | #1 |
| [5](issue-05.md) | CLI command surface & output | 1 | Must | #4 |
| [6](issue-06.md) | Two-level `cage.toml` loader | 1 | Must | #4 |
| [7](issue-07.md) | Security Layer path/volume validation | 1 | Must | #4 |
| [8](issue-08.md) | Docker/Podman runtime abstraction | 1 | Must | #4, #7 |
| [9](issue-09.md) | Hardening & resource limits | 1 | Must | #7, #8 |
| [10](issue-10.md) | Agent Adapter trait & built-ins | 1 | Must | #6 |
| [11](issue-11.md) | tmpfs credential injection | 1 | Must | #8, #10 |
| [12](issue-12.md) | `cage run` MVP | 1 | Must | #5–#11 |
| [13](issue-13.md) | Session persistence & baseline ref | 1 | Must | #12 |
| [14](issue-14.md) | `cage diff` | 2 | Must | #12, #13 |
| [15](issue-15.md) | `cage sync` approval & apply | 2 | Must | #14 |
| [16](issue-16.md) | `--from-volume` crash recovery | 2 | Must | #13, #15 |
| [17](issue-17.md) | Sync include/exclude & sensitive warnings | 2 | Should | #15 |
| [24](issue-24.md) | Binary-safe diff/sync (lost bug) | 2 | P1 | #14, #15 |
| [25](issue-25.md) | Subscription/OAuth credentials | 2 | Should | #10, #11 |
| [26](issue-26.md) | Host-context forwarding (ssh/hooks/settings) | 2 | Should | #11, #25 |
| [18](issue-18.md) | Security regression tests in CI | 3 | Must | #7, #9, #3 |
| [19](issue-19.md) | Podman compatibility | 3 | Must | #8 |
| [20](issue-20.md) | Safe DinD sidecar | 3 | Should | #8 |
| [21](issue-21.md) | `config`/`images`/`update` commands | 3 | Should | #5, #6 |
| [22](issue-22.md) | `cage team` up/down/status MVP | 3 | Must* | #12 |
| [23](issue-23.md) | Pre-alpha release packaging | 3 | — | #2 |
| [27](issue-27.md) | Startup latency / image cache (NFR-4) | 3 | P1 | #10, #21 |
| [28](issue-28.md) | Platform support matrix | 3 | — | #19, #26 |
| [29](issue-29.md) | `cage run --continue` resume | 3 | — | #11, #13, #16 |

\* PRD marks Teams as Must, but MVP scope is under review — see [issue-22.md](issue-22.md).

## Open decisions requiring human sign-off

These are the cross-cutting choices flagged during the 2026-07-06 requirements audit. They gate
the issues in the third column and should be resolved before those issues start.

| ID | Decision | Options | Gates |
|---|---|---|---|
| D-1 | Pre-alpha auth scope | (a) API-key only · (b) also OAuth/token-file injection | #25, #26 |
| D-2 | `--with-ssh` / `--with-hooks` | (a) implement per spec · (b) remove + explicit "unsupported" error | #26 |
| D-3 | Teams workspace model | (a) shared workspace + single team baseline · (b) per-agent workspace + shared dir excluded from sync | #22 |
| D-4 | DinD daemon endpoint | (a) plain TCP only with preventive network admission control · (b) TLS 2376 (required otherwise) | #20 |
| D-5 | Release channel priority | order of `cargo install` / GitHub Releases / Homebrew; SBOM & attestation for first release | #23 |
| D-6 | WSL2 pre-alpha status | validated vs explicitly unsupported | #28 |

## Provenance

Derived from the 2026-07-06 requirements-vs-issue audit. Full gap analysis and evidence:
Agents-Vault `01-Projects/Cage/TSK-20260706-cage-issue-requirements-audit/task.md`.
