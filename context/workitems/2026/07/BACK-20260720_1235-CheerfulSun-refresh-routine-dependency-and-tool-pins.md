---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260720_1235-CheerfulSun-refresh-routine-dependency-and-tool-pins
  created: '2026-07-20T12:35:14+00:00'
spec:
  title: Refresh routine dependency and tool pins after v0.28.1
  state: backlog
  type: task
  priority: medium
  description: Follow up on the v0.28.1 release-state report. Review and update routine
    non-security drift in base-image tools (ripgrep, eza, fzf, uv), documentation
    and language addon pins, infrastructure tools, Hermes Agent, and Cargo.lock-resolvable
    Rust dependencies. Review upstream release notes and rerun the full release validation
    surface. Cargo audit was clean and processkit v0.27.4 was current at release time.
  scope: release-engineering
---
