---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260515_0713-LucidFinch-repair-active-interlocutor-identity
  created: '2026-05-15T07:13:21+00:00'
  labels:
    source: pk-wrapup
    category: processkit-team
spec:
  title: Repair active interlocutor session identity
  state: backlog
  type: task
  priority: medium
  description: During pk-wrapup, `get_active_interlocutor(scope="project")` reported
    that `context/team/session-identity.json` is configured but references missing
    `TEAMMEMBER-20260508_2042-MigratedMember-avery`. Future sessions should repair
    or clear the active interlocutor binding so session start can show a valid TeamMember
    identity.
  scope: aibox
---
