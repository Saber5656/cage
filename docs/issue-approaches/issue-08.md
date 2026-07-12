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
     async fn start_attached(&self, id: &ContainerId) -> Result<AttachedSession>; // TTY: see #12
     async fn attach(&self, id: &ContainerId) -> Result<AttachedSession>; // already-started container
     async fn pause(&self, id: &ContainerId) -> Result<()>;
     async fn unpause(&self, id: &ContainerId) -> Result<()>;
     async fn stop(&self, id: &ContainerId) -> Result<()>;
     async fn remove(&self, id: &ContainerId) -> Result<()>;
     async fn exec(&self, id: &ContainerId, argv: &[String]) -> Result<Output>;
     async fn exec_with_stdin(&self, id: &ContainerId, argv: &[String], input: InputStream) -> Result<Output>;
     async fn cp_to(&self, id: &ContainerId, src: &Path, dst: &str) -> Result<()>;
     async fn cp_to_stream(&self, id: &ContainerId, tar: InputStream, dst: &str) -> Result<()>;
     async fn cp_from(&self, id: &ContainerId, src: &str, dst: &Path) -> Result<()>;
   }
   type InputStream = Pin<Box<dyn AsyncRead + Send>>;
   struct AttachedSession { /* concrete handle: ready barrier, streams, resize/signal, wait */ }
   ```
   `AttachedSession` is a concrete runtime-neutral handle (internally an enum or boxed private
   implementation), not a bare trait return; `wait()` yields the final `ExitStatus`.
2. **Execution**: `tokio::process::Command::new("docker"|"podman").args([...])` — no `sh -c`, ever.
3. **Detection order**: CLI `--podman` → `cage.toml environment.runtime` → `CAGE_RUNTIME` → auto
   (probe `docker version`, then `podman version`). Missing runtime → actionable error (exit 3).
4. Podman is API-compatible with the Docker CLI here; keep divergences (see #19) inside the impls.

## Acceptance criteria → approach

- Unit tests verify create args for both → build arg vectors as pure data, assert on them.
- Runtime-unavailable errors explain cause+action → detection returns `CageError::Container`.
- No shell string concat → enforced structurally + the FR-5.10 lint gate (#18).
- `AttachedSession` reports a connection-ready barrier before handoff and preserves full
  TTY/signal/exit semantics, including an immediately exiting PID 1. A running workspace can be
  paused/unpaused for a consistent #14 snapshot → explicit runtime methods.
- Differences documented → `docs/runtimes.md` (expanded by #19).

## QA gate

- Arg-vector and stdin-stream unit tests (no daemon needed) for create/exec/cp on Docker and Podman;
  streamed input is piped directly to the child process and never buffered to a host file.
- Unit/live: `start_attached` establishes the stream before bootstrap handoff and captures output +
  status from an immediate exit; `attach` handles an already-running container. Pause/unpause args
  are covered, with unsupported runtime behavior reported rather than silently ignored.
- Detection precedence test with env/flag combinations.

## Risks & notes

- `exec_with_stdin` is the non-shell framed-input primitive for #11 credential delivery and #12
  workspace unpacking through `cage-bootstrap`. `cp_to_stream` remains available only for generic
  non-secret copies whose runtime copy semantics are acceptable. Neither implementation stages
  streamed input on the host filesystem.
