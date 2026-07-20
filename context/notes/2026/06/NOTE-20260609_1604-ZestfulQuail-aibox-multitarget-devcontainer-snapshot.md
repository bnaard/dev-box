---
apiVersion: processkit.projectious.work/v2
kind: Note
metadata:
  id: NOTE-20260609_1604-ZestfulQuail-aibox-multitarget-devcontainer-snapshot
  created: '2026-06-09T16:04:16+00:00'
  updated: '2026-07-20T14:09:07+00:00'
spec:
  title: DISC-FirmRose state snapshot — aibox scripts + SSH + multi-target / devcontainer
    standard
  type: insight
  state: captured
  body: |
    # Discussion snapshot — DISC-20260608_1419-FirmRose

    > Captured 2026-06-09. The Discussion body is frozen at `open` (no MCP
    > body-update tool exists), so this Note records the evolved state.
    > Status: **active / pre-decisional** — converging, not yet decided.
    > Related charter: DISC-20260410_2242-CuriousRobin (core principles).

    ## Driving question

    Should aibox's `apply` generate a suite of bash scripts + a single
    entry-point (SSH-based attach) and extend from dev containers to
    cloud/bare-metal server provisioning via terraform/ansible?

    ## Proposal (owner), decomposed into 3 changes

    1. **Scripts as a first-class `apply` output.** Keep emitting
       compose/Dockerfile/devcontainer; additionally emit thin bash scripts
       that perform execution (build/run/attach). Tool-calling + dependency
       orchestration move OUT of the Rust binary into readable bash.
    2. **SSH attach replaces `docker exec`.** Container runs sshd; local
       attaches over SSH. `aibox up` removed → single generated entry-point
       script. Owner wants SSH **everywhere** (local + remote) for one code
       path: one "infra up" script + one "attach" script.
    3. **Cloud/bare-metal parity.** `apply` also generates terraform/ansible
       templates + driver scripts that provision a host, install sshd, and
       expose the same attach entry-point.

    Stated goals: harmonized provisioning across targets; decouple
    tool-calling/deps from Rust into legible bash; easier user hook scripts.

    ## Current architecture (v0.27.x), as verified in code

    - `apply` reconciles `aibox.toml` → generates `.devcontainer/`
      (Dockerfile, docker-compose, override, devcontainer.json) via Rust
      templating, runs processkit content diff, builds the image.
    - `up` attaches via **tmux over `docker exec -it -u <user>`**
      (`container.rs:830`, `runtime.rs:356`) — NOT a raw bash shell.
    - Runtime abstracted over docker/podman (`runtime.rs detect()`).
    - No terraform/ansible/host-provisioning today; `cloud-*` addons only
      install cloud **CLIs** into the container.

    → The proposal is partly a **mechanism swap** (exec→ssh, rust-orch→bash)
    and partly a **scope expansion** (containers→servers). Judged separately.

    ## Owner reframe (key turning point)

    aibox already **generates** compose without **owning** Docker. Generating
    terraform/ansible is the *same class of act*; the tool-*calling* moves to
    thin convenience scripts the user reads/runs. Owner: "I don't see any
    more infrastructure owning than today." → This largely **dissolves** the
    initial charter objection to change 3 *as a generator*.

    Owner also flagged the meta-worry: two undesirable extremes —
    (A) everything is user-owned infra → aibox becomes pointless;
    (B) aibox becomes a multi-target deployer + infra-management tool.
    Wants neither. Also questioned tool curation ("why hugo not any SSG?"
    vs. "latex build is genuinely useful — where else would it live?").

    ## Where the compose↔terraform symmetry actually breaks (the real risks)

    1. **Input asymmetry → aibox.toml scope trap (P2 relocated).** A compose
       file is ~100% derivable from aibox.toml. A useful terraform config
       needs provider/region/instance/VPC/SG/disk/keys/creds/state-backend —
       none in aibox.toml. Fork: (a) grow aibox.toml into provider config =
       inner-system fallacy one layer up; or (b) thin skeletons + a
       **user-owned tfvars/inventory** aibox scaffolds once and never
       re-owns. Only (b) honors the charter.
    2. **State asymmetry.** Compose is stateless (cattle). Terraform has
       tfstate (durable, sensitive, drift). A script that runs `terraform
       apply` decides *when state mutates* = more than generation. Ansible
       is better-behaved (idempotent). → **Ansible-first; if terraform, the
       state backend is a user-declared input, never an aibox default.**
    3. **SSH-everywhere cost.** Uniform attach is the win, but every local
       image now carries sshd + host-key + authorized_keys lifecycle + a
       port — friction vs. P11 slim base. Eyes-open trade, not free.
    4. **Cross-platform.** A bash entry-point is weaker than today's
       compiled binary on native Windows. Decide supported set (WSL/mac/linux?).
    5. **Generator/generated drift.** Rust still generates, bash executes —
       keep scripts thin enough the generator stays the source of truth.

    ## Canonical prior art: DevPod (answers "is there a canonical way?")

    DevPod = "DevContainers everywhere." Model: the **workspace**
    (devcontainer.json) is invariant; a **provider** is a thin swappable CLI
    (`provider.yaml`) that does create-or-connect + run (driver) + tunnel
    (attach transport). Stock providers: docker, ssh, kubernetes, aws,
    azure, gcloud, digitalocean. `devpod up` builds once, provider places.
    Unopinionated, **client-only, no management plane**.

    → This is the boundary that prevents BOTH extremes:
    **aibox owns the invariant + the contract; NOT the providers' guts.** A
    provider is fire-and-forget thin (no state plane, no drift mgmt). Adding
    a target = a thin adapter, not becoming a deployer. (Warning: DevPod
    itself grew a "Pro" management platform — the pull toward extreme B is
    real; stop consciously at the client/contract layer.)

    ## Option set

    - **A. Status quo** — Rust-orchestrated exec+tmux.
    - **B. Scripts + single entry-point only** — charter-aligned, captures
      legibility/hook wins, no SSH, no cloud.
    - **C. B + optional SSH attach**, container-only (exec stays default).
    - **D. Full proposal (1–3)** — requires charter amendment + cloud E2E.
    - **E. Remote target via SSH-only contract** — aibox manages
      workspace/context on any SSH-reachable host; provisioning is the
      user's job.
    - **F. Be placeable, don't build placement** (emerging front-runner) —
      aibox stays a workspace **compiler**; emit standard devcontainer output
      consumable by existing placement tools (Dev Container CLI / DevPod /
      Codespaces). Multi-target ≈ "we emit the standard artifact," not
      "we reimplement placement."

    ## devcontainer.json overlap with aibox.toml

    Only the **container third** overlaps; the AI/process/UX two-thirds have
    no standard equivalent.

    1. **Direct overlap (compiles down):** container.name→name/service,
       user→remoteUser, image/compose→dockerComposeFile,
       lifecycle.post_create→postCreateCommand, keepalive/gateway→
       postStartCommand, audio→portsAttributes, vscode ext/settings→
       customizations.vscode. aibox.toml is a higher-level *input* here — the
       slice most at risk of becoming "devcontainer.json with extra steps"
       (P2). Restraint here is the thing to protect.
    2. **Conceptual overlap, different impl:** `[addons.*]` tools+versions ≈
       devcontainer **Features** (composable, versioned installs) — but aibox
       bakes Dockerfile layers instead of emitting Features.
    3. **Zero overlap (the moat):** `[skills]`+`[processkit]`, `[ai]`
       (harnesses, execution policy, AGENTS.md, MCP gateway), `[customization]`
       tmux/theme/starship. The standard's `customizations` is a per-tool
       namespace → invites a `customizations.aibox` block to carry these
       inside a *valid, portable* devcontainer.json.

    ## devcontainer ecosystem facts

    - **Consumers:** Dev Container CLI (`@devcontainers/cli`, MIT — the
      reference impl; `devcontainer build/up/exec` = headless container
      creation), GitHub Codespaces, JetBrains, Visual Studio, Zed, DevPod,
      Coder, Daytona. Spec governed by the open `devcontainers` GitHub org
      (Microsoft-originated). List: containers.dev/supporting.
    - **Features distribution:** Features are **OCI artifacts** pushed to ANY
      OCI registry, referenced like images `<reg>/<ns>/<id>:<ver>`.
      **Decentralized — no central registry.** Curated *index* at
      containers.dev/features is discovery-only; canonical first-party set =
      github.com/devcontainers/features (→ ghcr.io/devcontainers/features/*),
      reviewed by spec maintainers.

    ## Supply-chain finding (reshapes the Features question)

    A Feature is an `install.sh` **run as root at build** — arbitrary code.
    The index is NOT a security gate; anyone can publish to their own
    registry and be referenced directly. Spec mandates **no signing, no
    central review, no provenance enforcement** (cosign/Notary/SBOM are
    opt-in, external). Pinning gives immutability, not initial trust. So a
    tainted third-party `python` Feature is a real, unmitigated threat —
    worse for an AI-agent workspace where an agent might add a Feature ref.

    → This **reframes aibox's baked-addon model as a deliberate strength**
    (curated, first-party, pinned, in-repo = closed auditable supply chain),
    and **qualifies** the earlier "emit Features instead of Dockerfile" idea:

    | Approach | Portability | Supply-chain trust |
    |---|---|---|
    | Today: baked Dockerfile addons | low | **high** |
    | Consume community Features | high | **low** |
    | **aibox authors its own first-party Features** | **high** | **high** |

    Synthesis: aibox should **author/publish** its own pinned (optionally
    signed) Features rather than **consume** the untrusted long tail —
    portability without importing the trust problem. Dockerfile-baking buys
    supply-chain control (worth keeping); Features buy portability (get it by
    authoring, not consuming). Curation ("why hugo") is doing real security
    work → it stays, just expressed portably.

    ## Emerging charter line (candidate DEC, not yet accepted)

    **aibox is a workspace compiler that targets the devcontainer standard
    while keeping a curated supply chain.** It owns invariants + contracts
    (workspace definition, context/processkit layer, attach contract,
    provider contract, addon mechanism) and a slim base. It does NOT own the
    variable catalog (which target, which tools) or an infra-management /
    state plane. Targets via thin adapters; tooling via the addon mechanism;
    the container third compiles down to a standard, portable devcontainer.json
    (non-standard layers ride in `customizations.aibox`).

    Tooling resolution: hugo AND latex both belong as addons (ideally
    user/community), not core blessings. "Nowhere else to go" is the smell
    that aibox is acting as a tool distro.

    ## Open / undecided

    1. **Build vs. be-placeable (F):** define aibox's *own* thin provider
       contract, or stay a compiler that existing tools (incl. DevPod) place?
    2. **Author-Features vs. keep baking:** does Dockerfile-baking buy enough
       (base/version/cache control) to keep, or move to self-authored Features?
    3. **SSH everywhere vs. remote-only** (cost vs. P11 slim base).
    4. **Cross-platform target set** for the bash entry-point.
    5. Is the candidate charter line above the resting point the owner wants?

    ## Next step

    When the owner confirms the charter line + picks on (1)/(2), promote to a
    **DecisionRecord** extending DISC-CuriousRobin, and `add_outcome` it to
    DISC-FirmRose.
  review_due: '2026-06-16'
  tags:
  - aibox
  - architecture
  - discussion-snapshot
  - devcontainer
  - scope
  - supply-chain
  source: DISC-20260608_1419-FirmRose-should-aibox-s-apply-generate-a
---

# Discussion snapshot — DISC-20260608_1419-FirmRose

> Captured 2026-06-09. The Discussion body is frozen at `open` (no MCP
> body-update tool exists), so this Note records the evolved state.
> Status: **active / pre-decisional** — converging, not yet decided.
> Related charter: DISC-20260410_2242-CuriousRobin (core principles).

## Driving question

Should aibox's `apply` generate a suite of bash scripts + a single
entry-point (SSH-based attach) and extend from dev containers to
cloud/bare-metal server provisioning via terraform/ansible?

## Proposal (owner), decomposed into 3 changes

1. **Scripts as a first-class `apply` output.** Keep emitting
   compose/Dockerfile/devcontainer; additionally emit thin bash scripts
   that perform execution (build/run/attach). Tool-calling + dependency
   orchestration move OUT of the Rust binary into readable bash.
2. **SSH attach replaces `docker exec`.** Container runs sshd; local
   attaches over SSH. `aibox up` removed → single generated entry-point
   script. Owner wants SSH **everywhere** (local + remote) for one code
   path: one "infra up" script + one "attach" script.
3. **Cloud/bare-metal parity.** `apply` also generates terraform/ansible
   templates + driver scripts that provision a host, install sshd, and
   expose the same attach entry-point.

Stated goals: harmonized provisioning across targets; decouple
tool-calling/deps from Rust into legible bash; easier user hook scripts.

## Current architecture (v0.27.x), as verified in code

- `apply` reconciles `aibox.toml` → generates `.devcontainer/`
  (Dockerfile, docker-compose, override, devcontainer.json) via Rust
  templating, runs processkit content diff, builds the image.
- `up` attaches via **tmux over `docker exec -it -u <user>`**
  (`container.rs:830`, `runtime.rs:356`) — NOT a raw bash shell.
- Runtime abstracted over docker/podman (`runtime.rs detect()`).
- No terraform/ansible/host-provisioning today; `cloud-*` addons only
  install cloud **CLIs** into the container.

→ The proposal is partly a **mechanism swap** (exec→ssh, rust-orch→bash)
and partly a **scope expansion** (containers→servers). Judged separately.

## Owner reframe (key turning point)

aibox already **generates** compose without **owning** Docker. Generating
terraform/ansible is the *same class of act*; the tool-*calling* moves to
thin convenience scripts the user reads/runs. Owner: "I don't see any
more infrastructure owning than today." → This largely **dissolves** the
initial charter objection to change 3 *as a generator*.

Owner also flagged the meta-worry: two undesirable extremes —
(A) everything is user-owned infra → aibox becomes pointless;
(B) aibox becomes a multi-target deployer + infra-management tool.
Wants neither. Also questioned tool curation ("why hugo not any SSG?"
vs. "latex build is genuinely useful — where else would it live?").

## Where the compose↔terraform symmetry actually breaks (the real risks)

1. **Input asymmetry → aibox.toml scope trap (P2 relocated).** A compose
   file is ~100% derivable from aibox.toml. A useful terraform config
   needs provider/region/instance/VPC/SG/disk/keys/creds/state-backend —
   none in aibox.toml. Fork: (a) grow aibox.toml into provider config =
   inner-system fallacy one layer up; or (b) thin skeletons + a
   **user-owned tfvars/inventory** aibox scaffolds once and never
   re-owns. Only (b) honors the charter.
2. **State asymmetry.** Compose is stateless (cattle). Terraform has
   tfstate (durable, sensitive, drift). A script that runs `terraform
   apply` decides *when state mutates* = more than generation. Ansible
   is better-behaved (idempotent). → **Ansible-first; if terraform, the
   state backend is a user-declared input, never an aibox default.**
3. **SSH-everywhere cost.** Uniform attach is the win, but every local
   image now carries sshd + host-key + authorized_keys lifecycle + a
   port — friction vs. P11 slim base. Eyes-open trade, not free.
4. **Cross-platform.** A bash entry-point is weaker than today's
   compiled binary on native Windows. Decide supported set (WSL/mac/linux?).
5. **Generator/generated drift.** Rust still generates, bash executes —
   keep scripts thin enough the generator stays the source of truth.

## Canonical prior art: DevPod (answers "is there a canonical way?")

DevPod = "DevContainers everywhere." Model: the **workspace**
(devcontainer.json) is invariant; a **provider** is a thin swappable CLI
(`provider.yaml`) that does create-or-connect + run (driver) + tunnel
(attach transport). Stock providers: docker, ssh, kubernetes, aws,
azure, gcloud, digitalocean. `devpod up` builds once, provider places.
Unopinionated, **client-only, no management plane**.

→ This is the boundary that prevents BOTH extremes:
**aibox owns the invariant + the contract; NOT the providers' guts.** A
provider is fire-and-forget thin (no state plane, no drift mgmt). Adding
a target = a thin adapter, not becoming a deployer. (Warning: DevPod
itself grew a "Pro" management platform — the pull toward extreme B is
real; stop consciously at the client/contract layer.)

## Option set

- **A. Status quo** — Rust-orchestrated exec+tmux.
- **B. Scripts + single entry-point only** — charter-aligned, captures
  legibility/hook wins, no SSH, no cloud.
- **C. B + optional SSH attach**, container-only (exec stays default).
- **D. Full proposal (1–3)** — requires charter amendment + cloud E2E.
- **E. Remote target via SSH-only contract** — aibox manages
  workspace/context on any SSH-reachable host; provisioning is the
  user's job.
- **F. Be placeable, don't build placement** (emerging front-runner) —
  aibox stays a workspace **compiler**; emit standard devcontainer output
  consumable by existing placement tools (Dev Container CLI / DevPod /
  Codespaces). Multi-target ≈ "we emit the standard artifact," not
  "we reimplement placement."

## devcontainer.json overlap with aibox.toml

Only the **container third** overlaps; the AI/process/UX two-thirds have
no standard equivalent.

1. **Direct overlap (compiles down):** container.name→name/service,
   user→remoteUser, image/compose→dockerComposeFile,
   lifecycle.post_create→postCreateCommand, keepalive/gateway→
   postStartCommand, audio→portsAttributes, vscode ext/settings→
   customizations.vscode. aibox.toml is a higher-level *input* here — the
   slice most at risk of becoming "devcontainer.json with extra steps"
   (P2). Restraint here is the thing to protect.
2. **Conceptual overlap, different impl:** `[addons.*]` tools+versions ≈
   devcontainer **Features** (composable, versioned installs) — but aibox
   bakes Dockerfile layers instead of emitting Features.
3. **Zero overlap (the moat):** `[skills]`+`[processkit]`, `[ai]`
   (harnesses, execution policy, AGENTS.md, MCP gateway), `[customization]`
   tmux/theme/starship. The standard's `customizations` is a per-tool
   namespace → invites a `customizations.aibox` block to carry these
   inside a *valid, portable* devcontainer.json.

## devcontainer ecosystem facts

- **Consumers:** Dev Container CLI (`@devcontainers/cli`, MIT — the
  reference impl; `devcontainer build/up/exec` = headless container
  creation), GitHub Codespaces, JetBrains, Visual Studio, Zed, DevPod,
  Coder, Daytona. Spec governed by the open `devcontainers` GitHub org
  (Microsoft-originated). List: containers.dev/supporting.
- **Features distribution:** Features are **OCI artifacts** pushed to ANY
  OCI registry, referenced like images `<reg>/<ns>/<id>:<ver>`.
  **Decentralized — no central registry.** Curated *index* at
  containers.dev/features is discovery-only; canonical first-party set =
  github.com/devcontainers/features (→ ghcr.io/devcontainers/features/*),
  reviewed by spec maintainers.

## Supply-chain finding (reshapes the Features question)

A Feature is an `install.sh` **run as root at build** — arbitrary code.
The index is NOT a security gate; anyone can publish to their own
registry and be referenced directly. Spec mandates **no signing, no
central review, no provenance enforcement** (cosign/Notary/SBOM are
opt-in, external). Pinning gives immutability, not initial trust. So a
tainted third-party `python` Feature is a real, unmitigated threat —
worse for an AI-agent workspace where an agent might add a Feature ref.

→ This **reframes aibox's baked-addon model as a deliberate strength**
(curated, first-party, pinned, in-repo = closed auditable supply chain),
and **qualifies** the earlier "emit Features instead of Dockerfile" idea:

| Approach | Portability | Supply-chain trust |
|---|---|---|
| Today: baked Dockerfile addons | low | **high** |
| Consume community Features | high | **low** |
| **aibox authors its own first-party Features** | **high** | **high** |

Synthesis: aibox should **author/publish** its own pinned (optionally
signed) Features rather than **consume** the untrusted long tail —
portability without importing the trust problem. Dockerfile-baking buys
supply-chain control (worth keeping); Features buy portability (get it by
authoring, not consuming). Curation ("why hugo") is doing real security
work → it stays, just expressed portably.

## Emerging charter line (candidate DEC, not yet accepted)

**aibox is a workspace compiler that targets the devcontainer standard
while keeping a curated supply chain.** It owns invariants + contracts
(workspace definition, context/processkit layer, attach contract,
provider contract, addon mechanism) and a slim base. It does NOT own the
variable catalog (which target, which tools) or an infra-management /
state plane. Targets via thin adapters; tooling via the addon mechanism;
the container third compiles down to a standard, portable devcontainer.json
(non-standard layers ride in `customizations.aibox`).

Tooling resolution: hugo AND latex both belong as addons (ideally
user/community), not core blessings. "Nowhere else to go" is the smell
that aibox is acting as a tool distro.

## Open / undecided

1. **Build vs. be-placeable (F):** define aibox's *own* thin provider
   contract, or stay a compiler that existing tools (incl. DevPod) place?
2. **Author-Features vs. keep baking:** does Dockerfile-baking buy enough
   (base/version/cache control) to keep, or move to self-authored Features?
3. **SSH everywhere vs. remote-only** (cost vs. P11 slim base).
4. **Cross-platform target set** for the bash entry-point.
5. Is the candidate charter line above the resting point the owner wants?

## Next step

When the owner confirms the charter line + picks on (1)/(2), promote to a
**DecisionRecord** extending DISC-CuriousRobin, and `add_outcome` it to
DISC-FirmRose.
