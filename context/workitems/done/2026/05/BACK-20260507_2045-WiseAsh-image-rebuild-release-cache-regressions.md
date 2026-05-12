---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260507_2045-WiseAsh-image-rebuild-release-cache-regressions
  created: '2026-05-07T20:45:00+00:00'
  updated: '2026-05-07T21:01:49+00:00'
spec:
  title: Resolve image rebuild and release cache regressions
  state: done
  type: bug
  priority: high
  description: Investigate and resolve GitHub issues and owner-reported regressions where aibox apply/release rebuilds images too often, e2e testrunner builds are too slow/heavy, Docusaurus install layers are slow, release uploads push unchanged layers, and the e2e companion appears to install unnecessary Podman components.
  scope: container-build-release
  started_at: '2026-05-07T20:45:14+00:00'
  completed_at: '2026-05-07T21:01:49+00:00'
---

## Transition note (2026-05-07T20:45:14+00:00)

Started investigation from owner report and open GitHub issues #68-#71.


## Transition note (2026-05-07T21:01:42+00:00)

Implementation and local validation complete. Moving through review state before terminal transition per workitem state machine.


## Transition note (2026-05-07T21:01:49+00:00)

Done after full local verification: cargo fmt --check, cargo test, targeted runtime/tmux/sync tests, bash -n for shell scripts, cargo check, and cargo clippy --all-targets -- -D warnings.
