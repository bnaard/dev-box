---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260508_2240-WarmLark-resolve-v0-25-6-deferred-items
  created: '2026-05-08T22:40:29+00:00'
spec:
  title: Resolve v0.25.6 deferred items and v1 legacy stale-process cleanup in v0.25.6 (no v0.25.7 deferral)
  state: accepted
  decision: 'Expand v0.25.6 scope to absorb (a) the six deferred items from DEC-20260508_2235-CuriousBadger that were already slated for v0.25.6 resolution (LuckyLily Q5/Q7, BR-CLEANUP-ARCH item 6, Hermes/OpenCode addon checksums, AWS GPG bundling, Codex seccomp acknowledgement) AND (b) the full stale-process cleanup surfaced this session. Cleanup scope: delete the 4 slug-named duplicate files in `context/processes/`, add a deprecation banner to `PROC-release.md` (and the other PROC- files) pointing to the live source (NOTE-20260411 + AGENTS.md:139), audit and archive the legacy `context/processes/` and `context/actors/` directories where superseded by v2 primitives, and file two upstream processkit issues (find_skill should down-weight v1 entities when a v2 successor exists; pk-doctor should grow a v1-entity-drift check). All work lands in v0.25.6 before tag.'
  context: "During the v0.25.6 cutover (2026-05-09 session), the agent investigated whether the canonical aibox release process was discoverable via processkit primitives. The investigation surfaced that `context/processes/PROC-release.md` (v1 schema) is **out of date** vs the actual aibox 2-phase flow (Phase 1 `./scripts/maintain.sh release` in-container, Phase 2 `./scripts/maintain.sh release-host` on macOS). The live canonical source is `context/notes/2026/04/NOTE-20260411_0001-LoyalSpruce-aibox-release-process.md` plus `AGENTS.md:139`. Worse, `processes/` contains 4 exact filename duplicates (`release.md` \u2261 `PROC-release.md`, plus three other pairs), and the directory itself is v1-schema legacy per the processkit skill catalog (process-management is marked \"Legacy/migration guidance for v1 Process entities\"). Wider audit shows ~3,450 v1 files across context/, with whole-directory legacy in `processes/` and `actors/` (the latter superseded by `team-members/` per DEC-20260422_0233-SpryTulip).\
    \ Owner triaged the cleanup options and chose full cleanup in v0.25.6 over v0.25.7 deferral."
  rationale: 'Owner''s call: shipping v0.25.6 with stale process docs in place (and known-misleading-to-the-next-agent) is a worse outcome than absorbing the cleanup cost. The agent today re-planned the release twice because PROC-release misled it; that cost is recurring until the staleness is fixed. The security-hardening followups (Hermes/OpenCode checksums, AWS GPG, Codex seccomp) are also genuine release-blocking gaps that are cheaper to land in v0.25.6 than re-explain in a v0.25.7 follow-up release. The duplicate-file deletion and deprecation-banner work is trivial. The whole-directory archival of `context/processes/` and `context/actors/` is medium effort (needs investigation + a migration-style move), but doable in this session.'
  alternatives:
  - option: Trivial-only cleanup in v0.25.6 (4 dup deletes + banners + 2 upstream issues), full audit in v0.25.7
    rejected_because: 'Owner explicitly chose full cleanup. The trivial-only option leaves the worst-of-both: stale process descriptions still discoverable, just with a banner that next agent may or may not read.'
  - option: Defer all cleanup to v0.25.7
    rejected_because: Owner rejected this option. Compounds the misleading-next-agent cost and leaves the v0.25.6 retrospective with a known-bad state.
  consequences: 'v0.25.6 scope materially larger than the previous session''s "content-complete" framing. Three tracking WorkItems filed in the same turn (compliance: same-turn entity creation): security-hardening followups, CI+code-quality followups, stale-process/v1-legacy cleanup. PluckyThorn stays in `review` until ALL three new WorkItems land plus PROC-release-canonical Phase 1 steps complete (version bump, tests, RELEASE-NOTES, tag, GH release, docs deploy). The stale-process cleanup may itself require its own DECs for any whole-directory archival decisions (e.g., "archive context/processes/ in full"), which will be recorded as the work proceeds. Two upstream processkit issues will be filed but resolution is upstream and not blocking v0.25.6 tag.'
  deciders:
  - TEAMMEMBER-20260422_0001-MigratedMember-thrifty-otter
  related_workitems:
  - BACK-20260508_1519-PluckyThorn-release-host-orchestration-rollout-cutover
  decided_at: '2026-05-08T22:40:29+00:00'
---
