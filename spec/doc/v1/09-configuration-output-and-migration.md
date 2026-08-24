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

Generated plans and run staging are replaced from inputs rather than migrated
in place. User-owned Compose overrides, Kubernetes overlays, Dockerfiles, and
Dev Container files are never rewritten by the generic migrator.

Before the clean implementation replaces Aibox v0 or the Rust v1 experiments,
the project MUST publish a concise lessons and capability disposition matrix.
Material behavior is classified as retain, redesign, discard, defer, or
investigate. Retained behavior becomes a requirement or black-box fixture; old
source code and module boundaries are not reused by default.

- **AIBOX-MIGRATION-010:** the disposition matrix MUST cover user-visible
  lifecycle behavior, migrations, ownership safeguards, evidence and recovery,
  packaging/platform failures, and downstream contracts that previously
  exposed defects or operational value.
- **AIBOX-MIGRATION-011:** the matrix is not a compatibility promise and MUST
  NOT require copying implementation code from Aibox v0 or the Rust line.

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
- normalized non-secret input, standard definition, lock and plan digests;
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

## Standard environment result

Every successful create or update operation produces one Aibox-owned,
versioned environment result. It describes the environment and workload facts
Aibox can substantiate, including stable environment and workload IDs, target
binding, resolved definition and image provenance, generic lifecycle and
readiness state, declared connection methods, retained storage, and evidence
references.

The result MAY reference an imported ainfra infrastructure result by immutable
identity and digest. It MUST NOT copy, extend, or claim ownership of that
artifact. Airunner registration, agent availability, model activity, and
Processkit entity state remain outside the Aibox result.

- **AIBOX-RESULT-001:** an environment result MUST remain useful when the
  target is local or explicitly configured and no ainfra result exists.
- **AIBOX-RESULT-002:** Aibox results MUST distinguish generic workload
  process readiness from application-specific readiness determined by Kaits,
  Airunner, or another consumer.
- **AIBOX-RESULT-003:** product resource IDs and an optional opaque external
  correlation ID MUST remain separate fields with separate semantics.

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
