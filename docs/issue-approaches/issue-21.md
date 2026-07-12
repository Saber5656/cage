# Issue #21 — Practical `config`, `images`, `update` commands

| | |
|---|---|
| Issue | https://github.com/Saber5656/cage/issues/21 |
| Phase | 3 |
| Priority | Should |
| Requirements | FR-5.5, FR-5.6, PRD §11.5 |
| Depends on | #5, #6 |
| Blocks | #27 (images build) |
| Legacy reference | `cage-demo/src/cli/{config,images,update}.rs` (all stubs) |
| Status | Not started |

## Goal

Replace the three `not yet implemented` stubs with real behavior, with confirmation on destructive ops.

## Approach

1. **`cage config`**: `list` and `get <key>` show effective merged values plus their source layer.
   Mutating `set <key> <value>` and `edit` require exactly one of `--global` (writes
   `~/.config/cage/cage.toml`) or `--local` (writes the project `cage.toml`), validate before an
   atomic write, and report when the other layer still wins. Reuse #6's loader + validators.
2. **`cage images`**: label every Cage-built image `io.cage.managed=true`; `list` shows managed
   images with size/created, while `pull <image>` and `remove <image>` operate on explicit names.
   `prune` first lists IDs with the positive Cage label and removes only those IDs—never delegate to
   an unfiltered runtime-wide prune. `remove`/`prune` require confirmation (`--yes`/`-y` to skip).
3. **`cage update`**: for pre-alpha, a no-op with guidance ("no releases yet; build from source")
   is acceptable per the issue — implement `--check` to report and otherwise print guidance. Full
   self-update is deferred to the release work (#23).
4. Errors + output follow the CLI UX spec (#5).

## Acceptance criteria → approach

- Stubs replaced with concrete behavior → per-subcommand impls.
- `images remove`/`prune` need confirmation → prompt unless `--yes`.
- Config mutations select a layer explicitly and reads identify the winning layer.
- Image prune cannot remove unrelated runtime images → positive-label filtering + ID allowlist.
- Pre-alpha `update` gives clear guidance → guidance path + `--check`.
- Help matches behavior → derived help reviewed against docs.

## QA gate

- Unit/`assert_cmd`: global/local config round-trip and project-shadow reporting; images confirmation
  prompt; an unrelated dangling image survives prune; update guidance text.

## Risks & notes

- `images build` (bake) is **not** here — it's introduced by #27, which builds on `images`.
