---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260809_1128-PolishedTiger-use-mandatory-core-and-impact-triggered
  created: '2026-08-09T11:28:15+00:00'
spec:
  title: Use mandatory core and impact-triggered host coverage for aibox releases
  state: accepted
  decision: Every release host gate must run the candidate-image lifecycle, --forget-tmux-state
    validation, native Darwin smoke, fail-closed cleanup, SBOM generation, and vulnerability
    scanning. Download-based addon build groups, the LaTeX watcher and preview lifecycle,
    and the rootless Podman runtime probe run as mandatory checks only when verified
    changed paths can affect them. Alpine smoke, companion reachability and tool-presence
    checks, and duplicate lifecycle wrappers are removed permanently.
  context: 'Issue #372 replaces privileged companion-based E2E coverage with local
    least-privilege tests plus a restricted macOS host gate. The remaining former
    host tests needed classification by release value and impact.'
  rationale: Core lifecycle, platform smoke, cleanup, and supply-chain evidence validate
    every candidate and therefore remain unconditional. Expensive addon, LaTeX, and
    nested-runtime tests add distinct behavioral surface only when relevant inputs
    change, so path-triggering preserves their value without making every release
    pay their cost. The removed checks either exercise unsupported Alpine, test infrastructure
    availability instead of product behavior, or duplicate the canonical lifecycle.
  alternatives:
  - option: Run all former host tests on every release
    rejected_because: High cost and flakiness without proportional test-surface value
      for unrelated changes.
  - option: Remove all former host-only tests
    rejected_because: Would lose unique addon build, LaTeX live-rebuild and preview,
      and rootless nested Podman behavioral coverage.
  - option: Keep companion reachability and duplicate wrapper tests
    rejected_because: They validate test infrastructure or duplicate the canonical
      candidate lifecycle rather than product behavior.
  consequences: Host preparation must attest the comparison revision and exact changed-path
    set. The gate must verify that provenance before selecting checks, record selected
    and skipped checks as evidence, fail closed for every selected check, and always
    perform cleanup. Broad generator, base-image, or addon-registry changes trigger
    all affected host probes. Documentation-only changes avoid expensive conditional
    builds.
  related_workitems:
  - BACK-20260808_1709-CuriousAnt-replace-privileged-e2e-companion-with-restricted
  decided_at: '2026-08-09T11:28:15+00:00'
---
