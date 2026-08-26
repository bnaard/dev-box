## Contract families

Aibox owns versioned schemas for deployment intent, local operational binding,
lockfile, saved plan, target import/binding, machine command result,
environment result, runtime-adapter capabilities, run evidence, and the
optional aiboxctl capability-driver registry. It does not own the ainfra
infrastructure-result schema or schemas for Dev Container Templates, Features,
`devcontainer.json`, Dockerfiles, Compose, or Kubernetes resources.

## Deployment intent

Illustrative `aibox.toml`:

```toml
schema_version = "1"

[environment]
definition = ".devcontainer/devcontainer.json"
purpose = "agent-workspace"

[profiles.local]
target = "local"
workload = "user-dev"
interaction = "human-ui"

[profiles.local.runtime]
candidates = ["devcontainer-cli"]

[profiles.local.security]
policy = "developer-interactive"

[profiles.local.storage.home]
class = "persistent"
implementation = "volume"

[profiles.remote-agent]
target = "ainfra://development/agent-runners"
workload = "headless-service"
interaction = "none"

[profiles.remote-agent.runtime]
candidates = ["envbuilder", "kubernetes-native"]

[profiles.remote-agent.security]
policy = "headless-agent"
secret_provider = "team-openbao"
require_feature_provenance = true
allow_target_agent = false
```

Feature selection and options remain directly in `devcontainer.json`. Local
Features MAY be checked in below `.devcontainer/`; tarball and OCI references
are also valid when the selected Dev Container implementation supports them.

- **AIBOX-CONTRACT-001:** aibox configuration MUST NOT duplicate Feature,
  Compose, Kubernetes, Dockerfile, or native application configuration.
- **AIBOX-CONTRACT-002:** unknown deployment-profile fields MUST follow the
  declared schema compatibility policy and MUST never be silently ignored when
  security or lifecycle behavior could change.
- **AIBOX-CONTRACT-003:** secrets are symbolic references with phase, delivery,
  and exposure policy; committed configuration contains no value.
- **AIBOX-CONTRACT-004:** project-owned standard files remain directly usable
  with their native tools as an escape path.

## Resolution and planning flow

```text
standard Dev Container definition + native target material
        + aibox deployment profile + local binding
        + local, explicit, or imported target + selected policy
                            │
                            ▼
             validate and resolve immutable inputs
                            │
                            ▼
        probe internal runtime adapters and external tools
                            │
                            ▼
             create secret-free normalized saved plan
                            │
                            ▼
     apply exact plan through selected established tooling
```

Aibox does not compile a proprietary deployment bundle. It may stage a
relocatable execution directory containing exact native inputs, generated
adapter metadata, locks, and provenance. Staging does not replace the canonical
project files and contains no secret values.

- **AIBOX-COMPILE-001:** identical normalized inputs, locks, target capability
  facts, adapter/tool versions, and policy MUST produce a semantically identical
  normalized plan.
- **AIBOX-COMPILE-002:** planning MUST NOT execute environment-supplied host
  shell snippets, decrypt secrets, or mutate a target.
- **AIBOX-COMPILE-003:** the plan MUST expose native inputs, effective runtime
  adapter, required target agent, privilege changes, storage effects, secret
  delivery metadata, and cleanup scope.
- **AIBOX-COMPILE-004:** apply MUST verify the saved plan binding immediately
  before mutation and refuse drift.
- **AIBOX-COMPILE-005:** resolved secret values MUST be absent from plans,
  staging directories, locks, command arguments, logs, and evidence.

## Standard source and provenance

Dev Container Templates and remote Features use their standard OCI identity and
distribution. Mutable tags resolve to immutable digests before a sensitive
build or deployment when selected policy requires it. Local Features and native
files are content-digested. Template application is explicit scaffolding; aibox
records origin when known but does not treat the applied Template as a perpetual
generator of project files.

- **AIBOX-SOURCE-001:** sensitive profiles MUST lock immutable identities for
  every remote Template, Feature, image, and runtime tool input that affects the
  result.
- **AIBOX-SOURCE-002:** acquisition MUST fail closed on checksum, containment,
  provenance, policy, or compatibility failure.
- **AIBOX-SOURCE-003:** private-registry credentials MUST remain outside locks,
  plans, staging directories, logs, and evidence.

## Runtime adapter contract

Adapters are internal Go packages compiled into `aibox`. Initial adapters wrap
the official Dev Container CLI, Compose, bounded SSH execution, Kubernetes
API/kubectl, and envbuilder where its conformance profile fits. DevPod is not a
supported dependency or adapter. Managed-container platforms whose atomic API
operation rents capacity and creates the declared OCI workload require a target
adapter of the same kind; Vast.ai is the reference evaluation case. A public
third-party plugin protocol is out of scope for v1.

Each adapter implements the applicable subset of `probe`, `resolve`, `plan`,
`build`, `deploy`, `inspect`, `exec`, `attach`, `stop`, `delete`, and evidence
collection. Capability discovery states supported definitions, targets,
headless/interactive behavior, multi-service fidelity, build-secret behavior,
structured results, and whether a target-resident agent is required.

Adapter ownership follows the lifecycle operation, not the implementation
technology. The existence of an OpenTofu provider does not make container
deployment an ainfra concern. If a service exposes an independently usable
host or cluster, ainfra may provision it and aibox subsequently deploys onto
it. If the service's atomic operation creates the declared container workload,
the aibox adapter owns that operation and its lifecycle state.

- **AIBOX-ADAPTER-001:** missing required capability MUST refuse rather than
  silently degrade or omit native semantics.
- **AIBOX-ADAPTER-002:** adapters MUST NOT decide authorization, secret policy,
  target identity, creator-boundary exceptions, or evidence signing.
- **AIBOX-ADAPTER-003:** every external command uses fixed executable/argument
  construction, cancellation, redaction, version probing, and result capture.
- **AIBOX-ADAPTER-004:** the internal interface MUST remain implementable by a
  future native adapter without making one necessary before a demonstrated gap.
- **AIBOX-ADAPTER-005:** deployment credentials used by a managed-container
  adapter MUST remain distinct from secrets delivered to the workload.
- **AIBOX-ADAPTER-006:** a managed-container plan MUST bind immutable image
  identity, capacity and price constraints, storage, networking, lifecycle and
  deletion effects before the platform creates or rents a workload.

## ainfra target handover

Aibox MAY consume the standardized, non-secret ainfra result projection with
capabilities, endpoints, access paths, symbolic secret-provider references,
trust material references, and deployment provenance. It does not perform broad
infrastructure autodiscovery.

- **AIBOX-HANDOVER-001:** handover contains no secret value or private key.
- **AIBOX-HANDOVER-002:** aibox verifies schema, digest, compatibility,
  signer/freshness policy, target identity, and required capabilities.
- **AIBOX-HANDOVER-003:** absent or incompatible handover data produces an
  actionable refusal; aibox does not guess endpoints or credentials.
- **AIBOX-HANDOVER-004:** live/remote attestation remains part of the future
  confidential-computing contract.
- **AIBOX-HANDOVER-005:** ainfra owns and versions the infrastructure result.
  Aibox MUST preserve the imported artifact or its immutable content-addressed
  reference and MUST NOT add environment or workload state to it.
- **AIBOX-HANDOVER-006:** after consuming a target, aibox MUST emit a separate
  Aibox-owned environment result for the operation it performed.
- **AIBOX-HANDOVER-007:** target acquisition MUST support three independent
  modes: local target, explicit existing target, and optional ainfra-result
  import. Absence of ainfra data MUST NOT be an error in the first two modes.
