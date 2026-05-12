---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260430_0842-SnappyQuail-add-global-theme-mode
  created: '2026-04-30T08:42:55+00:00'
spec:
  title: Add Global Theme Mode Command
  state: accepted
  decision: Implement a global `aibox theme` command that updates `[customization]`, regenerates mounted runtime theme files, and optionally restarts only the Zellij session. Do not stop or restart the container for theme switches.
  context: The user accepted the proposal for a global light/dark theme switch. aibox currently stores concrete theme names in `[customization].theme` and mounts `.aibox-home` runtime configuration into the running container.
  rationale: Theme changes are configuration-layer changes, not container lifecycle changes. Bind-mounted runtime files can be regenerated while the container is running, and the only disruptive operation needed for a full visual refresh is restarting UI processes such as the project Zellij session.
  alternatives:
  - option: Make mode a per-theme boolean
    reason_not_chosen: Concrete theme names already encode mode for existing themes, so a per-theme boolean is ambiguous and scales poorly.
  - option: Stop and restart the container on theme switch
    reason_not_chosen: Unnecessary and disruptive to companion services, shells, and agent state.
  - option: Only document manual aibox.toml edits
    reason_not_chosen: Provides a worse UX than a dedicated command and makes live runtime refresh less discoverable.
  consequences: Existing concrete theme configs remain compatible. New global theme commands can update runtime UI files without container restarts, while full visual refresh may still require restarting Zellij/Yazi/Vim/lazygit processes.
  decided_at: '2026-04-30T08:42:55+00:00'
---
