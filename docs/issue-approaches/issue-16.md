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

Recover artifacts from the persisted workspace + `.git` volumes after a container stops or crashes,
via `cage diff --from-volume` and `cage sync --from-volume`.

## Approach

1. Resolve recovery source from a session id or a workspace path → `(workspace_volume, git_volume)`.
2. **Mount both** in a throwaway container: `/repo` (workspace) + `/repo/.git` (git). The legacy
   `.git`-only mount (CAGE-BUG-002-A) could not restore worktree files — mount both.
3. Run `git diff refs/cage/baseline` inside that container; feed the result into the same diff/sync
   pipeline (#14/#15).
4. **Reject git-only sources** (no workspace) with a clear error — you cannot recover working-tree
   files from `.git` alone.
5. Clean up the recovery container idempotently.

## Acceptance criteria → approach

- Workspace + git paths resolvable from session id → resolution + test.
- Host-path recovery works safely → path validation (#7) on the provided path.
- Git-only paths rejected → explicit error + test.
- Live tests exist or skip clearly when Docker absent → `CAGE_INTEGRATION_DOCKER` gate.

## QA gate

- Unit: source resolution, git-only rejection.
- Live (#18): crash a run, recover diff+sync from volumes.

## Risks & notes

- Recovery volumes must outlive the container; tie their cleanup to session teardown, not container
  exit (otherwise a crash deletes the very data needed for recovery).
