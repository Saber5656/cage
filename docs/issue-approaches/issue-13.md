# Issue #13 — Session persistence & Cage baseline ref

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

Persist each session's workspace + `.git` as cage-managed volumes and a `refs/cage/baseline` ref,
so diff/sync work even after the container stops or crashes.

## Approach

1. **Layout**: `$CAGE_HOME/sessions/<id>/{meta.toml, workspace/, git/, credentials/}`.
   Mount `workspace/` → `/workspace` (RW) and `git/` → `/workspace/.git` (RW).
2. **`$CAGE_HOME` default (audit gap — make canonical here)**: `~/.cage`, overridable by
   `CAGE_HOME`. Document it in one place; PRD/architecture never pinned it.
3. **Baseline ref**: after `git init`, create `refs/cage/baseline` (host tree or empty tree) so
   `git diff refs/cage/baseline` (with `git add -N .`) surfaces new/untracked files.
4. **Metadata**: `SessionMeta { id, project_path, container_id, workspace_volume, git_volume,
   status }` with round-trip persist/load; `SessionStatus { Running, Stopped, Crashed, Synced }`.
5. **Latest-session resolution (audit gap G-18)**: deterministic (newest by monotonic timestamp in
   meta). Interactive: list candidates when ambiguous; non-interactive: require `--session`.
6. **Git edge cases (audit gap G-17 / CAGE-BUG-001)**: keep rejecting dangerous layouts (root
   `.git` worktree gitdir files, in-progress rebase, stale locks) but emit an explicit
   "unsupported Git layout" error + remediation. **Submodules & git-lfs are out of scope for
   pre-alpha** — reject/warn rather than produce surprising diffs.

## Acceptance criteria → approach

- Session metadata round-trips → persist/load test.
- Latest session resolvable → resolution fn + ambiguity test.
- Baseline ref created after `git init` → step 3 + test.
- Dangerous `.git`/worktree gitdir rejected → step 6 with explicit error message.

## QA gate

- Unit: meta round-trip, latest resolution (incl. concurrent-session ambiguity), baseline creation, layout rejection messages.

## Risks & notes

- The baseline choice (host tree vs empty tree) determines whether `diff` shows agent changes vs
  whole tree — must be the host tree at session start for meaningful diffs.
