---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260508_2241-ToughAsh-v0256-ci-code-quality-followups
  created: '2026-05-08T22:41:02+00:00'
  labels:
    track: ci-code-quality
    release: v0.25.6
    deferred_via: DEC-20260508_2235-CuriousBadger
spec:
  title: 'v0.25.6: CI + code-quality followups — Q5 release-smoke diffs, Q7 skills
    comment fact-check, BR-CLEANUP-ARCH item 6'
  state: backlog
  type: task
  priority: medium
  description: |
    ## Goal

    Resolve the three CI + code-quality items from the v0.25.6 deferred list before tagging.

    ## Items

    ### Q5 — Surface release-smoke status diffs in CI
    - File: `.github/workflows/*.yml` (find existing release/CI workflow) or a new step.
    - Spec from LuckyLily Q5 (BACK-20260508_1519-LuckyLily): after `cargo build --release`, run a minimal `release-runtime-smoke.sh` slice and emit a per-PR diff against the latest `dist/release-smoke/v0.25.X/` baseline for `tmux-state.txt` and `up-forget-tmux-state.log`. Failure on regex regression of known stable lines.
    - Practical adjustment: the smoke script needs Docker; for CI we may need a minimal alternative or a cached baseline diff that doesn't require running the full smoke. Investigate before implementing.

    ### Q7 — Streamline / fact-check skills `[skills]` inline comments in aibox.toml
    - File: root `aibox.toml`.
    - Spec from LuckyLily Q7: walk the comment list once and fix any descriptions that are stale or contradicted by the current SKILL.md.
    - Mechanical pass; one general-purpose subagent (Robin) reads each `# processkit; <description>` line, reads the matching SKILL.md, and updates if drifted.

    ### BR-CLEANUP-ARCH item 6 — Variant 3 Migration emission
    - File: `cli/src/migrate.rs` or wherever the runtime-cleanup architecture emits Migration entities.
    - Background: BR-CLEANUP-ARCH (the broader cleanup-architecture rework) had 6 items; items 1–5 shipped; item 6 (Variant 3 Migration emission) was deferred with a TODO comment.
    - Need to read the BR-CLEANUP-ARCH note / DEC to recover the exact spec, then implement.

    ## Dispatch hint
    - Q5: Avery (senior eng) — needs CI workflow knowledge and judgement on the Docker/non-Docker trade-off.
    - Q7: Robin (junior eng / mechanical) — large but mechanical pass.
    - BR item 6: Avery — touches the runtime-cleanup architecture; recovery of the exact spec is the first step.

    ## Acceptance
    - All three either land or get re-deferred via fresh DEC.
    - Release notes mention Q5 (CI gate) and BR-CLEANUP item 6 if shipped.
---
