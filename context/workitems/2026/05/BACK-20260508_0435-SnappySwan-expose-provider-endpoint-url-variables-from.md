---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260508_0435-SnappySwan-expose-provider-endpoint-url-variables-from
  created: '2026-05-08T04:35:12+00:00'
  labels:
    area: config
    component: providers
    release: next-patch
  updated: '2026-05-08T04:41:30+00:00'
spec:
  title: Expose provider endpoint URL variables from aibox.toml
  state: review
  type: task
  priority: high
  description: Extend provider/API environment projection so aibox.toml can expose
    provider endpoint/base URL variables in addition to existing key variables for
    each supported provider. Preserve provider-neutral naming and generated runtime
    behavior. Include docs/schema/tests so derived projects get endpoint variables
    without manual env hacks.
  parent: BACK-20260507_1455-CalmBison-aibox-v0250-tmux-runtime-redesign
  started_at: '2026-05-08T04:35:15+00:00'
---

## Transition note (2026-05-08T04:35:15+00:00)

Delegating implementation to GPT-5.3 Codex worker for next patch release.


## Transition note (2026-05-08T04:41:30+00:00)

Ready for review: provider endpoint/base URL env metadata, rendered aibox.toml comments, and configuration docs implemented; normal cargo test and clippy passed.
