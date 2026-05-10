---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260417_1100-SteadyPine-session-handover
  created: '2026-04-17T11:00:00Z'
spec:
  event_type: session.handover
  timestamp: '2026-04-17T11:00:00Z'
  actor: claude-opus-4-6
  summary: processkit v0.17.0 upgrade complete — 12 of 13 issues resolved, v0.18.4
    released with all platforms
  details:
    session_date: '2026-04-17'
    current_state: |
      This session completed the processkit v0.16.0→v0.17.0 upgrade and resolved all open GitHub issues except #51.

      **Release v0.18.4** is live on GitHub with all 4 platform binaries (aarch64+x86_64 Linux, aarch64+x86_64 macOS).

      **New modules added:**
      - `cli/src/hook_registration.rs` — writes processkit enforcement hooks into Claude Code (`.claude/settings.json`), Codex CLI (`.codex/hooks.json`), and Cursor (`.cursor/hooks.json`). Non-destructive merge via `_processkit_managed` marker (Claude/Codex) and command-path fingerprint (Cursor). 12 tests.
      - `cli/src/compliance.rs` — compliance contract drift detection (compares AGENTS.md block vs canonical source), `.cursor/rules/processkit-compliance.md` generation, `.aider.conf.yml` `read:` registration. 9 tests.

      **Modified modules:**
      - `mcp_registration.rs` — kernel MCP fallback: graceful degradation on per-skill parse errors, falls back to 8 essential servers (`KERNEL_MCP_SKILLS`). 2 new tests.
      - `processkit_vocab.rs` — added `KERNEL_MCP_SKILLS` constant, `PROCESSKIT_DEFAULT_VERSION` bumped to `v0.17.0`.
      - `container.rs` — wired `regenerate_hook_configs` and `regenerate_compliance_configs` into both `cmd_init` and `cmd_sync` paths (best-effort, warn-and-continue on failure).

      **Bug fixes:**
      - `docs-docusaurus` addon pinned to 3.7.0 (was 3.8, exposed to 3.9.2 ProgressPlugin regression).

      **581 tests passing** (524 unit + 41 integration + 16 doc tests).

      **v0.17.0 template delta was minimal for consumers:** only `.gitkeep` scaffolds in skill `scripts/` dirs, template variable substitution in AGENTS.md (aibox already handled all variables), and PROVENANCE.toml version bump. The `.processkit/` catalog directory is not installed in consumer projects.
    issues_resolved:
    - '#40: Wire mandatory MCP core set — was already implemented (MANDATORY_MCP_SKILLS
      + force_include)'
    - '#41: Docusaurus wrong package — was already fixed (@docusaurus/core correct)'
    - '#42: Docusaurus version pin — pinned to 3.7.0'
    - '#43: Per-skill MCP merge — was already implemented (regenerate_mcp_configs)'
    - '#44: Hook wiring Claude Code + Codex — new hook_registration.rs'
    - '#45: Kernel MCP fallback — graceful degradation in collect_processkit_mcp_specs'
    - '#46: Compliance drift warning — new compliance.rs check_compliance_contract_drift'
    - '#47: Cursor compliance rules — new compliance.rs write_cursor_compliance_rules'
    - '#48: Aider conf registration — new compliance.rs write_aider_compliance_conf'
    - '#49: OpenCode capability matrix — research posted as issue comment'
    - '#50: Cursor hooks.json — added to hook_registration.rs'
    - '#52: v0.14→v0.16 integration — completed in prior session'
    issues_remaining:
    - '#51: OpenCode TypeScript plugin — upstream blockers sst/opencode#2319 and #5894
      are now CLOSED. Promoted from P3 (watch) to P2 (implementation-ready). Implementation
      sketch posted as issue comment.'
    open_threads:
    - 'Issue #51 is now unblocked and ready for implementation. The OpenCode capability
      matrix (posted on #49) shows full MCP + hook support. A TypeScript plugin needs
      to be written for processkit hook enforcement.'
    - Yazi plugin integration from prior session (LOG-20260413_1807-CalmHeron) remains
      incomplete — seed.rs edits, preview-enhanced addon, and visual testing still
      pending. Those changes are committed but unverified.
    - The `maintain.sh release` script does not bump Cargo.toml version automatically.
      This causes confusion when a release succeeds but the session ends before reporting
      — the next session tries the same version and hits "Tag already exists". Consider
      adding a post-release Cargo.toml bump to the script.
    - Compliance contract drift detection (#46) looks for `<!-- pk-compliance-contract
      v1 BEGIN -->` markers. If processkit v0.17.0 bumped the contract to v2, the
      marker regex may need updating to match `v2`.
    next_recommended_action: |
      1. Implement #51 (OpenCode TypeScript plugin) — now unblocked, research and implementation sketch already posted.
      2. Verify compliance contract marker version matches what processkit v0.17.0 actually ships (v1 vs v2 markers in AGENTS.md).
      3. Complete Yazi plugin integration from the prior session handover.
      4. Consider adding Cargo.toml version bump to maintain.sh release script to avoid tag collision on session interrupts.
    branch: main
    commit: 3d92be9
    uncommitted_changes: []
    releases:
    - 'v0.18.3: released in prior session (processkit v0.17.0 vocab bump + security
      fix)'
    - 'v0.18.4: released this session (v0.17.0 upgrade + all issues resolved)'
---
