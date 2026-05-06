---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260506_2149-PluckyBison-redesign-aibox-native-zellij-status-bars
  created: '2026-05-06T21:49:54+00:00'
spec:
  title: Redesign aibox native Zellij status bars to match Zellij visuals
  state: accepted
  decision: Implement the aibox native Zellij key-hint and runtime-status bars as
    Zellij-style compact mode/key capsules and runtime segments, using the active
    Zellij palette instead of the previous bracketed custom ticker style.
  context: The user approved the visual redesign proposal after reviewing the current
    aibox status/key-binding bar behavior and Zellij's native status-bar conventions.
    The implementation should happen before the next minor release.
  rationale: The status and key-hint rows are part of the terminal workspace chrome
    and should look like a native Zellij extension. Matching Zellij's visual grammar
    reduces distraction, improves first-run trust, and avoids the current custom bracketed
    appearance.
  consequences: The native WASM status plugin rendering will change, while the two-row
    layout, toggle behavior, and runtime data collection stay intact. Tests should
    cover width degradation and the new visual grammar before a minor release.
  decided_at: '2026-05-06T21:49:54+00:00'
---
