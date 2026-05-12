---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260508_1518-HonestAnt-addon-installer-security-hardening-checksum
  created: '2026-05-08T15:18:34+00:00'
  labels:
    track: sec-harden
    release: v0.25.6
  updated: '2026-05-08T21:14:05+00:00'
spec:
  title: 'v0.25.6: Addon and installer security hardening'
  state: done
  type: task
  priority: high
  description: Migrated historical description; see git history for pre-migration full text.
  started_at: '2026-05-08T20:46:51+00:00'
  completed_at: '2026-05-08T21:14:05+00:00'
---

## Transition note (2026-05-08T20:46:51+00:00)

Dispatching to Avery (TEAMMEMBER-20260508_2042-MigratedMember-avery) — software-engineer/senior. Mechanical SHA pin / checksum verification work across ~12 addons + scripts/install.sh + seccomp gate.


## Transition note (2026-05-08T21:13:57+00:00)

Implementation complete in commit d9153b4. 11 addons hardened, [security] toml gate landed, MCP trust scope documented. 946 green.


## Transition note (2026-05-08T21:14:05+00:00)

Accepted as done. Two human-review caveats noted in commit message (AWS keyserver vs bundled key; manual [security] block for existing Codex projects).
