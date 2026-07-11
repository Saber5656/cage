# Issue #14 — Safely display sandbox changes with `cage diff`

| | |
|---|---|
| Issue | https://github.com/Saber5656/cage/issues/14 |
| Phase | 2 |
| Priority | Must |
| Requirements | FR-3.5 |
| Depends on | #12, #13 |
| Blocks | #15, #24 |
| Legacy reference | `cage-demo/src/cli/diff.rs`, `src/sync/diff.rs` |
| Status | Not started |

## Goal

Show the diff between the container workspace and the Cage baseline without touching local files.

## Approach

1. Resolve container from `--container` or `--session` (#13). For a running source, first persist a
   bounded pause lease and start an independent `cage unpause-watchdog` process; wait for its ready
   signal, then pause before traversal. Renew the lease with acknowledged heartbeats while extraction
   is healthy. If renewal fails or the safety margin is exhausted, cancel the helper, discard its
   entire diff, and wait for/perform unpause before returning an error. Normal completion unpauses and
   clears the lease; controller crash/kill makes the watchdog unpause at the deadline, and every later
   Cage invocation reconciles stale leases. If the runtime cannot pause or provide an equivalent
   snapshot, fail instead of reading a moving tree.
2. Create a short-lived Cage-owned diff helper: mount the stable workspace read-only at `/workspace`
   and `baseline.git/` from #13 read-only at `/baseline.git`. Put the index and new object store on
   helper tmpfs and set the complete trusted Git context explicitly:
   `GIT_DIR=/baseline.git`, `GIT_WORK_TREE=/workspace`, `GIT_INDEX_FILE=/scratch/index`,
   `GIT_OBJECT_DIRECTORY=/scratch/objects`, and
   `GIT_ALTERNATE_OBJECT_DIRECTORIES=/baseline.git/objects`. New blobs and the temporary index are
   writable without modifying the trusted repository. Never read the agent-writable `.git`.
3. Initialize the temporary index with `git read-tree refs/cage/baseline`, then update it from a
   Cage-controlled, NUL-safe leaf-file plan—never call `git add` or pass a directory pathspec. For
   each selected current regular file or symlink, hash the raw content into the temporary object
   store and batch its exact mode/OID/path through `git update-index -z --index-info`; emit an exact
   deletion record for each baseline leaf that is absent or replaced by a directory. Apply all
   removals before topologically ordered additions and reject any final file/path-prefix collision.
   Reject special files. If both baseline and selected-current sets are empty, skip index mutation.
   This includes selected files ignored by `.gitignore`, excludes unselected descendants during
   file↔directory replacement, and never lets Git expand the file set.
4. Extract with `git diff --cached --no-ext-diff --no-textconv refs/cage/baseline` (plus `--binary`
   when #24 lands), then parse unified diff → `FileDiff[]`.
5. Detect `.git/hooks/` paths and path traversal → warn + exclude from any downstream sync.
6. `--unified <n>` → wire to `git diff -U<n>`.

### Quoted/space path parsing (audit gap G-2 / CAGE-BUG-004 — lost fix)

Metadata-only diffs (e.g. new empty files) whose paths contain spaces were silently dropped by the
legacy `split_whitespace()` parse of `diff --git`. Fix: parse git's C-style quoted paths, **or**
run diff with `-c core.quotePath=false` and parse `a/`/`b/` prefixes position-aware. Add tests for
paths with spaces and non-ASCII. Carry over the metadata-only (zero-hunk) handling from legacy
commit `dfa7243`.

### Binary listing (ties to #24)

Binary changes must at least be **listed** as `Binary file changed (<size>)`, never omitted. Full
binary-safe sync is #24.

## Acceptance criteria → approach

- `--container <id>` shows files/insertions/deletions → parse + summary.
- `--session <id>` works via metadata → #13 resolution.
- Traversal diffs rejected → validator on every path.
- Hooks paths warned + excluded from sync → detection + exclusion flag.
- Ignored untracked files selected by Cage filters are visible → exact temporary-index update test.
- Rewriting the agent's `.git` cannot alter the result → trusted-baseline tamper test.
- A running agent cannot race traversal/staging or remain paused after controller death → renewable
  pause lease, fail-closed extraction cancellation, independent watchdog, and startup reconciliation
  tests.

## QA gate

- Unit: parser on real baseline diffs incl. empty files, spaced/non-ASCII paths, deletions
  (`--- a/…` fallback), ignored untracked files, empty selection, file↔directory replacement with
  excluded descendants, special-file rejection, hooks-path detection, traversal rejection, and
  agent-`.git` tamper isolation. Live: a concurrent writer is paused for extraction and resumes on
  success, injected helper failure, lease-renewal failure, and controller `SIGKILL` via the watchdog
  deadline; a slow extraction renews safely, while a failed renewal returns no partial diff.

## Risks & notes

- Keep parsing tolerant but never silently drop a file — an unparseable entry should surface as a
  visible warning, not a missing change.
