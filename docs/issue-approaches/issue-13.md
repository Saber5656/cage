# Issue #13 — Session persistence & trusted Cage baseline ref

| | |
|---|---|
| Issue | https://github.com/Saber5656/cage/issues/13 |
| Phase | 1 |
| Priority | Must |
| Requirements | FR-3.1, FR-3.1.1; architecture §3.4 |
| Depends on | #12 |
| Blocks | #14, #16, #29 |
| Legacy reference | `cage-demo/src/session/*`, `src/cli/run.rs` git/session helpers |
| Status | Not started |

## Goal

Persist each session's workspace and agent-local `.git` as Cage-managed volumes, while keeping the
trusted `refs/cage/baseline` outside the agent's write boundary, so diff/sync work after a stop or
crash without trusting Git metadata the agent can rewrite.

## Approach

1. **Layout**: `$CAGE_HOME/sessions/<id>/{meta.toml, workspace/, agent-git/, agent-state/,
   baseline.git/, baseline-paths.nul}`. Mount `workspace/` → `/workspace` (RW), `agent-git/` →
   `/workspace/.git` (RW), and optional `agent-state/` only at an adapter-declared non-credential
   conversation-state path. `baseline.git/` and its filtered path manifest are host-owned and never
   mounted into the agent. Credential/auth-state paths are separate tmpfs mounts; no credential
   material or credential directory is persisted in the session tree.
2. **`$CAGE_HOME` default (audit gap — make canonical here)**: `~/.cage`, overridable by
   `CAGE_HOME`. Document it in one place; PRD/architecture never pinned it.
3. **Trusted baseline transaction**: write `SessionStatus::Preparing`, then use one
   Cage-controlled filtered snapshot to populate the workspace and create `refs/cage/baseline` plus
   `baseline-paths.nul`. Persist a `snapshot_digest`, verify the workspace manifest and baseline tree
   against it, then atomically publish a generation commit marker and `SessionStatus::Ready` before
   agent launch. A stale/incomplete `Preparing` generation is never eligible for diff/resume and is
   cleaned or reported for recovery. The baseline is always the host tree at session start, never an
   empty-tree fallback, and agent-writable Git objects/refs are ignored.
4. **Metadata/run recipe**: `SessionMeta { id, generation, snapshot_digest, created_seq,
   created_at_utc, project_path, container_id, runtime, image_digest, adapter_id, adapter_version,
   resolved_argv, bootstrap_schema, profile_fingerprint, workspace_volume, agent_git_volume,
   agent_state_volume, status }` with round-trip persist/load. The recipe contains no credential
   values; command arguments that would embed secrets are forbidden. `SessionStatus { Preparing,
   Ready, Running, Stopped, Crashed, Synced }`.
5. **Latest-session resolution (audit gap G-18)**: allocate `created_seq` atomically under a
   `$CAGE_HOME` lock and select the maximum `(created_seq, id)` after project filtering. Use
   `created_at_utc` only for display. Interactive mode lists candidates when the project itself is
   ambiguous; non-interactive mode then requires `--session`.
6. **Git edge cases (audit gap G-17 / CAGE-BUG-001)**: keep rejecting dangerous layouts (root
   `.git` worktree gitdir files, in-progress rebase, stale locks) but emit an explicit
   "unsupported Git layout" error + remediation. **Submodules & git-lfs are out of scope for
   pre-alpha** — reject/warn rather than produce surprising diffs.

## Acceptance criteria → approach

- Session metadata round-trips → persist/load test.
- Latest session resolvable → persisted sequence + concurrent-creation/ambiguity tests.
- Trusted baseline records the host tree before launch and is inaccessible from the agent → step 3
  + mount-allowlist/tamper tests.
- Incomplete/mismatched generations are never published or selected → transaction failpoint tests.
- Session persistence contains a reconstructible non-secret run recipe and adapter conversation
  state, but no credential/auth-state path or secret bytes → layout/serialization tests.
- Dangerous `.git`/worktree gitdir rejected → step 6 with explicit error message.

## QA gate

- Unit: meta round-trip, atomic sequence/latest resolution (incl. concurrent creation), trusted
  baseline transaction failpoints/digest validation, mount allowlist/no-credentials layout,
  run-recipe reconstruction, agent-ref tamper isolation, and layout rejection messages.

## Risks & notes

- Treat the agent-local repository as untrusted session state. A deleted or rewritten agent ref must
  not change the trusted baseline or suppress a host-visible diff.
