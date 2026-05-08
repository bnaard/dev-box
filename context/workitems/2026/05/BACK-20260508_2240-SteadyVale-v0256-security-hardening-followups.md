---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260508_2240-SteadyVale-v0256-security-hardening-followups
  created: '2026-05-08T22:40:46+00:00'
  labels:
    track: security
    release: v0.25.6
    deferred_via: DEC-20260508_2235-CuriousBadger
  updated: '2026-05-08T23:11:27+00:00'
spec:
  title: 'v0.25.6: Security-hardening followups — addon checksums, AWS GPG, Codex
    seccomp'
  state: review
  type: task
  priority: high
  description: |
    ## Goal

    Resolve the three security-hardening items from the v0.25.6 deferred list before tagging.

    ## Items

    ### S1 — Hermes / OpenCode addons pinned without checksums — DEFERRED to v0.25.7 (2026-05-09 scope-pass)
    - Investigation 2026-05-09: both `addons/ai/ai-hermes.yaml` and `addons/ai/ai-opencode.yaml` already carry detailed `TODO(sec)` blocks documenting the upstream gap. Neither Nous Research nor opencode.ai publishes per-release SHA-256 or GPG signatures; the current pinned-versioned-download is the best practical posture until upstream ships verification material.
    - Tracked at `BACK-20260508_2257-BraveCrow-hermes-opencode-checksum-upstream-watch` (v0.25.7).
    - **Status: DEFERRED — not actionable in v0.25.6.**

    ### S2 — AWS GPG key fetched at build-time from keyserver — DROPPED (2026-05-09 scope-pass)
    - Investigation 2026-05-09: `grep -rn keyserver/recv-keys/AWS_GPG /workspace/images/ /workspace/cli/src/` returns zero matches. Either fixed in a prior release or never existed in the form described in the prior-session handover.
    - **Status: DROPPED — not in repo. No follow-up filed; if the pattern reappears in a future build, file a fresh WorkItem.**

    ### S3 — Codex seccomp acknowledgement gate — KEEP (target for v0.25.6)
    - File: `cli/src/security.rs` (or wherever the `[security].acknowledge_seccomp_unconfined` flag is read), plus the install/upgrade flow.
    - Today: existing Codex projects must manually set `[security].acknowledge_seccomp_unconfined = true` in their project before next `aibox apply` will run. There's no in-CLI prompt or migration helper.
    - Target: surface a clear migration prompt during `aibox apply` that detects the missing acknowledgement, explains the trade-off, and offers to write the flag (interactive) or refuses with a helpful error (non-interactive). Document in the v0.25.6 release notes.

    ## Dispatch hint
    - S1 + S2: Robin (junior eng / mechanical edits) for the file changes; Sage (CTO / architecture) reviews the bundling approach for S2.
    - S3: Avery (senior eng) — touches CLI flow and migration helper.

    ## Acceptance
    - All three items either land in v0.25.6 or get a fresh DEC re-deferring them with concrete reasons.
    - Release notes mention each (S1: "addon integrity hashes added"; S2: "AWS GPG key bundled, no longer keyserver-fetched"; S3: "Codex seccomp acknowledgement now prompted on apply").
  started_at: '2026-05-08T23:03:04+00:00'
---

## Transition note (2026-05-08T23:03:04+00:00)

Starting S3 (Codex seccomp ack prompt during aibox apply) via Avery. S1 deferred to v0.25.7 (BACK-20260508_2257-BraveCrow). S2 dropped (not in repo, see WorkItem description).


## Transition note (2026-05-08T23:11:27+00:00)

S3 (Codex seccomp ack consent gate) implemented and tested by Avery. Summary:

- New `evaluate_seccomp_consent()` decision function (pure) + `ensure_seccomp_consent()` orchestrator in `cli/src/container.rs`. Wired into `cmd_sync()` post-config-load, pre-generate.
- Interactive: TTY-detect via `std::io::IsTerminal`, dialoguer Confirm with default=false (stray Enter declines). On accept, persists `[security].acknowledge_seccomp_unconfined = true` to `aibox.toml` via `toml_edit` (preserves comments + key order).
- Non-interactive: refuses with helpful error pointing at the flag, the toml path, and how to re-run.
- Defers to user-declared compose-override seccomp=unconfined when present (matches existing generate.rs gate).
- 9 new unit tests in `cli/src/container.rs::mod tests`. Total seccomp-related: 17/17 pass.
- `cli/src/generate.rs` exports `compose_override_declares_codex_seccomp_pub()` thin wrapper for the override-already-declared path.

Acceptance covered:
- (a) flag set → silent proceed: tested.
- (d) flag unset + non-interactive → helpful refuse: tested (decision side + message content).
- (b/c) flag unset + interactive: decision branch tested; the dialoguer interactive glue (~10 lines) not unit-tested without a Prompter trait refactor (out of S3 scope; flagged as v0.25.7 follow-up if wanted).

S1 deferred to v0.25.7 (BACK-20260508_2257-BraveCrow). S2 dropped (not in repo, see WorkItem description).

Ready for owner review.
