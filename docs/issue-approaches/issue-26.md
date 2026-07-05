# Issue #26 — Host-context forwarding: settings sync, `--with-ssh`, `--with-hooks`

| | |
|---|---|
| Issue | https://github.com/Saber5656/cage/issues/26 |
| Phase | 2 |
| Priority | Should |
| Requirements | PRD §11.5; SEC-ISSUE-004, SEC-ISSUE-006; THREAT-AA-I-02, THREAT-AA-I-03 |
| Depends on | #11, #25 |
| Blocks | #28 (platform notes) |
| Legacy reference | `cage-demo/src/cli/run.rs:38-44` (parse-only); `docs/cli-ux/cli-ux-spec.md` §3.6-3.8 |
| Status | Not started |
| Gated by | **Decision D-2** (implement vs remove) |

## Goal

Bring selected host context into the sandbox safely. Today both flags are **parse-only** (a lost
bug, CAGE-BUG-006) — worse than absent, because users think a security feature is active when it is not.

## Approach

Decide **D-2** first: implement per spec, or remove the flags and fail with an explicit
"not supported yet" error. No parse-only flags may remain. If implemented:

1. **Host settings sync (§11.5)**: opt-in per-file mappings of `~/.claude/`
   (`settings.json`, `CLAUDE.md`, `plugins/`, `commands/`, `agents/`). `hooks/` **excluded by default**
   (SEC-ISSUE-006). Built-ins reuse the custom-adapter `config_files` mechanism.
2. **`--with-ssh` (SEC-ISSUE-004)**: forward only the **agent socket**, never private keys.
   Platform note: on Docker Desktop (macOS) the host `$SSH_AUTH_SOCK` can't be bind-mounted directly
   — use the documented magic path `/run/host-services/ssh-auth.sock`; on Linux forward the socket
   path as-is. Error E-018 when no agent is running. Document per-platform support (feeds #28).
3. **`--with-hooks` (SEC-ISSUE-006)**: preview hook contents (first 20 lines per spec), require
   confirmation, reject binary hooks (E-019). `cage.toml with_hooks = true` to always enable.
4. Credential-bearing files (e.g. `~/.claude` OAuth tokens) reuse #25's injection mechanics; this
   issue owns the host→container **mapping**, not the secret channel.

## Acceptance criteria → approach

- Flags work as specified **or** fail with explicit unimplemented errors → D-2.
- SSH private keys never copied → socket-only forwarding, asserted.
- Hooks sync needs preview+confirm, rejects binaries → §3.7 flow + E-019.
- Synced host files per adapter documented → `docs/host-context.md`, hooks default-off.

## QA gate

- Unit: hooks preview/confirm/binary-reject; settings mapping; ssh socket-only (no key path).
- Manual (per platform, #28): agent socket forwarding on macOS + Linux.

## Risks & notes

- `--with-ssh` gives the sandbox use of the user's SSH identity — keep it default-off and
  confirmation-gated; document the blast radius plainly.
