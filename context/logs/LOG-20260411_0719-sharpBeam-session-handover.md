---
apiVersion: processkit.projectious.work/v1
kind: LogEntry
metadata:
  id: LOG-20260411_0719-sharpBeam-session-handover
  created: '2026-04-11T07:19:02+00:00'
spec:
  event_type: session.handover
  timestamp: '2026-04-11T07:20:00Z'
  summary: Session handover — release process codified; v0.17.14 shipped; container
    rebuild needed for binary backfill
  actor: claude-sonnet-4-6
  details:
    session_date: '2026-04-11'
    current_state: 'Main branch is clean at 5b5e774. Two releases were shipped this
      session: v0.17.13 (mandatory MCP enforcement, Rust cross-compile addon, yazi
      git status, Zellij Ctrl+g) and v0.17.14 (docs-docusaurus fix: @docusaurus/core
      instead of create-docusaurus, version pinned to 3.8). Both releases are missing
      all binary assets (Linux and macOS) because `maintain.sh release` was bypassed
      in favour of manual `gh release create`. The release process gap has now been
      codified: AGENTS.md updated with two anti-pattern entries, and `context/processes/aibox-release.md`
      created as a proper Process entity. AGENTS.md has one uncommitted change (the
      anti-pattern additions).'
    open_threads:
    - AGENTS.md anti-pattern additions are uncommitted — need a `git add AGENTS.md
      && git commit` before closing
    - v0.17.13 and v0.17.14 are missing all binary assets — use retrofit recipe in
      context/processes/aibox-release.md once container is rebuilt
    - Container must be rebuilt to get gcc/cross-compile toolchain (Rust addon v1.1.0
      fix from this session). Until rebuilt, cargo build --release --target aarch64-unknown-linux-gnu
      fails with 'linker not found'
    - 'Phase 2 for both releases: run `./scripts/maintain.sh release-host 0.17.13`
      then `./scripts/maintain.sh release-host 0.17.14` on macOS host to get macOS
      binaries uploaded'
    - 'Entity IDs all show _0000 for the time component — investigation was interrupted;
      root cause not yet determined. ids.py uses datetime.now(timezone.utc) which
      should be correct, but all 51 entities have _0000. Suspicion: the MCP server
      process starts at midnight UTC (00:00 Berlin = 02:00 UTC) or the strftime result
      is being truncated somewhere'
    - Host-side Rust addon YAML (~/.config/aibox/addons/languages/rust.yaml on HOST)
      needs updating to match addons/languages/rust.yaml v1.1.0 — owner action required
      after container rebuild
    next_recommended_action: 'Commit the AGENTS.md change (`git add AGENTS.md && git
      commit -m ''docs: add release anti-patterns to AGENTS.md''`), then rebuild the
      container to activate the Rust addon v1.1.0 gcc/cross-compile fix, then use
      the retrofit recipe in context/processes/aibox-release.md to backfill Linux
      binaries for v0.17.13 and v0.17.14.'
    branch: main
    commit: 5b5e774
    uncommitted_changes:
    - AGENTS.md (modified — anti-pattern additions, not yet committed)
    behavioral_retrospective:
    - 'Bypassed maintain.sh release in favour of manual version bump + gh release
      create — produced two releases with no binary assets. Root cause: AGENTS.md
      said ''ship it'' without specifying HOW Phase 1 is done. Fixed by adding explicit
      anti-patterns to AGENTS.md and creating context/processes/aibox-release.md.'
    - Did not check skill-finder before acting on domain tasks (release, work-instruction
      creation). The process-management and session-handover skills were only consulted
      after the user pointed them out. Added to AGENTS.md already; will apply at next
      session start.
    - Proposed non-standard file path (context/work-instructions/RELEASE.md) for the
      work instruction — corrected to context/processes/aibox-release.md following
      processkit Process entity conventions after user pushback.
---
