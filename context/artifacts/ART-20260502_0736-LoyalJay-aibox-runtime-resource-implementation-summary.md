---
apiVersion: processkit.projectious.work/v1
kind: Artifact
metadata:
  id: ART-20260502_0736-LoyalJay-aibox-runtime-resource-implementation-summary
  created: '2026-05-02T07:36:15+00:00'
spec:
  name: Aibox Runtime Resource Recovery Implementation Summary
  kind: document
  format: markdown
  version: v1
  tags:
  - work-plan
  - implementation-summary
  - aibox
  - runtime
  - resource-usage
  - orbstack
  produced_at: '2026-05-02T07:36:15+00:00'
---

Approved implementation plan and completion summary for the 2026-05-02 aibox runtime/resource recovery work.

Plan slices:
1. Resolve open migrations and keep runtime-home migrations clean.
2. Fix stale generated Zellij layouts so aibox.toml harness selection is reflected in .aibox-home without overwriting genuine user edits.
3. Restore aibox apply --no-cache while preserving --rebuild compatibility.
4. Add explicit Compose project/image identity to improve Docker/OrbStack grouping.
5. Reduce eager process startup by suspending non-focused Zellij tabs and AI panes.
6. Add local runtime resource diagnostics for cgroup memory, OOM kills, total processes, and processkit MCP Python process count.
7. Make gh and lazygit selectable through an addon while preserving this repo's maintenance tooling by selecting that addon in aibox.toml.
8. Leave processkit MCP gateway implementation to processkit; aibox only prepares diagnostics and runtime behavior around it.

Completed implementation:
- Runtime-home provenance stored in aibox.lock [runtime_home], with same-version auto-repair for stale generated layouts and no repeated local-only migrations.
- aibox apply --no-cache restored; --rebuild remains a visible alias.
- Generated Compose files now include top-level name and explicit image fields.
- Zellij generated layouts use start_suspended true for non-focused editor/git/shell and secondary AI panes.
- aibox get runtime --resources added with table/json/yaml output.
- New git-ui addon owns gh and lazygit; base image no longer installs them unconditionally; Zellij lazygit tab is emitted only when git-ui.lazygit is selected.
- All six implementation WorkItems were transitioned to done.

Verification:
- cargo fmt --check
- cargo clippy --all-targets -- -D warnings
- cargo test --manifest-path cli/Cargo.toml
- aibox apply --no-container reports no runtime/processkit migration needed
- processkit pending and in-progress migration queues are empty
