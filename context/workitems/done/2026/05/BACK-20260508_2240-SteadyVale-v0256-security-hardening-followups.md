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
  updated: '2026-08-21T19:58:57+00:00'
spec:
  title: 'v0.25.6: Security-hardening followups — addon checksums, AWS GPG, Codex
    seccomp'
  state: done
  type: task
  priority: high
  description: Migrated historical description; see git history for pre-migration
    full text.
  started_at: '2026-05-08T23:03:04+00:00'
  completed_at: '2026-08-21T19:58:57+00:00'
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


## Transition note (2026-08-21T19:58:57+00:00)

Closed from the stale review queue by owner direction on 2026-08-21; subsequent v0.x releases supersede this historical review item.
