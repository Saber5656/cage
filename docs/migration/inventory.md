# Cage migration inventory

This repository, `Saber5656/cage`, is the canonical public source and working repository for Cage.
The sibling `cage-demo` repository is a legacy implementation and design reference only. New work,
reviews, and releases must be based on this repository.

Migration is intentionally staged through the owning GitHub issues. Files from `cage-demo` must
not be copied as a bulk import: each area is reviewed, adapted to the current requirements, tested,
and merged independently. The legacy Git history is not imported because it is not needed for the
public source and has not been audited for private paths, credentials, or other publishability
risks.

## Classification

| Classification | Meaning |
|---|---|
| `migrate-with-changes` | Reuse the design or implementation only after its owning issue applies current requirements, tests, and security review. |
| `reference-only` | Keep outside this repository; consult it while implementing the owning issue. |
| `exclude` | Never migrate this path or artifact. |

No legacy path is approved for `migrate-as-is`.

## Migration candidates

| Legacy path | Classification | Destination / owner | Required review or change |
|---|---|---|---|
| `docs/PRD.md` | `migrate-with-changes` | `docs/`; #2 and each feature owner (#5–#29) | Reconcile audit addenda and remove local-only provenance before publication. |
| `docs/architecture/architecture.md` | `migrate-with-changes` | `docs/architecture/`; #4, #8, #12, #13, #19, #20, #28 | Verify module boundaries against the implementation delivered by #4–#13. |
| `docs/architecture/tech-selection.md` | `migrate-with-changes` | `docs/architecture/`; #4, #8, #19, #23 | Revalidate Rust/MSRV and Docker/Podman support claims. |
| `docs/cli-ux/cli-ux-spec.md` | `migrate-with-changes` | `docs/cli-ux/`; #5, #12, #21, #22, #26, #29 | Remove or explicitly mark commands and flags that remain unimplemented. |
| `docs/security-design-cli-injection.md` | `migrate-with-changes` | `docs/security/`; #5–#12 | Security review against the final argument-array and validation implementation. |
| `docs/security-design-docker-socket.md` | `migrate-with-changes` | `docs/security/`; #7, #9, #18, #20 | Recheck socket aliases, rootless runtimes, and macOS canonicalization. |
| `docs/security/stride-threat-model.md` | `migrate-with-changes` | `docs/security/`; #7–#12, #18, #24–#26 | Update mitigations and residual risks after the owning implementations land. |
| `src/main.rs`, `src/lib.rs` | `migrate-with-changes` | #4 | Restore only the minimal crate entry points and current module skeleton. |
| `src/cli/**` | `migrate-with-changes` | #5, #11–#15, #21, #22, #25–#27, #29 | Split by command owner; remove parse-only or silent no-op behavior. |
| `src/config/**` | `migrate-with-changes` | #6 | Add trust confirmation, size limits, strict validation, and safe merge behavior. |
| `src/security/**` | `migrate-with-changes` | #7, #9 | Reconcile audit findings; do not weaken mandatory hardening through config. |
| `src/engine/**` | `migrate-with-changes` | #8, #9, #19, #20 | Preserve argument-array execution; separately review Podman and DinD boundaries. |
| `src/adapter/**` | `migrate-with-changes` | #10, #25, #26, #27 | Rework credential, image-prerequisite, host-context, and version handling. |
| `src/session/**` | `migrate-with-changes` | #13, #16, #29 | Validate lifecycle, crash recovery, ambiguity, and resume behavior. |
| `src/sync/**` | `migrate-with-changes` | #14–#17, #24 | Fix binary handling and retain approval and sensitive-path gates. |
| `src/team/**` | `migrate-with-changes` | #22 | Await the human-approved workspace model before implementation. |
| `tests/integration/mod.rs` | `migrate-with-changes` | #4, #18 | Keep a test target in the skeleton; adapt modules as features land. |
| `tests/integration/security_test.rs` | `migrate-with-changes` | #18 | Revalidate every security assertion and separate daemon-dependent tests. |
| `tests/integration/sync_test.rs` | `migrate-with-changes` | #14–#17, #24, #28 | Retain platform-specific path coverage and add binary-safe cases. |
| `seccomp/default.json` | `migrate-with-changes` | #9, #18 | Validate JSON and policy semantics; define how the profile is distributed and enabled. |
| `.github/workflows/ci.yml` | `migrate-with-changes` | #3, #4 | Recreate from current quality gates; allow only `push` and `pull_request` triggers. |
| `.github/workflows/security.yml` | `migrate-with-changes` | #3, #18 | Do not copy wholesale. The legacy workflow previously included a weekly schedule; that trigger has been removed from the current local reference. Migration must not reintroduce any `schedule`/`cron` trigger and must revalidate all commands. |
| `Cargo.toml`, `Cargo.lock` | `migrate-with-changes` | #4 | Audit crate metadata and dependencies; set and test the Rust 1.85 MSRV. |
| `.gitignore`, `rustfmt.toml`, `clippy.toml`, `deny.toml` | `migrate-with-changes` | #4 | Restore current quality/security policy without local-only patterns. |
| `cage.toml.example` | `migrate-with-changes` | #6, #10 | Match the current schema and exclude hardening-bypass or secret-bearing examples. |

## Explicit exclusions

| Legacy path or artifact | Classification | Reason |
|---|---|---|
| `.git/**` and all `cage-demo` commit history | `exclude` | The public repository has its own reviewed history; legacy history has not passed privacy and secret scanning. |
| `target/**` | `exclude` | Reproducible build output and local cache; never source material. |
| Scheduled workflow configuration (`schedule`, `cron`) | `exclude` | Public CI is limited to `push` and `pull_request`; #3 owns the enforcement guard. |
| Editor, OS, scratch, log, session, credential, and temporary files not listed above | `exclude` | Machine-local or sensitive artifacts are not project source. |
| Any private key, token, OAuth state, runtime socket, or generated credential material | `exclude` | Secrets and host runtime capabilities must never enter source control. |

## Items requiring follow-up review

- Documentation is a requirements reference until the owning implementation issue confirms that
  its behavior is accurate. It must not be presented as a released capability prematurely.
- Security-sensitive source, tests, workflow commands, and the seccomp policy require independent
  security review in their owning issues.
- Configuration examples require the same validators as CLI input and must not expose a way to
  disable mandatory hardening.
- Platform claims remain provisional until #19 and #28 validate supported OS/runtime combinations.
- Release and dependency supply-chain material remains gated by #23.

## Dependency order

| Stage | Owning issues | Dependency established by this inventory |
|---|---|---|
| Repository foundation | #2, #3, #4 | #2 and #4 build on this canonical-source decision; #3 defines the workflow-trigger policy used by #4 and #18. |
| Sandbox MVP | #5–#13 | Requires the buildable skeleton from #4; runtime and hardening follow the dependency graph in `docs/issue-approaches/README.md`. |
| Sync, recovery, and host context | #14–#17, #24–#26 | Requires the MVP/session/credential layers owned by #10–#13. |
| Runtime breadth and release readiness | #18–#23, #27–#29 | Requires the relevant security, runtime, command, and session owners to land first. |

The detailed issue graph and decisions requiring human approval remain in
[`docs/issue-approaches/README.md`](../issue-approaches/README.md). This inventory selects source
material; it does not expand or override any owning issue's scope.

## Verification checklist

- Compare this inventory with `find cage-demo` while excluding only `.git/**` and `target/**`.
- Confirm every source, test, documentation, workflow, seccomp, and configuration path maps to an
  owning issue or an explicit exclusion.
- Confirm no repository workflow contains a scheduled trigger before each workflow-related PR is
  merged.
