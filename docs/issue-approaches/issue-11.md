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

Pass API keys to the agent through tmpfs-backed files that are deleted right after startup, so
secrets never live in container metadata, image layers, logs, or `docker inspect`.

## Approach

1. **tmpfs mount**: `/run/cage-credentials` with `rw,noexec,nosuid,size=1m`.
2. **Delivery (audit gap G-12 — do NOT stage on host `/tmp`)**: the legacy flow wrote plaintext
   creds + entrypoint to host `std::env::temp_dir()` then `docker cp`. Replace with a host-FS-free
   path: `docker cp -` (tar on stdin) or `exec -i <id> sh -c 'cat > /run/cage-credentials/<name>'`
   with the value piped via stdin. If a temp file is truly unavoidable, use `tempfile` at mode
   `0600` with best-effort cleanup on panic.
3. **Timing**: tmpfs exists only after `create`; write creds **after create, before start**.
4. **Entrypoint** (`read → export → shred → exec`): read each `/run/cage-credentials/*` into an env
   var, `shred -n3 -u` (fallback `rm` where shred is absent), then `exec` the agent. Env inheritance
   is the intended propagation to child processes; files must be gone before `exec`.
5. **Masking (THREAT-CLI-I-02)**: scrub `*_KEY`/`*_TOKEN` values from any error/log/dry-run output.

## Acceptance criteria → approach

- tmpfs mount `noexec,nosuid,size=1m` → mount spec + test on the create args.
- Entrypoint deletes creds after load → generated script asserted in a unit test.
- Missing env var → understandable error → checked before container start (exit 4).
- Secrets never in logs/dry-run → masking helper + a test that dry-run output contains no value.

## QA gate

- Unit: entrypoint script content (shred present, order correct), masking helper, missing-var error.
- Live (#18): `docker inspect` shows no credential env; `/tmp` on host has no residue.

## Risks & notes

- Multi-process agents: only env inheritance is guaranteed; do not rely on the files persisting.
- File-/directory-backed credentials (OAuth token files) reuse this channel — designed in #25.
