# Issue #3 — Keep CI limited to push and pull_request triggers

| | |
|---|---|
| Issue | https://github.com/Saber5656/cage/issues/3 |
| Phase | 0 |
| Priority | P0 |
| Requirements | NFR-3 |
| Depends on | — |
| Blocks | #18 |
| Legacy reference | `cage-demo/.github/workflows/security.yml` |
| Status | Not started |

## Goal

Ensure every workflow runs only on `push` and `pull_request` — no `schedule`/`cron` — so the
public repo has no unattended, noisy, or cost-accruing runs while tests are incomplete.

## Approach

1. When workflows are introduced (CI in #4, security in #18), use only:
   ```yaml
   on:
     push: { branches: [main] }
     pull_request:
   ```
2. Strip any `schedule:` block carried from `cage-demo/security.yml` during migration.
3. Add a tiny CI guard step (or a `deny.toml`/grep check) that fails if any workflow file
   contains `schedule:` / `cron:`.

## Key decisions (defaults taken)

- Scheduled security scans (e.g. weekly `cargo audit`) are **deferred**, not adopted, until the
  test suite is stable — revisit post-alpha.

## Acceptance criteria → approach

- No `on.schedule` in any workflow → grep guard + review.
- `security.yml` has no weekly cron → covered by the migration strip in #18.
- Valid YAML → `actionlint` (or a parse step) in CI.

## QA gate

- `rg -n "schedule:|cron:" .github/workflows` → empty.
- `actionlint` passes.

## Risks & notes

- Purely a policy/guard issue; the actual security workflow content is owned by #18.
