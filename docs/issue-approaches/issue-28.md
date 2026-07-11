# Issue #28 — Platform support matrix

| | |
|---|---|
| Issue | https://github.com/Saber5656/cage/issues/28 |
| Phase | 3 |
| Priority | — |
| Requirements | NFR-6 (Must), NFR-7 (Should) |
| Depends on | #19, #26 |
| Blocks | — |
| Legacy reference | `tests/integration/sync_test.rs` `docker_tmpdir()` (/private/tmp); `src/security/path_validator.rs` |
| Status | Not started |
| Gated by | **Decision D-6** (WSL2 status) |

## Goal

Replace "works on the author's machine" with an explicit, tested support matrix across OS × runtime.

## Approach

1. **Matrix**: OS (macOS arm64/x86_64, Linux x86_64/aarch64, Windows WSL2) × runtime (Docker
   Desktop, Docker Engine, colima/OrbStack, Podman rootful/rootless) — each cell tested /
   best-effort / unsupported.
2. **Smoke tests**: run→diff→sync on at least macOS + Docker Desktop and Linux + Docker Engine.
3. **macOS specifics**: add mandatory regression tests proving the socket blocklist catches the
   Docker Desktop symlink chain (`/var/run/docker.sock` → `~/.docker/run/docker.sock`) and path
   canonicalization handles the in-scope `/tmp` → `/private/tmp` alias (legacy tests special-case
   this). These are automated regression requirements, not optional manual checks.
4. **WSL2 (Decision D-6)**: validate or explicitly mark unsupported for pre-alpha; document.
5. **CI coverage limits**: GitHub Actions Linux runners exercise live Docker tests; macOS runners
   have no Docker, so macOS coverage is manual/unit-level — state this so "skipped" ≠ "passed".

## Acceptance criteria → approach

- Support matrix + limitations documented → `docs/platforms.md` + README pointer.
- macOS+Docker Desktop and Linux+Docker Engine smoke tests recorded → manual checklist or CI.
- WSL2 status decided + documented → D-6.
- CI docs state which cells actually run → `docs/platforms.md` CI section.

## QA gate

- Recorded smoke-test runs for the two primary cells; macOS symlink/`/private/tmp` regression tests.

## Risks & notes

- The socket-blocklist symlink case is security-critical on macOS — a miss there defeats FR-1.4.1,
  so it must be a real regression test, not a manual note.
