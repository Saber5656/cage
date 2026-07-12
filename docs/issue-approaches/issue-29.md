# Issue #29 — `cage run --continue` session resume

| | |
|---|---|
| Issue | https://github.com/Saber5656/cage/issues/29 |
| Phase | 3 |
| Priority | — |
| Requirements | PRD §11.5; cli-ux-spec `--continue` |
| Depends on | #11, #13, #16 |
| Blocks | — |
| Legacy reference | `cage-demo/src/session/` SessionStatus lifecycle |
| Status | Not started |

## Goal

Resume the previous conversation in the same directory (modeled on claude-docker `--continue`),
or fail explicitly — never a silent no-op like the legacy parse-only flags.

## Approach

Decide pre-alpha stance: minimal resume vs explicit defer (either is fine; must be explicit). If implemented:
1. Resolve the latest session for the project (reuse #13 rules, incl. concurrent-session ambiguity
   handling: interactive lists candidates, non-interactive requires `--session`).
2. Restart the stopped bootstrap container, **or** recreate a removed container from #13's persisted
   non-secret run recipe and pinned image digest, mounting workspace, agent-git, and adapter
   conversation-state volumes read-write. Do not reuse #16's read-only diff-recovery helper. Refuse
   resume with an actionable error when the recipe, compatible bootstrap schema, state volume, or
   pinned image is unavailable.
3. **Re-inject credentials**: one-shot environment files were deleted before first launch, and
   session-scoped credential tmpfs content does not survive stop/recreation (FR-2.6), so resume runs
   the full #11 injection path again with the same guarantees.
4. Pass the agent's own resume flag through the adapter (e.g. `claude --continue`).
5. Define interaction with `--cleanup` (a cleaned session can't be resumed) and crashed sessions
   (resume vs `sync --from-volume` recovery).

## Acceptance criteria → approach

- `--continue` resumes latest **or** explicit unimplemented error — never silent no-op.
- Credential re-injection on resume keeps first-start guarantees → reuse #11.
- Removed-container resume reconstructs the adapter conversation from the pinned non-secret recipe
  and state volumes → recreation test; missing recipe/image/state fails explicitly.
- `--continue` with no prior session → clear error, not a panic.
- Interaction with `--cleanup` + crashed sessions documented → `docs/run-lifecycle.md`.

## QA gate

- Unit: no-prior-session error; resume resolves latest; recipe validation/reconstruction;
  `--cleanup` conflict message.
- Live: run → stop → `--continue` reattaches with creds working; remove the container and verify
  recipe-based recreation preserves conversation state without persisting auth state.

## Risks & notes

- Resume is where ephemeral credential state (#11) and "resume needs it again" collide — the only
  correct answer is re-injection from approved host sources, never persisting secrets in the session.
