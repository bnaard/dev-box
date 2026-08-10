---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260810_1604-SoftLark-gate-releases-on-actionable-container-vulnerabilities
  created: '2026-08-10T16:04:10+00:00'
spec:
  title: Gate releases on actionable container vulnerabilities
  state: accepted
  decision: The macOS release host gate always retains the complete Grype JSON report,
    groups terminal reporting by unique advisory, blocks High or Critical findings
    when Grype lists an available fixed version, and records High or Critical findings
    without a listed fix as explicit non-blocking warnings.
  context: Grype reported 180 High/Critical package matches for the Debian 13 candidate,
    including advisories that curl upstream and Debian classify as low or minor/no-DSA
    and for which Debian stable has no fixed package. Blocking every raw severity
    match makes releases impossible without improving the shipped image.
  rationale: This preserves complete supply-chain evidence and blocks remediable risk
    while avoiding false precision from scanner severity feeds and duplicate package
    matches. No-fix findings remain visible and auditable rather than suppressed.
  alternatives:
  - option: Block every High/Critical Grype match
    reason: Rejected because vendor/upstream severity can conflict and stable distributions
      may have no fix.
  - option: Ignore all no-fix findings entirely
    reason: Rejected because it would discard material security evidence.
  - option: Maintain a large manual CVE allowlist
    reason: Rejected because it becomes stale and hides changes in fix availability.
  consequences: The evidence manifest must include a machine-readable vulnerability
    policy summary. Release output must distinguish unique advisories, affected package
    matches, actionable findings, and no-fix findings. A newly available fixed version
    turns the corresponding High/Critical finding into a blocker on the next scan.
  deciders:
  - ACTOR-20260410_2209-SnappyFrog-bernhard
  related_workitems:
  - BACK-20260808_1709-CuriousAnt-replace-privileged-e2e-companion-with-restricted
  decided_at: '2026-08-10T16:04:10+00:00'
---
