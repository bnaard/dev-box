---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260723_1538-MindfulAnchor-fix-kubernetes-addon-checksums-v0286
  created: '2026-07-23T15:38:18+00:00'
  updated: '2026-08-01T07:52:13+00:00'
spec:
  title: Fix Kubernetes addon checksum verification and release v0.28.6
  state: done
  type: bug
  priority: high
  description: Repair derived-project container builds where Helm checksum files name
    the upstream archive but aibox downloads to /tmp/helm.tar.gz. Make checksum verification
    path-safe for Helm, Kustomize, and k9s; add regression coverage; port applicable
    changes across v0.x and v1.x; publish v0.28.6 and verify release artifacts.
  started_at: '2026-07-23T15:38:21+00:00'
  completed_at: '2026-08-01T07:52:13+00:00'
---

## Transition note (2026-07-23T15:38:21+00:00)

Confirmed the upstream checksum filename mismatch and began generator/test remediation on v0.x before porting to v1.x.


## Transition note (2026-08-01T07:52:12+00:00)

Reconciled against shipped evidence: tag v0.28.6 and published multi-platform release assets exist; commits d6e2e8b1 and 3c6fd414 implement the requested fixes.


## Transition note (2026-08-01T07:52:13+00:00)

Review accepted from authoritative tag, commit, and GitHub release evidence.
