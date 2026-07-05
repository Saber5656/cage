# Issue #25 — Subscription/OAuth agent credentials beyond API keys

| | |
|---|---|
| Issue | https://github.com/Saber5656/cage/issues/25 |
| Phase | 2 |
| Priority | Should (adoption-critical) |
| Requirements | FR-2.6, PRD §11.5; THREAT-AA-I-01 |
| Depends on | #10, #11 |
| Blocks | #26 (credential mechanics) |
| Legacy reference | credential injection in `cage-demo/src/cli/run.rs` |
| Status | Not started |
| Gated by | **Decision D-1** (see issue-approaches/README.md) |

## Goal

Support real-world auth: many users authenticate Claude Code via a claude.ai (Pro/Max)
subscription and Codex via a ChatGPT login — they have **no API key**, so `cage run claude`
currently fails at auth for a large share of users.

## Approach

1. **Decide & document** the per-adapter auth support matrix for pre-alpha (API key / OAuth token
   file / in-container login = supported | unsupported | planned). An explicit "API-key only" is an
   acceptable pre-alpha answer **if documented with clear errors** — that is Decision **D-1**.
2. Extend credential injection (#11) so **file-/directory-backed** credentials (not only env
   values) ride the same tmpfs channel with the same delete-after-start guarantees (e.g. map a host
   `~/.claude` token file into the agent home).
3. Decide whether interactive in-container login (device-code/OAuth run by the agent itself) is
   supported, and what happens to tokens the agent writes inside the container (they die with the
   session unless deliberately persisted).
4. Guarantee no credential material in image layers, logs, `--dry-run`, or `docker inspect`.

## Acceptance criteria → approach

- Per-adapter auth matrix documented → `docs/auth.md`.
- File-based creds inject via tmpfs + deleted after start → reuse #11 with a file source.
- Missing/expired creds → actionable error, exit 4 → auth precheck.
- No credential content observable → masking + inspect/dry-run tests.

## QA gate

- Unit: file-source injection + deletion; auth-error exit code; no secret in dry-run.

## Risks & notes

- Subscription tokens are more sensitive than API keys (broader account scope) — the "never on host
  `/tmp`, never in inspect" rules from #11 are mandatory, not optional, here.
