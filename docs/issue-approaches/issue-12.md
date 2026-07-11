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
3. `create` the container with hardening, tmpfs, resource limits, a TTY, and the waiting
   `cage-bootstrap` process as PID 1 (#8/#11).
4. Call `start_attached` in bootstrap mode and retain its connection-ready `AttachedSession` before
   setup. The live PID 1, tmpfs, workspace volume, and output stream must all be ready before any
   setup `exec` or streamed write. Bootstrap enforces a setup deadline and a controller lease so
   interruption rolls back staged credentials and stops an unlaunched container.
5. **Copy project → `/workspace` honoring excludes (audit gap G-16)**: stream a tar through the
   non-shell bootstrap helper that respects `[sync] exclude` / `.cageignore` and **skips root
   `.git`**. Never bind-mount the host project or its `.git`.
6. Build and validate the host-owned baseline generation from the exact filtered snapshot streamed
   to the workspace, then publish the session's `Ready` commit marker (#13). Separately, while the
   container is running, `exec git init` for the agent-local repository and set
   `core.hooksPath=/dev/null` (hooks defense L1); that writable `.git` is not trusted by diff/sync.
7. Inject credentials through the active tmpfs (#11).
8. With the `AttachedSession` already connected, signal bootstrap with the one-use handoff token to
   publish session-scoped credentials and `exec` the adapter command, then drive resize/signals and
   await output/exit status through that same handle.
9. `--dry-run`: print the planned container + security config and **exit before create**.

### TTY & signals (audit gap G-15 — functional, not polish)

Attached mode must allocate a TTY, propagate `SIGWINCH` (resize), forward signals, and return the
agent's exit code as `cage run`'s exit code (Ctrl+C → 130). TUI agents are unusable otherwise.

### Setup validation (audit gap G-14)

Before launch, verify the image provides the compatible `cage-bootstrap`, `git` (and `node`/`npm`
when the adapter needs them); fail with an actionable error (E-002 style) instead of a mid-session
setup failure.

### Startup observability (NFR-4)

Log per-step durations (create / copy / git init / cred inject / agent-ready) under `--verbose`.
Meeting the 10s target via image caching is owned by #27; this issue makes it **measurable**.

## Acceptance criteria → approach

- `cage run claude --dry-run` shows config → step 9.
- Real run creates→bootstraps→starts agent when runtime present → steps 1–8 (live-gated test).
- Root `.git` not bind-mounted → step 5 asserts no `.git` mount.
- Failure/cleanup policy documented → `docs/run-lifecycle.md` (partial container preserved for `--from-volume`).

## QA gate

- Unit: dry-run output, exclude honored in the tar, setup-validation error path.
- Live (#18): end-to-end bootstrap/setup/agent handoff, exit-code, and TTY on Docker; setup execs
  never target a stopped container. Fault injection after credential receive and before handoff
  proves controller loss triggers rollback/stop; an agent that prints and exits immediately still
  has complete output and status captured.

## Risks & notes

- Cold `npm install` in the entrypoint blows NFR-4; #27 caches it. Keep the entrypoint idempotent
  so a cached image simply skips install.
