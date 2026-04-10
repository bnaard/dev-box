# aibox v0.17.9

Feature release: `"latest"` sentinel for version fields in `aibox.toml`.
No processkit version change — still compatible with v0.8.0.

## `"latest"` sentinel for version fields

You can now set `version = "latest"` in `aibox.toml` for both version fields
instead of updating the pin after every release.

### `[aibox].version = "latest"`

```toml
[aibox]
version = "latest"
```

`aibox sync` suppresses the version-mismatch warning when set to `"latest"`.
Use this if you always want to run the newest aibox CLI without being prompted
to update the pin.

### `[processkit].version = "latest"`

```toml
[processkit]
version = "latest"
```

`aibox sync` resolves `"latest"` to the newest available tag at the source
before installing:

```
==> Resolved processkit 'latest' → v0.8.0
```

The resolved concrete version is written to `aibox.lock`; `"latest"` stays
in `aibox.toml`. The lock remains fully reproducible — a second developer
running sync on the same day gets the same concrete version from the lock.

Network errors or an empty tag list produce a warning and skip the install
rather than failing hard.
