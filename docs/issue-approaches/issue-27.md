# Issue #27 — Meet NFR-4 startup latency with cached agent images

| | |
|---|---|
| Issue | https://github.com/Saber5656/cage/issues/27 |
| Phase | 3 |
| Priority | P1 |
| Requirements | NFR-4; PRD §11.5 (`--rebuild`, `--no-cache`, `--agent-version`) |
| Depends on | #10, #21 |
| Blocks | — |
| Legacy reference | entrypoint `npm install -g` in `cage-demo/src/cli/run.rs` |
| Status | Not started |

## Goal

Reach agent-ready within 10 s (NFR-4). The legacy design runs `npm install -g <agent>` in the
entrypoint on **every** start, so cold starts routinely exceed 10 s, vary with the network, and
fail offline.

## Approach

1. **Bake path**: `cage images build <agent>` (or first-run auto-bake) producing a cached image with
   the agent preinstalled, keyed by `agent name + version`; later runs skip npm install.
2. Wire `--rebuild` / `--no-cache` (PRD §11.5) to invalidate the cache.
3. Wire `--agent-version <ver>` to pin the baked package version (adapter field from #10).
4. **Timing**: emit per-step durations under `--verbose` (create/copy/git init/cred/agent-ready);
   add a perf smoke test skipped when Docker is absent.
5. **Prebuilt publish?** Decide whether to publish images (e.g. GHCR) for pre-alpha and document it;
   supply-chain implications belong to #23.
6. **No creds in layers**: bake happens before credential injection — assert baked images are secret-free.

## Acceptance criteria → approach

- Warm start ≤ 10 s on a documented reference machine → cached image + measured smoke test.
- `--rebuild`/`--no-cache`/`--agent-version` behave as documented → cache-key + invalidation tests.
- Startup latency observable without a profiler → verbose timing.
- Baked images contain no credentials → build ordering + inspect test.

## QA gate

- Perf smoke test (warm vs cold); cache-key unit tests; image-secret scan.

## Risks & notes

- Keep the entrypoint idempotent: on a baked image it detects the agent is present and skips
  install, so the same entrypoint works cached or cold.
