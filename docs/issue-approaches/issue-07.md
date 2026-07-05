# Issue #7 — Security Layer path & volume validation

| | |
|---|---|
| Issue | https://github.com/Saber5656/cage/issues/7 |
| Phase | 1 |
| Priority | Must (critical) |
| Requirements | FR-1.4.1, FR-3.6, FR-5.8; SEC-ISSUE-001; SEC-DESIGN-001/002 |
| Depends on | #4 |
| Blocks | #8, #9, #12, #15 |
| Legacy reference | `cage-demo/src/security/{path_validator,volume_validator,mod}.rs` |
| Status | Not started |

## Goal

The trust boundary: reject dangerous host paths, mounts, and runtime sockets before anything
reaches the sandbox. This is the highest-severity subsystem — fail closed everywhere.

## Approach

1. **Path validation** (`validate_path_common`): reject empty, NUL byte, control chars; then
   `std::fs::canonicalize` (realpath) — **canonicalize failure = reject** (no unresolved-symlink bypass).
   Split validators: `validate_dir_path` (is_dir), `validate_file_path` (is_file),
   `validate_relative_sync_path` (no `..`, no absolute, stays under project root).
2. **Volume validation**: parse all mount forms — `-v`/`--volume`/`--volume=`, `--mount type=bind,source|src=…`,
   `--device` — extract host path, canonicalize, match against the blocklist.
3. **Blocklist (hard, no config override)**: `/var/run/docker.sock`, `/run/docker.sock`,
   `/var/run/docker/`, `/run/docker/`, containerd + podman socket dirs, `$HOME/.docker/`,
   `$XDG_RUNTIME_DIR/docker.sock`, `$XDG_RUNTIME_DIR/podman/`. Match = exact + path-prefix.
4. **Mount provenance**: `VolumeSource::{Internal, UserSpecified}` — internal cage mounts
   (`/workspace`, `.git`, tmpfs creds) skip the user-path blocklist; only `UserSpecified` is checked.

## API contract

```rust
fn validate_dir_path(input: &str) -> Result<PathBuf, SecurityError>;
fn validate_mounts(mounts: &[Mount]) -> Result<(), SecurityError>; // Mount { source: VolumeSource, host, container, opts }
```

## Acceptance criteria → approach

- docker.sock + symlinked socket rejected → blocklist + realpath, tested with a real symlink.
- `$HOME/.docker/` + rootless sockets rejected → blocklist entries.
- Internal mounts bypass user checks → `VolumeSource::Internal` path.
- Traversal + NUL tested → `path_validator` unit tests.

## QA gate

- Unit tests per SEC-DESIGN-001 §5 table (every mount form, containerd/podman, `~/.docker`, symlink, named volume OK).
- Live (Docker) escape tests belong to #18.

## Risks & notes

- `canonicalize` requires the path to exist. For not-yet-created outputs, canonicalize the parent
  and re-join the leaf — document this so the "must exist" rule doesn't break legitimate `--output`.
