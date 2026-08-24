## Product acceptance journeys

### Local interactive development

1. Apply a standard Dev Container Template or use an existing definition.
2. Select local or OCI Features directly in `devcontainer.json`, including
   processkit as ordinary project tooling where desired.
3. Create, review, and apply a secret-free saved plan.
4. Merge a user-owned Compose override without aibox rewriting it.
5. Build and start on a supported Docker-compatible engine.
6. Decrypt a SOPS reference with an OS-keychain identity and start a dedicated
   tmux process tree through exec-scoped environment delivery.
7. Prove unrelated existing processes and container configuration did not
   receive the exec-only value.
8. Change a provider-declared theme/layout through optional `aiboxctl` CLI and
   MCP without domain knowledge in aiboxctl.
9. Plan and apply destroy with exact resource, storage-retention, and
   temporary-material cleanup.

### Remote interactive development

1. Consume explicit remote target information or a verified ainfra handover.
2. Stage and transfer exact verified native inputs over SSH.
3. Combine a local copied or explicit remote-native Compose override and
   inspect its effective digest.
4. Invoke Compose on the remote host and validate readiness.
5. Enter through an SSH-protected engine exec/attach connection.
6. Exercise local-decrypt/secure-transfer and remote-decrypt SOPS variants.
7. Recover safely from interrupted transfer, deployment, and temporary-file
   cleanup.

### Headless workload

1. Validate autonomous service and batch definitions without `aiboxctl`, tmux,
   SSH, or UI assumptions.
2. Run an autonomous agent harness as the foreground workload, restore only
   declared durable state, enter its heartbeat/work loop, and report distinct
   startup, readiness, liveness, progress, and terminal outcomes.
3. Deploy locally and remotely using native Compose and on Kubernetes with
   equivalent target-native restart, health, resource, and storage semantics.
4. Exercise graceful shutdown, forced termination, restart budgets, duplicate
   agent prevention after controller loss, pause/kill-switch policy, and
   immutable upgrade/rollback.
5. Use OpenBao workload/agent delivery with denial, unattended authentication,
   renewal, rotation, outage, and restart behavior.
6. Enforce bounded CPU/GPU/memory/spend, controlled egress, structured logs,
   durable evidence, and separation of workspace, agent state, cache, and
   ephemeral secrets.
7. Prove no secret value entered plans, staging, args, logs, evidence, or image
   and no lifecycle or creator authority entered the workload.

### Managed-container GPU workload

1. Materialize and lock an OCI image for a headless model server or autonomous
   agent.
2. Plan a Vast.ai-style target through an internal managed-container adapter,
   including GPU/VRAM, reliability, price bounds, storage, ports, interruption
   policy, deployment credentials, and exact teardown effects.
3. Apply through a reviewed official platform API or machine-readable CLI,
   verify assigned access paths and readiness, and exercise logs and attach
   where supported.
4. Prove the platform credential never enters the workload and runtime secrets
   remain separately scoped and delivered.
5. Stop and delete the exact workload while honoring declared retained storage.
6. Demonstrate that an independently usable host/cluster instead follows the
   ordinary ainfra handover and aibox deployment path.

### Kubernetes workload

1. Render standard resources plus a user Kustomize overlay.
2. Apply from an authorized client without control-plane-node login.
3. Prove context, cluster identity, namespace, resource set, and rollout.
4. Exercise native Secret env/volume with documented state exposure and
   OpenBao Agent/CSI as the stronger path.
5. Enter an interactive Pod through Kubernetes exec/attach.
6. Prove generic plaintext exec-env emulation is not offered as secure mode.

### Build-time secret

1. Supply a SOPS- or OpenBao-derived value through a BuildKit secret mount.
2. Complete the build locally and through a remote builder path.
3. Prove the value is absent from source, context, layers, image config,
   cache-export contract, logs, staging, and evidence.

### Agent-native MCP lifecycle

1. Start a read-only stdio server bound to one project and explicit target set;
   inspect intent, standard definition, lock, plan, target capabilities, status,
   sanitized evidence, and diagnostics.
2. Enable planning and produce input-bound build/deployment/delete previews
   without acquiring secrets or mutating a target.
3. Independently authorize and execute build and deployment against a distinct
   disposable target, then recover durable status through a new MCP client and
   the CLI after simulated session loss.
4. Prepare a bounded connection reference without proxying an arbitrary
   terminal or accepting an arbitrary process command.
5. Stop and delete the exact target with separate destruction capability and
   authorization; prove CLI-equivalent effects, evidence, cleanup, and
   recovery.
6. Run aibox inside one isolated environment and manage a distinct downstream
   remote/headless target without creator callback, host bridge, mounted engine
   socket, or projected lifecycle credentials.
7. Prove that an ordinary environment without aibox/aiboxctl conforms, while a
   template-engineering environment may install aibox as a normal tool and
   `aiboxctl` remains current-environment-only.

## Migration acceptance

- Diagnose representative v0 configurations without mutation.
- Deterministically migrate fields with a unique semantic mapping.
- Convert addon selections with unique mappings into direct local or OCI
  Feature references and preserve supported explicit version intent.
- Convert processkit installation into ordinary Feature/project tooling where
  a unique mapping exists.
- Identify themes, harnesses, yazi/tmux customization, and other former engine
  features requiring template selection or native override.
- Refuse ambiguous conversions with an actionable report usable in headless
  CI; AI assistance is optional, not required.
- Never rewrite user-owned Compose overrides or Kubernetes content.
- Preserve a recoverable v0 source and document rollback/non-rollback.

## Security acceptance

- Threat model and trust boundaries are public and tested.
- Redaction precedes every output/log/evidence sink.
- No secret values occur in committed intent, locks, plans, staging, command args,
  examples, fixtures, generated YAML, image layers, or run evidence.
- Temporary secret artifacts use exact lifecycle cleanup and recovery.
- Provider denial, expired identity, unavailable broker, lease rotation,
  cancellation, and partial failure fail closed.
- Remote SSH host identity, transferred bytes, remote override bytes, working
  directory, and exact cleanup are verified.
- Native weaker options remain available with accurate assessment unless an
  explicit policy prohibits them.
- Future KBS claims do not ship before live attestation conformance exists.
- Prompt injection, tool poisoning/rug-pull/shadowing, capability escalation,
  project/target/context/namespace substitution, conversational self-approval,
  secret extraction, unsafe remediation, replay, and session loss are refused
  before unauthorized native-tool effects.
- Executor and creator-boundary checks fail closed; no managed environment
  receives a host bridge, engine socket, or aibox lifecycle authority.

## Template acceptance

- A clean-room author produces a standard Dev Container Template using only
  public upstream and aibox conformance material.
- The Template composes local and OCI Features with explicit versions without
  engine feature conditionals or an aibox content DSL.
- At least one interactive, one autonomous headless-service, one batch, one
  remote Compose, one Kubernetes, and one managed-container template/profile
  combination passes disposable conformance.
- Native overrides/overlays work without aibox schema duplication.
- Unsupported combinations fail during validation with precise capability
  diagnostics.

## Implementation roadmap

[`roadmap.yaml`](roadmap.yaml) is canonical. Status semantics follow the
company roadmap standard: `planned` is committed sequencing, `idea` is
non-committal, and `shipped` requires release and development evidence.

Phase 0 accepts this specification. Subsequent phases establish Go and owned
contracts before adding builders, targets, runtime features, or providers.
OpenBao follows core provider/delivery ports; confidential KBS and live
attestation remain a later idea phase dependent on ainfra/platform outcomes.

## Implementation-time selections

The following choices remain open until their named phase and require focused
review, fixtures, and documentation:

| Selection | Required criteria |
|---|---|
| TOML parser/editor | comments and unknown ordering preservation, strict types, maintained Go support, safe atomic rewrite |
| JSON Schema implementation | supported drafts/formats, unknown-field enforcement, deterministic errors, maintained dependency |
| Compose adapter | native CLI compatibility, effective config, supported Docker/Podman paths, cancellation and inspect contracts |
| Container exec adapter | Docker-compatible capability support, SSH protection, environment handling, attach/TTY correctness |
| Kubernetes adapter | standard client behavior, context authority, Kustomize integration, stable machine diagnostics |
| SSH implementation | host verification, agent/keychain support, proxy/bastion, structured args, cancellation, transfer integrity |
| SOPS invocation | version support, stdin/file modes, keychain identities, redaction, temporary-material behavior |
| OpenBao integration modes | standard API/Agent/CSI, workload authentication, renewal, denial, outage and version support |
| MCP SDK and protocol version | maintained implementation, strict schema/resource/tool support, cancellation, stdio interoperability, low domain coupling, and compatibility with the company agent-native interface standard |
| Bundle signing | subject model, DSSE/in-toto compatibility, signer/freshness policy, keyless availability, offline verification |

An implementation convenience does not become a durable product contract
without a reviewed specification amendment.
