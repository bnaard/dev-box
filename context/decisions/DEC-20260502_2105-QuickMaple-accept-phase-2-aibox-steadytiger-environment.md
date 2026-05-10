---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260502_2105-QuickMaple-accept-phase-2-aibox-steadytiger-environment
  created: '2026-05-02T21:05:25+00:00'
spec:
  title: Accept phase 2 aibox SteadyTiger environment contract plan
  state: accepted
  decision: 'Phase 2 of the next aibox release will implement the broader SteadyTiger
    aibox environment-contract work after phase 1: harden human-dev and headless-runner
    profile validation; add profile-aware addon metadata including profile_intent,
    usage_class, compatible profiles, and exported surfaces; promote workspace-manifest
    beyond preview while preserving processkit schema ownership; emit provider-backend
    metadata and automation/manual-only distinctions; add an image-provenance baseline
    and profile labels; make headless-runner ready enough for future runner/broker
    consumption without implementing AgentRun; add doctor checks for addon usage class,
    subscription CLI leakage, manifest/profile mismatch, provider automation mismatch,
    and image label/provenance mismatch; update docs and tests.'
  context: The owner clarified that the next release should contain both processkit
    v0.25.0 integration and the SteadyTiger changes. Phase 1 was already recorded
    and accepted. The phase 2 plan was proposed as a separate gate and the owner directed
    that phase 1 and phase 2 plans be recorded before phase 1 implementation starts.
  rationale: Phase 2 extends the release beyond processkit gateway integration into
    the aibox-owned environment materialization contract, while maintaining the boundary
    that aibox prepares boxes and does not run agents. It keeps the selected-components
    model in aibox.toml and leaves runtime AgentRun/provider-router/SEP concerns outside
    aibox.
  alternatives:
  - option: Merge phase 2 into phase 1 implementation
    reason_rejected: Would make the gateway/processkit v0.25.0 integration too broad
      and hard to validate.
  - option: Defer all SteadyTiger environment contract work until after release
    reason_rejected: The owner wants the next aibox release to include both the processkit
      release integration and SteadyTiger changes.
  - option: Implement AgentRun/runtime behavior inside aibox
    reason_rejected: 'Contradicts the accepted boundary: aibox prepares boxes; it
      does not run agents.'
  consequences: Phase 2 implementation follows phase 1. If phases 1 and 2 are solid
    under release gates, aibox release work proceeds before phase 3. Phase 3 remains
    deeper follow-up work with investigation and discussion.
  decided_at: '2026-05-02T21:05:25+00:00'
---
