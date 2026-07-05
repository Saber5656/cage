# Issue #6 — Two-level `cage.toml` configuration loader

| | |
|---|---|
| Issue | https://github.com/Saber5656/cage/issues/6 |
| Phase | 1 |
| Priority | Must |
| Requirements | FR-5.5, FR-5.9; THREAT-CLI-S-01, THREAT-CLI-E-01, THREAT-SEC-T-01, THREAT-CLI-D-01 |
| Depends on | #4 |
| Blocks | #10, #12, #21 |
| Legacy reference | `cage-demo/src/config/{mod,loader,profile}.rs`, `cage.toml.example` |
| Status | Not started |

## Goal

Merge global (`~/.config/cage/cage.toml`) and project (`./cage.toml`) config with project winning,
falling back to safe defaults — and treat config values as untrusted input.

## Approach

1. Structs: `CageConfig { environment, profiles, sync, adapters, defaults }`,
   `EnvironmentConfig`, `ProfileConfig { image, memory, cpus, env, gpus, with_hooks, dind }`,
   `SyncConfig { include, exclude }`, `CustomAdapterConfig`.
2. Resolution order (highest wins): CLI flag → project → global → built-in default.
3. Invalid TOML → typed error (exit 1), **never panic** (fuzz-safe deserialize).

### Security requirements (from audit / STRIDE)

- **Validate config values with the same validators as CLI args (FR-5.9)**: paths →
  realpath/NUL/control-char checks (#7); image names → `^[A-Za-z0-9._\-/:@]+$`; env entries are
  **names**, not values.
- **Hardening cannot be weakened via config (THREAT-CLI-E-01/SEC-T-01)**: reject or ignore-with-warning
  any hardening-adjacent key (`privileged`, `cap_add`, `security_opt`, …); the Security Layer (#9)
  always assembles the final flag set.
- **First-run trust confirmation (FR-5.9 / THREAT-CLI-S-01)**: on first load of a project config
  (and whenever its content hash changes), print the effective settings and require confirmation;
  cache the acknowledgement under `$CAGE_HOME`. Non-interactive/`--auto` runs fail closed if unacknowledged.
- **DoS guard (THREAT-CLI-D-01)**: cap file size (e.g. 1 MiB) with a clear error.

## Acceptance criteria → approach

- Global-only / project-only / merged tested → three fixture tests.
- Profile `memory/cpus/env/image` resolvable → `ProfileConfig` + merge test.
- `environment.runtime` feeds runtime selection → exposed to #8's detection.
- Invalid TOML never panics → `Result`-returning loader + fuzz/negative test.

## QA gate

- Unit tests for merge precedence, malicious values (`image = "ubuntu; curl…"` rejected), oversized file, hash-change re-prompt.

## Risks & notes

- The trust-confirmation cache key must be `(project path, content hash)` so editing the file
  re-prompts but an unchanged file does not nag.
