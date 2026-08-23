## Principles

- Go for both host CLI and optional in-container controller.
- One dependency direction from commands toward application use cases and
  ports, then outward through adapters.
- Pure compilation separated from filesystem, process, network, clock, and
  credential effects.
- Standards and target capabilities over product-name conditionals.
- Small interfaces owned by consumers; explicit constructors; no global
  mutable state.
- Standard library preferred when clear and testable.
- Template content treated as data and native source, never engine code.

## Proposed source layout

```text
cmd/
├── aibox/
└── aiboxctl/
internal/
├── app/              # use cases and orchestration
├── command/          # CLI parsing and rendering
├── contract/         # schemas, version negotiation, normalized models
├── config/           # layered config and migration
├── template/         # acquisition, verification, manifest, lock
├── compile/          # pure bundle compilation
├── bundle/           # layout, digests, provenance
├── target/           # target capabilities and handover
├── compose/          # standard Compose adapter
├── kubernetes/       # kubectl/Kustomize adapter
├── container/        # narrow exec/attach/inspect adapter
├── remote/           # SSH transfer and bounded remote command execution
├── source/           # project-source placement/synchronization
├── secret/           # reference model and provider/delivery ports
├── sops/             # external SOPS adapter
├── openbao/          # OpenBao configuration/integration adapter
├── runtimefeature/   # optional aiboxctl contract and application
├── diagnostic/       # doctor findings and effective configuration
├── evidence/         # run records and cleanup evidence
├── logging/          # structured redacted operational events
├── output/           # human/plain/JSON result rendering
├── execx/            # child-process boundary
└── fsx/              # contained filesystem boundary
```

Exact package names may change during foundation implementation, but the
responsibility and dependency rules are normative.

## Core dependency flow

```text
cmd → command → app → domain contracts/ports
                         ↑
        adapters implement ports
```

Compilation depends only on validated normalized models and an abstract output
writer. It does not depend on Compose, Kubernetes, SSH, SOPS, OpenBao, tmux,
processkit, or an addon implementation.

- **AIBOX-ARCH-001:** `internal/compile` MUST NOT import target, process,
  network, secret-provider, or template-specific feature adapters.
- **AIBOX-ARCH-002:** `app` MUST invoke external behavior through bounded
  interfaces suitable for fakes and contract tests.
- **AIBOX-ARCH-003:** adapter packages MUST NOT call each other through hidden
  global state; application use cases coordinate them.
- **AIBOX-ARCH-004:** no production package outside template parsing/tests MAY
  contain processkit-specific installation or migration logic.
- **AIBOX-ARCH-005:** target support is selected through manifest and probed
  capability contracts, not runtime-brand switches except for documented
  compatibility workarounds isolated in adapters.

## Template-engine boundary

The engine understands generic concepts: addon selection, version constraints,
file contributions, native build/deploy inputs, runtime feature declarations,
requirements, conflicts, checks, and secret references. It does not understand
what Go, processkit, vim, yazi, a harness, or a theme does.

Templates use standard tools in priority order:

1. Dev Container Features or equivalent portable feature packages where fit;
2. Dockerfile/BuildKit for image construction;
3. native package managers and installers inside image build stages;
4. Compose for Docker-compatible topology;
5. manifests/Kustomize for Kubernetes topology; and
6. declarative aibox metadata only for orchestration not expressible natively.

Arbitrary host-side bash hooks are prohibited. Container-side shell remains
ordinary Dockerfile/entrypoint content and is reviewed under that boundary.

## aibox and aiboxctl relationship

The binaries MAY share contract and pure domain packages, but they are separate
applications and release artifacts. `aiboxctl` does not import deployment,
remote, container-engine, template acquisition, or secret-provider adapters.
Its binary size and dependency set remain deliberately small.

## Concurrency and cancellation

Concurrency is bounded and used only for independent acquisition, validation,
or target operations. Every goroutine has an owner, cancellation path, and
joined completion. Child processes and remote commands receive contexts and
bounded shutdown before force termination of the exact child. Output ordering
uses correlation IDs and never invents a false sequential lifecycle.

## Dependency policy

Likely reviewed dependencies include TOML parsing/edit preservation, JSON
Schema validation, SSH, and container API clients. Native CLIs remain valid
adapters when their contracts are stable and using them avoids large SDK
surface. Every dependency requires purpose, maintenance, license, provenance,
platform, vulnerability, and transitive-weight review.

Go language selection criteria, now decided, remain release constraints:

- portable single binaries for Linux/macOS and container images;
- strong process, filesystem, API, concurrency, and testing support;
- maintainable typed contracts and compatibility adapters;
- reuse of ainfra engineering conventions where product semantics agree; and
- no shared ainfra/aibox core until duplicated stable behavior proves a narrow
  reusable library boundary. Premature framework extraction is prohibited.

## Size and review signals

Large files, broad interfaces, cyclic conceptual ownership, feature-name
switches, mutable singletons, and repeated target logic trigger design review.
Generated schema bindings are isolated. Security-sensitive adapters receive
focused negative tests and independent review.
