# Issue #8 — Docker & Podman runtime abstraction

| | |
|---|---|
| Issue | https://github.com/Saber5656/cage/issues/8 |
| Phase | 1 |
| Priority | Must |
| Requirements | NFR-5, FR-5.7; SEC-DESIGN-002 |
| Depends on | #4, #7 |
| Blocks | #9, #11, #12, #19, #20 |
| Legacy reference | `cage-demo/src/engine/*`; `docs/security-design-cli-injection.md` |
| Status | Not started |

## Goal

One `ContainerRuntime` trait behind which Docker and Podman are interchangeable, executed via
argument arrays (never a shell).

## Approach

1. **Trait**:
   ```rust
   trait ContainerRuntime {
     async fn create(&self, cfg: &ContainerConfig) -> Result<ContainerId>;
     async fn start(&self, id: &ContainerId) -> Result<()>;
     async fn start_attached(&self, id: &ContainerId) -> Result<ExitStatus>; // TTY: see #12
     async fn stop(&self, id: &ContainerId) -> Result<()>;
     async fn remove(&self, id: &ContainerId) -> Result<()>;
     async fn exec(&self, id: &ContainerId, argv: &[String]) -> Result<Output>;
     async fn cp_to(&self, id: &ContainerId, src: &Path, dst: &str) -> Result<()>;
     async fn cp_from(&self, id: &ContainerId, src: &str, dst: &Path) -> Result<()>;
   }
   ```
2. **Execution**: `tokio::process::Command::new("docker"|"podman").args([...])` — no `sh -c`, ever.
3. **Detection order**: CLI `--podman` → `cage.toml environment.runtime` → `CAGE_RUNTIME` → auto
   (probe `docker version`, then `podman version`). Missing runtime → actionable error (exit 3).
4. Podman is API-compatible with the Docker CLI here; keep divergences (see #19) inside the impls.

## Acceptance criteria → approach

- Unit tests verify create args for both → build arg vectors as pure data, assert on them.
- Runtime-unavailable errors explain cause+action → detection returns `CageError::Container`.
- No shell string concat → enforced structurally + the FR-5.10 lint gate (#18).
- Differences documented → `docs/runtimes.md` (expanded by #19).

## QA gate

- Arg-vector unit tests (no daemon needed) for create/exec/cp on Docker and Podman.
- Detection precedence test with env/flag combinations.

## Risks & notes

- `cp_to`/`cp_from` should support tar-stream on stdin (`cp -`) so #11 (creds) and #12 (project
  copy honoring excludes) can avoid staging files on the host.
