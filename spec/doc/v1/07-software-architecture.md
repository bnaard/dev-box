## Principles

- Go for host `aibox` and optional in-environment `aiboxctl`.
- One application core per binary, shared by its CLI and MCP adapters.
- Standard Dev Container and target-native files are leading artifacts.
- Runtime mechanics are internal adapter packages around established tools.
- Policy, authorization, target identity, secrets, and evidence remain core.
- No global mutable state; bounded subprocesses; explicit cancellation.
- No DevPod dependency or provider compatibility promise.

## Proposed source layout

```text
cmd/
├── aibox/
└── aiboxctl/
internal/
├── aiboxapp/             # host lifecycle use cases
├── ctlapp/               # current-environment use cases
├── cli/                  # CLI adapters
├── mcp/                  # guarded MCP adapters for both applications
├── contract/             # owned schemas and result envelopes
├── config/               # deployment intent, local binding, migration
├── devcontainer/         # standard definition/template/Feature validation
├── runtime/
│   ├── runtime.go        # consumer-owned internal interface
│   ├── devcontainercli/  # official CLI adapter
│   ├── compose/          # Compose adapter
│   ├── ssh/              # bounded remote execution
│   ├── kubernetes/       # API/kubectl adapter
│   └── envbuilder/       # conforming Kubernetes build adapter
├── target/               # target identity, capabilities, ainfra handover
├── policy/               # secure-agent and deployment policy
├── secret/               # provider/delivery orchestration
├── storage/              # logical storage class binding
├── plan/                 # normalized saved plan and binding
├── run/                  # durable operation state
├── evidence/             # sanitized evidence and future attestation
├── ctlprovider/          # local provider discovery/protocol/policy
├── diagnostic/
├── output/
├── execx/
└── fsx/
```

Exact names may change, but responsibilities and dependency direction are
normative.

## Dependency flow

```text
CLI/MCP → application use cases → domain contracts/ports
                                      ↑
                         internal adapters implement ports
```

- **AIBOX-ARCH-001:** runtime adapters are internal Go packages compiled into
  `aibox`; v1 ships no public plugin protocol or separate adapter artifacts.
- **AIBOX-ARCH-002:** adapters invoke external tools only through the bounded
  subprocess boundary and return normalized capability, plan, event, and result
  models.
- **AIBOX-ARCH-003:** adapters MUST NOT call each other through hidden state;
  application use cases coordinate them.
- **AIBOX-ARCH-004:** production code contains no processkit-, harness-, editor-,
  language-, theme-, or Feature-specific installation behavior.
- **AIBOX-ARCH-005:** selection is based on probed capability and policy, not
  silent runtime-brand fallback.
- **AIBOX-ARCH-006:** CLI and MCP share application use cases; domain packages
  do not import transport SDK types.
- **AIBOX-ARCH-007:** a future external adapter bridge requires demonstrated
  demand and a separate security, versioning, distribution, and support design.

## Standard-tool boundary

The official Dev Container CLI owns reference build and Feature semantics for
the profiles it supports. Compose owns Docker-compatible multi-service topology.
Kubernetes resources and API/kubectl own Kubernetes topology and exec/attach.
Envbuilder may own a Kubernetes build where its probed semantics conform.
Aibox supplies orchestration and guarantees around these tools; it does not
fork their configuration languages.

External tool versions are probed and recorded. Missing versions, unstructured
failure, privilege changes, or unsupported semantics produce explicit findings.
Commercial distributions preserve applicable MIT, Apache-2.0, and transitive
third-party notices.

## aibox and aiboxctl

The binaries are separate release artifacts and MAY share pure contracts,
protocol helpers, diagnostics, and redaction packages. `aiboxctl` does not link
deployment, target, remote, container-engine, Kubernetes, template-acquisition,
or secret-provider adapters.

`aiboxctl` may load current-environment capability providers through a narrow
versioned local protocol. This is not the host runtime-adapter interface. A
provider is installed as environment content, runs without a shell and without
elevated creator authority, and owns all implementation knowledge for its
domain. The aiboxctl core owns discovery, namespace collision checks, policy,
approval, timeouts, result normalization, audit, and MCP/CLI projection.

## Concurrency and dependencies

Every goroutine has an owner, cancellation path, and joined completion. Child
processes and remote commands receive contexts and bounded shutdown. Likely
dependencies include TOML/JSON parsing, JSON Schema validation, SSH, MCP, and
selected Kubernetes libraries; established CLIs remain preferred when they
avoid a larger unstable SDK surface.

Every dependency and external tool receives maintenance, license, provenance,
platform, vulnerability, and transitive-weight review. A shared ainfra/aibox
library may emerge only for stable run/result/MCP conventions; a shared product
core is prohibited without demonstrated need.
