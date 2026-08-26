## Authoring posture

Aibox-compatible Templates are standard Dev Container Templates. Authors use
`devcontainer-template.json`, `.devcontainer/devcontainer.json`, Dockerfiles,
standard Features, Compose, native application configuration, and optional
Kubernetes material. There is no `aibox-template.toml`, addon DSL, component
format, or `values.toml` contract.

Template options are limited to standard scaffolding choices. Detailed Feature
options remain directly on Feature references. Application settings remain in
their native files. After application, generated project files are canonical
and user-owned.

- **AIBOX-TEMPLATE-001:** a conforming Template MUST satisfy the upstream Dev
  Container Template format and include a valid environment definition.
- **AIBOX-TEMPLATE-002:** installable tool contributions SHOULD use existing
  or local Dev Container Features before custom Dockerfile logic.
- **AIBOX-TEMPLATE-003:** local Features MUST remain contained below the
  `.devcontainer/` tree and receive the same digest and policy evaluation as
  remote Features.
- **AIBOX-TEMPLATE-004:** target-specific topology uses Compose or
  Kubernetes-native sources; aibox configuration MUST NOT duplicate it.
- **AIBOX-TEMPLATE-005:** Template application and upgrade are explicit; normal
  lifecycle commands MUST NOT overwrite user-edited standard files.

## Former v0.x features

| Concern | v1 authoring location |
|---|---|
| Languages, CLIs, harnesses | Dev Container Features |
| Processkit binary/tool setup | ordinary Feature; repository context remains processkit-owned |
| Base operating system | image or Dockerfile |
| tmux, Yazi, prompts, themes | Feature-installed and template/project-native configuration |
| Yazi preview dependencies/plugins | Feature plus native Yazi configuration |
| LaTeX toolchain | Feature |
| LaTeX preview companion | Compose service/include and Kubernetes-native equivalent |
| Audio client | Feature |
| Audio host/container connection | target-native mount/network configuration plus adapter capability and policy |
| Auxiliary n8n/database/browser service | Compose include or Kubernetes-native resource |

A definition may support terminal/tmux operation and VS Code simultaneously.
Editor customizations are optional standard `customizations` entries and do not
change the terminal-first runtime.

## Runtime capability drivers

Runtime behavior is implemented by content installed with the Template or
Feature, not by adding domain code to `aiboxctl`. Official projectious.work
drivers default to auditable POSIX shell scripts that validate typed arguments
and invoke established tools.

```text
.devcontainer/runtime/providers/latex/
├── provider.json
├── aibox-latex-build
├── aibox-latex-watch
└── aibox-latex-status
```

Illustrative metadata:

```json
{
  "schemaVersion": "aibox.runtime-provider/v1",
  "id": "latex",
  "protocol": "fixed-actions",
  "actions": {
    "build": {"command": ["/usr/local/bin/aibox-latex-build"], "mutability": "workspace-write"},
    "watch": {"command": ["/usr/local/bin/aibox-latex-watch"], "mutability": "runtime"},
    "preview_status": {"command": ["/usr/local/bin/aibox-latex-status"], "mutability": "read-only"}
  }
}
```

The Feature owns these scripts, the companion endpoint, tmux command, file
layout, process-lifecycle choice, and application API. A foreground watcher,
template-declared service, or Feature-owned supervisor integration are all
possible without teaching aiboxctl LaTeX semantics. `aiboxctl` owns only
generic discovery, policy, fixed-action invocation, result handling, audit, and
MCP/CLI projection. Rich domains MAY expose a dedicated MCP-over-stdio provider,
but a compiled helper is not the default. Neither path permits shell command
strings.

- **AIBOX-RUNTIME-001:** driver identity and bytes MUST be covered by the
  environment definition/image provenance and retained in runtime evidence.
- **AIBOX-RUNTIME-002:** driver tools MUST be namespaced and collision-free,
  with input/output schemas, mutability, persistence effect, timeout, and
  approval metadata.
- **AIBOX-RUNTIME-003:** drivers MUST run as the environment user unless an
  explicit current-environment policy authorizes narrower elevation.
- **AIBOX-RUNTIME-004:** drivers MUST NOT receive host, creator, deployment,
  engine, Kubernetes, or general secret-provider authority.
- **AIBOX-RUNTIME-005:** a driver controlling a sidecar uses a scoped
  template-owned service-network protocol; it never reaches through a host
  lifecycle bridge.
- **AIBOX-RUNTIME-006:** projectious.work drivers SHOULD be POSIX-shell-first;
  shell scripts MUST use strict error handling, fixed executable invocation,
  validated inputs, bounded output, and `exec` for the foreground process
  where appropriate. Bash or another implementation language requires
  demonstrated complexity or portability need.

## Conformance

Template and Feature conformance combines upstream validation with aibox checks
for provenance, secure-agent policy compatibility, target declarations,
runtime-capability-driver descriptors, native material, storage effects, and
fixtures.
Checks run in disposable environments and must cover both expected success and
unsupported combinations.
