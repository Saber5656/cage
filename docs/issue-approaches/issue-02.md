# Issue #2 — OSS public repository baseline documents

| | |
|---|---|
| Issue | https://github.com/Saber5656/cage/issues/2 |
| Phase | 0 |
| Priority | P0 |
| Requirements | PRD P-6; SEC-ISSUE-002/003 (release posture) |
| Depends on | #1 |
| Blocks | #23 |
| Legacy reference | `cage-demo/Cargo.toml` (`MIT OR Apache-2.0`) |
| Status | Not started |

## Goal

Add the baseline documents a public security tool needs so readers correctly judge its
maturity and know how to report vulnerabilities.

## Approach

1. **README**: state `pre-alpha`, `experimental`, `not production ready`, `no release yet`.
   One-paragraph "what it is" + "what it is not (yet)". Link to the security policy.
2. **LICENSE**: dual `MIT OR Apache-2.0` → add `LICENSE-MIT` and `LICENSE-APACHE` (standard
   dual-license layout) and reference both from `Cargo.toml` (already declared).
3. **SECURITY.md**: private reporting path (GitHub private vulnerability reporting), supported
   versions table (pre-alpha → "no supported release; report against `main`"), response SLA.
4. **CONTRIBUTING.md**: issue/PR expectations, required checks (fmt/clippy/test), the
   "security-sensitive changes need owner review" rule.
5. **.github/CODEOWNERS**: assign `/.github/`, `SECURITY.md`, `seccomp/`, and `src/security/`
   to the maintainer so those paths require owner review.

## Key decisions (defaults taken)

- Dual license kept as declared in legacy `Cargo.toml`.
- Use GitHub-native private vulnerability reporting rather than an email inbox (no address to leak/rotate).

## Acceptance criteria → approach

- README pre-alpha + no-release → README wording above.
- LICENSE at root → dual license files.
- Security reporting path → SECURITY.md.
- CODEOWNERS covers workflows → CODEOWNERS entry for `/.github/`.

## QA gate

- GitHub renders SECURITY.md in the Security tab; CODEOWNERS validates (no unknown owners).
- Links in README resolve.

## Risks & notes

- Keep SECURITY.md honest: a pre-alpha sandbox must not imply hardening guarantees it has not
  yet regression-tested (#18).
