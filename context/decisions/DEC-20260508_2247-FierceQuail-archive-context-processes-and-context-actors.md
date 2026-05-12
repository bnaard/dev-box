---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260508_2247-FierceQuail-archive-context-processes-and-context-actors
  created: '2026-05-08T22:47:17+00:00'
spec:
  title: Archive context/processes/ and context/actors/ as v1 legacy directories
  state: accepted
  decision: "Move `context/processes/` \u2192 `context/archive/processes-v1/` and `context/actors/` \u2192 `context/archive/actors-v1/` in v0.25.6. Add a README in each archive directory pointing to the v2 successors (NOTE-20260411 + AGENTS.md:139 for processes; team-members/ for actors). Remove the original directories from the live tree so `find_skill`, `search_entities`, and route_task no longer surface them as authoritative."
  context: "DEC-20260508_2240-WarmLark authorised the full stale-process / v1-legacy cleanup for v0.25.6 and explicitly noted \"C3 ... Needs a fresh DEC if the answer is 'archive whole directory'.\" Investigation confirmed: (a) all 4 slug/PROC file pairs in `context/processes/` are byte-identical duplicates, (b) all 4 PROC- files describe v1 generic processes superseded by AGENTS.md content (release process specifically lives in NOTE-20260411 per AGENTS.md:139), (c) `context/actors/` holds 9 v1 entities \u2014 `ACTOR-20260410_2209-SnappyFrog-bernhard` (the human owner, duplicated as TEAMMEMBER-20260422_0001-MigratedMember-thrifty-otter) and 8 generic role-template Actors that are obsolete since the actual team is now named TeamMembers (Cora/Sage/Avery/Robin/Jordan). The skill catalog marks both `process-management` and `actor-profile` as legacy / superseded."
  rationale: "Banner-then-keep gives the next agent two discoverable paths and the wrong one will keep being picked up by routing. Archive-and-redirect gives one path. The directories carry no live signal \u2014 every Actor entity has been replaced (Bernhard) or made obsolete (8 role templates), and every PROC- file is a generic v1 placeholder that doesn't match the actual aibox flow. Keeping them in `context/archive/` (vs `git rm`) preserves history at a glance for any agent looking back at why the directories went away, and gives processkit a reference point for the migration entities aibox should have emitted when the v2 transitions happened."
  consequences: |
    - `context/processes/` and `context/actors/` no longer present in the live tree.
    - `route_task("release process")` and `find_skill("release process")` should stop returning `PROC-release` after the next reindex.
    - A `context/archive/processes-v1/README.md` and `context/archive/actors-v1/README.md` document the move and point at the v2 successors.
    - The `context-archiving` skill SHOULD ideally drive this move; the same operation done via `git mv` is functionally equivalent for v1 directories that have no live MCP tooling left and aren't in the v2 archive policy. Logged event preserves the audit trail.
    - The 4 slug-named duplicates (`release.md`, `bug-fix.md`, `code-review.md`, `feature-development.md`) get `git rm`'d outright since they're byte-identical to the PROC- files which will themselves move to the archive.
    - Q: do the 9 ACTOR entities have any cross-references that need to be patched? A: a follow-up grep is included in the execution plan; references are expected to be in already-archived/historical workitems and decisions, which can stay as-is (immutable historical record).
  deciders:
  - TEAMMEMBER-20260422_0001-MigratedMember-thrifty-otter
  related_workitems:
  - BACK-20260508_2241-QuietLark-v0256-stale-process-v1-cleanup
  decided_at: '2026-05-08T22:47:17+00:00'
---
