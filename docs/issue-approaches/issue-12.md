# Issue #12 — `cage run` MVP

| | |
|---|---|
| Issue | https://github.com/Saber5656/cage/issues/12 |
| Phase | 1 |
| Priority | Must |
| Requirements | FR-1, FR-5.1, NFR-4 |
| Depends on | #5, #6, #7, #8, #9, #10, #11 |
| Blocks | #13, #14, #22, #29 |
| Legacy reference | `cage-demo/src/cli/run.rs`; `docs/architecture/architecture.md` §3.3 |
| Status | Not started |

## Goal

The core command: copy the project into a hardened container, `git init` + baseline, inject creds,
and launch the agent — streaming its output in attached mode.

## Approach (pipeline)

1. Parse `RunArgs` → resolve config (#6) → profile → adapter (#10).
2. Security-validate path args + mounts (#7); build hardening (#9).
3. `create` container (hardening + tmpfs + resource limits) (#8).
4. **Copy project → `/workspace` honoring excludes (audit gap G-16)**: stream a tar that respects
   `[sync] exclude` / `.cageignore` and **skips root `.git`** (seeded separately in #13). Never
   bind-mount the host project or its `.git`.
5. `exec git init` + create baseline ref + `git config core.hooksPath /dev/null` (hooks defense L1).
6. Inject credentials + entrypoint (#11).
7. `start_attached` (see TTY below).
8. `--dry-run`: print the planned container + security config and **exit before create**.

### TTY & signals (audit gap G-15 — functional, not polish)

Attached mode must allocate a TTY, propagate `SIGWINCH` (resize), forward signals, and return the
agent's exit code as `cage run`'s exit code (Ctrl+C → 130). TUI agents are unusable otherwise.

### Setup validation (audit gap G-14)

Before launch, verify the image provides `git` (and `node`/`npm` when the adapter needs them);
fail with an actionable error (E-002 style) instead of a mid-session git failure.

### Startup observability (NFR-4)

Log per-step durations (create / copy / git init / cred inject / agent-ready) under `--verbose`.
Meeting the 10s target via image caching is owned by #27; this issue makes it **measurable**.

## Acceptance criteria → approach

- `cage run claude --dry-run` shows config → step 8.
- Real run creates→starts agent when runtime present → steps 1–7 (live-gated test).
- Root `.git` not bind-mounted → step 4 asserts no `.git` mount.
- Failure/cleanup policy documented → `docs/run-lifecycle.md` (partial container preserved for `--from-volume`).

## QA gate

- Unit: dry-run output, exclude honored in the tar, setup-validation error path.
- Live (#18): end-to-end run/exit-code/TTY on Docker.

## Risks & notes

- Cold `npm install` in the entrypoint blows NFR-4; #27 caches it. Keep the entrypoint idempotent
  so a cached image simply skips install.
