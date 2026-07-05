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

1. **`cage config`**: `list` (effective merged config), `get <key>`, `set <key> <value>` (writes
   global `~/.config/cage/cage.toml`), `edit` (`$EDITOR`), `validate`. Reuse #6's loader + validators.
2. **`cage images`**: `list` (size/created), `pull <image>`, `remove <image>`, `prune`. `remove`/`prune`
   **require confirmation** (`--yes`/`-y` to skip). Delegates to the runtime abstraction (#8).
3. **`cage update`**: for pre-alpha, a no-op with guidance ("no releases yet; build from source")
   is acceptable per the issue — implement `--check` to report and otherwise print guidance. Full
   self-update is deferred to the release work (#23).
4. Errors + output follow the CLI UX spec (#5).

## Acceptance criteria → approach

- Stubs replaced with concrete behavior → per-subcommand impls.
- `images remove`/`prune` need confirmation → prompt unless `--yes`.
- Pre-alpha `update` gives clear guidance → guidance path + `--check`.
- Help matches behavior → derived help reviewed against docs.

## QA gate

- Unit/`assert_cmd`: config get/set/list round-trip; images confirmation prompt; update guidance text.

## Risks & notes

- `images build` (bake) is **not** here — it's introduced by #27, which builds on `images`.
