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
spec:
  title: 'v0.25.6: Security-hardening followups — addon checksums, AWS GPG, Codex
    seccomp'
  state: backlog
  type: task
  priority: high
  description: |
    ## Goal

    Resolve the three security-hardening items from the v0.25.6 deferred list before tagging.

    ## Items

    ### S1 — Hermes / OpenCode addons pinned without checksums
    - Files: addon yaml files under `context/skills/<name>/config/` or `cli/src/templates/addons/` (locate exact paths first).
    - Today: Hermes and OpenCode addon installers pin a version but do not record a SHA256/integrity hash, leaving us trusting only TLS + version pinning. TODO comments mark the gap.
    - Target: add a `sha256:` (or equivalent integrity field) for each pinned addon release. If the upstream provides no signature/checksum, document the vendor-gap in a comment and verify via a separate channel (release page, download stats).

    ### S2 — AWS GPG key fetched at build-time from keyserver
    - File: likely `images/base-debian/Dockerfile` or a build-time script under `images/base-debian/`.
    - Today: an AWS GPG signing key is fetched from a public keyserver during image build, which is brittle (keyserver outages) and weaker than bundling the trusted key in-tree.
    - Target: bundle the AWS GPG key in `images/base-debian/keys/` (or equivalent), reference it directly during build, eliminate the keyserver fetch.

    ### S3 — Codex seccomp acknowledgement gate
    - File: `cli/src/security.rs` (or wherever the `[security].acknowledge_seccomp_unconfined` flag is read), plus the install/upgrade flow.
    - Today: existing Codex projects must manually set `[security].acknowledge_seccomp_unconfined = true` in their project before next `aibox apply` will run. There's no in-CLI prompt or migration helper.
    - Target: surface a clear migration prompt during `aibox apply` that detects the missing acknowledgement, explains the trade-off, and offers to write the flag (interactive) or refuses with a helpful error (non-interactive). Document in the v0.25.6 release notes.

    ## Dispatch hint
    - S1 + S2: Robin (junior eng / mechanical edits) for the file changes; Sage (CTO / architecture) reviews the bundling approach for S2.
    - S3: Avery (senior eng) — touches CLI flow and migration helper.

    ## Acceptance
    - All three items either land in v0.25.6 or get a fresh DEC re-deferring them with concrete reasons.
    - Release notes mention each (S1: "addon integrity hashes added"; S2: "AWS GPG key bundled, no longer keyserver-fetched"; S3: "Codex seccomp acknowledgement now prompted on apply").
---
