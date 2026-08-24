## Agent-native product posture

aibox is the agent-native environment execution boundary for reproducible AI
workspaces. Agents may interpret goals, select or author standard Dev Container
definitions, explain plans, and propose remediation. Aibox validates deployment
intent and definitions, resolves immutable inputs, creates secret-free saved
plans, verifies authority, invokes established tools through internal adapters,
records evidence, and recovers through deterministic product mechanisms.

The product follows the company
[agent-native product interfaces standard][agent-standard]. MCP is the primary
agent interaction surface wherever aibox is installed. CLI remains complete
for humans, CI, diagnostics, recovery, and break-glass operation. Both are
adapters over the same Go application core.

Aibox does not claim that container engines, remote hosts, clusters, networks,
or workloads are deterministic. It makes the process by which an agent or
human materializes and operates an environment explicit, inspectable,
reproducible for declared inputs, attributable, and recoverable.

- **AIBOX-AGENT-001:** natural-language intent and model-generated template
  content MUST remain proposed inputs and MUST NOT become mutation authority or
  execution truth.
- **AIBOX-AGENT-002:** MCP, CLI, and future adapters MUST preserve equivalent
  validation, authorization, lifecycle, native-tool effects, evidence,
  cancellation, cleanup, and recovery for overlapping application use cases.
- **AIBOX-AGENT-003:** aibox MUST NOT expose arbitrary shell, container-engine,
  Kubernetes, SSH, filesystem, or natural-language deployment tools.

## Executor and target boundary

The **executor environment** is the environment in which the `aibox` process
runs. A **target environment** is a distinct environment that aibox plans,
builds, deploys, inspects, connects to, stops, or deletes.

An aibox process may run on a bare host, in an independently managed container,
in an aibox environment intended for template/environment engineering, in CI,
or on an authorized remote executor. The installation location does not change
its product semantics: aibox manages other environments.

Most aibox-managed environments do not need the `aibox` binary. A template MAY
install aibox as an ordinary tool when the workload is intended to author
templates or manage distinct downstream environments. That does not grant the
current environment control over its creator or underlying host.

`aiboxctl` has the inverse scope: it manages only bounded runtime-provider
capabilities of its current environment. It does not manage another environment
and cannot proxy aibox deployment authority. Its CLI and stdio MCP mode share
the same application core.

- **AIBOX-AGENT-010:** aibox MUST NOT include its executor environment or the
  resources that create or control it in the managed target set. A shared
  remote control plane is permissible only when authenticated authority and
  target scope exclude executor and creator resources.
- **AIBOX-AGENT-011:** aibox MUST NOT create a host bridge, mount a creator's
  container-engine socket, project host lifecycle credentials, or install an
  implicit callback that lets a managed environment control its creator.
- **AIBOX-AGENT-012:** executor and target identities, resource-scope and
  relationship checks, and residual limitations MUST be explicit before
  mutation and retained in redacted evidence.
- **AIBOX-AGENT-013:** `aiboxctl` MUST operate only on its current environment's
  provider registry and MUST NOT expose aibox lifecycle operations.
- **AIBOX-AGENT-014:** `aiboxctl mcp serve --stdio` MAY expose built-in
  inspection and policy-allowed namespaced provider tools; it MUST NOT expose a
  network listener, generic shell, deployment lifecycle, or creator bridge.

## MCP command and transport

The initial server command is:

```text
aibox mcp serve --stdio [--project PATH]
```

It runs in the same executor environment as the equivalent CLI. Stdio frames
use stdout exclusively; diagnostics and redacted operational logs use stderr
or configured sinks. The server is non-interactive and binds one canonical
project plus an explicit allowed target set at startup.

A remote transport is outside the initial MCP phase. It requires a separate
authentication, authorization, TLS, origin, request-isolation, rate-limit,
tenant, credential, deployment, and recovery threat model. It MUST NOT expose
local stdio assumptions directly on a network socket.

## Capability model

The default tool registry is read-only. Additional capability groups are
explicit server-start allowlists; visibility is not approval.

| Capability | Bounded application operations |
|---|---|
| Default read-only | Inspect intent, effective configuration, standard definition, lock, saved plan, target capabilities, status, readiness, sanitized evidence/logs, drift, and diagnostics. |
| `planning` | Preview validation, resolution, build, transfer, deployment, connection, stop, deletion, secret delivery, and cleanup without acquiring secret values or mutating a target. |
| `build` | Execute an authorized, input-bound image-build operation through the selected standard builder. |
| `deployment` | Apply an exact saved plan, transfer staged native inputs, deploy/start/update an environment, and perform bounded non-destructive runtime operations. |
| `connection` | Prepare a declared non-interactive exec or return a short-lived typed connection reference for an authorized external client. |
| `destruction` | Stop or delete the exact recorded target; deletion requires separate explicit authorization. |

Interactive terminal attachment remains a CLI/client concern. An MCP tool MAY
prepare and describe a connection but MUST NOT become an unrestricted terminal
or byte-stream proxy. `connection` does not grant arbitrary process execution;
the command/process contract is template-declared and allowlisted.

- **AIBOX-AGENT-020:** every tool MUST represent one bounded application use
  case with strict versioned schemas, stable result codes, side-effect class,
  affected project/target identity, timeout, and cancellation behavior.
- **AIBOX-AGENT-021:** Templates, Features, overrides, images, remote hosts, and
  Kubernetes resources MUST NOT add or alter tool names, descriptions, schemas,
  annotations, capability membership, or authorization rules.
- **AIBOX-AGENT-022:** the server MUST reject duplicate, shadowed, ambiguous,
  or provenance-conflicting tool/resource identities and use a stable
  capability snapshot for its lifetime.
- **AIBOX-AGENT-023:** MCP request fields MUST NOT select arbitrary Docker
  endpoints, SSH hosts, Kubernetes contexts/namespaces, executables, host paths,
  credential profiles, secret-delivery modes, or native project names outside
  startup and reviewed project policy.

## Authorization, operations, and results

Capability enablement, target reachability, available credentials, and client
confirmation do not authorize a mutation. Consequential operations require an
independently verifiable grant or local policy decision bound to canonical
project, target, operation, input/plan digest, caller, intent, capability, and
expiry. Plan creators cannot self-approve unless explicit external policy
grants both roles.

Mutations carry a durable replay-resistant operation identity before effects
begin. Another authorized MCP client or the CLI can inspect an interrupted
operation. Retry, cancellation, proxy failure, or session loss cannot silently
duplicate a build, transfer, deployment, exec, stop, deletion, or cleanup.

Structured results identify applicability, blocking/advisory diagnostics,
required authorization, target identity, durable operation, completed and
unknown effects, permitted next actions from a closed set, cleanup, evidence,
and recovery. Human explanation cannot replace native configuration, a plan,
authorization, or evidence.

- **AIBOX-AGENT-030:** approval MUST be independent of conversation text,
  model confidence, tool annotations, and client confirmation UI.
- **AIBOX-AGENT-031:** secret providers resolve values only inside the normal
  phase-scoped lifecycle; MCP schemas, requests, resources, results,
  diagnostics, explanations, and evidence MUST NOT carry secret values.
- **AIBOX-AGENT-032:** generated next actions MUST reference registered aibox
  operations and MUST NOT reflect executable instructions from untrusted
  Template, Feature, override, image, native-tool, or target output.
- **AIBOX-AGENT-033:** lost-session recovery MUST never depend on private
  conversation state or chain-of-thought.

## Agent-specific security and acceptance

The threat model and negative tests include prompt injection through Template
documentation, Features, Dockerfiles, image/build output, Compose metadata,
Kubernetes objects, remote-host output, and native overrides; tool poisoning,
rug pulls, collision, and shadowing; target/root/context/namespace substitution;
host-engine access; credential extraction; unsafe remediation; self-approval;
request flooding; retry, cancellation, replay, and session loss.

Acceptance proves:

1. an external agent completes inspect, plan, authorize, build, deploy, status,
   connection preparation, stop, and deletion against a distinct disposable
   target through MCP;
2. the CLI produces equivalent lifecycle effects, evidence, and recovery;
3. aibox running inside one environment manages a distinct remote/headless
   target without a bridge to its creator or underlying host;
4. a normal Dev Container definition without aibox or aiboxctl remains fully conforming;
5. a Template-engineering environment may install aibox as an ordinary tool
   without gaining creator authority;
6. `aiboxctl` changes only its current policy-allowed provider capabilities; and
7. every adversarial case is refused before unauthorized native-tool effects.

[agent-standard]: https://github.com/projectious-work/internal/blob/main/docs/standards/agent-native-product-interfaces.md
