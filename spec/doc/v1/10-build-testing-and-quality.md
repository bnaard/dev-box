## Application profiles

Aibox declares these company-standard profiles:

- CLI application: host `aibox` and `aiboxctl`;
- schema/protocol package: owned contracts and machine results;
- infrastructure/template product: template and deployment conformance;
- documentation/website; and
- host-gated release overlay where container-engine, native macOS, signing, or
  publication authority is unavailable in the development environment.

## Developer interface

The Go foundation MUST provide direct, documented commands for formatting,
vet/static analysis, tests, race tests, vulnerability analysis, build, schema
validation, examples, and documentation. A convenience script may sequence
them but cannot be the only interface.

## Test layers

| Layer | Required coverage |
|---|---|
| Unit | parsers, normalization, migration, merge, graphs, redaction, path containment, capability decisions |
| Component | definition resolver, plan binding, internal runtime adapters, provider/delivery orchestration, remote transfer, child-process contracts |
| Black box | CLI help/errors/results, config precedence, migration, plan/apply/up/attach/stop/destroy with fakes |
| MCP contract | Capability discovery, strict schemas, CLI parity, authorization, durable operations, transport, and adversarial agent-interface behavior |
| Integration | supported Compose engines, Docker/OrbStack/Podman capabilities, SSH, kubectl/Kustomize, managed-container API/CLI, SOPS, OpenBao contracts |
| Disposable E2E | local/remote interactive, autonomous headless-service and batch deployments, Kubernetes, managed-container targets, build secrets, recovery and exact cleanup |

Default tests are deterministic, credential-free, offline, parallel-safe, and
isolated from real user home, keychain, Docker context, kubeconfig, SSH config,
and environment. Live tests are opt-in and declare credentials, cost,
isolation, timeout, and cleanup.

## Required risk coverage

- positive and negative fixtures for every owned schema/version;
- migration idempotence and comment/semantic preservation;
- canonical user ownership and native override handling;
- archive/path traversal, symlink, special-file, size, and race attacks;
- argument injection and malicious template/remote output;
- redaction across chunks, concurrency, errors, logs, evidence, and machine
  output;
- secret absence from plans/staging, args, layers, image config, logs, evidence, and
  fixtures;
- temporary-file cleanup on success, failure, cancellation, restart, and
  reboot-recovery simulation;
- engine inspection rather than human-output parsing;
- SSH host verification, transfer digest mismatch, partial transfer, remote
  override drift, timeout, and cancellation;
- Kubernetes context/namespace authority, apply refusal, rollout failure,
  exec/attach, Secret state warnings, and cleanup;
- managed-container capacity/cost binding, immutable image identity, platform
  credential isolation, interruption, dynamic access paths, and exact deletion;
- autonomous-agent startup/readiness/liveness/progress, restart budget, durable
  state recovery, duplicate prevention, resource/spend bounds, egress policy,
  graceful/forced termination, kill switch, and rollback;
- tmux new/existing-server environment behavior;
- OpenBao authentication, lease/rotation, outage, denial, and revocation; and
- future KBS reference/attestation negative cases before that phase ships.
- MCP prompt/tool injection, immutable tool manifests, collision/shadowing,
  capability and authorization denial, target substitution, secret extraction,
  retry/replay, cancellation, and lost-session recovery;
- executor/target identity and creator-boundary refusal, including an aibox
  process inside one container managing only a distinct downstream target; and
- proof that no host bridge or engine socket is projected into managed
  environments and `aiboxctl` cannot reach external lifecycle authority.
- runtime-capability-driver discovery, namespace collision, protocol mismatch, fixed
  executable invocation, MCP policy, timeout, persistence effect, and refusal
  of host/deployment authority;

## Runtime compatibility matrix

The release gate exercises supported versions and capability combinations, not
only product names. The initial intended matrix covers:

- Linux Docker Engine/Compose;
- macOS Docker Desktop;
- macOS OrbStack;
- rootless Linux Podman with its supported Compose/API path;
- at least one local Kubernetes cluster; and
- at least one remote Kubernetes API and remote Linux Compose host.

Unsupported or untested combinations remain explicit. A compatibility claim
requires real-tool evidence for the claimed operation set.

## Go quality and supply chain

Required checks include `gofmt`, `go vet`, `go test ./...`, `go test -race
./...` on supported Linux, selected static analysis, `govulncheck`, secret
scanning, dependency/license review, and artifact scanning. Release artifacts
include checksums, SBOMs, source/toolchain identity, and signatures or
attestations under the company release standard.

Dependencies are locked through `go.mod`/`go.sum`; build and test tool versions
are pinned or constrained through the selected standard tooling/template. Go
release builds use reproducible metadata and avoid embedding local paths or
credentials.

## Host-gated verification

When development runs inside an isolated container without a host engine,
native macOS, signing identity, or publication authority, it produces an
immutable version-bound handover. The host verifier validates digests and
schema before using reviewed capability adapters. It does not grant the
development container a host socket, shell, credentials, broad mount, or
privilege.

Applicable conformance includes Docker Desktop, OrbStack, Podman, exact image
and container cleanup, changed-surface selection, vulnerability disposition,
credential-free validation, retryable publication, and independent
post-publication verification.

## Release gate

No required check may skip unexpectedly. A release candidate must have clean
worktree identity, complete supported-platform builds, schemas/examples/docs
in agreement, security evidence, disposable lifecycle evidence, and migration
journeys. Evidence binds the exact source and artifact digests.
