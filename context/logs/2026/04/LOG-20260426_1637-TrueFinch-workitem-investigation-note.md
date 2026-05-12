---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260426_1637-TrueFinch-workitem-investigation-note
  created: '2026-04-26T16:37:53+00:00'
spec:
  event_type: workitem.investigation.note
  timestamp: '2026-04-26T16:37:53+00:00'
  summary: Classifier logic at content_diff.rs:144 looks correct in isolation; false positives must come from upstream parameter computation, not the match arms themselves.
  actor: claude-opus-4-7
  subject: BACK-20260426_1627-TrueRaven-investigate-content-diff-conflict
  subject_kind: WorkItem
  details:
    function_location: cli/src/content_diff.rs:144-164 fn classify()
    match_logic: cache_eq = (reference == cache); live_eq = live==reference; (false,true) -> ChangedUpstreamOnly; (false,false) -> Conflict
    expected_for_canonical_no_conflict: live==old_mirror, upstream changed -> cache_eq=false, live_eq=true -> ChangedUpstreamOnly (correct)
    v0.21.0_fix: commit b1b57cf tightened RemovedUpstreamStale reclassification pass (lines 326-349) to compare live_sha vs older-mirror SHA before reclassifying. The classify() function itself was not changed.
    existing_tests:
    - classify_unchanged() L975
    - classify_changed_locally_only() L983
    - classify_changed_upstream_only() L996
    - classify_conflict() L1004
    - classify_new_upstream() L1018
    - classify_removed_upstream() L1030
    open_question: 'If classify() is correct, why did v0.22.0->v0.23.0 produce 50 false-positive Conflicts on files where the user did not edit? Three hypotheses: (1) rendered-template SHA drift between mirror-render and live-install-render breaks the DEC-034 invariant; (2) wrong arg order at the call site (something other than old_mirror passed as `reference`); (3) live_sha computation includes line-ending or trailing-newline normalization that the mirror SHA does not.'
    next_step: 'Reproduce a single false-positive: pick one file, capture the three SHAs actually passed to classify() at runtime, and trace which hypothesis matches.'
    agent: Explore subagent (a396de16bb79cb4fe), 150s, 38 tool uses
---
