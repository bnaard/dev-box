## Concepts

### Project intent

`aibox.toml` is committed, human-readable intent. It identifies contract
version, template, selected target/profile dimensions, addon selections and
supported version overrides, runtime-feature defaults, source strategy, and
secret references or policies. It contains no secret values and does not
duplicate complete native deployment schemas.

### Local operational configuration

`.aibox-local.toml` is optional, uncommitted configuration for target endpoints,
local path selections, credential references, and invocation defaults. It does
not legitimize plaintext secrets; secrets remain provider inputs.

### Template

A template is a versioned package containing a manifest, image sources,
standards-based deployment sources, addon definitions or references, runtime
assets, documentation, examples, and conformance fixtures. Processkit is one
possible addon selected by a template or project.

### Lock

`aibox.lock` records exact template identity, content digest, addon resolution,
tool or image selections that affect reproducibility, and compatible contract
versions. It contains no credentials or target-local secret values.

### Deployment bundle

A bundle is the compilation result. It is a directory, not an opaque binary.
It contains target-native sources, a redacted manifest, locks, and provenance.
Humans can inspect it and invoke its declared standard tools without aibox.

### Environment and run

An environment is a named desired deployment such as `local-dev` or
`remote-runner`. A run is one bounded aibox operation with correlation ID,
inputs, result, diagnostics, and cleanup evidence.

### Addon and runtime feature

An addon changes image or deployment content. A runtime feature changes
declared behavior of an already-running environment when the template supports
it. Themes and tmux layouts are examples; neither is an engine concern.

## Orthogonal dimensions

| Dimension | Initial values |
|---|---|
| Workload | `user-dev`, `headless` |
| Target | `local`, `remote-host`, `kubernetes` |
| Interaction | `exec-attach`, `ssh` only when the target explicitly requires it, `none` |
| Image origin | build from template, use pinned published image |
| Deployment engine | Compose-compatible, Kubernetes manifests/Kustomize; later Helm when justified |
| Secret provider | user-managed, SOPS, OpenBao; future KBS/custom |
| Secret phase | build, deploy/create, interactive entry, runtime |
| Secret delivery | build secret, container environment, exec environment, mounted file, agent/CSI, application API |
| Secret scope | build step, container, service/file, process tree, application |
| Source placement | local, synchronized remote, remote-native, image-baked |

- **AIBOX-CONCEPT-001:** these dimensions MUST remain independently
  representable; a target MUST NOT silently imply an unrelated workload,
  interaction, provider, or feature choice.
- **AIBOX-CONCEPT-002:** templates MAY constrain invalid combinations and MUST
  publish those constraints as capabilities with actionable diagnostics.
- **AIBOX-CONCEPT-003:** security policy MAY prohibit a technically possible
  combination, but a default recommendation MUST NOT be represented as a
  runtime impossibility.

## Project layout

```text
project/
├── aibox.toml
├── aibox.lock
├── .aibox-local.toml                # optional, ignored
├── docker-compose.override.yaml     # optional, user-owned
├── .devcontainer/                   # optional standard user-owned inputs
├── deploy/                          # optional Kustomize/manifests/overlays
└── .aibox/
    ├── bundles/                     # generated, disposable/reproducible
    ├── runs/                        # redacted run evidence
    └── cache/                       # content-addressed template cache
```

The established `.aibox-local.toml` spelling remains the v1 migration input and
initial local-operations filename. Any later rename is a public contract change
and requires a deterministic migration and specification amendment.

## Template layout

```text
template/
├── aibox-template.toml
├── README.md
├── image/
│   ├── Dockerfile
│   └── devcontainer-feature.json    # optional standard feature package
├── deploy/
│   ├── compose.yaml                 # optional
│   └── kubernetes/                  # optional manifests/Kustomize
├── addons/
├── runtime/
│   ├── aiboxctl.toml                # optional feature contract
│   └── assets/
├── examples/
└── tests/
```

Templates SHOULD reuse Dev Container Features where the standard expresses the
installation cleanly. Dockerfile/BuildKit remains the universal image-build
escape hatch. Compose and Kubernetes sources remain authoritative for target
topology.

## Bundle layout

```text
bundle/
├── bundle.json
├── aibox.lock
├── compose.yaml                     # when selected
├── compose.override.yaml            # copied local override, when selected
├── kubernetes/                      # when selected
├── image/
├── runtime/
└── provenance/
    ├── inputs.json
    └── digests.json
```

- **AIBOX-LAYOUT-001:** generated bundles MUST contain no secret values.
- **AIBOX-LAYOUT-002:** generated files MUST carry ownership markers and MUST
  NOT be silently merged with user edits.
- **AIBOX-LAYOUT-003:** local and remote-native overrides MUST be distinguishable
  in effective-configuration and provenance output.
- **AIBOX-LAYOUT-004:** bundle paths MUST be relocatable; producer-local
  absolute paths MAY be diagnostic metadata but MUST NOT be the only locator.
