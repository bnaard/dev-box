---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260505_1408-KeenPlum-represent-audio-features-under-audio-config
  created: '2026-05-05T14:08:47+00:00'
spec:
  title: Represent audio features under audio config
  state: accepted
  decision: Move user-facing audio and voice feature configuration in aibox.toml under a top-level [audio] namespace. Keep [container.audio] as a backward-compatible legacy alias, and continue resolving enabled installable audio support to the internal audio-voice addon recipe.
  context: 'Audio support currently lives under [container.audio] and selects the audio-voice addon internally. After consolidating AI harness selection under [ai] while keeping provider addons internal, the same semantic split applies to audio: users are expressing a feature intent, not choosing a generic tool bundle.'
  rationale: A top-level [audio] section makes the public config easier to scan and keeps voice/audio settings together. Keeping audio-voice as internal recipe plumbing preserves the existing package/install behavior and avoids exposing implementation details as user-facing addon selection.
  alternatives:
  - option: Keep audio only under [container.audio]
    status: rejected
    reason: It is defensible as container plumbing, but it splits feature intent from the rest of semantic user-facing configuration.
  - option: Expose audio-voice as the only public selection mechanism
    status: rejected
    reason: It leaks implementation details and treats a feature area like a generic tool bundle.
  - option: Support both [audio] and [container.audio] permanently as equal public APIs
    status: rejected
    reason: It creates two sources of truth; [container.audio] should remain compatibility input rather than the scaffolded form.
  consequences: New scaffolded configs should write [audio]. The config loader should accept both [audio] and [container.audio], with [audio] taking precedence when both are present. The resolver should select audio-voice only when audio is enabled and install is true. Documentation should describe audio as a feature namespace rather than an addon block.
  decided_at: '2026-05-05T14:08:47+00:00'
---
