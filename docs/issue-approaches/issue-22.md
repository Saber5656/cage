# Issue #22 — `cage team` up/down/status MVP

| | |
|---|---|
| Issue | https://github.com/Saber5656/cage/issues/22 |
| Phase | 3 |
| Priority | Must (PRD) / MVP scope under review |
| Requirements | FR-4.1–4.4, FR-5.4 |
| Depends on | #12 |
| Blocks | — |
| Legacy reference | `cage-demo/src/cli/team.rs` + `src/team/*` (stubs) |
| Status | Not started |
| Gated by | **Decision D-3** (workspace model) |

## Goal

Sandboxed multi-agent lifecycle: start/stop/status of a team of agent containers sharing a
workspace and an internal network. Conflict resolution and per-agent permissions stay out of scope.

## Approach

**Resolve Decision D-3 first** — this is the reason `src/team/*` stayed a stub. FR-3 assumes one
workspace + one baseline per session, but FR-4.2's shared team volume breaks that (multiple agents
mutate one tree; whose baseline does `sync --team` diff against?). Pick and record:
- **(a)** one shared workspace volume + a single team-level baseline; `sync --team` → one unified diff; or
- **(b)** per-agent workspaces + an explicitly shared directory excluded from sync.

Then:
1. **Team config schema** (`cage-team.toml`): agents (name, adapter/profile), shared volume, network.
   Parse + validate.
2. **`team up`**: create the shared volume + `cage-net-<team>` network, start each agent container
   (reusing #11 credential injection per container — no shared credential volume), print status.
3. **`team status`**: table of name / container id / state / uptime / resource usage; flag stopped agents.
4. **`team down`**: stop + remove containers and the network. Preserve the shared workspace volume
   by default; remove it only with explicit `--remove-volumes`, with `--force` available to skip the
   destructive confirmation.
5. **Naming/collision (audit gap G-20)**: use the common runtime-name helper keyed by a
   Cage-generated collision-resistant id, rather than deriving names from agent/team labels, so two
   teams (or a team + solo runs) on one host cannot collide.
6. **Resource multiplication (THREAT-TM-D-01/02)**: `up` prints aggregate limits and warns when the
   sum exceeds host memory.
7. **`cage sync --team`** scope: define per D-3 or return an explicit unimplemented error.

## Acceptance criteria → approach

- Team config parsing + validation tests → schema + tests.
- `up/down/status` functional (not stubs) → impls above.
- Shared workspace responsibilities documented → `docs/teams.md` (D-3 outcome).
- `sync --team` supported scope clear → documented or explicit error.

## QA gate

- Unit: schema validation, naming collisions, aggregate-limit warning.
- Live: up→status→down leaves no containers/network but preserves the workspace volume; a second
  teardown with `--remove-volumes` removes that volume and leaves nothing.

## Risks & notes

- Without D-3, any implementation risks a second stubbed subsystem. Do not start coding until the
  workspace model is signed off.
