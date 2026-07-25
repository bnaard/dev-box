# aibox v1 workplan — image and deployment orchestration

Status: implementation in progress — M0 through M2 implemented for integration  
Source: [GitHub issue #179](https://github.com/projectious-work/aibox/issues/179)  
Implementation: M0 boundary ledger, M1 contract alpha, and M2 deterministic
configuration compiler are implemented on the v1 feature stack; M3 is next.

## 1. Outcome and boundary

V1 makes aibox the owner of workspace image construction, desired fleet topology,
deployment onto pre-existing targets, lifecycle visibility, and connection to a
running workspace. Processkit owns its complete distribution and installation
policy. Ainfra-templates may produce target references, but aibox does not provision
or administer infrastructure.

The delivery strategy is contract-first and vertical-slice-first: establish the
canonical domain and safety contracts, prove them through the Compose backend,
delegate processkit through its released protocol, then add Kubernetes without
forking the domain model.

## 2. Planning principles

1. Develop v1 features on feature branches inside the independent v1.x line. Continue
   supporting v0.x alongside v1.x after v1 becomes stable. At that point v0.x enters
   maintenance mode and receives patches only; removal happens only after adoption
   evidence shows projects have moved successfully.
2. Add no new processkit policy to the v0 compatibility bridge.
3. Treat schemas and fixtures as public API. Version them independently from CLI
   implementation modules and validate backward/forward compatibility explicitly.
4. Prove each lifecycle operation end-to-end on Compose before generalizing it.
5. Model capabilities honestly. Unsupported operations fail during validation or
   planning, before mutation.
6. Never infer ownership during destroy. Require a deployment ID, desired-spec
   digest, and backend ownership labels that agree.
7. Store only references to credentials and secrets. Add automated absence tests
   over manifests, records, logs, diagnostics, and release fixtures.
8. Make processkit delegation producer-gated: request/result integration can be
   developed against fixtures, but policy removal waits for a released compatible
   processkit CLI and proven parity.

## 3. Workstreams

### WS-A — Current-state inventory and boundary ledger

Deliverables:

- Machine-readable inventory of processkit-specific code, constants, templates,
  generated files, migrations, tests, docs, and release steps.
- Ownership classification for every item: remove, replace with opaque protocol,
  retain as v0 bridge, or retain as generic content-source/runtime machinery.
- Command-semantics inventory for `apply`, `up`, `down`, attach, status, logs,
  generated runtime, and release-host behavior.
- Compatibility ledger listing v0 behaviors that must remain available during the
  transition and their removal criteria.

Exit gate: every production reference to processkit policy has an owner and planned
disposition; ambiguous items block removal work.

### WS-B — Contract package and error vocabulary

Publish versioned schemas and Rust models for:

- `WorkspaceImageSpec/v1alpha1`
- `WorkspaceFleetSpec/v1alpha1`
- `DeploymentTarget/v1alpha1`
- `DeploymentRecord/v1alpha1`
- `BackendDescriptor/v1alpha1`
- `ConnectionTarget/v1alpha1`
- narrow references to the processkit install request/result

For each contract provide:

- JSON Schema, Rust serialization model, documentation, and canonical examples;
- valid, invalid, minimum, and forward-compatible fixtures;
- stable identifiers, digest canonicalization rules, and redaction rules;
- typed error codes split into validation, capability, planning, mutation,
  observation, connection, ownership, and delegated-installer failures;
- schema-version negotiation and unknown-field policy.

`DeploymentRecord` is the durable receipt for one named deployment. It is not the
desired configuration and not a copy of all backend state. It connects desired intent
to what a backend actually created: deployment and target identity, canonical spec
digest, immutable image digest, owned service/resource identities, last operation and
status, connection endpoints, and opaque processkit result provenance. It contains
references rather than credentials. Status refreshes observed fields; apply and destroy
use the record together with target ownership labels to avoid acting on foreign resources.

The accepted storage model is split representation:

- keep the complete record project-locally;
- put deployment ID, desired-spec digest, immutable image digest, and ownership facts
  on target resources as labels/annotations;
- permit a minimal non-secret target-side receipt where the backend supports it;
- reconstruct a missing local record from verified target ownership metadata;
- do not require a controller, central database, or target-side secret storage.

Exit gate: schema fixtures pass independently of a backend; secrets injected into
test inputs never appear in serialized outputs or diagnostics.

### WS-C — Configuration compiler

Compile user-facing `aibox.toml` into the canonical models without backend-specific
maps leaking into the domain.

Tasks:

- Define typed image, fleet, target, deployment, and connection configuration.
- Define migration from current container/addon/compose configuration.
- Separate semantic defaults from backend rendering defaults.
- Emit a deterministic normalized plan and desired-spec digest.
- Make image building an explicit, opt-in planned action. Remote apply must not hide a
  mutable image build; it resolves or consumes an immutable image artifact.
- Add `aibox config compile` or an equivalent diagnostic surface for inspecting the
  canonical result without mutation.

Exit gate: one input fixture deterministically produces the same canonical models
on Linux and macOS; invalid backend configuration fails before runtime discovery.

### WS-D — Backend interface and registry

Introduce a built-in backend registry with typed operations:

`descriptor`, `validate`, `plan`, `apply`, `status`, `destroy`, `connection`, and
`logs`.

Tasks:

- Define request/result types and async/process execution boundaries.
- Define capability declaration and preflight enforcement.
- Define planned-action vocabulary and mutation receipts.
- Define cancellation/interruption behavior and partial-failure reporting.
- Ensure backend commands are typed argument vectors, not persisted shell strings.

Exit gate: fake backends demonstrate every capability outcome, unsupported-operation
failure, interruption, redaction, and idempotence contract.

### WS-E — Compose vertical slice

Reimplement current local behavior as the first backend.

Increment 1 — plan and render:

- Compile the canonical fleet into Compose and devcontainer artifacts.
- Produce stable ownership labels and golden rendering fixtures.
- Keep Docker, Podman, and OrbStack runtime detection behind the backend.

Increment 2 — apply and record:

- Reconcile desired state idempotently.
- Resolve deployed images to immutable digests.
- Persist a `DeploymentRecord` with desired and observed identities.
- Verify unchanged second apply performs no unintended mutation.

Increment 3 — observe and remove:

- Implement backend-neutral status and logs.
- Implement guarded destroy using labels and deployment identity.
- Refuse foreign, unlabeled, or digest-mismatched resources.

Increment 4 — connect:

- Return `ConnectionTarget` using `compose-exec`.
- Support interactive shell and noninteractive command execution separately.

Compatibility:

- During v1 prereleases, `aibox up` retains an explicit compatibility mode for implicit
  attachment. Stable v1 defines `up` as apply-only; users connect separately.
- `aibox down` maps to guarded `deploy destroy`.
- Existing local project generation remains usable while the new backend path is
  compared against golden outputs.

Exit gate: the standard local lifecycle passes through canonical models and backend
contracts with parity evidence, idempotence evidence, and guarded-destroy tests.

### WS-F — Deployment record and state policy

Resolve and document:

- Which state is local, which is observed from the target, and which is reconstructible.
- Atomic record writes, locking, concurrent invocations, and interrupted operations.
- Deployment ID generation and relation to project/fleet/service identity.
- Drift classification: desired, observed, degraded, unavailable, orphaned.
- Record retention, sensitive-data redaction, and migration between schema versions.

Exit gate: recovery tests cover missing local state, stale state, target drift,
interrupted apply, and concurrent status/apply without unsafe destruction.

### WS-G — Processkit protocol delegation

Dependency: processkit issue #118.

Tasks before the producer release:

- Jointly freeze request/result fixtures, error semantics, availability discovery,
  profile/harness facts, cancellation, migration, and provenance fields.
- Implement a transport adapter against recorded fixtures and a fake CLI.
- Preserve opaque user intent only: enabled, source/channel/version, profile,
  harnesses, root, and environment/path facts.

Tasks after a compatible processkit CLI release:

- Ensure the processkit CLI is available inside the workspace.
- Invoke it and capture the structured result without interpreting contents.
- Compare v0 installer outcomes and v1 CLI outcomes on representative golden projects.
- Prove failure, retry, interruption, migration, and rollback behavior.
- Move old policy behind a bounded v0 bridge, then delete policy only when the
  inventory ledger and parity gates are complete.

Exit gate: changing compatible processkit layouts, skills, catalogs, MCP servers, or
harness projections requires no aibox source change; processkit-disabled deployment
and direct in-workspace processkit invocation both pass.

### WS-H — Kubernetes backend

Implement one backend for local and remote kubeconfig targets.

Increments:

1. Typed target validation, explicit context/namespace authorization, capability
   descriptor, and server-side discovery without mutation.
2. Golden rendering from the same canonical fleet used by Compose.
3. Plan/apply with namespace-scoped ownership labels and immutable image digests.
4. Status/logs and drift reporting using fake clients first.
5. Guarded destroy refusing foreign resources.
6. `kubernetes-exec` connection and typed port-forward targets.
7. Ingress reconciliation and DNS record reconciliation through explicit, typed target
   capabilities and credential references. Aibox may manage records/resources owned by
   the deployment, but does not provision DNS zones, accounts, clusters, networks, or
   ingress controllers.
   DNS records are created only in pre-existing zones. Ingress uses an existing
   `IngressClass` or `GatewayClass`; installing controllers is outside aibox scope.
8. Disposable-cluster integration tests for apply, second apply, drift, connect,
   logs, and destroy.

Exit gate: local and remote targets differ only in target configuration and UX
policy; neither path provisions a cluster or persists kubeconfig contents.

### WS-I — Command and UX convergence

Deliver the final command surface:

- `aibox image build|inspect`
- `aibox deploy plan|apply|status|destroy|logs`
- `aibox connect [service] [-- command ...]`
- compatibility aliases `up` and `down`

Tasks:

- Define machine-readable output alongside human output.
- Define noninteractive behavior, exit codes, progress events, and cancellation.
- Separate deployment completion from connection explicitly.
- Add deprecation messaging and a dated removal criterion for implicit attach.

Exit gate: scripted and interactive journeys are documented and tested across both
backends without assuming SSH.

### WS-J — Documentation, migration, and release readiness

- Rewrite architecture and configuration documentation around image, fleet, target,
  deployment, connection, and optional processkit installation.
- Publish v0-to-v1 migration and rollback guides with supported boundaries.
- Support operational coexistence and reversible configuration rollback: existing v0
  deployments may remain active while v1 is evaluated, and v1 configuration migration
  must be previewable and restorable to a v0-compatible backup. V0 is not required to
  understand, manage, or destroy deployments created by v1.
- Document infrastructure prerequisites and the ainfra target-output boundary.
- Add supply-chain, provenance, secret-safety, and ownership threat-model sections.
- Run schema compatibility, unit, integration, E2E, security, and release audits.

Exit gate: a fresh user can deploy locally without processkit, migrate a supported v0
project, connect explicitly, roll back within the documented boundary, and understand
what aibox will never provision or destroy.

## 4. Milestone sequence

| Milestone | Scope | Depends on | Demonstration |
|---|---|---|---|
| M0 Boundary baseline | WS-A | issue #179 | complete ownership and compatibility ledger |
| M1 Contract alpha | WS-B, state decisions from WS-F | M0 | schemas, fixtures, fake-backend contract suite |
| M2 Configuration compiler | WS-C | M1 | TOML to deterministic canonical plans |
| M3 Compose plan | WS-D + WS-E increment 1 | M2 | backend descriptor and golden non-mutating plan |
| M4 Compose lifecycle | WS-E increments 2–4, WS-F | M3 | apply/status/logs/connect/destroy with records |
| M5 Processkit delegation | WS-G | M1 and processkit#118 release | opaque request/result integration and parity; mandatory before stable v1 |
| M6 Kubernetes plan | WS-H increments 1–2 | M3 | same fleet model renders to Kubernetes |
| M7 Kubernetes lifecycle | WS-H increments 3–7 | M4, M6 | disposable-cluster lifecycle and connection |
| M8 UX convergence | WS-I | M4, M5, M7 | final commands and alias transition |
| M9 v1 release gate | WS-J | all | migration, rollback, security, docs, release audit |

M5 and M6 may proceed in parallel after their dependencies. Compose work must not wait
for processkit or ainfra-templates, but M5 is a mandatory stable-v1 release gate.

## 5. Version-line and support policy

- V1 work uses feature branches rooted in the independent v1.x development line.
- V0.x and v1.x remain supported concurrently when v1 becomes stable.
- Stable v1 moves v0.x into maintenance mode: compatibility and defect patches only,
  with no new architectural capabilities.
- V0.x removal is adoption-driven, not date-driven. Before removal, collect evidence
  that supported projects have migrated, publish warnings and migration tooling, and
  announce a final-support release.
- The v1 implementation is not backported wholesale into v0.x; shared security and
  correctness fixes are ported deliberately where applicable.

## 6. Cross-project dependencies

### processkit#118 — blocking only for M5 production integration

Required producer outputs:

- versioned request/result schemas and golden fixtures;
- CLI availability/bootstrap contract;
- profiles and harness intent fields;
- structured errors, cancellation, retry, and partial-result semantics;
- migration/rollback and provenance semantics;
- secret/redaction contract.

### ainfra-templates — non-blocking for Compose and Kubernetes core

Define a versioned target-output reference containing non-secret target identity,
context/namespace/project facts, and credential references. Aibox validates and
consumes it but does not run provisioning or administration workflows.

## 7. Mandatory test matrix

- Schema: valid/invalid/minimum/future-field fixtures and canonical digest vectors.
- Backend contract: capability, preflight, plan purity, cancellation, typed errors.
- Lifecycle: first apply, unchanged second apply, changed apply, drift, degraded,
  unavailable, interrupted apply, guarded destroy, repeat destroy.
- Rendering: one canonical fleet to Compose and Kubernetes golden outputs.
- Connection: interactive, noninteractive, unsupported transport, service ambiguity,
  exit propagation, port-forward lifecycle.
- Processkit: absent, success, no-op, update, migration, failure, interruption, retry,
  rollback, malformed response, incompatible protocol.
- Security: credential canaries absent from every artifact, record, manifest, log,
  diagnostic, and error; ownership spoofing and foreign-resource destroy refusal.
- Compatibility: representative v0 projects, alias behavior, migration, rollback,
  and bounded v0 bridge removal gates.

## 8. Review gates and evidence

Each milestone requires:

1. Contract/schema review for public-model changes.
2. Threat/ownership review for mutation, destroy, credentials, or connection changes.
3. Golden fixtures and test evidence checked into the same change.
4. Migration and compatibility impact stated explicitly.
5. No implementation milestone may silently broaden infrastructure ownership.

## 9. Refinement status and open questions

Accepted:

1. Develop on feature branches inside the v1.x line.
2. Stable v1 makes `aibox up` apply-only.
3. Image builds are explicit and opt-in; remote apply does not build implicitly.
4. V1 Kubernetes scope includes ingress and DNS reconciliation.
5. Processkit delegation is mandatory before the stable v1 release, but need not block
   earlier Kubernetes development.
6. V0.x remains supported alongside stable v1, then enters patch-only maintenance and
   is removed only after sufficient adoption and migration evidence.

7. Deployment records use split representation: complete project-local record,
   reconstructible ownership metadata on target resources, and an optional minimal
   target receipt without a required controller or database.
8. Rollback supports operational v0/v1 coexistence and reversible configuration
   migration. V0 does not manage v1-created deployments.
9. DNS reconciliation creates records only in pre-existing zones.
10. Ingress reconciliation consumes existing `IngressClass`/`GatewayClass` resources;
    aibox does not install ingress controllers or provision infrastructure.

No architectural refinement questions remain open. Provider-specific DNS adapter
selection and exact contract shapes are M1 design work constrained by these decisions.

## 10. Definition of ready for implementation

The owner has accepted:

- the responsibility boundary and non-goals;
- the milestone ordering and producer dependencies;
- the state/record authority model;
- `up`/connect transition semantics;
- image build versus deploy behavior;
- the initial Kubernetes resource scope;
- the compatibility and rollback policy.

The plan has been decomposed into implementation work items. Implementation was
authorized on 2026-07-24 and is proceeding milestone-by-milestone on the independent
v1.x development line.
