# Issue #5 — CLI command surface & output conventions

| | |
|---|---|
| Issue | https://github.com/Saber5656/cage/issues/5 |
| Phase | 1 |
| Priority | Must |
| Requirements | FR-5.1–5.6, FR-5.7 (typed parser), NFR-9 |
| Depends on | #4 |
| Blocks | #12 and every command |
| Legacy reference | `cage-demo/src/cli/mod.rs`; `docs/cli-ux/cli-ux-spec.md` |
| Status | Not started |

## Goal

Define the whole command tree with `clap` derive, plus the shared output/exit-code/error
conventions every command reuses.

## Approach

1. **Command tree** (clap derive): `run`, `sync`, `diff`, `team {up,down,status}`,
   `config {get,set,list,edit,validate}`, `images {list,pull,remove,prune}`, `update`.
   Global flags: `--verbose`, `--no-color`, `--version`, `--help`.
2. **Output module** (`cli::output`): every line prefixed `[cage]`; helpers `info/success/warn/error/action`
   with the spec's color mapping. Auto-disable color when `NO_COLOR`, `--no-color`, or non-TTY.
3. **Exit codes** (`cli::exit`): `0` ok, `1` general, `2` security, `3` container, `4` auth, `130` interrupt.
   Map error types → codes centrally (a `CageError` enum with a `.exit_code()`).
4. **Error format** (NFR-9): `summary` → `原因/cause` → `対処/actions` → `→ next`. One constructor so
   every command emits the same shape.
5. Unimplemented commands return an explicit `CageError::NotImplemented` (exit 1), never `panic!`.

## API contract

```rust
#[derive(Parser)] struct Cli { #[command(subcommand)] cmd: Command, /* global flags */ }
enum Command { Run(RunArgs), Sync(SyncArgs), Diff(DiffArgs), Team(TeamArgs), Config(..), Images(..), Update(..) }
trait CommandExit { fn exit_code(&self) -> i32; }
```

## Acceptance criteria → approach

- `cage --help` lists commands → derived from the tree.
- `run/sync/diff --help` ≈ spec → arg definitions mirror `cli-ux-spec.md`.
- Unimplemented → explicit error → `NotImplemented` variant, asserted by test.
- Output & exit codes match docs → `cli::output` + `cli::exit`, unit-tested.

## QA gate

- `assert_cmd` tests: `--help` snapshots, exit code per command, `[cage]` prefix present, `--no-color` strips ANSI.

## Risks & notes

- Keep `agent_args` as clap `last = true` (`-- …`) so agent flags are never parsed by cage
  (prevents flag collisions and injection via option smuggling).
