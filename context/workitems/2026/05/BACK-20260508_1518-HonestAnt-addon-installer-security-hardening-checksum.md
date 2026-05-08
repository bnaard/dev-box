---
apiVersion: processkit.projectious.work/v2
kind: WorkItem
metadata:
  id: BACK-20260508_1518-HonestAnt-addon-installer-security-hardening-checksum
  created: '2026-05-08T15:18:34+00:00'
  labels:
    track: sec-harden
    release: v0.25.6
spec:
  title: 'v0.25.6: Addon and installer security hardening'
  state: backlog
  type: task
  priority: high
  description: |
    ## Goal
    Resolve every concrete supply-chain risk identified in the v0.25.6 review. Every binary download must be checksum-verified and no addon may use unverified `curl | bash`.

    ## Items

    ### S1 — Replace `curl | bash` for ai-hermes (HIGH)
    - File: `addons/ai/ai-hermes.yaml:27`.
    - Pattern to follow: `addons/languages/node.yaml:50-55` (the bun replacement).
    - Determine vendor's official binary release asset + signature/checksum and adopt download → verify → install steps.

    ### S2 — SHA-256 verification for aws-cli (MEDIUM)
    - File: `addons/tools/cloud-aws.yaml:21`.
    - AWS publishes signed installers (https://docs.aws.amazon.com/cli/latest/userguide/install-cliv2-linux.html#install-cliv2-linux-verify) with a separate `.sig` and the AWS-CLI public key. Adopt full GPG verification path.
    - Same review pass for azure-cli and gcloud-cli — document if they already verify or if hardening is needed.

    ### S3 — Checksum verification for packer (MEDIUM)
    - File: `addons/tools/infrastructure.yaml:52`.
    - HashiCorp publishes `*_SHA256SUMS` and `*_SHA256SUMS.sig` per release. Mirror the OpenTofu pattern already at `infrastructure.yaml:43-45`.

    ### S4 — Checksum verification in scripts/install.sh (MEDIUM)
    - File: `scripts/install.sh:152-176`.
    - Add SHA-256 verification of the downloaded release tarball against a checksum either pinned in the script or fetched from the release manifest. Print the computed and expected checksum on failure.

    ### S5 — Gate seccomp=unconfined behind explicit consent (LOW)
    - File: `cli/src/templates/docker-compose.yml.j2:31-33`.
    - Currently emits `seccomp=unconfined` whenever `codex_sandbox_seccomp` is true. Add a top-level `[security].acknowledge_seccomp_unconfined = false` (default) gate; if false but the codex sandbox needs it, error in `aibox apply` with remediation pointer.

    ### S6 — Document MCP gateway trust scope (architectural note, not a code change)
    - File: a short note under `docs-site/.../security.md` or in AGENTS.md.
    - Make explicit that any installed processkit skill's `mcp/server.py` runs with project-user trust inside the container (`mcp_registration.rs:898`). Recommend a review checklist before installing third-party skills.

    ### S7 — General audit pass on remaining addon yamls
    - Grep `addons/**/*.yaml` for `curl ` / `wget ` / `RUN apt` and assert each has a checksum or signed apt-source + GPG key import. Document any exception with rationale.

    ## Acceptance criteria
    - `grep -rEn "curl[^|]*\|.*sh|wget.*\|.*sh" addons/` returns 0.
    - `grep -rEn "(curl|wget) " addons/ scripts/install.sh | grep -v "sha256sum\|gpg\|--checksum"` returns 0 unverified downloads (each one either followed by checksum verification or annotated with a justification comment).
    - `aibox doctor` warns when the project is using seccomp-unconfined without explicit consent.

    ## Dispatch hint for next session
    One general-purpose subagent. Largely mechanical; the agent must look up the official verification path for each vendor before editing.
---
