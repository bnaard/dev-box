## Contract families

Aibox owns independently versioned schemas for:

- project intent;
- local operational configuration;
- template manifest;
- lockfile;
- deployment-bundle manifest;
- target handover consumed from ainfra or another provisioner;
- machine command results; and
- optional runtime-feature contract consumed by `aiboxctl`.

Each contract defines unknown-version and unknown-field behavior, supported
version window, compatibility rules, deprecation, migration, and positive and
negative fixtures.

## Project intent

Illustrative intent:

```toml
schema = "aibox.project/v1"

[template]
source = "git+https://github.com/example/aibox-templates.git//go-dev"
version = "1.2.0"

[environment]
workload = "user-dev"
target = "remote-host"
interaction = "exec-attach"

[addons.processkit]
enabled = true
version = "0.30.0"

[addons.go]
enabled = true
version = "1.26.5"

[runtime.theme]
default = "projectious-dark"
changeable = true

[[secrets]]
name = "github-token"
provider = "developer-sops"
phase = "interactive-entry"
delivery = "exec-env"
scope = "process-tree"
target = "GH_TOKEN"
```

Addon selection MUST preserve the current useful controls: enable/disable,
supported explicit version, template default, and deliberately unpinned/latest
where policy permits. A template, not the CLI, defines addon names, defaults,
dependencies, conflicts, supported versions, installation, and checks.

## Native overrides and the inner-system boundary

For Compose targets, a user-owned `docker-compose.override.yaml` MAY augment or
override generated Compose. A local override may be copied into the bundle; a
remote-native override may be selected from the target host. Aibox MUST NOT
rewrite either.

For Kubernetes, templates and users use native manifests and Kustomize
overlays/patches initially. Helm support MAY be added after its ownership,
rendering, and provenance contract is specified.

- **AIBOX-CONTRACT-001:** aibox configuration MUST NOT duplicate arbitrary
  Compose or Kubernetes fields.
- **AIBOX-CONTRACT-002:** aibox MUST expose the effective native deployment
  configuration and identify generated, local override, and remote override
  provenance.
- **AIBOX-CONTRACT-003:** aibox MAY warn when an override weakens a recommended
  posture; it MUST reject it only when technically invalid or prohibited by an
  explicitly selected security policy.
- **AIBOX-CONTRACT-004:** invoking native tools directly on the bundle MUST be
  documented and supported as an escape path.

## Compilation flow

```text
aibox.toml + local operation config
        + template + lock
        + selected native overrides
                    │
                    ▼
          validate and resolve
                    │
                    ▼
       deterministic compilation
                    │
                    ▼
  secret-free target-native bundle
                    │
           ┌────────┴─────────┐
           ▼                  ▼
   image builder       Compose/Kubernetes
```

Compilation is pure with respect to declared inputs. Capability discovery and
remote state are explicit inputs or later lifecycle checks, not hidden
template-rendering variables.

- **AIBOX-COMPILE-001:** identical normalized intent, lock, template bytes,
  selected override bytes, compiler version, and declared capability inputs
  MUST produce semantically identical bundle content.
- **AIBOX-COMPILE-002:** compilation MUST NOT execute template-supplied shell
  snippets on the host.
- **AIBOX-COMPILE-003:** generated native sources MUST be inspectable before
  build or deployment.
- **AIBOX-COMPILE-004:** bundle metadata MUST bind normalized input digests,
  template identity, addon resolution, compiler identity, and output digests.
- **AIBOX-COMPILE-005:** secret references MAY appear in the bundle; resolved
  secret values MUST NOT.

## Template sources and locking

Initial sources are contained local directories and immutable Git repository
revisions with optional subdirectories. Mutable references are resolved to an
immutable revision and content digest before compilation. Archives are
validated for traversal, links, special files, size, and entry limits.

- **AIBOX-SOURCE-001:** production use MUST lock immutable source identity and
  digest.
- **AIBOX-SOURCE-002:** source acquisition MUST fail closed on checksum,
  containment, or compatibility failure.
- **AIBOX-SOURCE-003:** credentials used to acquire private templates MUST NOT
  enter locks, bundles, logs, or evidence.

## ainfra target handover

Aibox MAY consume a versioned, non-secret ainfra target projection describing
capabilities, endpoints, access paths, symbolic credential and secret-provider
references, trust material references, and deployment provenance. Aibox MUST
not attempt broad infrastructure autodiscovery.

The handover is one result-contract projection, not a competing Ansible
inventory. Ainfra owns production; aibox owns validation and consumption.
Signatures prove binding to declared bytes and execution provenance; until the
future confidential phase, they do not prove that remote infrastructure is
currently live or still matches those bytes.

- **AIBOX-HANDOVER-001:** target handover MUST contain no secret value or
  private key.
- **AIBOX-HANDOVER-002:** aibox MUST verify schema, digest, compatibility,
  signer/freshness policy when required, and declared target capabilities.
- **AIBOX-HANDOVER-003:** absent or incompatible handover data MUST produce an
  actionable refusal; aibox MUST NOT guess endpoints or credentials.
- **AIBOX-HANDOVER-004:** live/remote attestation is reserved for the future
  confidential-computing contract.
