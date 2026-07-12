# Issue #10 — Agent Adapter trait & built-in adapters

| | |
|---|---|
| Issue | https://github.com/Saber5656/cage/issues/10 |
| Phase | 1 |
| Priority | Must |
| Requirements | FR-2.1–2.5, FR-2.7, NFR-8 |
| Depends on | #6 |
| Blocks | #11, #12, #25, #27 |
| Legacy reference | `cage-demo/src/adapter/*`, `cage.toml.example` |
| Status | Not started |

## Goal

A uniform interface that lets cage launch any agent CLI, with built-ins for Claude, Codex,
Gemini, aider, and custom adapters from `cage.toml`.

## Approach

1. **Trait**:
   ```rust
   trait AgentAdapter {
     fn name(&self) -> &str;
     fn default_image(&self) -> &str;
     fn entrypoint(&self) -> Vec<String>;
     fn required_env(&self) -> Vec<String>;      // NAMES only, never values
     fn npm_package(&self) -> Option<NpmPackage>; // { name, version: Option }
     fn image_prereqs(&self) -> &[&str];          // e.g. ["git", "node"]
   }
   fn resolve(name: &str, cfg: &CageConfig) -> Result<Box<dyn AgentAdapter>>;
   ```
2. **Built-ins**: claude=`@anthropic-ai/claude-code`/`ANTHROPIC_API_KEY`; codex=`@openai/codex`/`OPENAI_API_KEY`;
   gemini=`@google/gemini-cli`/`GOOGLE_API_KEY`; aider=image `paulgauthier/aider:latest`.
3. **Custom adapters** from `cage.toml [adapters.<name>]` (image, command, env, config_files).
4. **`--agent-version <ver>`** (PRD §11.5) → pins `npm_package.version`; consumed by image bake (#27).
5. **Image prerequisites (audit gap G-14)**: adapters declare needs (`git`, `node`); #12 verifies
   them at setup with a clear error instead of a mid-session failure.
6. **NFR-8**: add `docs/adapters.md` describing how to add an adapter (trait impl + custom config).

## Acceptance criteria → approach

- `resolve("claude")` etc. return built-ins → registry + tests.
- Custom adapters resolvable from `cage.toml` → `resolve` reads `[adapters.*]`.
- Required env treated as names → `required_env()` returns names; values injected by #11.
- Unknown agents → clear error → `CageError::UnknownAgent` with the known list.

## QA gate

- Unit tests: each built-in's fields; custom-adapter resolution; unknown-agent error; docs/adapters.md exists.

## Risks & notes

- Auth beyond API keys (Claude Pro/Max OAuth, Codex ChatGPT login) is **out of scope here** and
  owned by #25; this issue only guarantees required env **names** are declared.
