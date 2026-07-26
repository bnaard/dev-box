# V1 processkit installer integration

The M5 consumer is aligned with the producer-owned
`processkit.projectious.work/installer/v1alpha1` contract implemented by
processkit PR #123. Aibox supplies only the operation, target root, release
input paths, profiles, harness intent, and explicit mutation acknowledgement.
Processkit owns all installation policy and interprets the request without
aibox inspecting layouts, skills, templates, migrations, MCP topology, or
harness projections.

The request is written to a private temporary file and passed as typed argv to
`processkit execute --request <path>`, never through a shell. The file is
removed after invocation. Aibox validates the versioned result envelope and
preserves producer extensions without interpreting processkit-owned state,
changes, conflicts, warnings, errors, or provenance.

For development compatibility testing against a processkit checkout:

```sh
./scripts/test-processkit-v1-consumer.sh /path/to/processkit
```

The gate builds the standalone producer and exercises install, verify, update,
and uninstall through the aibox adapter in an arbitrary disposable project.
Unit coverage additionally fixes request validation, forward-compatible result
decoding, exit/status agreement, request-file cleanup, and recover-before-retry
semantics.

## Stable-release gate

The bounded v0 installer remains intact. Stable v1 stays blocked until a tagged
processkit prerelease contains this protocol, the producer's
`scripts/test-installer-local.sh` passes for that tag, and this consumer gate
passes against the same tag. Cancellation is process cancellation; the only
supported next operation is `recover`, followed by one retry.
