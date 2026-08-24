## Security posture

Aibox offers secure supported integrations without prohibiting standard
Compose or Kubernetes mechanisms. Provider, lifecycle phase, delivery channel,
exposure scope, target, authentication, and renewal remain orthogonal.

Security-sensitive behavior fails closed. A warning may describe a weaker but
valid native mechanism; an explicitly selected organizational policy may turn
that warning into a refusal.

## Assets and trust boundaries

Protected assets include secret values and decryption identities, source and
template integrity, project files, remote host and cluster credentials,
container-engine authority, workload source, generated bundles, and run
evidence.

Trust boundaries include:

- committed project and template content versus the invoking user;
- local aibox process versus native child tools;
- local host versus remote host over SSH;
- client versus container-engine or Kubernetes API;
- workload versus secret provider;
- ainfra-produced target handover versus current target state; and
- ordinary workload identity versus future attested confidential identity.
- MCP client versus the aibox application and its allowed project/targets;
- aibox executor environment versus every distinct managed target; and
- current-environment `aiboxctl` authority versus external aibox lifecycle
  authority, with no bridge between them.

Container-engine socket access is highly privileged. Remote hosts, cluster
administrators, runtime administrators, and processes with sufficient same-UID
or root access may observe workload secrets regardless of aibox delivery
choices. Aibox documentation MUST state this boundary plainly.

Agent-interface threats include prompt injection in template documentation,
addons, Dockerfiles, image/build output, Compose metadata, Kubernetes objects,
remote-host output, native overrides, diagnostics, and logs; tool poisoning,
rug pulls, collision, and shadowing; project, target, engine, SSH host,
Kubernetes context/namespace, credential, and operation substitution;
conversational approval; unsafe generated remediation; credential extraction;
request flooding; replay, cancellation, and session loss; and attempts to gain
authority over the executor or creator boundary.

- **AIBOX-SEC-007:** MCP tools, resources, schemas, descriptions, annotations,
  namespaces, and capability membership MUST be trusted versioned aibox assets
  and MUST NOT be supplied or modified by templates or target content.
- **AIBOX-SEC-008:** a managed environment MUST NOT receive the executor's
  container-engine socket, lifecycle credentials, MCP authority, or an implicit
  route to its creator as an aibox implementation mechanism.
- **AIBOX-SEC-009:** mutating MCP operations MUST have durable replay-resistant
  identity and fail closed on ambiguous target identity, authority, prior
  effects, or session-loss recovery.
- **AIBOX-SEC-010:** diagnostics, explanations, and next actions MUST treat
  template and target output as untrusted data and MUST NOT reflect arbitrary
  executable instructions.

## Secret model

| Dimension | Values |
|---|---|
| Provider | user-managed, SOPS, OpenBao, future KBS/custom |
| Phase | build, deployment/create, interactive entry, runtime |
| Delivery | build secret, container environment, exec environment, mounted file, agent/CSI, application API |
| Exposure | build step, container, service/file, process tree, application |
| Authentication | user/keychain, remote user/machine identity, workload identity, attestation |
| Renewal | none/static, on recreate, on re-entry, provider-managed lease/rotation |

- **AIBOX-SEC-001:** `aibox.toml`, locks, bundles, provenance, logs, evidence,
  examples, and test fixtures MUST contain references or placeholders, never
  resolved secret values.
- **AIBOX-SEC-002:** redaction MUST occur before buffering, rendering, logging,
  telemetry, evidence, or error construction and cover chunked/concurrent
  boundaries.
- **AIBOX-SEC-003:** secret values MUST NOT be placed in command arguments,
  shell source, generated Compose/Kubernetes YAML, or build context.
- **AIBOX-SEC-004:** aibox MUST minimize plaintext lifetime and memory copies,
  avoid swap/dump claims it cannot prove, and document remaining exposure.
- **AIBOX-SEC-005:** secret references MUST declare provider, phase, delivery,
  exposure scope, and target binding; defaults MUST be visible.
- **AIBOX-SEC-006:** unknown provider, incompatible delivery, failed cleanup,
  failed authentication, or missing target identity MUST fail closed.

## Supported provider posture

### Native/user-managed

Users may use native Compose `environment`, `env_file`, secrets, bind mounts,
Kubernetes Secrets, environment references, projected volumes, CSI drivers,
and custom overrides/overlays. Aibox passes through valid native content and
reports detectable consequences. It does not claim to secure user-managed
plaintext sources.

### SOPS

SOPS is the lightweight supported provider, primarily for user-mediated
development. Aibox invokes a reviewed installed SOPS binary; it does not
implement SOPS cryptography.

Acquisition modes:

1. decrypt locally using a user identity protected by the OS keychain;
2. decrypt locally and transfer or stream over authenticated SSH;
3. decrypt remotely using a remote user/machine identity; or
4. let a user-owned native override manage SOPS independently.

Local keychain use requires a supported key backend, unlocked user session,
SOPS availability on the decryption host, and policy allowing user-present
decryption. Headless SOPS is technically possible with a machine identity but
is not the recommended general headless provider because unattended key
custody, rotation, and revocation become operator responsibilities.

### OpenBao

OpenBao is supported in every scenario and is the recommended initial brokered
provider for headless, Kubernetes, renewable, audited, and stronger local-dev
requirements. It may be provisioned by ainfra, a platform team, the user, or an
existing service.

Aibox configures or consumes standard OpenBao API, Agent, authentication, and
Kubernetes integrations. It does not automatically deploy one vault per
workspace. OpenBao Agent/CSI/application-native retrieval SHOULD keep runtime
secrets on the workload side where feasible.

### KBS

KBS is a future confidential-computing interface. Secret release is bound to
verified workload attestation and policy. Ainfra/platform tooling provisions
TEE, attestation, Trustee/KBS, trust roots, and target handover; aibox binds the
workload deployment to declared measurements and provider references. KBS is
not a facade that all ordinary OpenBao access must implement.

## Delivery assessment

| Mechanism | Scope | Runtime state/exposure | Host plaintext | Assessment |
|---|---|---|---|---|
| Runtime exec API environment | New process tree | Not container creation config; present in exec request and descendant environments | Memory-only path possible | Preferred lightweight interactive Docker-compatible delivery |
| `compose exec -e NAME` | New process tree | Same container-side scope; Compose child receives host environment | Memory-only path possible | Supported compatibility path; direct API gives tighter control |
| Service `env_file` | Container-wide | Values persist in container engine configuration and survive restart of the same container | Source may be deleted after successful creation | Standard and convenient; privileged inspect can reveal values |
| Compose CLI `--env-file` plus interpolation | Depends on rendered use, usually container-wide | Resolved values may appear in effective config and persist in container configuration | Source may be temporary | Operational input, not independently a delivery mechanism |
| Compose runtime secret | Explicit services, file | Read-only `/run/secrets/...`; mount metadata visible, value not an environment field | Backing plaintext generally required for local Compose lifetime | Narrower application exposure; local Compose is not an encrypted store |
| Ordinary read-only bind mount | File/directory | Mount source/destination visible | Backing plaintext required for mount lifetime | Standard escape hatch; lifecycle and permissions are user/template owned |
| Kubernetes Secret environment | Pod/container | Reference/value resides in Kubernetes state; environment is process-wide | No node file required | Supported native method; cluster encryption/RBAC are prerequisites for stronger posture |
| Kubernetes Secret/projected volume | Selected Pod/container file | Secret remains Kubernetes state and mounted file | No portable client-host file | Preferable to env when application supports file access |
| OpenBao Agent/CSI/application API | Service/application | Provider-authenticated retrieval, leases and renewal possible | None required on client host | Recommended brokered runtime path |
| BuildKit secret mount | Build step | Temporary builder mount; must not enter layer | Memory/file input depends on provider adapter | Required build-time pattern |

No mechanism is universally most secure. Environment delivery avoids a
long-lived host file but broadens inheritance and inspection. File delivery can
narrow application access but may lengthen plaintext backing-file lifetime.
Brokered application/workload delivery improves renewal and policy at the cost
of infrastructure and identity complexity.

## SOPS delivery flows

### Container-wide Compose environment

```text
encrypted SOPS source
   → decrypt locally or remotely
   → user-private remote temporary env file (0600/runtime directory)
   → remote compose up
   → verify create/recreate completed
   → delete exact temporary file on success or failure
```

Compose reads the file during creation. It can then disappear, but resolved
values persist in the engine's container configuration. Recreating the
container requires reacquisition; restarting the same container does not.

### Interactive Docker-compatible process tree

```text
SOPS/keychain → local aibox memory
             → exec API through protected socket/SSH tunnel
             → dedicated tmux server
             → panes, shells, and descendants
```

The value is not added to unrelated existing processes or the container's
creation configuration. It remains observable to sufficiently privileged
runtime/host actors and processes able to inspect the tmux server.

An existing tmux server does not reliably replace every stored environment
value on attach. The template SHOULD use a dedicated named tmux server/socket
created with the intended environment. Aibox MUST NOT pass the value through a
`tmux set-environment` shell command.

### Mounted Compose file

A SOPS-decrypted mount source generally must remain available for the
container lifetime. Aibox MAY use a user-private tmpfs/runtime directory with
strict permissions and recovery cleanup. Persistent disk is supported only as
an explicitly visible weaker host-retention choice. OpenBao Agent is preferred
for renewal and robust long-lived file delivery.

### Kubernetes

Kubernetes has no equally clean generic exec-environment field. Aibox MUST NOT
advertise `kubectl exec -- env NAME=value ...` as secure generic injection;
the value becomes part of an exec command/audit surface. SOPS MAY be decrypted
and applied through standard Kubernetes Secret tooling when the user accepts
Kubernetes state exposure. OpenBao Agent/CSI/application retrieval is the
recommended managed runtime mechanism.

## Build-time secrets

Build secrets use the selected builder's native secret mount, initially
BuildKit/Compose build secrets. SOPS or OpenBao supplies the input on the build
execution side. Dockerfile `ARG`, `ENV`, `COPY`, generated source, and ordinary
build context MUST NOT carry secret values. Tests MUST prove the value is
absent from final layers, image configuration, bundle, logs, and provenance.

## Scenario recommendations

| Scenario | Lightweight supported option | Stronger recommended option | Preconditions and criticism |
|---|---|---|---|
| Local interactive Docker-compatible | SOPS + OS keychain + exec environment | OpenBao | User session/keychain required; process-tree scope but host/runtime admins remain trusted |
| Remote interactive Docker-compatible | Local or remote SOPS + exec over SSH-protected API | Remote OpenBao integration | Remote host trusted; local decryption crosses SSH, remote decryption needs remote identity |
| Local headless Compose | SOPS machine identity + temporary env/file input | OpenBao | SOPS possible but unattended key custody and renewal are operator burdens |
| Remote headless Compose | SOPS + protected remote temporary input | OpenBao | Static engine environment remains inspectable; broker preferred |
| Interactive Kubernetes | Native Secret env/volume or user overlay | OpenBao Agent/CSI | Generic exec-env is not a secure portable option |
| Headless Kubernetes | Native Secret/volume | OpenBao Agent/CSI | Kubernetes RBAC/encryption-at-rest must be assessed |
| Build | SOPS + BuildKit secret | Short-lived OpenBao credential | Builder and cache are trusted; prove no layer retention |
| Confidential workload | None of the ordinary methods proves workload identity | Attestation-gated KBS | Future phase; depends on ainfra/platform confidential infrastructure |

These are recommendations. Native standard mechanisms remain available unless
an explicitly selected policy forbids them.

## Temporary material and cleanup

- Temporary files use user-private runtime storage where supported, restrictive
  umask/mode, unpredictable exact paths, atomic creation, and no symlink
  following.
- Cleanup is installed before secret material is created and runs on success,
  failure, cancellation, and the next recovery/doctor pass.
- Reboot/logout cleanup is defense in depth, not the only lifecycle guarantee.
- Aibox records redacted cleanup identity and result, never content.
- Cleanup failure is visible and security policy MAY block further operation.
- Aibox never uses broad directory deletion, prune, or global process killing.

## Process, plugin, and template safety

External commands are selected by trusted configuration and capability probes,
not by template shell strings. Arguments remain structured. Template hooks are
container build/runtime content executed by the native target under its normal
security boundary, not privileged host extensions.

Provider adapters use narrow interfaces and MUST document authentication,
network destinations, caching, renewal, redaction, error behavior, and
supported versions. A third-party adapter is not trusted merely because its
configuration came from a template.

## Confidential computing roadmap boundary

The ordinary v1 phases may bind a deployment to signed bundle and ainfra
handover provenance, but exclude live/remote attestation. The future
confidential phase adds measured workload identity, reference-value policy,
attestation verification, KBS secret release, freshness, revocation, and
evidence binding current attested infrastructure to the deployment. Aibox owns
workload integration; ainfra/platform owns confidential infrastructure and
attestation services; the secret provider owns release policy.
