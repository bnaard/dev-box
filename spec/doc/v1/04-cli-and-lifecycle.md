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

The environment creates its declared PID 1. `attach` separately connects to the
user or agent workload through engine exec/attach, Kubernetes exec/attach, or
explicitly selected workload SSH. A successful native deployment command is not
itself readiness.

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

Instead, a Template or Feature MAY install runtime capability providers and
signed/locked provider manifests. A provider knows its own companion or native
configuration. `aiboxctl` discovers provider metadata, applies policy, starts
the fixed executable without a shell, routes versioned structured requests,
namespaces MCP tools, and records changes.

```text
aiboxctl CLI/MCP
       │ authorize and route
       ├── theme provider ── native theme files
       ├── tmux provider ─── tmux server
       └── latex provider ── scoped companion API
```

The LaTeX provider may call a private service-network HTTP endpoint or another
template-owned local protocol. It receives no host, engine, Kubernetes, aibox,
or general secret-provider authority.

- **AIBOX-CTL-001:** `aiboxctl` MUST NOT acquire, store, broker, or receive
  general secret values and MUST NOT deploy infrastructure or containers.
- **AIBOX-CTL-002:** absence of `aiboxctl` and providers is conforming.
- **AIBOX-CTL-003:** provider operations MUST be addressed only to the current
  environment, namespaced, schema-validated, time-bounded, and audited.
- **AIBOX-CTL-004:** provider manifests MUST declare executable identity, tool
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
