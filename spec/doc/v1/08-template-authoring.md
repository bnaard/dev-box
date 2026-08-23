## Authoring promise

A template author can create a conforming template from published schemas,
examples, documentation, and `aibox template validate` without reading aibox
implementation source.

## Manifest responsibilities

The template manifest declares:

- identity, version, license, source and compatibility;
- supported workload, target, interaction, architecture, and builder
  dimensions;
- image and deployment entry points;
- addons, defaults, dependencies, conflicts, version choices and checks;
- runtime-changeable features and their allowed values;
- required target and native-tool capabilities;
- named secret requirements without values;
- generated/user-owned paths; and
- examples, tests, migration notices, and documentation.

- **AIBOX-TEMPLATE-001:** templates MUST NOT rely on undocumented engine
  behavior or known-template branching.
- **AIBOX-TEMPLATE-002:** every installed addon version MUST be explicit,
  constrained, or deliberately `latest` with a visible reproducibility warning.
- **AIBOX-TEMPLATE-003:** addon disablement MUST remove its declared image and
  deployment contributions on the next clean compilation/build.
- **AIBOX-TEMPLATE-004:** conflicts and prerequisites MUST be declarative and
  diagnosed before build.
- **AIBOX-TEMPLATE-005:** a template MUST document native escape hatches and
  the effect of user overrides on its support claim.

## Image tooling

Dev Container configuration is a first-class interoperability surface, not the
entire aibox template. A template may provide `devcontainer.json`, Features,
Dockerfile/BuildKit sources, and Compose. Aibox does not invent another package
installation language.

Templates SHOULD use multi-stage builds, verified downloads, non-root runtime
users, explicit base-image identity, build cache boundaries, and native secret
mounts. They MUST publish supported builders and architectures.

## Deployment tooling

Compose templates publish a generated base file. Users may add
`docker-compose.override.yaml`; aibox neither rewrites nor mirrors its complete
option surface in TOML.

Kubernetes templates publish manifests/Kustomize with documented overlay
points. Cluster-specific admission, policy, service mesh, storage, ingress,
Secret, and workload settings remain native configuration.

## Feature location

Previously built-in features move to templates/addons, including:

- processkit and AI harness installation;
- editor and terminal tools;
- yazi integrations and previews;
- archive preview;
- themes and prompts;
- tmux layouts and status configuration;
- language and cloud toolchains; and
- documentation/LaTeX helpers.

The engine can select and compose them only through generic contracts.

## Runtime-changeable features

A template may declare a bounded runtime contract:

```toml
schema = "aibox.runtime-features/v1"

[features.theme]
values = ["projectious-dark", "projectious-light"]
default = "projectious-dark"
apply = ["/usr/local/libexec/aibox/apply-theme"]

[features.tmux-layout]
values = ["focus", "review"]
default = "focus"
apply = ["/usr/local/libexec/aibox/apply-tmux-layout"]
```

`aiboxctl` validates values and invokes only manifest-declared executable/argv
templates installed in trusted image locations. It may display a terminal UI
or tmux popup, but the non-interactive CLI contract remains complete.

- **AIBOX-RUNTIME-001:** runtime-feature commands MUST NOT be supplied by
  mutable project content unless the selected policy explicitly trusts it.
- **AIBOX-RUNTIME-002:** runtime settings MUST identify persistence scope:
  process/session, environment volume, or user preference.
- **AIBOX-RUNTIME-003:** features MUST document conflicts, restart needs,
  reversibility, and headless behavior.
- **AIBOX-RUNTIME-004:** `aiboxctl` MUST operate without container-engine
  socket, deployment credentials, SOPS keys, vault root tokens, or KBS policy
  authority.

## Conformance

Template validation includes schema and unknown-field checks, contained paths,
supported dimension combinations, addon graph resolution, example compilation,
native config validation when tools are available, secret-reference checks,
generated/user ownership, and documentation presence. Clean-room authoring is
required before claiming the v1 template contract stable.
