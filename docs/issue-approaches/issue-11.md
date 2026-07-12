# Issue #11 — tmpfs credential injection

| | |
|---|---|
| Issue | https://github.com/Saber5656/cage/issues/11 |
| Phase | 1 |
| Priority | Must |
| Requirements | FR-2.6; THREAT-CE-I-01, THREAT-AA-I-01, THREAT-CLI-I-02 |
| Depends on | #8, #10 |
| Blocks | #12, #25, #26, #29 |
| Legacy reference | `cage-demo/src/cli/run.rs` credential injection (lines ~1108–1222) |
| Status | Not started |

## Goal

Pass credentials through tmpfs-backed files so secrets never live in container metadata, image
layers, logs, or `docker inspect`. One-shot environment delivery files are deleted before agent
launch; file/directory credentials remain only on tmpfs for as long as the adapter requires them.

## Approach

1. **tmpfs mount**: `/run/cage-credentials` with `rw,noexec,nosuid,size=1m`.
2. **Delivery (audit gap G-12 — do NOT stage on host `/tmp`)**: include a Cage-owned
   `cage-bootstrap receive-credentials` helper in runnable images. Stream a length-delimited bundle
   to that process over `exec -i`. The manifest permits only regular files and directories with
   unique, normalized relative paths; reject symlinks, hard links, devices, FIFOs/sockets, duplicate
   paths, parent/child type collisions, and unsafe components. Pre-open a nonce-scoped `.incoming/`
   root on tmpfs and create entries descriptor-relative with no-follow/beneath resolution
   (`openat2` where available, otherwise component-wise `openat`/`mkdirat` with `O_NOFOLLOW`) at
   mode `0600`/`0700`. On complete input, verify manifest/digests and atomically rename the directory
   to `.ready/<generation>`; an interrupted or invalid stream removes `.incoming/` before exit. Do
   not use archive extraction, `sh -c`, runtime copy APIs for the tmpfs, or a host-tempfile fallback.
3. **Timing**: create the container with the tmpfs, start it in a waiting bootstrap phase, and
   confirm bootstrap readiness before delivery. Injection into a created-but-stopped container is
   forbidden because the live tmpfs is not mounted and bytes could reach the writable layer. PID 1
   gives each ready generation a one-use handoff token and setup deadline; controller disconnect or
   timeout before launch wipes unclaimed generations and stops the bootstrap container.
4. **Launch handoff and lifetime**: PID 1 reads environment-backed values, unlinks those one-shot
   delivery files, clears temporary buffers, consumes the matching handoff token, and then `exec`s
   the agent. For adapter-specific file/directory credentials from #25, atomically move the validated
   generation within tmpfs to an adapter-declared live path and point the agent there via an
   argument, environment variable, or non-secret symlink. Keep that path only while the session
   runs; container stop/cleanup removes it with the tmpfs. Never copy credential bytes into the
   container writable layer.
5. **Masking (THREAT-CLI-I-02)**: the credential resolver registers every environment, file, and
   directory source with a source-aware redactor before any error/log/dry-run rendering. Redact the
   actual resolved values and known secret fields (API keys, passwords, OAuth/access/refresh tokens),
   rather than relying only on variable-name suffixes such as `_KEY` or `_TOKEN`.

## Acceptance criteria → approach

- tmpfs mount `noexec,nosuid,size=1m` → mount spec + test on the create args.
- Delivery starts only after bootstrap readiness → state-machine test; no shell, runtime-copy, or
  host-tempfile delivery path exists.
- Partial input, failed validation, controller loss, and setup timeout leave no staged credential
  generation and do not launch the agent → transactional/watchdog tests.
- Directory bundles cannot escape or alias their nonce root → unsafe-entry rejection and no-follow
  descriptor-walk tests.
- Bootstrap deletes one-shot environment files before agent exec; required file/directory sources
  remain tmpfs-only during the session and disappear on stop → lifecycle tests.
- Missing credential source → understandable error before agent launch (exit 4).
- Secrets never in logs/dry-run → source-aware redactor tests prove resolved API-key, password,
  OAuth-token, file, and directory values never appear.

## QA gate

- Unit: bootstrap state/order, destination validation, no-shell delivery command, source-aware
  redaction, transactional receive/rollback, symlink/hardlink/special-file and collision rejection,
  descriptor-relative containment, one-use handoff token, setup watchdog, and missing-source error.
- Live (#18): delivery succeeds only after the tmpfs is active; `docker inspect` shows no credential
  env; the container writable layer and host temp directories have no residue; one-shot files are
  gone before agent exec; session-scoped file/directory credentials resolve inside tmpfs while
  running and are gone after stop.

## Risks & notes

- Environment-backed credentials propagate by inheritance. Adapter-declared file/directory paths
  remain available to child processes only for the running container's tmpfs lifetime.
- File-/directory-backed credentials (OAuth token files) reuse this channel — designed in #25.
