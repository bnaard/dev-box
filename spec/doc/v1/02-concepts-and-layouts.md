## Concepts

### Environment definition

The portable environment definition is the standard `.devcontainer/` tree:
`devcontainer.json`, referenced Dockerfiles or images, Compose files, Dev
Container Features, and native application configuration. These files are
canonical and user-editable; aibox does not regenerate them from another
content description.

### Dev Container Template and Feature

A Template is the standard OCI-distributed Dev Container scaffolding package.
Applying it initializes project-owned files. A Feature is the standard local,
tarball, or OCI-distributed unit for installing tools and container metadata.
Aibox defines neither a competing template format nor addon/component model.

### Deployment intent

`aibox.toml` is committed, human-readable deployment intent. It binds the
environment definition to named deployment profiles, interaction and workload
requirements, target references, security policy, storage classes, runtime
adapter candidates, and symbolic secret-provider references. It MUST NOT mirror
Feature options, Dockerfile content, application configuration, or arbitrary
Compose/Kubernetes fields.

`.aibox-local.toml` is optional, uncommitted operational binding for concrete
endpoints, local paths, ainfra results, secret-provider configuration, and
invocation defaults. It contains references rather than secret values.

### Lock, plan, materialization, environment and run

`aibox.lock` records immutable identities and digests for the Dev Container
definition, Features, images, selected runtime tools, and compatible contracts.
A plan binds normalized deployment intent, native inputs, target identity,
adapter/tool versions, security findings, storage choices, and intended actions.
A materialization is the runtime-native result produced by applying that saved
plan; it is not a second source of environment truth. An environment is the
named deployed instance. A run is one bounded operation with correlation ID,
events, result, diagnostics, and cleanup evidence.

### Runtime capability driver

A runtime capability driver is template- or Feature-owned current-environment
content: a versioned descriptor plus one or more fixed executables. Official
projectious.work capabilities use small POSIX shell scripts where practical and
invoke established tools; they do not require another fleet of compiled helper
binaries. A richer implementation MAY expose the same bounded domain through a
language-neutral stdio protocol. `aiboxctl` discovers, authorizes, invokes, and
audits drivers without knowing their implementation.

## Orthogonal dimensions

| Dimension | Initial values |
|---|---|
| Workload | `user-dev`, `headless-service`, `batch` |
| Target | `local`, `remote-host`, `kubernetes`, managed-container platform |
| Interaction | `human-ui`, `exec-attach`, explicit SSH, `none` |
| Definition | image, Dockerfile, Docker Compose |
| Deployment runtime | Dev Container CLI, Compose, Kubernetes API/kubectl, envbuilder where conforming, managed-container API/CLI adapter |
| Secret provider | user-managed, SOPS, OpenBao; future KBS/custom |
| Secret phase | build, deploy/create, interactive entry, runtime |
| Secret delivery | build secret, container environment, exec environment, mounted file, agent/CSI, application API |
| Secret scope | build step, container, service/file, process tree, application |
| Storage class | persistent, rebuildable cache, ephemeral, confidential |
| Source placement | local, synchronized remote, remote-native, image-baked |

- **AIBOX-CONCEPT-001:** these dimensions MUST remain independently
  representable; a target MUST NOT silently imply workload, interaction,
  provider, persistence, or runtime choice.
- **AIBOX-CONCEPT-002:** native definitions and selected policy MAY constrain
  invalid combinations and MUST produce actionable diagnostics.
- **AIBOX-CONCEPT-003:** runtime selection MUST be capability-based and MUST
  fail rather than silently omit unsupported semantics.

## Project layout

```text
project/
├── .devcontainer/
│   ├── devcontainer.json
│   ├── Dockerfile                    # optional
│   ├── compose.yaml                  # optional
│   ├── compose.override.yaml         # optional, user-owned
│   ├── features/                     # optional local Features
│   └── runtime/                      # optional native configs/providers
├── deploy/kubernetes/                # optional native target material
├── aibox.toml                        # deployment intent
├── aibox.lock
├── .aibox-local.toml                 # optional, ignored
└── .aibox/
    ├── plans/
    ├── runs/
    └── cache/
```

The established `.aibox-local.toml` spelling remains the initial local
operations filename. A later rename requires deterministic migration.

## Standard Template layout

```text
template/
├── devcontainer-template.json
├── .devcontainer/
│   ├── devcontainer.json
│   ├── Dockerfile
│   ├── compose.yaml
│   ├── features/
│   └── runtime/
├── deploy/kubernetes/                # optional aibox-compatible extension
├── README.md
└── tests/
```

Applying a Template is an explicit scaffolding operation. Resulting files belong
to the project. Aibox MAY record provenance and offer an explicit three-way
upgrade preview, but `up` and `apply` MUST NOT silently regenerate edited files.

## Storage model

Logical stores are distinct: workspace source, user home, rebuildable cache,
durable application data, and ephemeral secret delivery. Native definitions
declare mount points; deployment profiles bind storage implementation and
retention policy. Local home persistence defaults to a named engine volume,
with an explicit host bind supported. Remote hosts use remote volumes or paths;
Kubernetes uses PVCs; disposable headless runs may use ephemeral volumes or
tmpfs; confidential profiles require approved encrypted/attested storage.

- **AIBOX-LAYOUT-001:** plans and run evidence MUST contain no secret values.
- **AIBOX-LAYOUT-002:** aibox MUST NOT overwrite canonical Dev Container or
  native deployment files during ordinary lifecycle operations.
- **AIBOX-LAYOUT-003:** local and target-native overrides MUST remain
  distinguishable in effective configuration and provenance.
- **AIBOX-LAYOUT-004:** storage retention and deletion effects MUST be explicit
  in every destructive plan.
