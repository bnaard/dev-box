---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260509_2125-CoolFrog-powerkit-status-format-slot-order-discipline
  created: '2026-05-09T21:25:39+00:00'
spec:
  title: PowerKit status-format slot-order discipline extends to line1-left + plugin
    label-emission contract
  state: proposed
  decision: |
    Extend DEC-20260508_2115-SilentFern slot-order discipline to cover tmux status line1-left, and codify the label-emission contract for aibox_* PowerKit plugins:

    1. **Plugin label-emission contract:** PowerKit aibox_* plugin scripts MUST emit data-only in `plugin_render`. Icons/labels are produced by `plugin_get_icon` and prepended by the framework. Emitting a label literal inside `plugin_render` (e.g. `printf 'OOM %s/%s'`) is a contract violation — it produces double-rendered labels in the rendered status line ("OOM OOM 3/4" instead of "OOM 3/4"). Applies to all six current aibox_* plugins (oom, log, proc, ai, mcp, mig) and any future siblings.

    2. **status-left composition:** `status-left` carries `#S` followed by a window list using the format `#{W:#I:#W ,#[reverse]#I:#W#[noreverse] }`, so window navigation stays visible alongside the session name in the powerkit double-bar layout.

    3. **status-left-length:** `status-left-length 80` is the standard minimum to keep multi-window lists from being truncated.

    4. **Slot-order discipline scope expansion:** The schema-bump-plus-paired-Migration requirement from DEC-SilentFern (originally line2 only) now also applies to line1-left composition. Any change to status-left structure (adding/removing/reordering segments) requires the same gate.
  context: |
    DEC-20260508_2115-SilentFern established that the v0.25.6 PowerKit status-format slot order is intentionally fixed and that any reordering / addition / removal of slots requires (a) a schema bump and (b) a paired Migration entity. SilentFern's original scope covered line2 only (git/github/kubernetes/terraform/cloud/cloudstatus + aibox metric slots).

    The SilentFjord WorkItem (label-doubling fix in 6 powerkit plugins + addition of a window list to line1-left of status-left) materially changes line1-left's contract. The T1 implementation (commit e6e1e9a on branch v0.25.7/terminal-stack) adds a window-list segment to status-left and exposes a contract about how aibox_* plugin scripts must emit labels.

    Without this DecisionRecord, line1-left changes would slip the SilentFern slot-order discipline and create audit drift.
  rationale: |
    The label-doubling bug surfaced because plugin authors had to remember an implicit contract about who prepends the label. Codifying it as an explicit decision record makes the contract enforceable in code review and catches regressions early.

    Adding line1-left to the slot-order discipline is the conservative extension: SilentFern's reasoning (status-line slot order is user-visible API; reordering needs explicit migration) applies equally to status-left. Splitting the contract by line would create gratuitous boundaries that future plugins would step on.
  alternatives:
  - option: Limit contract to label-emission only; leave line1-left out of slot-order
      discipline
    rejected_because: Creates an artificial boundary between line1 and line2; future
      structural changes to status-left would slip the audit trail SilentFern was
      designed to enforce.
  - option: Skip the DecisionRecord; treat the SilentFjord fix as pure bugfix
    rejected_because: The window-list addition to line1-left is a structural change,
      not a pure bugfix. Skipping creates exactly the audit drift SilentFern was designed
      to prevent.
  - option: Make the label contract a Skill-level lint instead of a Decision
    rejected_because: A lint catches mechanically; a Decision documents the why and
      gives reviewers context. Both can coexist later, but the Decision is the load-bearing
      artifact.
  consequences: |
    - Future aibox_* plugin PRs must be reviewed against the data-only-render contract.
    - Future status-left changes (segment additions, format string edits, length-cap changes) require a schema bump and paired Migration, same as line2.
    - The fix in commit e6e1e9a (T1 / v0.25.7/terminal-stack) ships under this decision; no Migration is required for the bugfix portion (it restores the contract, not changes it), but the line1-left window-list addition is a structural change that DOES require a paired Migration entity per the SilentFern rule. That Migration should be authored alongside this decision before T1 lands on main.
    - AIBOX_LAYOUT_AGENT_SPLIT and AIBOX_LAYOUT_AGENT_RATIO env-vars introduced by SnappyWolf (commit 917f160) are NOT covered by this decision — they govern layout, not status-line slot order.
  deciders:
  - TEAMMEMBER-thrifty-otter
  related_workitems:
  - BACK-20260509_1316-SilentFjord
---
