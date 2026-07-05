# Issue #24 — Make `cage diff`/`sync` binary-safe (restore lost fix)

| | |
|---|---|
| Issue | https://github.com/Saber5656/cage/issues/24 |
| Phase | 2 |
| Priority | P1 (bug) |
| Requirements | FR-3.2, FR-3.3, FR-3.4 |
| Depends on | #14, #15 |
| Blocks | — |
| Legacy reference | `cage-demo/src/sync/diff.rs:163-171, 192-200`; Vault TSK-1325 / CAGE-BUG-005 |
| Status | Not started (regression from repo recreation) |

## Goal

Let binary artifacts produced in the sandbox be diffed and synced. The legacy extraction used
plain `git diff` (no `--binary`), so `git apply` failed with
`cannot apply binary patch … without full index line`. This bug issue was lost when the public
repo was recreated on 2026-07-04; this restores it.

## Approach

1. Extract with `git diff --binary` so patches carry full index data for binary files.
2. Apply binary patches through the existing `git apply` path (stdin), keeping hooks/traversal
   validation unchanged.
3. **Preview**: show binary changes as a summary line (`Binary file changed (1.2 MiB)`), not hunks;
   approval stays file-level.
4. **Size warning** for very large binary artifacts (shares thresholds with #17 / THREAT-AS-D-01).

## Acceptance criteria → approach

- Binary add/modify/delete approvable + syncable → `--binary` extraction + apply test.
- Text diff/sync unchanged → existing tests still green.
- Previews don't dump raw content → summary line for binary.
- Oversized binaries warn before approval → threshold check.

## QA gate

- Unit/live: add a binary file in-sandbox, approve, apply to host; assert byte-identical result.

## Risks & notes

- Quoted/space-path parsing is **separate** (#14). Keep this issue about binary content only.
