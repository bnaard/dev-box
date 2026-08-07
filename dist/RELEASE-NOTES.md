# aibox v0.31.1 — 2026-08-07

**Summary:** This patch makes processkit upgrades self-healing when an old shared cache is incomplete and eliminates the stale commands, missing skill dependencies, and false MCP header drift reported after `aibox apply`. Projects using `processkit.version = "latest"` receive the paired processkit v0.28.6 manifest fix automatically.

## Added

- Add provenance-inventory validation for cached processkit releases.
- Add transitive installation of dependencies declared through `metadata.processkit.uses`.

## Changed

- Consume processkit v0.28.6, whose release manifest records the shipped source tree's MCP dependency headers independently from the upstream dogfood tree.

## Fixed

- Refetch a signed processkit release when its shared cache is missing files declared by `PROVENANCE.toml`.
- Remove stale generated `pk-*` harness commands even when the older processkit mirror is incomplete.
- Install referenced skills such as `release-semver` when selected skills declare them as dependencies.

## Removed

- No configuration or addon surface was removed.

## Upgrade notes

Run `aibox apply`. Incomplete cached processkit releases are detected and replaced automatically.

[v0.31.1]: https://github.com/projectious-work/aibox/compare/v0.31.0...v0.31.1
