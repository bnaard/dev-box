## Target principle

Deployment planning and execution are separate. Aibox resolves and plans where
the user invokes it, then runs the standard deployment tool in an
authorized environment whose filesystem and API semantics are unambiguous.
The executor is not a managed target. When aibox runs in a container, it may
manage a distinct remote host, cluster, nested isolated runtime, or other
explicit target, but it receives no bridge to the host/engine that created the
executor.

| Target | Default execution location | Native tool |
|---|---|---|
| Local Docker-compatible | Local host | Compose-compatible CLI and engine API |
| Remote Docker-compatible | Remote workload host over SSH | Remote Compose-compatible CLI; protected engine API for bounded exec/inspect |
| Kubernetes | Local host or authorized bastion | `kubectl`/Kustomize against cluster API |

Kubernetes deployment never requires `kubectl` on a control-plane node. A
remote API endpoint is the normal cluster boundary.

## Remote Compose

The default remote flow is:

```text
local deployment intent and native definition
          │ resolve + plan
          ▼
secret-free staged native inputs
          │ verify + SSH transfer
          ▼
remote user-private deployment directory
          │
          ▼
remote compose config / up / down
```

This makes remote bind paths, build contexts, `env_file`, secrets, and
user-owned overrides behave exactly as they do for a user logged into that
host. It avoids exposing a Docker socket over a network and avoids
reimplementing Compose.

- **AIBOX-TARGET-001:** remote transfers MUST use authenticated, host-verified
  SSH or an equally reviewed secure transport.
- **AIBOX-TARGET-002:** aibox MUST stage into a user-private directory, verify
  transferred digests before use, use atomic activation, and clean only exact
  run-owned staging paths.
- **AIBOX-TARGET-003:** remote commands MUST use fixed argument boundaries,
  controlled working directories, allowlisted environment, timeouts,
  cancellation, and captured/redacted output.
- **AIBOX-TARGET-004:** remote template or override content MUST NOT select the
  SSH executable, arbitrary host command, identity file, or unrelated path.
- **AIBOX-TARGET-005:** a remote-native override MUST be explicitly selected,
  read/hashable by the deploying identity, and included in effective
  provenance before mutation.

Running local Compose against a remote engine MAY be added as an advanced
adapter. It MUST document the split semantics: client-side files and
interpolation are local, while bind-mount paths and engine state are remote.
It is not the default.

## Remote project source

Source placement is explicit and template-constrained:

- `sync`: aibox transfers selected source with excludes and digest evidence;
- `git`: the remote host checks out a declared immutable or user-selected
  revision using credentials outside staged inputs;
- `remote-path`: the user declares an existing remote directory; or
- `image`: no host source mount is required.

- **AIBOX-TARGET-006:** aibox MUST NOT silently infer or mix source-placement
  modes.
- **AIBOX-TARGET-007:** synchronization MUST define symlink, ignore, deletion,
  ownership, partial-transfer, and conflict behavior and MUST NOT perform broad
  remote deletion.
- **AIBOX-TARGET-008:** `doctor` MUST validate referenced remote paths and
  permissions before `up` where possible.

## Docker-compatible runtime adapters

The Docker Engine API is a widely implemented interface but not an OCI
management standard. OCI standardizes runtime/image contracts, not one daemon
API. Docker, OrbStack, and Podman's Docker compatibility service may satisfy a
shared adapter when capability probes and conformance tests pass. Product-name
recognition alone is insufficient.

Initial probed capabilities include:

- Compose version and config merge;
- image build and required BuildKit features;
- exec with environment and attach;
- inspect without parsing human `ps` output;
- bind mounts and read-only secret mounts;
- remote connection transport; and
- exact resource labels/identifiers used for cleanup.

- **AIBOX-TARGET-009:** engine sockets MUST remain host-side and MUST NOT be
  mounted into aibox-managed workloads as an implementation shortcut.
- **AIBOX-TARGET-010:** remote engine connections MUST use SSH forwarding or
  mutually authenticated TLS; unauthenticated TCP sockets are prohibited.
- **AIBOX-TARGET-011:** missing capabilities produce a refusal or a documented
  alternative adapter, never silent behavior degradation.
- **AIBOX-TARGET-012:** target probing MUST establish sufficient identity,
  topology, and resource scope to reject the executor itself and detected
  resources that create or control it before mutation.
- **AIBOX-TARGET-013:** inability to distinguish a proposed target from the
  executor or its creator boundary MUST fail closed; a user acknowledgement is
  not a substitute for target identity.

## Kubernetes targets

Projects or Templates provide standard manifests and optional Kustomize
bases/overlays. Aibox validates references and capabilities, selects a
conforming Kubernetes/envbuilder adapter, invokes the native client, and records
sanitized results. Kubernetes owns scheduling, Pod state,
Secret objects, volumes, exec/attach, rollout, and deletion semantics.

- **AIBOX-K8S-001:** Kubernetes interaction MUST use kubeconfig/context and
  API authorization supplied by the user, ainfra handover, or platform policy.
- **AIBOX-K8S-002:** a committed project MUST NOT silently select a privileged
  kubeconfig, context, namespace, executable, or cluster endpoint.
- **AIBOX-K8S-003:** the effective context, cluster identity, namespace, and
  resource set MUST be shown before mutation and represented in redacted run
  evidence.
- **AIBOX-K8S-004:** `enter` SHOULD use exec/attach. SSH inside Pods is an
  explicit exceptional template capability, not a default.
- **AIBOX-K8S-005:** portable secret delivery MUST NOT depend on node-local
  `hostPath` files.

## ainfra relationship

Ainfra may provision a remote host, Kubernetes cluster, OpenBao, KBS, and
access paths, then return a non-secret target handover. Aibox consumes that
contract. It does not mutate ainfra state or assume that a signed historical
handover proves current liveness. `doctor` performs bounded connectivity and
capability checks appropriate to the requested action.
