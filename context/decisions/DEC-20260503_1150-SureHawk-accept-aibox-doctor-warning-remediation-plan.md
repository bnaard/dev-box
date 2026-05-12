---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260503_1150-SureHawk-accept-aibox-doctor-warning-remediation-plan
  created: '2026-05-03T11:50:58+00:00'
spec:
  title: Accept aibox doctor warning remediation plan
  state: accepted
  decision: 'Implement the aibox doctor warning remediation plan before returning to SnappyCrane: remove stale processkit context extra-file warnings, fix schema/version domain comparisons, gate Linux container-only checks when doctor runs on the host, make generated-file .gitignore warnings aware of tracked files, and make image provenance warnings aware of resolved latest tags.'
  context: The owner reran host-side aibox doctor and captured 8250 warnings with 0 errors. Review showed 8241 warnings came from stale recursive context schema checks after processkit became owner of most context content. Remaining warnings were host/container context mismatches, literal latest version comparisons, optional audit tool availability, and separate generated Dockerfile addon drift.
  rationale: Doctor should surface actionable project health signals, not flood the owner with expected processkit-owned files or compare unrelated version domains. Fixing doctor noise first makes subsequent generated-state and SnappyCrane work easier to validate.
  alternatives:
  - option: Treat all doctor warnings as real project cleanup
    assessment: Rejected because most warnings are expected processkit-owned files and would drive destructive or meaningless cleanup.
  - option: Ignore doctor output
    assessment: Rejected because several warnings reveal real doctor defects and some generated-state drift worth separating.
  consequences: This decision authorizes doctor remediation implementation only. SnappyCrane remains on hold. Optional audit tooling and generated Dockerfile addon drift may remain as separate follow-up signals if not directly solved by the doctor patch.
  deciders:
  - TEAMMEMBER-20260422_0001-MigratedMember-thrifty-otter
  related_workitems:
  - BACK-20260423_2050-EagerStone-extend-aibox-doctor-to
  decided_at: '2026-05-03T11:50:58+00:00'
---
