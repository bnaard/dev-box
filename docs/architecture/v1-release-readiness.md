# V1 release readiness, threat model, and canaries

This document defines the stable-v1 release boundary. It is intentionally stricter than local unit-test success: deployment ownership and producer integration cross trust boundaries that fake clients cannot fully establish.

## Assets and trust boundaries

| Asset | Boundary | Required control |
| --- | --- | --- |
| `aibox.toml` and v0 backup | local project filesystem | preview never echoes contents; exact backup is written before atomic replacement; backups are confined to `.aibox/backups/v1-config` and mode 0600 on Unix |
| Deployment record | local `.aibox/deployments` and verified Kubernetes annotation | operation lock, record digest, deployment ID, image digest, desired-spec digest, namespace, and owner labels must all agree |
| Kubernetes/Compose resources | backend runtime | apply/status/logs/destroy address only the recorded deployment; destroy refuses untracked, foreign, unlabeled, or digest-mismatched resources |
| Credentials | operator secret store/environment/filesystem | contracts contain locators only; plans, records, diagnostics, and process argv must never contain secret values |
| Processkit installer request | aibox-to-producer CLI | typed argv, secret-name filtering, producer provenance, and no aibox interpretation of producer content |
| Release evidence | exact producer and disposable-cluster CI | generated artifacts tied to a release-candidate commit; no fixture or fake-client result satisfies M5 interruption/recovery or M7c |

## Threats and canaries

| Threat | Canary | Expected result |
| --- | --- | --- |
| A config preview leaks a secret | migration preview test injects `AIBOX_V1_SECRET_CANARY_DO_NOT_LEAK` | JSON report contains only paths, operation, and digests; canary is absent |
| A failed migration loses v0 config | migration apply test compares backup byte-for-byte before accepting the new boundary | exact backup exists before config replacement |
| A rollback is redirected to an attacker-controlled path | restore test supplies a backup outside the confined backup directory | restore refuses it |
| v0 lifecycle touches a v1 deployment | migration/restore coexistence test places a v1 deployment receipt before rollback | receipt is byte-for-byte unchanged |
| Destroy deletes a foreign resource | Compose and Kubernetes ownership tests change/miss labels or digests | guarded destroy returns the ownership error and performs no deletion |
| Processkit request leaks a credential | protocol test injects an environment secret canary | serialized argv omits it |
| Alpha pin drifts or unsigned assets are accepted | exact alpha.3 consumer gate verifies both published checksums and the signing key | only reviewed release assets reach the producer |
| Interrupted producer state is retried unsafely | real-producer gate interrupts after a durable journal, refuses normal retry, then runs recover and one retry | retry proceeds only from an unambiguous recovered state |
| A producer or recovery path leaks a credential | real-producer canaries scan argv, diagnostics, journal/state, logs, and recovery output | canary values are absent |
| Fake Kubernetes success is mistaken for live evidence | release audit requires complete M7c disposable-cluster lifecycle evidence | audit blocks when file is absent, malformed, incomplete, or bound to another candidate |

## Stable-v1 release audit

`aibox config release-readiness` is the machine/human gate. It must never declare readiness merely because a marker says “passed.” Today it blocks for:

1. M5 exact alpha.3 lifecycle, interruption/recovery, coexistence/rollback,
   and secret-safety evidence is incomplete.
2. M7c complete live disposable-cluster lifecycle evidence is absent.

M5 is split into explicit gates so the checked-in adapter and retained
release-candidate evidence cannot be conflated. The alpha pipeline runs the
exact alpha.3 consumer gate and checks its SHA-256 pins before tag creation.
M7c requires a CI-generated attestation with the candidate commit, cluster,
Kubernetes command, timestamp, and passed first/unchanged/changed apply,
drift/recovery, status/logs, exec/port-forward, ingress, and foreign-destroy
refusal scenarios. Release review retains the CI run alongside the attestation.

## Non-goals

- Provisioning Kubernetes clusters, ingress controllers, gateways, DNS zones, or secret stores.
- Recovering or deleting v1 deployments during v0 config rollback.
- Treating a local fake client, a rendered manifest, or a unit-test fixture as live-cluster release evidence.
- Shipping stable v1 before the two blocking gates pass.
