---
apiVersion: processkit.projectious.work/v2
kind: Discussion
metadata:
  id: DISC-20260608_1419-FirmRose-should-aibox-s-apply-generate-a
  created: '2026-06-08T14:19:07+00:00'
spec:
  question: Should aibox's `apply` generate a suite of bash scripts + a single entry-point
    (with SSH-based attach) and extend from dev containers to cloud/bare-metal server
    provisioning via terraform/ansible?
  state: active
  opened_at: '2026-06-08T14:19:07+00:00'
  participants:
  - ACTOR-20260410_2209-SnappyFrog-bernhard
  related:
  - DISC-20260410_2242-CuriousRobin-what-are-the-core
---

# aibox: scripts-as-output + SSH attach + cloud/bare-metal provisioning

## 1. Problem statement

The owner proposes restructuring the aibox CLI so it serves both
container workspaces *and* infrastructure deployments on bare/cloud
servers, around three coupled changes:

1. **Scripts as a first-class `apply` output.** `apply` continues to
   emit docker-compose/Dockerfile/devcontainer files, but *additionally*
   emits a suite of generated bash scripts that perform execution
   (build, run, attach). Tool invocation and dependency orchestration
   move out of the Rust binary into readable, user-inspectable bash.
2. **SSH attach replaces `docker exec`.** Containers run an SSH server;
   the local side attaches over SSH instead of `docker exec -it … tmux`.
   `aibox up` is removed in favour of a single generated entry-point
   script.
3. **Cloud/bare-metal parity.** For servers, `apply` generates
   terraform/ansible templates plus driver scripts that provision the
   host, install an SSH server, and expose the same single entry-point
   attach experience.

Stated advantages: harmonized provisioning across container and server
targets; decoupling of tool-calling/deps from Rust into bash (more
legible to users); easier user-supplied hook scripts.

## 2. Current architecture (as built, v0.27.x)

- `apply` reconciles `aibox.toml` → generates `.devcontainer/`
  (Dockerfile.j2, docker-compose.yml.j2, override, devcontainer.json),
  runs the processkit content diff, builds the image. Generation is
  Jinja-style templating inside the Rust crate (`cli/src/templates/`).
- `up` seeds `.aibox-home/`, ensures the container runs, attaches via
  **tmux over `docker exec -it -u <user>`** (`container.rs:830`,
  `runtime.rs:356 exec_interactive`). Layouts: dev/focus/cowork/ai.
- Runtime is abstracted over docker/podman (`runtime.rs detect()`),
  preferring docker (OrbStack).
- No terraform/ansible/cloud-provisioning today; `cloud-*` addons only
  install cloud *CLIs* into the container. There is **no** server/host
  provisioning surface.

So the proposal is partly a **mechanism swap** (exec→ssh, rust-orch→bash)
and partly a **scope expansion** (containers→servers). These must be
judged separately.

## 3. Collision with documented scope (DISC core-principles)

The current charter (DISC-20260410_2242-CuriousRobin) is explicit:

- **P1 Dev container first** — primary artifact is a dev container.
- **P2 No inner-system fallacy** — aibox does NOT re-expose
  Docker/compose options behind its own config layer.
- **What aibox is NOT:** "Not a CI/CD system (provides dev environment;
  build/deploy is the project's concern)"; "Not a Docker wrapper
  (scaffolds containers; does not abstract Docker)."

→ **Change 3 (cloud/bare-metal provisioning)** is the part most in
tension with the charter: provisioning servers is build/deploy of
infrastructure, explicitly out of scope. Accepting it is a charter
amendment, not just a feature — it should be decided as such.

→ **Change 1 (scripts as output)** is *aligned* with the charter's
"standard, understandable output" value and P2 (thin the binary, make
the artifacts the contract). This is the strongest part of the idea.

→ **Change 2 (SSH attach)** is a neutral mechanism trade — judged on
engineering merit below.

## 4. Pro / con by change

### Change 1 — scripts as `apply` output, single entry-point
PRO
- Legibility: users read/modify the actual run logic; less "magic Rust".
- Hookability: pre/post hook points are trivial in bash.
- Testability of the *contract*: generated scripts are diffable
  artifacts (fits existing content-diff/drift checks).
- Reduces Rust surface that wraps docker invocations.
CON
- Two sources of truth for orchestration logic (Rust still generates,
  bash still executes) — drift risk between generator and generated.
- Cross-platform fragility: bash entry-point on macOS/Windows
  (Git-Bash/WSL) is weaker than a compiled binary; aibox currently runs
  the same on every host.
- Error handling / structured output (`--json`, doctor signals) is
  harder and less uniform in bash than in Rust.
- Security/UX of "run this generated script" vs. "run the signed binary".

### Change 2 — SSH server in container, attach over SSH
PRO
- Uniform attach verb across container and remote host (the harmonizing
  win the owner is after).
- Real TTY/agent-forwarding/port-forwarding, scp, IDE remote-SSH,
  multiple independent sessions — richer than `docker exec`.
- Decouples attach from the container runtime (no docker/podman exec
  semantics differences).
CON
- New attack surface + key/secret management inside every workspace
  (sshd, host keys, authorized_keys lifecycle, port allocation).
- Heavier image + a daemon to supervise; conflicts with P11 "slim base".
- `docker exec` needs no network, no port, no keys — SSH adds moving
  parts to the common local case for little local benefit.
- tmux-over-exec already gives persistent sessions/layouts today; SSH
  duplicates plumbing aibox already solved.

### Change 3 — cloud/bare-metal provisioning (terraform/ansible)
PRO
- One tool to stand up an AI workspace anywhere (laptop → cloud VM).
- Reuses the same context/skills/processkit layer on a remote host.
CON
- Direct charter conflict (P1, "not CI/CD", "not Docker wrapper" →
  now also "not infra wrapper").
- Enormous surface: state files, cloud creds, provider drift, secrets,
  teardown/cost control, partial-failure recovery — this is a product,
  not a feature.
- terraform/ansible already *are* the harmonization layer; wrapping them
  in generated bash risks the P2 inner-system fallacy at a larger scale.
- Maintenance/test burden (E2E against real clouds) dwarfs the current
  container E2E suite.

## 5. Key reframing / challenge to the premise

The owner frames the goal as *harmonized provisioning*. But a cleaner
seam already exists: **SSH is the only thing that actually needs to be
uniform.** "A reachable host with an SSH endpoint + our context layer"
is the real abstraction — *how* that host comes to exist (compose,
terraform, a hand-provisioned VM, a colleague's box) can stay outside
aibox. That preserves the charter while still delivering the
single-entry-point attach experience the owner wants.

This suggests decoupling the three changes rather than shipping them as
one structure.

## 6. Options on the table

- **A. Status quo** — keep Rust-orchestrated exec+tmux.
- **B. Scripts + single entry-point only** (Change 1), no SSH, no cloud.
  Lowest risk, charter-aligned, captures the legibility/hook wins.
- **C. B + optional SSH attach** (Changes 1–2), container-only. SSH as
  an opt-in transport, `docker exec` remains default.
- **D. Full proposal** (Changes 1–3) — requires a charter amendment and
  a much larger commitment.
- **E. "Remote target" via SSH-only contract** — aibox manages the
  workspace/context on *any* host you can SSH to; provisioning that host
  is explicitly the user's job (terraform/ansible as user-supplied, not
  aibox-generated). Harmonizes attach without owning provisioning.

## 7. Open questions for the owner
1. Is the real need *remote AI workspaces* (E) or genuinely *aibox owns
   server provisioning* (D)? These have very different costs.
2. Local container case: is `docker exec`+tmux actually a pain point, or
   is SSH only wanted for the remote case?
3. Are you prepared to amend the core-principles charter (P1, "not
   CI/CD") — and maintain cloud E2E — or keep provisioning out of scope?
4. Cross-platform: must the entry-point work on native Windows, or is
   WSL/macOS/Linux sufficient?

## Decisions (DEC-NNN records)
- _none yet — pre-decisional._
