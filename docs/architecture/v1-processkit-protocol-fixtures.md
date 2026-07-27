# V1 processkit installer integration

The M5 consumer is aligned with the producer-owned
`processkit.projectious.work/installer/v1alpha1` contract implemented by
processkit `v1.0.0-alpha.3` (tagged commit
`61929f9160b9b97063c5b8f10ad7cbff33c55e5c`). Aibox supplies only the
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

The gate exercises signed plan, install, verify, unchanged update, and
uninstall through the aibox adapter in an arbitrary disposable project. The
release candidate must additionally retain real-producer evidence for a durable
interruption, normal-retry refusal, `recover`, and one successful retry. Unit
coverage fixes request validation, forward-compatible result decoding,
exit/status agreement, request-file cleanup, and recover-before-retry
semantics; it does not replace that real-producer evidence.

Alpha.3 intentionally does not infer or mutate an existing v0 layout. The
consumer evidence is therefore operational coexistence, explicit v1 rollback,
and v1-only uninstall: v0 content survives a failed or removed v1 install, and
the bounded v0 bridge remains available. It is not an in-place v0 layout
conversion.

Secret canaries are supplied through environment, path, profile, and harness
inputs. Candidate logs, argv, request/result diagnostics, journals, state, and
recovery output must not contain their values.

## Stable-release gate

The bounded v0 installer remains intact. The tagged producer dependency is now
satisfied by processkit `v1.0.0-alpha.3`; aibox `v1.0.0-alpha.1` is the first
appropriate consumer prerelease target. Stable v1 remains blocked until the
release candidate has retained parity, coexistence/rollback, interruption and
recovery, secret-safety, and complete M7c evidence. Cancellation is process
cancellation; the only supported next operation is `recover`, followed by one
retry.
