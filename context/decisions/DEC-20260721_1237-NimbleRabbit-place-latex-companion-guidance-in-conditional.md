---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260721_1237-NimbleRabbit-place-latex-companion-guidance-in-conditional
  created: '2026-07-21T12:37:50+00:00'
spec:
  title: Place LaTeX companion guidance in conditional AGENTS.md content
  state: accepted
  decision: Derived projects with configured LaTeX documents will receive a concise,
    aibox-managed conditional guidance block in AGENTS.md instead of a root-level
    AIBOX-LATEX.md file.
  rationale: AGENTS.md is the primary instruction surface an AI agent reads; conditional
    insertion keeps guidance discoverable without adding a project-root companion
    file.
  alternatives:
  - option: Generate AIBOX-LATEX.md in the project root
    reason_rejected: Discoverability depends on another instruction pointing to the
      file and it adds root-level clutter.
  - option: Create a processkit skill
    reason_rejected: The companion lifecycle and endpoints are aibox-specific runtime
      behavior, not a reusable processkit workflow.
  consequences: aibox must manage a narrow, idempotent LaTeX-specific block without
    modifying the processkit-owned canonical template content. The block must document
    in-container health checks, build/watch commands, and user-facing preview guidance.
  decided_at: '2026-07-21T12:37:50+00:00'
---
