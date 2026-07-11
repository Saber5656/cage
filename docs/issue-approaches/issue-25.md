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
   values) ride the same no-host-temp, no-writable-layer channel (e.g. import a host `~/.claude`
   token file or OAuth-state directory through an adapter-declared mapping). Keep credential paths
   that the agent reads after launch on session-scoped tmpfs, then remove them on stop/cleanup.
3. Decide whether interactive in-container login (device-code/OAuth run by the agent itself) is
   supported. If supported, the adapter must redirect all generated/refreshed auth state to its
   session-scoped tmpfs destination before login starts; if the tool cannot relocate that state,
   mark interactive login unsupported. Never let agent-created tokens fall back to the container
   writable layer or persisted `agent-state/` volume.
4. Guarantee no credential material in image layers, logs, `--dry-run`, or `docker inspect`.

## Acceptance criteria → approach

- Per-adapter auth matrix documented → `docs/auth.md`.
- File-/directory-backed creds inject via tmpfs and never persist beyond the running session → reuse
  #11 with both source kinds and adapter-declared destinations/lifetimes.
- Missing/expired creds → actionable error, exit 4 → auth precheck.
- No credential content observable → masking + inspect/dry-run tests.
- Interactive login is either tmpfs-confined or explicitly unsupported → adapter capability test.

## QA gate

- Unit: file- and directory-source injection (including OAuth state), one-shot vs session-scoped
  lifecycle, stop-time deletion, interactive-login destination enforcement, auth-error exit code,
  and no source value in dry-run/log output.

## Risks & notes

- Subscription tokens are more sensitive than API keys (broader account scope) — the "never on host
  `/tmp`, never in inspect" rules from #11 are mandatory, not optional, here.
