# 

LLMS index: [llms.txt](/aibox/v1.x/v1.0.0-alpha.1/llms.txt)

---

# Moving from v0 containers to v1 deployments

v1 deployment orchestration is opt-in. A v0 project remains a v0 container project until its `aibox.toml` explicitly enables `[orchestration]`; `aibox apply`, `up`, and `down` from the v0 lifecycle do not discover, modify, or remove a v1 deployment receipt.

## Prepare the configuration

Preview first. This reads only `aibox.toml`, reports digests rather than printing configuration contents, and does not contact a container runtime or a cluster:

```sh
aibox config migrate-v1 --output json
```

The preview maps the safe, deterministic part of the old configuration:
`container.name` becomes the proposed fleet, first service, and deployment
name. It then returns an `unresolvedDecisions` array for facts that aibox must
not guess:

- immutable image reference and digest;
- target platform;
- Compose context/scope or Kubernetes context/namespace;
- stable deployment owner;
- connection transports;
- credential references for any v0 environment entries;
- a remove-or-redesign disposition for host bind mounts.

The report never includes environment values. `readyToEnable` remains false
until these operator decisions have explicit v1 values. This makes the command
a migration planner, rather than a textual marker that implies the v0
configuration was fully converted.

Create a reviewed TOML document containing only one complete
`[orchestration]` tree, then apply it with the migration:

```sh
aibox config migrate-v1 --apply --intent-file v1-intent.toml --output json
```

The intent file may say `enabled = true` for validation, but the migration
always writes it as `enabled = false`. Aibox validates the complete image,
fleet, target, deployment, connection, and credential-reference contract
offline before it creates the backup or changes `aibox.toml`. Extra top-level
tables, incomplete intent, symlinked files, and raw credential values are
rejected. The result reports `readyToEnable: true`; activation remains a
separate reviewed edit followed by `aibox config compile` and
`aibox deploy plan`.

Apply the narrow migration only after reviewing the preview:

```sh
aibox config migrate-v1 --apply
```

The command creates an exact original copy under `.aibox/backups/v1-config/` before atomically replacing `aibox.toml`. The only new configuration is:

```toml
[orchestration]
enabled = false
```

That disabled boundary is intentional. Add the reported unresolved values
explicitly, run `aibox config compile`, review `aibox deploy plan`, and only
then set `enabled = true`.

## Roll back configuration

The apply result prints the backup path. Restore it explicitly:

```sh
aibox config migrate-v1 --restore .aibox/backups/v1-config/v0-<stamp>-<digest>.toml
```

Restore accepts only regular backups inside the project backup directory and uses an atomic replacement. It restores the config alone: it deliberately does not read, alter, or delete `.aibox/deployments/` records or any remote resource. Use `aibox deploy destroy` while the v1 configuration and ownership record are still available if you intend to remove an existing v1 deployment.

## Coexistence boundary

Do not point v0 generated Compose files at v1 deployment artifacts. v1 Compose deployments use their own rendered artifacts and ownership labels; Kubernetes deployments use namespace-scoped labels and a durable `DeploymentRecord`. The v0 lifecycle has no authority to operate either form of v1 state.

This boundary makes rollback safe but not magical: reverting a config does not roll back a remote deployment. Treat deployment removal as a separate, guarded operation with its own record and evidence.
