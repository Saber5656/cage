# Issue #15 — `cage sync` approval flow & safe patch application

| | |
|---|---|
| Issue | https://github.com/Saber5656/cage/issues/15 |
| Phase | 2 |
| Priority | Must |
| Requirements | FR-3.2, FR-3.3, FR-3.4, FR-3.6 |
| Depends on | #14 |
| Blocks | #16, #17, #24 |
| Legacy reference | `cage-demo/src/cli/sync.rs`, `src/sync/{approval,apply}.rs` |
| Status | Not started |

## Goal

Preview the diff, get per-file user approval, and apply only approved changes to the host — with
path-traversal and hooks defenses at apply time.

## Approach

1. Extract diff (reuse #14).
2. Interactive approval (`dialoguer`): per-file approve/skip; show file summary + hunks.
3. `--auto` (explicit opt-in, CI): approve all **except** the guarded classes (see #17).
4. Build a patch from approved files/hunks; validate with `git apply --check`, then apply the exact
   approved bytes via plain `git apply -` (stdin, no shell). Do not normalize whitespace implicitly;
   any future normalization must be an explicit opt-in whose transformed patch is previewed again.
5. **Defenses at apply (SEC-ISSUE-001)**: reject `..`, absolute paths, and any `.git/hooks/**`
   (hooks defense L2/L3) before writing; nothing is applied outside the project dir.
6. Print synced file count on completion.

## API contract

```rust
struct FileDecision { path, approved: bool, hunks: Vec<HunkDecision> } // zero-hunk = metadata-only, still appliable
fn apply(decisions: &[FileDecision], project: &Path) -> Result<AppliedSummary>;
```

## Acceptance criteria → approach

- Interactive per-file approve/skip → approval flow.
- No auto-apply without `--auto` → default is interactive; non-TTY without `--auto` → E-017 (exit 1).
- `.git/hooks/**` never applied → apply-time reject test.
- Writes outside project rejected → traversal test with `../` targets.
- Applied content is byte-for-byte the approved patch → no implicit whitespace-fix behavior.

## QA gate

- Unit: approval selection → patch subset; apply rejects traversal + hooks; whitespace is preserved;
  metadata-only empty file applies (carry `dfa7243`); synced-count correct.

## Risks & notes

- Metadata-only and (later) binary patches must survive the "approved but zero text hunks" path —
  don't gate application on hunk count.
