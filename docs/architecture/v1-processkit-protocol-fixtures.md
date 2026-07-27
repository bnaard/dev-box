# V1 processkit installer integration

The M5 consumer is aligned with the producer-owned
`processkit.projectious.work/installer/v1alpha1` contract implemented by
processkit `v1.0.0-alpha.2` (tagged commit
`c232656a695dfb72dafaaddc19f406eba0e24c6a`). Aibox supplies only the
operation, target root, release input paths, profiles, harness intent, and
explicit mutation acknowledgement.
Processkit owns all installation policy and interprets the request without
aibox inspecting layouts, skills, templates, migrations, MCP topology, or
harness projections.

The request is written to a private temporary file and passed as typed argv to
`processkit execute --request <path>`, never through a shell. The file is
removed after invocation. Aibox validates the versioned result envelope and
preserves producer extensions without interpreting processkit-owned state,
changes, conflicts, warnings, errors, or provenance.

The release gate downloads the exact-pinned source archive, native Linux
aarch64 installer, signed release envelope, signature, and public key. It
verifies the published archive and installer SHA-256 values before exercising
the signed-release request path:

```sh
./scripts/test-processkit-v1-consumer.sh
```

For development compatibility testing against a processkit checkout, pass its
path explicitly:

```sh
./scripts/test-processkit-v1-consumer.sh /path/to/processkit
```

The gate exercises plan, install, verify, unchanged update, and uninstall
through the aibox adapter in an arbitrary disposable project.
Unit coverage additionally fixes request validation, forward-compatible result
decoding, exit/status agreement, request-file cleanup, and recover-before-retry
semantics.

## Stable-release gate

The bounded v0 installer remains intact. The tagged producer dependency is now
satisfied by processkit `v1.0.0-alpha.2`; stable v1 remains blocked on the
remaining parity, migration, rollback, interruption, and secret-safety gates.
Cancellation is process cancellation; the only supported next operation is
`recover`, followed by one retry.
