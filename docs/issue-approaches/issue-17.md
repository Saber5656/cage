# Issue #17 — Sync include/exclude filters & sensitive-file warnings

| | |
|---|---|
| Issue | https://github.com/Saber5656/cage/issues/17 |
| Phase | 2 |
| Priority | Should |
| Requirements | FR-3.7; THREAT-AS-I-01, THREAT-AS-E-01, THREAT-AS-D-01 |
| Depends on | #15 |
| Blocks | — |
| Legacy reference | `cage-demo/src/sync/diff.rs` |
| Status | Not started |

## Goal

Apply `cage.toml [sync] include/exclude` to diff & sync, warn on sensitive/executable changes, and
guard against oversized diffs.

## Approach

1. Apply glob include/exclude from config to the file set (diff + sync).
2. **Precedence**: CLI `--file` filters **intersect (narrow)** the config result — never widen.
   Sensitive-file warnings apply **after** all filtering.
3. **Sensitive patterns**: warn on `.env`, `*.key`, `*.pem`, and similar.
4. **Executable bit added** → require explicit confirmation (THREAT-AS-E-01).
5. **`--auto` interaction (audit gap G-19)**: under `--auto`, sensitive-pattern and exec-bit changes
   are **skipped with a warning** by default, applied only with an extra opt-in
   (`--auto-include-sensitive`). Non-interactive **without** `--auto` still fails per E-017.
6. **Large-diff thresholds (THREAT-AS-D-01/CE-D-02)**: configurable under `[sync]` (defaults e.g.
   warn > 5 MB/file or > 500 files); state whether they block under `--auto`.

## Acceptance criteria → approach

- Include/exclude unit-tested → glob matcher tests.
- `--file` narrows as expected → intersection test.
- Sensitive changes warn → pattern matcher + warning.
- Executable sync needs confirmation → prompt (interactive) / skip-unless-opt-in (`--auto`).

## QA gate

- Unit: glob include/exclude, `--file` intersection, sensitive/exec detection, `--auto` skip behavior, size thresholds.

## Risks & notes

- Default excludes should ship sane (`target/**`, `node_modules/**`, `.env`) so a first-time user
  doesn't accidentally sync build junk or secrets.
