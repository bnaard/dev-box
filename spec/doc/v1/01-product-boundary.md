## Goals

- Replace the current implementation with a maintainable Go CLI and a small,
  optional Go `aiboxctl` binary.
- Compile human-readable project intent and versioned templates into
  inspectable, target-native image and deployment artifacts.
- Keep engine behavior independent of template-installed tools, addons,
  harnesses, themes, and processkit.
- Support interactive and headless workloads on local and remote
  Docker-compatible hosts and Kubernetes clusters.
- Preserve standard Compose and Kubernetes customization rather than
  reproducing those systems in aibox configuration.
- Offer secure, composable secret-provider and delivery choices without
  forcing one provider, exposure scope, or runtime mechanism on every user.
- Integrate with ainfra through a versioned, non-secret target handover while
  keeping infrastructure provisioning outside aibox.
- Provide MCP as the primary agent interface and a complete equivalent CLI over
  one application core.
- Preserve a one-way trust boundary: aibox manages distinct other
  environments, while optional `aiboxctl` manages only its current environment.

## Target users

- developers who need reproducible interactive AI-enabled workspaces;
- teams operating remote development hosts or Kubernetes development Pods;
- operators building headless agent or automation images;
- template authors packaging reusable image and deployment behavior; and
- platform teams supplying infrastructure and secret providers.
- AI agents and humans authoring templates or managing distinct downstream
  aibox environments from a suitable executor.

## Core use cases

1. Initialize a project from a template and explicit target/profile choices.
2. Validate and migrate project intent deterministically.
3. Resolve and lock a template plus selected addon versions.
4. Compile a secret-free image/deployment bundle and inspect the effective
   native configuration.
5. Build and start a local interactive container.
6. Transfer a bundle over SSH and run Compose on a remote development host.
7. Apply rendered resources to a local or remote Kubernetes API.
8. Enter an interactive environment through runtime-native exec/attach.
9. Run a headless workload with brokered, renewable secrets.
10. Change template-declared runtime features such as theme or tmux layout
    through optional `aiboxctl` without rebuilding the image.
11. Perform the bounded lifecycle against a distinct target through guarded
    MCP tools with equivalent CLI recovery.

## Non-goals

- **AIBOX-NONGOAL-001:** aibox is not an infrastructure provisioner; ainfra,
  platform tooling, or the operator supplies hosts, clusters, networking, and
  secret services.
- **AIBOX-NONGOAL-002:** aibox is not a container runtime, image builder,
  Compose implementation, Kubernetes distribution, or package manager.
- **AIBOX-NONGOAL-003:** `aibox.toml` is not a second Compose, Kubernetes,
  Dev Container, Dockerfile, Helm, or Kustomize language.
- **AIBOX-NONGOAL-004:** aibox and `aiboxctl` are not vaults, KBS
  implementations, cryptographic libraries, or general secret receivers.
- **AIBOX-NONGOAL-005:** processkit, AI harnesses, editors, tmux, yazi, themes,
  previews, and language toolchains are not built-in engine features.
- **AIBOX-NONGOAL-006:** v1 does not promise transparent compatibility with
  every v0 configuration or generated file.
- **AIBOX-NONGOAL-007:** aibox does not require SSH inside Kubernetes Pods;
  cluster-native exec/attach is the default interaction mechanism.
- **AIBOX-NONGOAL-008:** v1 does not create a proprietary plugin scripting
  language for installation or deployment.
- **AIBOX-NONGOAL-009:** aibox does not manage the environment in which its own
  process runs and does not bridge a managed environment to its creator host,
  engine, cluster authority, or lifecycle credentials.
- **AIBOX-NONGOAL-010:** `aiboxctl` is not a proxy for aibox and does not manage
  another environment.

## Ownership boundary

| Owner | Responsibilities |
|---|---|
| aibox application core | Intent parsing, deterministic migration, template acquisition and lock, compilation, capability probing, bundle transfer, native-tool invocation, diagnostics, evidence, cleanup for distinct target environments |
| MCP and CLI adapters | Typed request conversion, capability presentation, result rendering, transport behavior, and delegation to identical application use cases |
| Template | Image sources, addons, supported versions, deployment sources, interaction features, runtime assets, target/profile support, documentation and tests |
| Native tools | Dev Container interpretation, image builds, Compose merging/lifecycle, Kubernetes rendering/apply, runtime exec/attach |
| `aiboxctl` | Optional bounded application of template-declared runtime settings inside its current running environment only |
| Secret provider | Encryption, storage, authentication, authorization, lease/renewal, revocation, or attested release |
| ainfra/platform | Infrastructure provisioning, target capabilities and endpoints, trust and credential references, OpenBao/KBS deployment |
| User override/overlay | Native target-specific customization and accepted security consequences |

- **AIBOX-BOUNDARY-001:** production engine code MUST NOT branch on a known
  addon, processkit, harness, editor, theme, or template feature name.
- **AIBOX-BOUNDARY-002:** a template MUST express installation and deployment
  specifics through native sources and versioned declarative metadata, not
  arbitrary snippets executed by a privileged aibox shell.
- **AIBOX-BOUNDARY-003:** aibox MUST preserve a documented path from generated
  output to direct use of the underlying standard tools.
- **AIBOX-BOUNDARY-004:** an aibox installation MAY itself run in any suitable
  executor, including a container, but every managed target MUST be a distinct
  environment with no authority path back to its creator.
- **AIBOX-BOUNDARY-005:** ordinary templates MUST NOT require aibox or
  `aiboxctl`; installing either is an explicit template/workload choice.

## Supported delivery

The host CLI is distributed as signed or attested Linux and macOS binaries for
`amd64` and `arm64`. Windows native distribution is deferred unless a roadmap
amendment supplies compatibility and release evidence. Templates MAY build
Linux container images for any architecture their manifest and builders prove.

`aiboxctl` is optional per template and image. It MUST NOT be required for a
valid headless image or for deployments that do not declare runtime-changeable
features.
