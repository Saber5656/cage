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
3. Add `scripts/check-workflow-triggers.sh` (or an equivalent repository script) that parses each
   workflow YAML file and rejects an `on.schedule` key. CI invokes the script by name, so the
   prohibited trigger text is not embedded in a workflow and cannot make the guard match itself.

## Key decisions (defaults taken)

- Scheduled security scans (e.g. weekly `cargo audit`) are **deferred**, not adopted, until the
  test suite is stable — revisit post-alpha.

## Acceptance criteria → approach

- No `on.schedule` in any workflow → external YAML-aware guard + review.
- `security.yml` has no weekly cron → covered by the migration strip in #18.
- Valid YAML → `actionlint` (or a parse step) in CI.

## QA gate

- `scripts/check-workflow-triggers.sh` passes without finding a scheduled trigger.
- `actionlint` passes.

## Risks & notes

- Purely a policy/guard issue; the actual security workflow content is owned by #18.
