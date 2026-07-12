# Issue #19 — Podman compatibility & runtime differences

| | |
|---|---|
| Issue | https://github.com/Saber5656/cage/issues/19 |
| Phase | 3 |
| Priority | Must |
| Requirements | NFR-5 |
| Depends on | #8 |
| Blocks | #28 |
| Legacy reference | `cage-demo/src/engine/podman.rs`; `docs/architecture/tech-selection.md` §2 |
| Status | Not started |

## Goal

Validate Podman behind the runtime abstraction — flags, networking, security options, and rootless
behavior — without breaking Docker.

## Approach

1. Availability detection: `--podman` and `CAGE_RUNTIME` win over auto-detection; missing Podman →
   understandable error.
2. Verify create/start/exec/cp/network parity with Docker (arg-vector tests + a live suite).
3. **Rootless specifics (audit gap G-23)**:
   - `--user <uid>:<gid>` interacts with user namespaces → support `--userns=keep-id`; verify
     workspace file ownership survives `podman cp` round-trips.
   - `--memory`/`--pids-limit` need cgroups v2 delegation and may be unavailable → decide
     warn-and-continue vs fail-closed and test both.
   - macOS `podman machine` adds a VM boundary (volume/tmpfs semantics differ) → mark tested/best-effort in #28.
4. Document Podman-in-Podman as the `--dind` counterpart or defer it (#20).
5. Keep all divergences inside the Podman impl; `docs/runtimes.md` records them.

## Acceptance criteria → approach

- `--podman` selects Podman → detection precedence + test.
- Missing Podman → understandable error → error path.
- Podman differences documented → `docs/runtimes.md`.
- Podman tests added without breaking Docker → parallel arg-vector + live suites.

## QA gate

- Arg-vector parity tests; live Podman suite where a rootless Podman is available (skips otherwise).

## Risks & notes

- Rootless resource-limit gaps are the most likely silent failure — pick fail-closed unless the
  user opts into best-effort, and say so in the error.
