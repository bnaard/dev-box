## Configuration layers

From lower to higher precedence:

```text
built-in defaults
→ system policy
→ user policy/preferences
→ committed aibox.toml
→ uncommitted .aibox-local.toml
→ environment variables
→ invocation flags
```

Authority remains separate from precedence. Committed project/template content
cannot silently select executables, credentials, privileged targets, writable
host paths, logging egress, or weaker system security policy.

- **AIBOX-CONFIG-001:** path anchors, absent/empty/null behavior, environment
  names, unknown fields, and merge behavior MUST be documented and tested.
- **AIBOX-CONFIG-002:** `aibox config show --format json` MUST report every
  supported effective key once with redacted value, winning source, loaded
  layers, and rejected settings.
- **AIBOX-CONFIG-003:** project intent MUST remain human-readable TOML with
  comments permitted; aibox MUST NOT rewrite it except through an explicit or
  apply-triggered deterministic owned migration.

## Migration

Owned contracts carry explicit schema versions. Migrations are ordered,
deterministic transformations with before/after fixtures and idempotence tests.
`aibox apply` performs safe migrations by default and reports them. `aibox
config migrate --check` supports CI and review; `aibox config migrate` provides
an explicit UX.

A migration that requires intent, cannot preserve semantics/comments safely,
or conflicts with native user-owned files stops with an actionable preview.
AI-assisted migration MAY be documented as a user workflow but is not an
automatic conformance mechanism and cannot be required for headless operation.

Generated bundles are replaced from inputs rather than migrated in place.
User-owned Compose overrides, Kubernetes overlays, Dockerfiles, and Dev
Container files are never rewritten by the generic migrator.

## Output, logs, and evidence

Three channels remain separate:

1. requested command results;
2. operational diagnostics/logs; and
3. durable run/deployment evidence.

Human output uses stdout/stderr conventionally. Machine mode emits exactly one
versioned envelope on stdout and diagnostics on stderr. It distinguishes
success, refusal, cancellation, timeout, partial failure, and complete failure.

Structured operational events include timestamp, severity, stable event name,
component, operation, run ID, target attribution, outcome/error category, and
already-redacted fields. Logs rotate and never substitute for authoritative
run evidence.

## Run evidence

Evidence records, as applicable:

- command, actor/executor attribution, timestamps and outcome;
- product and contract versions;
- normalized non-secret input, template, lock and bundle digests;
- target-handover identity and verified signer/freshness status;
- effective native configuration digest;
- selected capabilities and adapter versions;
- child/remote operation outcomes;
- exact created/stopped/deleted resource identifiers;
- secret provider/reference/delivery metadata without values; and
- cleanup and recovery status.

Evidence does not contain Compose/Kubernetes secret values, SOPS plaintext,
private target data not necessary for recovery, raw environment dumps, or
vault tokens. A future attestation phase adds live confidential evidence
without retroactively overstating ordinary run records.

## Locking and recovery

Mutating operations acquire an environment-scoped lock with owner and stale
recovery information. Interruption leaves a recoverable run record. The next
operation or `doctor` reconciles exact owned staging, temporary secret paths,
tunnels, and deployment state. It never uses global container prune, arbitrary
remote deletion, or broad process termination.

## Exit categories

Stable categories include success, usage/configuration, compatibility,
prerequisite/capability, security refusal, target/dependency failure,
interruption/timeout, partial/recovery required, and internal error. Numeric
assignments are frozen with the CLI machine contract.
