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

1. Resolve container from `--container` or `--session` (#13).
2. Extract: `exec git add -N . && git diff refs/cage/baseline` (container-side; the trust boundary
   is documented — the container's git is not trusted, mount points are controlled).
3. Parse unified diff → `FileDiff[]`.
4. Detect `.git/hooks/` paths and path traversal → warn + exclude from any downstream sync.
5. `--unified <n>` → wire to `git diff -U<n>`.

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

## QA gate

- Unit: parser on real baseline diffs incl. empty files, spaced/non-ASCII paths, deletions
  (`--- a/…` fallback), hooks-path detection, traversal rejection.

## Risks & notes

- Keep parsing tolerant but never silently drop a file — an unparseable entry should surface as a
  visible warning, not a missing change.
