---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260723_1849-WildHare-fix-infrastructure-addon-checksum-verification-on
  created: '2026-07-23T18:49:25+00:00'
  updated: '2026-07-23T18:53:00+00:00'
spec:
  title: Fix infrastructure addon checksum verification on ARM64
  state: done
  type: bug
  priority: high
  description: Repair generated infrastructure builder verification for OpenTofu and
    Packer when upstream SHA256SUMS entries name the archive but aibox downloads to
    a temporary path. Download by upstream filename under /tmp, verify from /tmp,
    add renderer regression coverage, and port the applicable fix from v0.x to v1.x.
  started_at: '2026-07-23T18:49:33+00:00'
  completed_at: '2026-07-23T18:53:00+00:00'
---

## Transition note (2026-07-23T18:49:33+00:00)

Confirmed failing ARM64 OpenTofu checksum entry and matching latent Packer failure; implementing path-safe verification and regression coverage.


## Transition note (2026-07-23T18:52:56+00:00)

OpenTofu and Packer checksum verification now uses upstream archive filenames; validated against current ARM64 checksum manifests and merged across v0.x/v1.x.


## Transition note (2026-07-23T18:53:00+00:00)

Completed: v0.x release merge, v1.x development/pre-release port, full test gates, and cross-line port gates all pass.
