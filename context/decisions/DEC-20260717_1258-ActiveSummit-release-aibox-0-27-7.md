---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260717_1258-ActiveSummit-release-aibox-0-27-7
  created: '2026-07-17T12:58:02+00:00'
spec:
  title: Release aibox 0.27.7
  state: accepted
  decision: Release patch version 0.27.7 after committing and pushing the current
    validated fixes.
  context: The owner explicitly requested committing all current changes, pushing
    main, and creating a new patch release.
  rationale: The release includes resolved GitHub issues, processkit health fixes,
    and the TeX Live archive TLS mirror repair.
  consequences: The release workflow will tag, publish GitHub assets, and deploy documentation.
  decided_at: '2026-07-17T12:58:02+00:00'
---
