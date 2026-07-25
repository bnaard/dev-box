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
| Release evidence | disposable-cluster CI | generated evidence artifact tied to a release-candidate commit; no fixture or fake-client result satisfies M7c |

## Threats and canaries

| Threat | Canary | Expected result |
| --- | --- | --- |
| A config preview leaks a secret | migration preview test injects `AIBOX_V1_SECRET_CANARY_DO_NOT_LEAK` | JSON report contains only paths, operation, and digests; canary is absent |
| A failed migration loses v0 config | migration apply test compares backup byte-for-byte before accepting the new boundary | exact backup exists before config replacement |
| A rollback is redirected to an attacker-controlled path | restore test supplies a backup outside the confined backup directory | restore refuses it |
| v0 lifecycle touches a v1 deployment | migration/restore coexistence test places a v1 deployment receipt before rollback | receipt is byte-for-byte unchanged |
| Destroy deletes a foreign resource | Compose and Kubernetes ownership tests change/miss labels or digests | guarded destroy returns the ownership error and performs no deletion |
| Processkit request leaks a credential | protocol test injects an environment secret canary | serialized argv omits it |
| Fixture success is mistaken for production readiness | release audit reads the compiled M5 provisional marker | audit blocks until real producer integration replaces it |
| Fake Kubernetes success is mistaken for live evidence | release audit requires complete M7c disposable-cluster evidence | audit blocks when file is absent, malformed, or incomplete |

## Stable-v1 release audit

`aibox config release-readiness` is the machine/human gate. It must never declare readiness merely because a marker says “passed.” Today it blocks for:

1. M5: the processkit #118 producer protocol remains fixture-only.
2. M7c: no live disposable-cluster attestation is present.

The M5 check is source-backed: it detects the compiled provisional-protocol marker, rather than accepting a manually created project file. M7c requires a CI-generated attestation with a commit, cluster identifier, Kubernetes command, and timestamp. Release review must retain the CI run itself alongside that attestation.

## Non-goals

- Provisioning Kubernetes clusters, ingress controllers, gateways, DNS zones, or secret stores.
- Recovering or deleting v1 deployments during v0 config rollback.
- Treating a local fake client, a rendered manifest, or a unit-test fixture as live-cluster release evidence.
- Shipping stable v1 before the two blocking gates pass.
