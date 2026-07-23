---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260723_0830-BoldSpark-require-tracked-ports-between-maintained-version
  created: '2026-07-23T08:30:01+00:00'
spec:
  title: Require tracked ports between maintained version lines
  state: accepted
  decision: Every fix merged into a maintained v0.x or v1.x line must either be ported
    to the other line with traceable source and target commits or explicitly marked
    not applicable with rationale. Release checks must block a target line while required
    port items for that line remain open.
  rationale: The maintained lines have already diverged and applicable fixes were
    not consistently propagated. Automated tracking plus a release-time gate makes
    omissions visible and prevents publication with known missing ports.
  consequences: Repository automation will create cross-line port tracking issues
    for unclassified fixes, port commits will close them by source commit reference,
    and release validation will reject open target-line port obligations.
  related_workitems:
  - BACK-20260723_0829-ValiantCliff-ship-v0285-port-cross-line-fixes
  decided_at: '2026-07-23T08:30:01+00:00'
---
