# Issue #9 — Container hardening & resource limits

| | |
|---|---|
| Issue | https://github.com/Saber5656/cage/issues/9 |
| Phase | 1 |
| Priority | Must |
| Requirements | FR-1.3, FR-1.4; THREAT-SEC-E-02, THREAT-CE-D-01; NFR-3 |
| Depends on | #7, #8 |
| Blocks | #12, #18 |
| Legacy reference | `cage-demo/src/security/hardening.rs`, `src/engine/mod.rs`, `seccomp/default.json` |
| Status | Not started |

## Goal

Apply non-negotiable hardening + resource limits to every agent container, with no config path
that disables them.

## Approach

1. **Mandatory flags** (stable order): `--security-opt no-new-privileges`, `--cap-drop ALL`,
   `--user <uid>:<gid>` (non-root), `--security-opt seccomp=<profile>`.
2. **Resource defaults**: memory `4g`, cpus `2.0`, pids-limit `512`; overridable **upward/downward
   by profile/flag but never removable**.
3. **Seccomp distribution (audit gap G-13)**: legacy ships `seccomp/default.json` but
   `SecurityPolicy::default()` sets `seccomp_profile: None`, so installed binaries get **no** seccomp.
   Fix: `include_str!` the profile into the binary, materialize to `$CAGE_HOME/seccomp/default.json`
   on first run, and apply **by default**. Config/flags may select only a named Cage-shipped,
   validated profile; no `none`, `unconfined`, or disable setting exists.
4. **Profile validation**: parse the JSON, assert the required `defaultAction` and restrictions, and
   reject an unknown or invalid profile before passing `--security-opt seccomp=…`; a broken profile
   is a clear startup error, not a runtime stack trace.

## API contract

```rust
struct SecurityPolicy { /* private */ }
impl SecurityPolicy { fn hardening_args(&self) -> Vec<String>; fn resource_args(&self) -> Vec<String>; }
```

## Acceptance criteria → approach

- Create args include hardening in stable order → `hardening_args()` deterministic + test.
- Defaults 4g / 2.0 / 512 → `resource_args()` + test.
- No config disables mandatory hardening → policy owns final flags; config cannot unset (#6).
- Seccomp applied only when profile available → **profile is always available** (embedded) so it
  applies by default; absence path still guarded.

## QA gate

- Unit: hardening arg order, resource defaults, seccomp present by default, and disable/unknown
  profile requests are unavailable or rejected.
- JSON validity of `seccomp/default.json` (also gated in CI by #18).

## Risks & notes

- On macOS/Podman, seccomp applies inside the Linux VM; document that behavior may differ on
  rootless Podman (see #19).
