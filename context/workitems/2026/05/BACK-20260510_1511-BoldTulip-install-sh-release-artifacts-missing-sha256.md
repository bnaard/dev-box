---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260510_1511-BoldTulip-install-sh-release-artifacts-missing-sha256
  created: '2026-05-10T15:11:19+00:00'
  labels:
    version: v0.25.8-candidate
    area: release-tooling
    kind: security
spec:
  title: 'install.sh: release artifacts missing .sha256 checksum files; install skips
    verification'
  state: backlog
  type: bug
  priority: high
  description: |
    ## Symptom

    Running the install one-liner against v0.25.7:

    ```
    curl -fsSL https://raw.githubusercontent.com/projectious-work/aibox/main/scripts/install.sh | bash
    ```

    surfaces:

    ```
    ! Checksum file not found at https://github.com/projectious-work/aibox/releases/download/v0.25.7/aibox-v0.25.7-aarch64-apple-darwin.tar.gz.sha256 — skipping verification.
    ! Consider pinning VERSION=0.25.7 and verifying manually:
    !   sha256sum aibox-v0.25.7-aarch64-apple-darwin.tar.gz
    ```

    The install proceeds without verifying the binary, falling back to bare-tarball install. This is a security regression: the v0.25.6 BR-SEC-HARDEN work mandated checksum verification for every addon binary download path, and the aibox binary install should follow the same rule.

    ## Root cause (likely)

    The `scripts/maintain.sh release` flow builds + uploads the `.tar.gz` artifacts (`aibox-v0.25.7-aarch64-apple-darwin.tar.gz`, `aibox-v0.25.7-x86_64-apple-darwin.tar.gz`, plus aarch64+x86_64 Linux variants) but does NOT generate paired `.sha256` sidecar files and upload them alongside.

    `scripts/install.sh` correctly looks for `<artifact>.sha256` adjacent to each tarball — its check is right; the release script just doesn't produce the checksum files.

    ## Affected releases

    All four v0.25.7 binaries (Linux aarch64, Linux x86_64, macOS aarch64, macOS x86_64). Likely affected on prior releases too, depending on when checksum sidecar generation was last present in `scripts/maintain.sh`.

    ## Fix

    In `scripts/maintain.sh::cmd_release` (around the binary upload step):

    1. After each `cp` or `tar -czf` that produces an artifact in `dist/`, also produce a `.sha256` sidecar:
       ```bash
       sha256sum dist/aibox-v${version}-${target}.tar.gz | awk '{print $1}' > dist/aibox-v${version}-${target}.tar.gz.sha256
       ```
    2. Include `dist/aibox-v${version}-*.tar.gz.sha256` in the `gh release create --upload` arg list.
    3. The macOS `release-host` flow should mirror the same — generate `.sha256` sidecars locally and upload them via `gh release upload`.

    Verify `scripts/install.sh`'s expected sha256 file format matches what `sha256sum` produces (likely just the hex digest on a line; install.sh may strip filename suffixes — confirm before changing).

    ## Backfill for v0.25.7

    After the fix lands, generate `.sha256` sidecars for the existing v0.25.7 binaries by downloading them, hashing them locally, and uploading the sidecars via `gh release upload v0.25.7 *.sha256`. This unblocks existing v0.25.7 installs without requiring a re-release.

    ## Acceptance

    - After fix lands: `gh release view v0.25.7` shows 8 assets (4 tarballs + 4 .sha256 sidecars).
    - Re-running the install one-liner displays "✓ Checksum verified" and proceeds.
    - v0.25.8+ releases automatically include sidecars without manual intervention.

    ## Refs

    - `scripts/install.sh` (the consumer that looks for `.sha256`)
    - `scripts/maintain.sh::cmd_release` and `cmd_release_host` (the producers that need to generate sidecars)
    - DEC-20260508_1515-SilentAsh / BR-SEC-HARDEN — the security-hardening epic that mandated checksum verification for binary downloads.
    - Surfaced 2026-05-10 by owner running install one-liner against v0.25.7 fresh install.
---
