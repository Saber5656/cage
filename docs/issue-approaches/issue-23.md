# Issue #23 — Pre-alpha release packaging

| | |
|---|---|
| Issue | https://github.com/Saber5656/cage/issues/23 |
| Phase | 3 |
| Priority | — (policy) |
| Requirements | SEC-ISSUE-002, SEC-ISSUE-003; tech-selection §4 |
| Depends on | #2 |
| Blocks | — |
| Legacy reference | `docs/architecture/tech-selection.md` §4; PRD §11.5 |
| Status | Not started |
| Gated by | **Decision D-5** (channel priority, SBOM/attestation) |

## Goal

Document how future releases will work — without publishing anything yet — and make any release
path tag/manual-gated and checksum-verified.

## Approach

1. **README release policy**: state "no release yet"; document the intended order of
   `cargo install` / GitHub Releases / Homebrew (**Decision D-5**).
2. **Release workflow (when added)**: trigger only on `v*` tags or `workflow_dispatch` — **never** on
   a `main` merge. (Consistent with #3's no-schedule rule.)
3. **Install script (if any)**: must verify SHA256 before executing; `curl | bash` direct execution
   is forbidden (SEC-ISSUE-002/003). Shell-RC alias injection is opt-in only.
4. **Supply chain (D-5)**: decide whether binary checksums, SBOMs, and artifact attestations are
   required for the first release; record the decision.
5. **Secrets posture**: document that publish tokens are **not** stored as repo secrets.

## Acceptance criteria → approach

- README states no-release + release policy → README section.
- Release workflow uses only manual/tag gates → `on: { push: { tags: ['v*'] }, workflow_dispatch: }`.
- Install script verifies SHA256 → checksum step (or no script).
- Publish tokens not stored as repo secrets → documented.

## QA gate

- `actionlint` on any release workflow; a review checklist confirms no `main`-merge publish path.

## Risks & notes

- This is a policy/doc issue for pre-alpha; actual publishing waits until the security regression
  suite (#18) and platform matrix (#28) are green.
