## Command surface

| Command | Purpose |
|---|---|
| `aibox init [PATH]` | Create minimal intent and select a template/profile without installing template-owned content implicitly. |
| `aibox apply [ENVIRONMENT]` | Validate/migrate intent, resolve locks, compile the bundle, and reconcile declared generated project artifacts. |
| `aibox plan [ENVIRONMENT]` | Show compilation, transfer, build, deployment, secret-delivery, and cleanup actions without executing them. |
| `aibox up [ENVIRONMENT]` | Apply if required, transfer when remote, invoke the native deployment tool, and reach declared readiness. |
| `aibox enter [ENVIRONMENT]` | Start or attach to the declared interactive process through runtime-native exec/attach. |
| `aibox down [ENVIRONMENT]` | Stop the deployment through its native tool without deleting durable declared data unless native configuration says so. |
| `aibox delete environment ENVIRONMENT` | Delete the selected environment after an exact-target confirmation boundary. |
| `aibox build [ENVIRONMENT]` | Build the selected image using the template-declared standard builder. |
| `aibox doctor [ENVIRONMENT]` | Diagnose project, template, target, provider, runtime, generated output, and drift. |
| `aibox config migrate [--check]` | Deterministically migrate owned configuration or report required changes. |
| `aibox config show [--format json]` | Show redacted effective configuration and provenance. |
| `aibox template validate PATH` | Validate a template and run bounded available conformance checks. |
| `aibox version` | Report product, contract, and build identity. |
| `aibox self update` | Update only the CLI binary; never mutate project configuration. |

Subcommand names are normative design intent but MAY receive minor usability
adjustments before CLI contract freeze. `aibox apply` remains the ordinary
place for deterministic configuration migration because it consumes project
intent; `self update` does not.

## Common lifecycle

1. Discover project and configuration layers.
2. Parse the declared schema version without reinterpretation.
3. Preview or apply deterministic owned-document migrations.
4. Acquire and verify the locked template.
5. Resolve addons and target/profile constraints.
6. Compile or reuse an input-bound secret-free bundle.
7. Probe required standard-tool and target capabilities.
8. Acquire secrets only at the phase that needs them.
9. Build, transfer, deploy, or exec through bounded adapters.
10. Record redacted results and exact cleanup.

- **AIBOX-CLI-001:** commands MUST support deterministic plain output for
  non-interactive use and a versioned JSON result envelope where specified.
- **AIBOX-CLI-002:** dry-run/plan MUST NOT decrypt secrets, mutate targets,
  create containers, apply Kubernetes resources, or transfer sensitive data.
- **AIBOX-CLI-003:** cancellation and signals MUST reach child tools and remote
  operations, after which aibox MUST record partial state and attempt only
  exact owned cleanup.
- **AIBOX-CLI-004:** diagnostics MUST name the failing layer and the direct
  standard-tool command or remediation where safe.

## Apply and configuration migration

`apply` MAY migrate `aibox.toml` automatically only when every transition is
deterministic, semantics-preserving, atomic, and covered by fixtures. It first
creates a recoverable backup or uses atomic replacement. A semantic choice,
removed feature without a unique mapping, or conflicting user edit produces a
preview and refusal rather than probabilistic rewriting.

Comments and unknown user ordering SHOULD be preserved when the TOML editing
library permits. Canonical meaning is defined by the schema and normalized
diagnostics, not by forcing one textual serialization.

## Up

For a local Compose target, aibox invokes local Compose. For a remote Compose
target, the default is to transfer the bundle over SSH and invoke Compose on
the remote host. For Kubernetes, aibox invokes the selected local or bastion
client against the target API; it does not log into a control-plane node.

Readiness is template-declared and bounded. A successful `compose up` or
`kubectl apply` is not by itself proof that the user workload is ready.

## Enter

The deployment creates the container or Pod and its template-defined PID 1.
`enter` separately starts or attaches to the user workspace:

- Docker-compatible engine: exec/attach through a protected local socket or
  SSH-protected engine connection;
- Kubernetes: Kubernetes exec/attach API;
- SSH inside the workload: only when explicitly selected and supported by the
  template/target, never a Kubernetes requirement.

An interactive template MAY start a dedicated tmux server. Exec-scoped
environment values belong to that tmux server and its descendants, not to
unrelated existing container processes.

## Down and delete

`down` delegates normal stop behavior to the native deployment tool. `delete`
resolves exact environment identity, previews resources and retained data,
requires explicit authorization appropriate to interactivity, and performs no
broad prune. Cleanup targets only run-owned temporary files, tunnels, and
known deployment resources.

## Optional `aiboxctl`

`aiboxctl` MAY expose template-declared runtime settings such as themes and
tmux layouts through a small CLI or tmux popup. It reads a versioned runtime
feature contract and invokes allowlisted template-provided operations.

- **AIBOX-CTL-001:** `aiboxctl` MUST NOT acquire, store, broker, or receive
  general secret values.
- **AIBOX-CTL-002:** it MUST NOT deploy infrastructure or containers.
- **AIBOX-CTL-003:** its absence MUST be valid for headless templates and
  templates without runtime-changeable features.
- **AIBOX-CTL-004:** runtime changes MUST be reversible or document why not,
  and MUST distinguish ephemeral from persisted user preference.
- **AIBOX-CTL-005:** `aiboxctl` MUST reject any operation addressed to another
  environment and MUST NOT access deployment, remote, container-engine,
  Kubernetes, template-acquisition, or secret-provider authority.
- **AIBOX-CTL-006:** aibox and `aiboxctl` MUST NOT communicate through an
  implicit creator/guest bridge; their responsibilities meet only through
  versioned template and runtime-feature artifacts.
