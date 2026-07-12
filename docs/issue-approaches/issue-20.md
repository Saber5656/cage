# Issue #20 — Safe DinD sidecar mode

| | |
|---|---|
| Issue | https://github.com/Saber5656/cage/issues/20 |
| Phase | 3 |
| Priority | Should |
| Requirements | FR-1.4.2; THREAT-CE-E-01, THREAT-CE-S-02, THREAT-TM-T-02 |
| Depends on | #8 |
| Blocks | — |
| Legacy reference | `cage-demo/src/engine/dind.rs`; `docs/security-design-docker-socket.md` §4 |
| Status | Not started |
| Gated by | **Decision D-4** (TCP vs TLS endpoint) |

## Goal

When `--dind` is given, run a dedicated Docker-in-Docker sidecar on an isolated network so nested
container use never touches the host daemon socket.

## Approach

1. Start `cage-dind-<session>` (`docker:27-dind`, `--privileged`, own `/var/lib/docker`) on a
   dedicated, labeled `cage-net-<session>` network used only by the sidecar and its agent.
2. Point the agent container at it via `DOCKER_HOST` (endpoint per **D-4**).
3. **No host volumes** on the sidecar; the agent container keeps FR-1.4 hardening (sidecar is exempt
   only because `--privileged` is required).
4. **Resource limits on the sidecar (audit gap G-21)**: apply memory/cpus/pids like any cage
   container, despite `--privileged`.
5. **Endpoint decision (D-4)**: plain TCP on 2375 is admissible only when the runtime offers a
   preventive admission-control primitive that rejects every non-allowlisted container **before**
   it can join/reach the network. Membership inspection, event watching, polling, random names, and
   post-connect teardown are not sufficient. Standard Docker/Podman bridge networks provide no such
   ACL, so they must use mutually authenticated TLS 2376 with per-session certificates. If a future
   runtime proves preventive enforcement, Cage still verifies exactly the sidecar + agent members;
   inability to prove either property fails closed with no insecure fallback. Record the selected
   contract and evidence.
6. **Readiness wait**: poll `docker version` through the sidecar before handing control to the agent
   (dockerd takes seconds to come up, else the agent's first call fails).
7. **Cleanup**: remove sidecar + network + its anonymous `/var/lib/docker` volume; idempotent.

## Acceptance criteria → approach

- Host `/var/run/docker.sock` never mounted even in DinD → design forbids it; test asserts absence.
- Sidecar + network names include collision-resistant session id → naming scheme (shared with #22).
- Cleanup idempotent → teardown returns ok when already gone.
- DinD privileged risks documented → `docs/dind.md`.
- Standard runtimes reject plain-TCP mode and use mTLS; any future plain-TCP runtime must prove
  preventive admission control plus exact membership → capability/assertion tests.

## QA gate

- Unit/live: Docker/Podman refuse plain-TCP configuration; mTLS rejects a client without the
  per-session certificate; `--dind` can otherwise run `docker run hello-world` inside the sandbox;
  host socket is absent; cleanup leaves nothing.

## Risks & notes

- The whole point is to avoid THREAT-SEC-E-01 (socket mount); a lazy "just mount the socket"
  shortcut must never be added as a fallback.
