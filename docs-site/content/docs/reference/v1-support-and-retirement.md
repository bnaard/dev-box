---
weight: 9
title: V1 support, deprecation, and retirement
---

# V1 support, deprecation, and retirement

V1 prereleases are evaluation releases. Until a stable v1 release satisfies
every candidate-bound readiness gate, v0 remains supported as the stable line.
V1 and v0 may coexist while projects evaluate migration; neither line may
manage or destroy the other line's deployment records or resources.

## Product boundaries

- **ainfra provisions** accounts, hosts, networks, clusters, identities, DNS
  zones, and other infrastructure, then exposes non-secret target references.
- **aibox deploys** immutable AI workspace images onto existing Compose or
  Kubernetes targets. It does not provision infrastructure.
- **processkit owns** process distribution, installation, migrations, MCP,
  skills, schemas, and harness projections. Aibox's v1 integration is disabled
  or delegates one opaque request to the processkit CLI.

These boundaries are release requirements. The portfolio audit fails if the v1
production path begins interpreting processkit policy or provisioning
infrastructure.

## Support and deprecation

- Alpha and beta users should pin the exact prerelease and retain a known-good
  v0 installer version.
- Corrective contract changes require compatibility review and an updated
  contract-freeze manifest. Incompatible changes require a new API version.
- A deprecated v0 compatibility surface must identify its replacement and
  removal criteria. A date alone is not sufficient retirement authority.
- Security reporting and response follow the repository `SECURITY.md`.

## Rollback and coexistence

Binary rollback does not delete v1 deployments. Before reinstalling v0, use the
matching v1 CLI to inspect and, when intended, destroy v1 resources through its
ownership-guarded lifecycle. Restoring a v0 configuration changes only the
configuration backup boundary; it does not delete v1 deployments or receipts.

Stable release rehearsal must retain checksummed archives for both Linux and
both macOS targets, container- and host-release logs, and an exact-version
rollback/reinstall log. `scripts/record-v1-platform-rehearsal.sh` validates and
records those artifacts against the exact candidate.

## V0 retirement criteria

Retirement is evidence-based. V0 remains available until all of these are true:

1. representative new, migrated, Kubernetes, and direct-processkit journeys
   pass on candidate-bound technical evidence;
2. external pilots have recorded migration friction, failures, recovery steps,
   terminology problems, and documentation gaps;
3. migration and coexistence documentation covers unresolved decisions and
   manual v1 cleanup;
4. supported projects have a reviewed migration outcome with no known data
   loss or destructive ownership defect;
5. all four native artifacts, release phases, and rollback have been rehearsed
   from the final candidate;
6. the ainfra/aibox/processkit portfolio-boundary audit passes.

Retirement requires a reviewed decision after this evidence exists. Automated
journeys cannot stand in for external operator feedback.
