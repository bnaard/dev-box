## Command surface

The primary lifecycle deliberately follows ainfra.

| Command | Purpose |
|---|---|
| `aibox init [DEPLOYMENT]` | Initialize minimal deployment intent and optionally apply a standard Dev Container Template without overwriting existing files. |
| `aibox doctor [TARGET]` | Diagnose environment definition, deployment profile, target, tools, policy, storage, latest run, and safe remediation. |
| `aibox plan [DEPLOYMENT] [--destroy]` | Create a saved apply or destroy plan bound to native inputs, target identity, adapter/tool versions, policy, and storage effects. |
| `aibox apply [DEPLOYMENT] --plan RUN_ID` | Verify all bindings and apply the exact reviewed plan. |
| `aibox output [DEPLOYMENT] --run RUN_ID` | Show the sanitized standardized environment result. |
| `aibox status [DEPLOYMENT]` | Summarize desired/deployed state, readiness, failures, interruption, and recovery. |
| `aibox destroy [DEPLOYMENT] --plan RUN_ID` | Apply the exact reviewed destroy plan and record retained storage. |
| `aibox build [DEPLOYMENT]` | Build through the selected conforming runtime adapter without deploying. |
| `aibox start|stop [DEPLOYMENT]` | Start or stop an existing environment without changing durable data. |
| `aibox attach [DEPLOYMENT]` | Attach through runtime-native exec/attach or explicit SSH where selected. |
| `aibox exec [DEPLOYMENT] -- COMMAND...` | Execute a bounded command through the runtime adapter. |
| `aibox up [DEPLOYMENT]` | Local interactive convenience for doctor, plan, authorized apply, and attach using the same application core. |
| `aibox config migrate [--check]` | Deterministically migrate owned aibox configuration or report required choices. |
| `aibox config show [--format json]` | Show redacted effective deployment configuration and provenance. |
| `aibox template apply OCI_REF` | Apply a standard Dev Container Template as explicit scaffolding. |
| `aibox version` / `aibox self update` | Report or update the aibox binary; self update never mutates project files. |

Remote, headless, shared, privileged, and destructive operations default to the
explicit `plan` then `apply`/`destroy` sequence. `up` MUST retain a saved plan,
run ID, and evidence and MUST NOT bypass authorization.

## Common lifecycle

1. Discover standard environment files and aibox configuration layers.
2. Preview or apply deterministic owned-document migrations.
3. Validate the Dev Container definition and native target material.
4. Resolve and lock Features, images, target handover, and policy inputs.
5. Probe internal adapters and installed external tools.
6. Select a conforming adapter deterministically.
7. Create a secret-free saved plan.
8. Verify plan binding and authorization.
9. Acquire secrets only for the phase that needs them.
10. Build, transfer, deploy, attach, or delete through the selected adapter.
11. Record sanitized result, events, retained storage, and exact cleanup.

- **AIBOX-CLI-001:** commands MUST provide deterministic human output and a
  versioned machine-result envelope where specified.
- **AIBOX-CLI-002:** plan MUST NOT decrypt secrets, mutate targets, create
  workloads, or transfer sensitive data.
- **AIBOX-CLI-003:** cancellation MUST reach child tools and remote operations;
  partial state and safe next actions are recorded.
- **AIBOX-CLI-004:** diagnostics MUST name the failing definition, target,
  policy, adapter, external tool, or provider layer.
- **AIBOX-CLI-005:** apply and destroy MUST reject changes to any saved-plan
  binding rather than silently re-plan.

## Target execution

Local Docker/Compose initially uses the official Dev Container CLI and Compose.
For a remote Compose target, aibox transfers exact staged native inputs over SSH
and invokes the required standard tooling on the remote host. Kubernetes uses a
local or approved bastion client against the Kubernetes API; it does not require
login to a control-plane node. Envbuilder MAY be selected for a Kubernetes build
only when its probed conformance satisfies the plan.

For a managed-container platform, a target adapter MAY use the platform's
reviewed official API or machine-readable CLI to select capacity and create the
declared OCI workload when those are one atomic platform operation. The adapter
then owns inspection, readiness, logs, attach where supported, stop, and exact
deletion. It MUST NOT introduce OpenTofu state merely because a community
provider exists; selection of API, CLI, or provider is an implementation-time
qualification decision.

The environment creates its declared PID 1. `attach` separately connects to the
user or agent workload through engine exec/attach, Kubernetes exec/attach, or
explicitly selected workload SSH. A successful native deployment command is not
itself readiness.

Airunner is one possible PID 1, installed and configured as standard box
content. Aibox does not interpret Airunner assignments, model turns,
heartbeats, checkpoints, or registration. An orchestrator such as Kaits MAY
pass opaque feature configuration and correlation metadata, then determines
agent availability from Airunner's independent registration or health
contract. Aibox success means that the requested environment and workload were
deployed and reached their declared generic readiness condition.

- **AIBOX-LIFECYCLE-020:** Airunner, Processkit, and other harness/runtime
  products MUST remain optional Features or native image content and MUST NOT
  create domain branches in the Aibox application core.
- **AIBOX-LIFECYCLE-021:** a caller-supplied correlation identifier is opaque
  evidence metadata. It MUST NOT grant authority, select secrets, or become an
  Aibox resource identity.
- **AIBOX-LIFECYCLE-022:** mutating operations MUST accept an idempotency key
  or equivalent durable replay guard suitable for an external orchestrator.

## Autonomous headless and batch workloads

Headless operation is a first-class service lifecycle, not an interactive
container with its terminal removed. An autonomous agent harness normally runs
as the declared foreground process and owns its heartbeat/work loop. Aibox
deploys, observes, stops, and replaces that workload; it does not implement the
agent loop or require tmux, SSH, aiboxctl, or a UI.

- **AIBOX-HEADLESS-001:** intent MUST distinguish `headless-service` from
  finite `batch` work and MUST declare applicable startup, readiness, liveness,
  progress, completion, restart, and failure-budget semantics.
- **AIBOX-HEADLESS-002:** durable agent state, workspace, rebuildable model/tool
  cache, and ephemeral secret material MUST be independently bindable storage
  classes.
- **AIBOX-HEADLESS-003:** unattended workloads MUST declare authentication and
  secret-renewal behavior, resource and spend bounds where the target supports
  them, network-egress policy, graceful shutdown, forced termination, and
  operator pause or kill-switch behavior.
- **AIBOX-HEADLESS-004:** recovery after executor or controller loss MUST avoid
  creating a second active agent for the same exclusive workload identity.
- **AIBOX-HEADLESS-005:** updates bind immutable image identity and declared
  state compatibility and retain a target-native rollback or actionable
  recovery path.
- **AIBOX-HEADLESS-006:** structured logs, progress, terminal outcome, restart
  history, and sanitized deployment evidence MUST remain observable without
  interactive attachment.

## Whole-stack workflow

```sh
# Provision target infrastructure.
ainfra doctor deployment development
ainfra plan development
ainfra apply development --plan INFRA_RUN
ainfra output development --run INFRA_RUN

# Bind and deploy the environment onto the signed result.
aibox doctor agent-runner
aibox plan agent-runner --target ainfra://development/runs/INFRA_RUN
aibox apply agent-runner --plan AIBOX_RUN
aibox output agent-runner --run AIBOX_RUN

# Use the environment when interactive.
aibox attach agent-runner
```

CLI and MCP operations preserve the same order: ainfra produces the target
result; aibox consumes it, produces a reviewed environment plan, and returns an
environment result suitable for kaits or another authorized consumer.

## Optional aiboxctl

`aiboxctl` manages only the current environment. Its CLI and stdio MCP mode use
one bounded application core. It MUST NOT know template-specific implementations
such as a LaTeX companion API, Yazi plugin layout, or theme file format.

Instead, a Template or Feature MAY install runtime capability drivers and
signed/locked descriptors. A driver knows its own companion or native
configuration. For simple domains the descriptor maps typed actions to fixed
Feature-owned scripts; richer domains MAY use a versioned stdio provider.
`aiboxctl` discovers metadata, applies policy, invokes fixed argument vectors
without shell evaluation, namespaces MCP tools, and records changes.

```text
aiboxctl CLI/MCP
       │ authorize and route
       ├── theme driver ── Feature-owned script ── native theme files
       ├── tmux driver ─── Feature-owned script ── tmux server
       └── latex driver ── Feature-owned scripts ── tool/companion API
```

For example, an official LaTeX Feature may install POSIX shell scripts named
`aibox-latex-build`, `aibox-latex-watch`, and `aibox-latex-status`. Those
scripts validate typed inputs and invoke established tools such as `latexmk`
or a scoped private service endpoint. The Feature, not aiboxctl, owns and
versions the scripts and companion behavior. They receive no host, engine,
Kubernetes, aibox, or general secret-provider authority.

- **AIBOX-CTL-001:** `aiboxctl` MUST NOT acquire, store, broker, or receive
  general secret values and MUST NOT deploy infrastructure or containers.
- **AIBOX-CTL-002:** absence of `aiboxctl` and providers is conforming.
- **AIBOX-CTL-003:** provider operations MUST be addressed only to the current
  environment, namespaced, schema-validated, time-bounded, and audited.
- **AIBOX-CTL-004:** driver descriptors MUST declare executable identity, tool
  metadata, mutability class, persistence effect, required approval, and
  protocol compatibility; shell command strings are prohibited.
- **AIBOX-CTL-005:** providers run with current-environment authority only and
  MUST NOT receive a host socket, creator callback, deployment credential, or
  implicit bridge to aibox.
- **AIBOX-CTL-006:** `aiboxctl mcp serve --stdio` MUST expose only built-in
  inspection plus the policy-allowed namespaced tools of discovered providers;
  it MUST NOT be a generic shell or network listener.
- **AIBOX-CTL-007:** runtime changes distinguish ephemeral preference,
  persistent home state, and project-owned configuration and MUST NOT rewrite
  project-owned files without an explicit provider contract and authorization.
- **AIBOX-CTL-008:** official projectious.work runtime drivers SHOULD use
  auditable POSIX shell scripts that orchestrate established tools; Bash,
  Python, a dedicated executable, or an MCP-over-stdio process requires a
  documented need and does not become part of the aiboxctl domain model.
