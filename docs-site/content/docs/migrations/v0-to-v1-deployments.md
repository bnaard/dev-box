---
weight: 3
---

# Moving from v0 containers to v1 deployments

v1 deployment orchestration is opt-in. A v0 project remains a v0 container project until its `aibox.toml` explicitly enables `[orchestration]`; `aibox apply`, `up`, and `down` from the v0 lifecycle do not discover, modify, or remove a v1 deployment receipt.

## Prepare the configuration

Preview first. This reads only `aibox.toml`, reports digests rather than printing configuration contents, and does not contact a container runtime or a cluster:

```sh
aibox config migrate-v1 --output json
```

Apply the narrow migration only after reviewing the preview:

```sh
aibox config migrate-v1 --apply
```

The command creates an exact original copy under `.aibox/backups/v1-config/` before atomically replacing `aibox.toml`. The only new configuration is:

```toml
[orchestration]
enabled = false
```

That disabled boundary is intentional: aibox cannot safely infer an immutable image digest, deployment owner, target, namespace, credential references, or ingress/DNS prerequisites from a v0 devcontainer configuration. Add those values explicitly, review `aibox deploy plan`, and only then set `enabled = true`.

## Roll back configuration

The apply result prints the backup path. Restore it explicitly:

```sh
aibox config migrate-v1 --restore .aibox/backups/v1-config/v0-<stamp>-<digest>.toml
```

Restore accepts only regular backups inside the project backup directory and uses an atomic replacement. It restores the config alone: it deliberately does not read, alter, or delete `.aibox/deployments/` records or any remote resource. Use `aibox deploy destroy` while the v1 configuration and ownership record are still available if you intend to remove an existing v1 deployment.

## Coexistence boundary

Do not point v0 generated Compose files at v1 deployment artifacts. v1 Compose deployments use their own rendered artifacts and ownership labels; Kubernetes deployments use namespace-scoped labels and a durable `DeploymentRecord`. The v0 lifecycle has no authority to operate either form of v1 state.

This boundary makes rollback safe but not magical: reverting a config does not roll back a remote deployment. Treat deployment removal as a separate, guarded operation with its own record and evidence.
