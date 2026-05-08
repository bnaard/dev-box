---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260508_2235-CuriousBadger-v0-25-6-deferred-item-triage
  created: '2026-05-08T22:35:17+00:00'
spec:
  title: v0.25.6 deferred-item triage — drop Q2, defer Q3, resolve the rest
  state: accepted
  decision: 'For v0.25.6 cutover: (1) Drop LuckyLily Q2 (skills CLI surface — `aibox
    skills add/remove`) entirely. Owner never explicitly approved Q2 and on review
    does not see the need. Dead — not deferred, not backlogged. (2) Defer LuckyLily
    Q3 (further `cli/src/seed.rs` split to clear the <2,400-line acceptance criterion;
    current 2,929 lines) to v0.25.7. WorkItem BACK-20260508_2234-WiseTulip filed in
    this same turn. (3) Resolve for v0.25.6 the remaining six items: LuckyLily Q5
    (CI release-smoke status diffs), LuckyLily Q7 (skills inline-comment fact-check),
    BR-CLEANUP-ARCH item 6 (Variant 3 Migration emission), Hermes/OpenCode addons
    checksum gap, AWS GPG keyserver-vs-bundled, Codex seccomp acknowledgement. Each
    gets investigated and either implemented or formally re-deferred via a fresh DEC.'
  context: 'During v0.25.6 cutover review (session of 2026-05-09), the prior session''s
    handover (commit 4cafa9e) listed several deferred items as "open threads" without
    filing them as WorkItems. Sources: LuckyLily completion notes (Q2/Q3/Q5/Q7) and
    prior-session handover open-threads list (BR-CLEANUP-ARCH item 6, Hermes/OpenCode
    checksums, AWS GPG, Codex seccomp). Owner triaged each item explicitly.'
  rationale: 'Q2 was a "nice-to-have" the previous session bundled into LuckyLily
    without explicit owner approval. Owner''s read: not needed; existing inline-comment
    toggling is fine. Recording as a "won''t do" DEC prevents drift. Q3 hit a partial
    result (3,100+ → 2,929 lines, 537 lines extracted into cli/src/tmux/) but missed
    <2,400. Reaching it needs a second extraction pass over non-tmux code (banners,
    runtime-file synthesis) — independent work, would have churned v0.25.6 mid-flight.
    The remaining six items are real release-blocking gaps (security hardening, CI
    gate, schema emission, doc-only fixes) cheaper to land in v0.25.6 than re-explain
    in v0.25.7.'
  alternatives:
  - option: Defer all to v0.25.7 (ship v0.25.6 now)
    rejected_because: Six items include security hardening and a known CI gap; shipping
      them in v0.25.6 is materially better.
  - option: Backlog Q2 too
    rejected_because: Owner explicitly does not want Q2; tracking invites scope creep.
  consequences: 'Q2 closed; resurrect only on fresh evidence + new DEC. Q3 v0.25.7
    backlog filed (compliance: same-turn entity creation). Six items expand this session''s
    scope into investigate-then-implement-or-redefer dispatch via the team. PluckyThorn
    stays in `review` until all six resolve AND PROC-release canonical steps (version
    bump, RELEASE-NOTES, tag, GH release, docs deploy) complete.'
  deciders:
  - TEAMMEMBER-thrifty-otter
  related_workitems:
  - BACK-20260508_1519-LuckyLily-code-quality-aibox-toml-seed-split
  - BACK-20260508_1519-PluckyThorn-release-host-orchestration-rollout-cutover
  - BACK-20260508_2234-WiseTulip-seed-rs-further-split-v0257
  decided_at: '2026-05-08T22:35:17+00:00'
---
