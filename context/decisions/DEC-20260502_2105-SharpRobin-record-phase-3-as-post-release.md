---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260502_2105-SharpRobin-record-phase-3-as-post-release
  created: '2026-05-02T21:05:45+00:00'
spec:
  title: Record phase 3 as post-release SteadyTiger follow-up
  state: accepted
  decision: 'Phase 3 is recorded as the post-release SteadyTiger follow-up for aibox-owned work that remains after phases 1 and 2: full crate/repo restructuring; true published human-dev and headless-runner image streams; complete SBOM/provenance signing and registry attachment; Cilium/Tetragon integration; full workspace-manifest canonicalization as processkit-backed Artifact emission; complete addon-spec Artifact publishing; complete provider-backend-spec semantics; formal MCP projection compiler and shared drift manifest; remaining doctor checks; and conditional future items such as policy-spec emitter, multi-tenant manifests, kagent projection, ACP/session-state extensions, and hook-inbox layout addon-specs.'
  context: 'The owner asked what remains after phase 1 and phase 2, accepted that gap analysis as phase 3, and noted that phase 3 likely needs significant investigation and discussion. The owner also set the sequence: implement phase 1, then phase 2, release if solid, then implement phase 3 later.'
  rationale: Phase 3 contains deeper structural and ecosystem work whose requirements are less settled and whose blast radius is larger than the next release should carry before the gateway/profile work is proven. Recording it preserves the SteadyTiger request without blocking phase 1 and phase 2 release execution.
  alternatives:
  - option: Include phase 3 in the same release as phases 1 and 2
    reason_rejected: Too much investigation and architectural churn for the current release path.
  - option: Drop phase 3 because it is long-horizon
    reason_rejected: It is still requested by the SteadyTiger aibox work plan and should be preserved as a planned follow-up.
  - option: Start with full crate restructuring before gateway/profile work
    reason_rejected: That would delay the immediate processkit v0.25.0 integration and runtime process-count improvements.
  consequences: Phase 3 is not part of the immediate pre-release implementation path. It should receive separate investigation, discussion, and likely additional decision records before implementation. The next active work is phase 1 implementation.
  decided_at: '2026-05-02T21:05:45+00:00'
---
