---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260410_2238-CalmSeal-starship-prompts-asciinema-previews
  created: '2026-04-10T22:38:01+00:00'
  labels:
    old_id: BACK-062
    area: docs
spec:
  title: Starship prompts + asciinema previews for all themes and prompts in docs
  state: backlog
  type: task
  priority: medium
  description: "(A) New prompts: add 2-3 more Starship prompt presets \u2014 one powerline/chevron-style, one minimal single-line variant. (B) Asciinema recordings: record a short cast for each prompt preset showing it in action. (C) Docs: add asciinema player embeds to Themes and Prompts pages in docs-site/ \u2014 one cast per theme showing vim+zellij rendering, one per prompt. Implementation order: design prompts \u2192 record casts \u2192 add to docs. See scripts/record-asciinema.sh and docs-site/docs/customization/. Old ID: BACK-062."
---
