# V1 processkit responsibility-boundary report

This report is the human-readable companion to the machine-readable
[boundary ledger](v1-processkit-boundary-ledger.json). It implements WS-A of
the [v1 workplan](../v1-workplan.md) and is based on the scope in GitHub #179.
It is an inventory, not an implementation change.

## Result

The current tree contains aibox-owned processkit policy in four places that
must not survive the v1 boundary:

1. `processkit_vocab.rs` duplicates producer vocabulary: paths, package/tier
   names, skill/server lists, templates, and historical layout aliases.
2. `content_install.rs` and its diff/migration/init siblings decide how
   processkit is distributed, installed, reconciled, and migrated.
3. `mcp_registration.rs`, `hook_registration.rs`, `preauth.rs`, and doctor
   checks discover and project processkit MCP/harness policy.
4. generated runtime, documentation, and release scripts assume aibox owns
   processkit update and runtime topology.

The ledger gives every identified surface one of four dispositions. It is
deliberately conservative: a group remains a `bounded-v0-bridge` until the
producer CLI protocol, migration, rollback, and representative golden parity
are demonstrated. No new processkit policy may be added to that bridge.

## Required v1 boundary

Aibox owns workspace images, fleet intent, backend selection, deployment
lifecycle, observed state, and connection. Processkit owns its distribution,
installation/update/reconciliation/migration, profiles, templates, MCP
catalogs, and harness projections. Ainfra-templates supplies references to
pre-existing targets only.

The only durable v1 aibox/processkit interface is a versioned opaque install
request/result. Aibox may retain user intent (`enabled`, source/channel/version,
profile, harnesses, root, and environment/path facts), ensure the producer CLI
is available, invoke it using typed arguments, and save result provenance. It
must not interpret paths, layouts, packages, skills, templates, migrations,
MCP topology, or harness projections.

## Compatibility ledger

| Current behavior | v1 treatment | Removal evidence |
| --- | --- | --- |
| `apply` installs and projects processkit | bounded v0 bridge; v1 sends an opaque request | producer CLI success/failure/interruption fixtures and golden project parity |
| `sync`/update reconciles processkit content | bounded v0 bridge | direct producer update/reconcile and rollback parity |
| generated runtime starts/observes processkit MCP | bridge only | workspace runs processkit-disabled and direct in-workspace invocation succeeds |
| host doctor validates catalog/gateway/layout | replace with delegated diagnosis | aibox observes structured producer result without catalog/layout checks |
| release scripts track processkit releases | remove | an aibox release has no producer vocabulary or catalog maintenance gate |
| `up` couples lifecycle and attach | bridge while commands migrate | Compose backend plan/apply/status/destroy/logs/connect parity; `up` is apply alias |

## Ambiguity policy and exit gate

There are no unclassified inventory entries. Entries marked
`retain-generic` require a code review confirming they are content-source,
addon, workspace-template, or historical-record machinery without a
processkit-specific name, path, layout, or policy branch. If that proof fails,
the entry becomes a removal or adapter task before implementation proceeds.

Removal work is blocked until all of the following are true:

- processkit publishes the compatible CLI protocol from processkit#118;
- request/result fixtures cover availability, failure, retry, cancellation,
  migration, rollback, and provenance;
- v0 installer and v1 producer CLI outcomes have golden parity on
  representative projects;
- a processkit-disabled aibox deployment and direct processkit invocation in a
  bare running workspace both pass; and
- every `bounded-v0-bridge` entry has either met its criterion or has a dated,
  documented support/removal decision.

## Inventory method and validation

The inventory searched tracked non-`context/` files for processkit references,
then grouped closely coupled modules only where they share a single ownership
decision. `context/` is processkit-owned input and is listed as an excluded
migration surface rather than read or modified. Evidence strings in the ledger
identify the policy actually found in each path; they are intentionally
traceable to a file/module rather than asserting a future implementation.

Validate the ledger syntax and minimum entry contract with:

```sh
jq -e '.ledger_version == "v1alpha1" and ([.entries[] | has("id") and has("path") and has("evidence") and has("classification") and has("criterion")] | all)' docs/architecture/v1-processkit-boundary-ledger.json
```
