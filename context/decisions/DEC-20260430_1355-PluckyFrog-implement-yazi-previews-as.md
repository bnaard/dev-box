---
apiVersion: processkit.projectious.work/v1
kind: DecisionRecord
metadata:
  id: DEC-20260430_1355-PluckyFrog-implement-yazi-previews-as
  created: '2026-04-30T13:55:25+00:00'
spec:
  title: Implement Yazi Previews as Addon-Gated Renderer plus Explicit Watch Helpers
  state: accepted
  decision: Implement Markdown preview through the preview-enhanced addon using the
    rich-preview Yazi plugin backed by python3-rich, implement horizontal preview
    scrolling as an explicit less -R -S pager shortcut, and implement live PDF preview
    as a pdf-watch helper command plus Yazi binding rather than attempting continuous
    refresh inside Yazi's passive preview pane.
  context: The owner approved implementation of Markdown preview, horizontal preview
    scrolling, and PDF live-watch support for aibox's Yazi/runtime environment. Yazi's
    preview pane supports vertical seek but does not provide a clean provider-neutral
    horizontal scrolling API for all previewers, and live PDF watch behavior is better
    modeled as an active watcher in a pane.
  rationale: This keeps default Yazi behavior simple, avoids brittle preview-pane
    hacks, makes dependencies explicit through the existing preview-enhanced addon,
    and supports existing containers by seeding a project-local pdf-watch helper while
    future images also include /usr/local/bin/pdf-watch.
  alternatives:
  - option: Force horizontal scrolling into the Yazi preview pane
    assessment: Rejected because it would depend on previewer-specific behavior and
      would be less predictable than a normal pager.
  - option: Make PDF live watch part of the built-in PDF previewer
    assessment: Rejected because passive Yazi previewers are not a good fit for long-running
      file watchers.
  - option: Add a heavyweight standalone Markdown renderer binary
    assessment: Rejected for now because python3 is already in the base image and
      python3-rich is sufficient when preview-enhanced is enabled.
  consequences: Projects enable rendered Markdown previews by enabling preview-enhanced.
    Wide previews are accessed through a dedicated keybinding. PDF watch preview runs
    in a blocking pane command and updates when the PDF changes.
  decided_at: '2026-04-30T13:55:25+00:00'
---
