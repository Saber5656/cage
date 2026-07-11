# Issue #1 — Canonicalize repo & confirm migration scope

| | |
|---|---|
| Issue | https://github.com/Saber5656/cage/issues/1 |
| Phase | 0 |
| Priority | P0 |
| Requirements | PRD (whole), P-6 |
| Depends on | — |
| Blocks | #2, #4 (and transitively all implementation) |
| Legacy reference | `cage-demo/` (entire tree) |
| Status | Not started |

## Goal

Establish the repository root for `Saber5656/cage` as the canonical working tree, and produce a
written inventory of what migrates from `cage-demo` and what is deliberately excluded.

## Approach

1. Add `docs/migration/inventory.md` classifying every `cage-demo` path into one of:
   `migrate-as-is`, `migrate-with-changes`, `reference-only`, `exclude`.
   - `src/**`, `tests/**`, `seccomp/**`, `cage.toml.example` → migrate-with-changes (carry the
     bug fixes and audit addenda; do **not** paste verbatim).
   - `docs/**` → migrate the PRD + architecture + security design as the in-repo spec.
   - `.github/workflows/security.yml` → migrate **without** the `schedule` trigger (see #3).
   - Local scratch / session artifacts / `target/` → exclude.
2. Record, per migrated area, which follow-up issue owns it (traceability table).
3. State in the repo (README or this doc) that `cage-demo` is legacy reference only.

## Key decisions (defaults taken)

- Migration is **staged by issue**, not a single bulk import PR — each subsystem lands with its
  own tests through its owning issue.
- Commit history of `cage-demo` is **not** imported (clean-room start; avoids leaking any
  local paths/secrets from old history).

## Acceptance criteria → approach

- README/note names Cage as canonical → add the statement in `docs/migration/inventory.md` and link from README.
- Migration inventory lists move/exclude → the classification table above.
- Scheduled workflow excluded → asserted in inventory, enforced by #3.
- Follow-up dependencies documented → traceability column mapping paths → issues.

## QA gate

- The external YAML-aware guard from #3 passes over `.github/workflows`.
- Inventory reviewed against `cage-demo` tree so no source file is silently dropped.

## Risks & notes

- The biggest risk is bulk-pasting legacy `src` and re-importing already-fixed bugs
  (CAGE-BUG-001..007). The inventory must point each area at the issue that carries its fix.
