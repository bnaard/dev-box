---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260508_2257-SmoothFalcon-no-github-actions-ci-for-aibox
  created: '2026-05-08T22:57:50+00:00'
spec:
  title: No GitHub Actions CI for aibox — release verification stays in scripts +
    e2e companion container
  state: accepted
  decision: aibox does not adopt GitHub Actions or any other paid CI service. Release
    verification stays in the existing local scripts (`scripts/release-check-state.sh`,
    `scripts/release-runtime-smoke.sh`, `scripts/aibox-upgrade-test.sh`, `scripts/maintain.sh
    test-e2e*`) and the e2e companion container, all run from the developer's own
    host as part of `./scripts/maintain.sh release` Phase 1. Any future "add CI" suggestion
    must address the cost gate (the project does not have a paid GitHub account; GitHub
    Actions minutes are billed) before it can be considered.
  context: 'During the v0.25.6 deferred-item scope-pass on 2026-05-09, item Q5 ("Surface
    release-smoke status diffs in CI", originally from BACK-20260508_1519-LuckyLily)
    was investigated. Finding: `/workspace/.github/workflows/` does not exist; `.github/`
    contains only `repository-metadata.md`. The repo has no CI/CD pipeline at all.
    Owner clarified: "I have not paid github account and CI costs money for actions.
    We continue to do smoke tests during release with our scripts and our e2e companion
    container." This makes Q5 not just out of v0.25.6 scope but architecturally not
    on the project''s roadmap until the cost gate is resolved.'
  rationale: aibox is a single-developer / small-team project. The release ritual
    is already structured around host-run scripts and an e2e companion container that
    captures runtime UX. Adding GitHub Actions would (a) cost money (the project's
    GitHub account is on the free tier with limited Actions minutes — and the owner's
    stated stance is they have not paid), (b) duplicate verification capability already
    present in the local scripts, and (c) introduce a parallel verification surface
    that could drift from the canonical local one. The cost-gate framing ("address
    the cost gate before suggesting CI") prevents future agents from re-proposing
    CI as an obvious win.
  alternatives:
  - option: Adopt GitHub Actions on the free tier
    rejected_because: Free-tier minutes are limited; release flows that build images
      and run e2e are likely to exceed them. Owner's stance is no paid plan.
  - option: Use a different free-tier CI (CircleCI, Drone, self-hosted Actions runner)
    rejected_because: Each adds setup + maintenance overhead with no clear win over
      the existing host-run scripts. If the cost gate is ever lifted, this could be
      revisited.
  - option: Punt Q5 to v0.25.7 instead of dropping
    rejected_because: Punting implies the work is intended; the architectural answer
      is 'not on the roadmap'. Dropping with explicit DEC rationale prevents Q5 reappearing
      in handovers.
  consequences: |
    - Q5 (LuckyLily Q5 from BACK-20260508_1519-LuckyLily) is dropped from v0.25.6 AND not deferred to a future release. Closed with this DEC as the rationale.
    - Future "add CI" or "add GitHub Actions" proposals are pre-disqualified unless they include a cost-coverage plan.
    - Release verification remains a local/host responsibility; the release-runtime-smoke baseline is the smoke-baseline contract.
    - This DEC supersedes the v0.25.6 portion of LuckyLily Q5 commitment. The corresponding bullet in BACK-20260508_2241-ToughAsh's description should be marked DROPPED (separate WorkItem update).
    - LuckyLily's original Q5 spec was based on the prior session's planning without an explicit cost-gate check — this DEC corrects that.
  deciders:
  - TEAMMEMBER-thrifty-otter
  related_workitems:
  - BACK-20260508_2241-ToughAsh-v0256-ci-code-quality-followups
  - BACK-20260508_1519-LuckyLily-code-quality-aibox-toml-seed-split
  decided_at: '2026-05-08T22:57:50+00:00'
---
