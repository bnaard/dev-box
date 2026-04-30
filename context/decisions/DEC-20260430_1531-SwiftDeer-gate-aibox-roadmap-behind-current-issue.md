---
apiVersion: processkit.projectious.work/v1
kind: DecisionRecord
metadata:
  id: DEC-20260430_1531-SwiftDeer-gate-aibox-roadmap-behind-current-issue
  created: '2026-04-30T15:31:50+00:00'
spec:
  title: Gate aibox roadmap behind current issue fixes and processkit schema releases
  state: accepted
  decision: 'Proceed with implementation by first resolving current aibox GitHub issues
    #58 and #59 and shipping a patch release if those fixes change shipped behavior;
    then work the LivelyMoss roadmap in parallel with processkit SmoothRiver work,
    while gating canonical processkit artifact emission on a future processkit release
    that declares the required schemas and migrations.'
  context: The owner approved the adapted plan based on LivelyMoss and SmoothRiver.
    aibox v0.22.0 currently pins processkit v0.24.0. The SmoothRiver processkit work
    plan is in progress and owns schema/known-kind additions; LivelyMoss assigns emitters,
    images, projectors, and aibox-doctor behavior to aibox.
  rationale: 'Issues #58 and #59 affect generated processkit harness behavior in derived
    projects and should be stabilized before large roadmap changes. Most aibox roadmap
    machinery can be built internally without waiting for processkit, but canonical
    Artifact kinds such as workspace-manifest, addon-spec, provider-backend-spec,
    container-image-spec, and image-provenance-policy must wait until processkit owns
    and releases their schema surface.'
  alternatives:
  - option: Wait for the next processkit release before any aibox work
    reason_rejected: Would unnecessarily block aibox-owned internal modeling, refactors,
      image groundwork, and issue fixes.
  - option: Implement and emit canonical artifacts immediately against draft schema
      names
    reason_rejected: Would bypass processkit schema ownership and risk invalid context
      entities in downstream projects.
  consequences: 'Implementation starts with #58/#59. A patch release is expected if
    those fixes alter released behavior. Roadmap work may proceed behind compatibility
    gates and soft validation, but hard validation and canonical processkit artifact
    writes wait for a processkit release with matching schema support.'
  related_workitems:
  - BACK-20260424_0019-DaringCliff-github-51-research-opencode
  decided_at: '2026-04-30T15:31:50+00:00'
---
