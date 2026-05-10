---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260502_0829-WarmBrook-use-compose-init-reaper-and-preserve
  created: '2026-05-02T08:29:58+00:00'
spec:
  title: Use compose init reaper and preserve unprivileged Codex bubblewrap sandboxing
  state: accepted
  decision: Generated aibox devcontainers will use a real init/reaper for the main
    service while keeping Codex bubblewrap sandboxing unprivileged by default. Codex-specific
    broad grants such as privileged mode or SYS_ADMIN remain out of the generated
    baseline; only targeted project-local fallback guidance may mention seccomp relaxation.
  context: 'The owner accepted the bwrap zombie remediation plan after live evidence
    showed more than 500 zombie bwrap processes reparented to PID 1, while the generated
    compose still used bare `sleep infinity` as PID 1. The plan has two tracks: verify
    Codex bubblewrap prerequisites and improve process runtime reaping.'
  rationale: Zombie bwrap processes are dead children that need a parent reaper; changing
    Codex sandbox privileges would not solve zombie reaping and would weaken the security
    model. Compose init support addresses the observed process-count driver directly
    while preserving aibox.toml as the declarative source of truth.
  alternatives:
  - option: Privileged or SYS_ADMIN devcontainer
    reason: Rejected for generated baseline because it weakens the main development
      container and is broader than Codex bubblewrap requires.
  - option: Disable Codex bubblewrap
    reason: Rejected because it reduces command sandboxing instead of fixing process
      reaping.
  - option: Do nothing until processkit gateway
    reason: Rejected because the zombie bwrap issue is independent of MCP process
      count and can be fixed in aibox now.
  consequences: Freshly generated devcontainers should reap orphaned sandbox helper
    processes. Existing containers need restart/recreation to clear already accumulated
    zombies. Runtime diagnostics and docs should distinguish sandbox-prerequisite
    failures from PID 1 reaping failures.
  related_workitems:
  - BACK-20260502_0829-SnappyPine-fix-bwrap-zombie-accumulation
  decided_at: '2026-05-02T08:29:58+00:00'
---
