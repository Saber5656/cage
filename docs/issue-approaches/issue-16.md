# Issue #16 — `--from-volume` sync & diff for crash recovery

| | |
|---|---|
| Issue | https://github.com/Saber5656/cage/issues/16 |
| Phase | 2 |
| Priority | Must |
| Requirements | FR-3.1.1 |
| Depends on | #13, #15 |
| Blocks | #29 |
| Legacy reference | `cage-demo/src/sync/diff.rs` recovery source; `tests/integration/sync_test.rs` |
| Status | Not started |

## Goal

Recover artifacts from the persisted workspace and trusted baseline after a container stops or
crashes, via `cage diff --from-volume` and `cage sync --from-volume`.

## Approach

1. Resolve a recovery source from a session id or session path → `(workspace_volume,
   trusted_baseline_repo, baseline_path_manifest, generation, snapshot_digest)` and verify the
   atomically published `Ready` generation marker from #13 before mounting anything.
2. Mount the workspace read-only and the trusted baseline read-only in the Cage-owned helper from
   #14, with its writable index and object store on tmpfs. Do not mount or trust the agent-local
   `.git` for recovery.
3. Run the shared exact temporary-index diff extraction from #14 and feed the result into #15.
4. Reject a source missing either the workspace or trusted baseline, a `Preparing`/partial
   generation, or a manifest/tree digest mismatch with a clear error. The legacy `.git`-only mount
   (CAGE-BUG-002-A) cannot restore worktree files, and a workspace without its trusted baseline
   cannot prove what changed.
5. Clean up the recovery container idempotently.

## Acceptance criteria → approach

- Workspace + trusted-baseline paths resolvable from session id → resolution + test.
- Host-path recovery works safely → path validation (#7) on the provided path.
- Incomplete workspace/baseline pairs and unpublished/mismatched generations rejected → explicit
  errors + failpoint tests.
- Live tests exist or skip clearly when Docker absent → `CAGE_INTEGRATION_DOCKER` gate.

## QA gate

- Unit: source resolution, incomplete-pair/generation rejection, digest validation, and
  agent-`.git` tamper isolation.
- Live (#18): crash a run, recover diff+sync from volumes.

## Risks & notes

- Recovery volumes must outlive the container; tie their cleanup to session teardown, not container
  exit (otherwise a crash deletes the very data needed for recovery).
