---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260514_1305-MellowBison-accept-id-vocabulary-doctor-findings-as
  created: '2026-05-14T13:05:07+00:00'
spec:
  title: Accept id-vocabulary doctor findings as upstream-tracked policy
  state: accepted
  decision: 'Accept the two id-vocabulary doctor finding classes as known upstream
    limitations, not project-actionable: (1) `id-vocabulary.default-pair-capacity-low`
    is structurally unresolvable at project level — filed upstream as projectious-work/processkit#49;
    (2) the 20 `id-vocabulary.lexical-token-ambiguous` findings (AmberBrook, AmberOak,
    AmberWren, BoldSky, BraveHare, BraveLark, BrightStream, BrightThorn, CalmBison,
    CalmFern, CleverDew, CleverMoss, CuriousOwl, DaringSky, DaringVale, DeepAnt, DeepCliff,
    EagerDew, EagerStone, FierceCrane) are historical shorthand collisions; doctor''s
    own guidance is "prefer full IDs for these entities" — no renaming.'
  context: pk-doctor v0.26.7 ships a new id_vocabulary check that flags the bare default-pair
    capacity (120 adj × 120 nouns = 14400, target 50000+) and 20 colliding shorthand
    tokens across 1076 indexed entities. The capacity check ignores per-kind palette
    configuration (WorkItem already uses tagged double_adjective at 1.24M capacity),
    so no settings.toml flip will clear it. The ambiguity check's only suggested resolution
    is guidance — there's no rename-to-clear path that doesn't break decision/log/workitem
    cross-references.
  rationale: 'Filing upstream is the right escalation path: the doctor check needs
    to either walk configured per-kind palettes or expose a project threshold. Renaming
    20 colliding entities to fix historical shorthand collisions would invalidate
    cross-entity references and provide no end-user benefit — agents are instructed
    to prefer full IDs in long-form references already.'
  alternatives:
  - option: Hand-edit _ADJECTIVES/_NOUNS in _lib/processkit/ids.py
    rejected_because: upstream-managed file; would drift on next aibox apply
  - option: Rename the 20 colliding entities
    rejected_because: would invalidate historical decision/log cross-references; collides
      with agent guidance to prefer full IDs
  - option: Lower NORMAL_PAIR_TARGET from 50000 to 14400
    rejected_because: same upstream-file edit problem
  consequences: pk-doctor will continue to report 21 actionable id-vocabulary WARNs
    until processkit ships a check refinement. These should be treated as accepted-policy
    noise, not action items, until the upstream issue closes.
  decided_at: '2026-05-14T13:05:07+00:00'
---
