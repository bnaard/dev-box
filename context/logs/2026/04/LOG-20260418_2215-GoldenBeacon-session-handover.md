---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260418_2215-GoldenBeacon-session-handover
  created: '2026-04-18T20:15:52Z'
spec:
  event_type: session.handover
  timestamp: '2026-04-18T20:15:52Z'
  actor: claude-opus-4-7
  summary: v0.18.7 shipped — processkit v0.18.2 integrated, Linux Phase 1 + macOS
    Phase 2 complete, GitHub release and docs deployed; 3 MCP writes remain deferred
    pending harness restart.
  details:
    session_date: '2026-04-18'
    current_state: |
      **v0.18.7 is live.** Tag `v0.18.7` pushed; GitHub release
      created with both Linux binaries
      (`aibox-v0.18.7-aarch64-unknown-linux-gnu.tar.gz`,
      `aibox-v0.18.7-x86_64-unknown-linux-gnu.tar.gz`); docs
      deployed to gh-pages (clean skip on Pages auto-config — the
      v0.18.7 probe-first fix worked); macOS Phase 2 confirmed
      done by the owner on host.

      **Main is clean at `5db8a34`.** Session landed three commits
      on top of the prior v0.18.7 code-complete commit (`26d3b0e`):
      1. `b010680` feat(v0.18.7): integrate processkit v0.18.2
         — bumped `PROCESSKIT_DEFAULT_VERSION` v0.18.1→v0.18.2 in
         `cli/src/processkit_vocab.rs`, bumped v0.18.7
         COMPAT_TABLE entry's `processkit_version` in
         `cli/src/compat.rs`, refreshed `aibox.lock` +
         `.devcontainer/*` against v0.18.7, absorbed 5 in-place
         processkit v0.18.2 skill updates (skill-gate server/hooks,
         session-handover SKILL, standup-context command,
         compliance-contract asset).
      2. `044832f` chore: track carried session artifacts +
         v0.18.2 template snapshot — committed the two prior
         session handovers (BrightSummit, QuietForge), the
         applied v0.18.5→v0.18.6 migration doc, three pending
         migration docs, and the v0.18.6 aibox-home +
         v0.18.2 processkit template snapshots. Added
         `__pycache__/` and `*.pyc` to `.gitignore` (processkit
         MCP helper libs generate these). Required to pass the
         release preflight cleanliness check.
      3. `5db8a34` style: cargo fmt — collapse two-line assertion
         — caught by `maintain.sh release`'s fmt preflight; no
         behavior change.

      **`.mcp.json` is now valid end-to-end.** 16/16 server
      script paths exist on disk (verified after sync). The
      processkit#8 upstream fix landed in v0.18.2, so all
      per-skill `mcp-config.json` files ship with the correct
      `processkit/` category prefix from upstream and the
      local hotpatch is no longer needed. Claude Code's
      next handshake should load all 16 servers; in-session
      the harness still has only the 4 pre-restart servers
      (skill-finder, skill-gate, task-router, artifact-management),
      which is why the three deferred writes (see below) could not
      be filed this session.

      **Release flow actually executed:**
      Phase 1 (Linux + docs, from inside the devcontainer):
      preflight OK → cargo fmt check OK (after `5db8a34`) →
      617/617 tests pass → cargo audit clean → built aarch64 +
      x86_64 binaries → `aibox --version` = 0.18.7 → tag pushed →
      GitHub release created with both binaries → docs built by
      Docusaurus → gh-pages force-pushed → GitHub Pages
      auto-config correctly skipped (no spurious warning —
      v0.18.7 probe-first worked).
      Phase 2 (macOS + GHCR, from host):
      `./scripts/maintain.sh release-host 0.18.7` — confirmed
      done by owner.

      **Addon definitions workaround applied:**
      `aibox sync` in the devcontainer failed on first run with
      "Addon definitions not found at
      /home/aibox/.config/aibox/addons". Worked around by
      `cp -r /workspace/addons /home/aibox/.config/aibox/addons`.
      Same dev-env limitation noted in the prior handover. Worth
      filing as a real issue: `aibox init` (host) and `aibox sync`
      (container) should find addon definitions without requiring
      the install script — or the devcontainer image should
      include them at a predictable path. Low priority but aged.
    issues_resolved:
    - processkit v0.18.2 pulled and integrated — resolves processkit#8 upstream-side;
      aibox`s safety rail in v0.18.7 ensures any future regression of the same class
      is caught at sync time instead of silently emitting broken config.
    - '`.mcp.json` regenerated with all 16 server paths valid on disk. The drift diagnosed
      in the prior session (`/workspace/.mcp.json` had 12 broken paths despite per-skill
      `mcp-config.json` hotpatch) is resolved by the upstream fix — no more local
      hotpatch needed going forward.'
    - cargo fmt drift in `cli/src/mcp_registration.rs` test — caught by release preflight,
      fixed in `5db8a34`. Not a blocker but a reminder that pre-commit `cargo fmt`
      is an unwritten norm here.
    - Carried session artifacts committed to git — the accumulation of handover logs,
      migration docs, and template snapshots that had been untracked across multiple
      sessions (because the compliance gate was shut) are now properly tracked. Matches
      the prior pattern of tracked siblings (context/logs/, context/migrations/applied/,
      context/templates/aibox-home/0.17.20/).
    - 'v0.18.7 released end-to-end: Linux Phase 1 and macOS Phase 2 both complete;
      GitHub release live; docs deployed; Pages auto-config warning no longer emitted.'
    issues_remaining:
    - Three MCP writes still deferred, one more than inherited — CarefulFalcon DecisionRecord
      (unchanged from prior), MIG-20260418T111856 apply (unchanged), MIG-20260418T195315
      apply (NEW — the processkit v0.18.1→v0.18.2 migration generated this session;
      18 conflicting files recorded in `context/migrations/pending/`), and either
      a release.prepared or release.shipped event log for tag v0.18.7 / commit 5db8a34.
      All blocked on Claude Code harness re-handshaking `.mcp.json` (needs a restart).
    - MIG-RUNTIME-20260418T175825 runtime migration doc also pending in `context/migrations/pending/`
      — origin unclear (possibly from the prior devcontainer rebuild). Worth an `apply_migration`
      check in the next session.
    - Addon-definitions-not-found on sync inside the devcontainer — had to manually
      `cp /workspace/addons → /home/aibox/.config/aibox/addons`. Worth a real fix
      (bake into image, or fall back to the in-repo copy), but not pressing.
    - Compliance contract drift between AGENTS.md and canonical at `context/skills/processkit/skill-gate/assets/compliance-contract.md`
      — sync warned and suggested `aibox sync --fix-compliance-contract`, but that
      flag is not implemented yet (only referenced in the warning message). Prospective
      feature for a future release.
    - 'Decision-language capture: the owner said "Option B" / "Let''s go" this session.
      Per the v2 compliance contract, those moments should either produce a `record_decision`
      or a `skip_decision_record(reason=...)`. Neither tool was loaded (decision-record
      MCP server offline; `skip_decision_record` not yet shipped upstream). The sequencing
      decision ("release before MCP writes") is operational rather than cross-cutting;
      likely skippable, but worth surfacing to the next session for a formal acknowledgement.'
    - BACK-20260411_0000-SoundRabbit (critical, self-hosted deployment) — unchanged
      across many sessions. Still needs a grooming session.
    - BACK-AmberWren, BACK-CoolBear, BACK-JollyWren, BACK-LoyalSeal — 4 high-priority
      backlog items, still unstarted.
    - Yazi Tier-3 interactive verification — untouched this session; still waiting
      on a host-side run with DISPLAY/TTY.
    open_threads:
    - 'AFTER SESSION RESTART (first 5 minutes): verify that all 16 processkit MCP
      tool surfaces are reachable. Start with `check_contract_acknowledged` → `acknowledge_contract(version="v2")`
      (contract is now v2 in-session as of this session). Then `query_entities(kind="LogEntry",
      event_type="session.handover", limit=1)` as a smoke test that index-management
      and event-log are live.'
    - 'AFTER RESTART — file the deferred entities in one pass: (a) `record_decision`
      for CarefulFalcon (warn-and-continue + bare-name keys; close `BACK-20260418_1145-CarefulFalcon`);
      (b) `apply_migration("MIG-20260418T111856")`; (c) `apply_migration("MIG-20260418T195315")`
      — this one is substantive (18 conflicts from the processkit v0.18.1→v0.18.2
      delta); (d) `apply_migration("MIG-RUNTIME-20260418T175825")` if applicable;
      (e) `create_log_entry(event_type="release.shipped", ...)` for tag v0.18.7 /
      commit 5db8a34 (release is already live, so "prepared" is past tense — "shipped"
      is accurate).'
    - 'AFTER RESTART — retro the decision-language moments: `record_decision` or `skip_decision_record(reason="operational
      sequencing, not cross-cutting")` for the "release before MCP writes" choice
      the owner made this session ("Option B").'
    - MIG-20260418T195315 is not trivial — 18 conflicts across AGENTS and skills/processkit.
      The migration doc at `context/migrations/pending/MIG-20260418T195315.md` lists
      them. Sync decided to force-apply 5 of them (skill-gate server/hooks + a few
      others — those are in `b010680`); the remaining 13 are conflicts that require
      a human decide-keep-local-or-take-upstream pass. Apply_migration will walk the
      conflict list interactively or auto-resolve per configured policy.
    - Compliance contract v1 → v2 transition is complete enough for this session (the
      contract text in SessionStart/UserPromptSubmit hooks shifted to v2 mid-session
      and the v0.18.7 aibox drift checker tolerates both). But the canonical AGENTS.md
      at project root still differs from the canonical source — see `aibox sync --fix-compliance-contract`
      warning above. When the flag is actually implemented upstream, run it.
    - Addon-definitions-not-found workaround is not committed anywhere — lives in
      `/home/aibox/.config/aibox/addons/` which is outside the repo. If the devcontainer
      is rebuilt again without an updated image that bakes it in, the same workaround
      is needed. File a proper fix if the pattern recurs.
    - BACK-20260411_0000-SoundRabbit still needs a grooming session; 4 other high-priority
      backlog items (AmberWren, CoolBear, JollyWren, LoyalSeal) still unstarted.
    next_recommended_action: |
      **Restart Claude Code first, before anything else.** The
      in-session MCP tool surface is still the 4 pre-restart
      servers; the 16-server config on disk is correct and will
      load on the next handshake.

      Then in the opening minutes of the next session, in this
      order:

      1. Smoke-test MCP: `check_contract_acknowledged`,
         `acknowledge_contract("v2")`, `query_entities(kind=LogEntry, limit=1)`.
         If all 16 servers are up, proceed; if any are still
         missing, diagnose before writing anything.

      2. File the deferred entities as a single opening pass:
         - `record_decision` CarefulFalcon → close
           BACK-20260418_1145-CarefulFalcon.
         - `apply_migration("MIG-20260418T111856")` (trivial —
           AGENTS.md v1→v2 marker; drift checker already tolerates
           both).
         - `apply_migration("MIG-20260418T195315")` (18 conflicts
           from processkit v0.18.1→v0.18.2; walk them).
         - `apply_migration("MIG-RUNTIME-20260418T175825")` if
           index-management still lists it as pending.
         - `create_log_entry(event_type="release.shipped", ...)`
           for commit `5db8a34` / tag `v0.18.7`.
         - Either `record_decision` or `skip_decision_record` for
           the owner's "release before MCP writes" call this
           session.

      3. Once the context is compliance-clean, pick from the
         backlog. BACK-SoundRabbit is the most overdue; the 4
         high-priority reviews (AmberWren / CoolBear / JollyWren
         / LoyalSeal) are next tier.
    branch: main
    commit: 5db8a34
    tag: v0.18.7
    uncommitted_changes: []
    stashes: []
    releases:
    - 'v0.18.7: shipped this session. Phase 1 (Linux binaries, GitHub release, docs
      deploy) completed by claude-opus-4-7 inside the devcontainer. Phase 2 (macOS
      binaries + GHCR push) confirmed done by owner. GitHub release live: https://github.com/projectious-work/aibox/releases/tag/v0.18.7'
    behavioral_retrospective:
    - 'Committed work before running `cargo fmt` → release preflight caught the drift.
      Cost one extra commit (`5db8a34`). Encoded for next session: when preparing
      to ship, run `cd cli && cargo fmt --check` BEFORE the first integration commit,
      not as a mid-release patch. Not a process gap — the safety net worked as designed
      — but an avoidable retry.'
    - Delegated pre-integration research to an Explore agent (processkit_vocab.rs
      default, compat.rs entry, aibox.toml/lock state, release script phases, sync
      semantics). Under 300 words, structured report, one turn, zero main-thread context
      spent on filesystem walking. This is the pattern the prior handover encoded
      ("agents are great for research, diff drafting, and read-only investigation;
      main thread does the actual edits") — confirmed useful in practice this session.
    - Distinguished "edit a file under `context/templates/`" (forbidden by contract)
      from "commit an already-present file under `context/templates/`" (correct behavior
      — those template snapshots are meant to be tracked; siblings like the v0.17.20
      snapshot are already in git). The contract text reads as a blanket prohibition;
      the nuance is that the snapshots are WRITTEN by aibox sync automatically and
      should simply be committed without hand editing. Worth surfacing upstream if
      the contract wording drifts this rule.
    - 'Decision-language compliance gap — the v2 contract introduced a new rule: when
      user messages contain approval language ("ok", "yes", "Option B", "let''s go"),
      either `record_decision` or `skip_decision_record` in the same turn. Both tools
      were unavailable this session (server offline / tool not yet shipped upstream).
      Transparent in-text acknowledgement is the honest fallback, and next session
      can retroactively file or skip. Encoded as an open_thread so it doesn''t get
      forgotten.'
    - Addon-definitions workaround (`cp -r /workspace/addons → /home/aibox/.config/aibox/addons`)
      is identical to a workaround pattern noted in the prior handover. Two sessions
      in a row with the same manual step; worth filing as a real aibox-side issue
      so the third session doesn't repeat.
---

# Session summary

This session **integrated processkit v0.18.2 and shipped aibox
v0.18.7 end-to-end** — closing the release hold that the prior
handover had explicitly flagged as waiting on processkit's in-flight
update.

Flow:
- `aibox sync` pulled processkit v0.18.2 (resolving "latest"),
  installed 372 files, regenerated `.mcp.json` with all 16 server
  paths valid (the long-running diagnosis from prior sessions —
  processkit#8 — is now upstream-fixed), and generated a new
  pending migration doc (`MIG-20260418T195315`) recording 18
  conflicting files between project-local and upstream v0.18.2.
- CLI rebuilt at v0.18.7; default processkit version bumped to
  v0.18.2 in `processkit_vocab.rs` and `compat.rs` COMPAT_TABLE
  entry.
- 617/617 tests pass against the v0.18.7 binary.
- Three commits on `main`: integration (`b010680`), tracked session
  artifacts (`044832f`), cargo fmt nit (`5db8a34`).
- `scripts/maintain.sh release 0.18.7` executed Phase 1 inside the
  devcontainer — preflight, fmt check, tests, audit, aarch64 +
  x86_64 binaries, tag push, GitHub release creation, docs build
  and gh-pages deploy, Pages auto-config (correctly skipped —
  probe-first fix worked).
- Owner ran Phase 2 on macOS host (`./scripts/maintain.sh
  release-host 0.18.7`), uploading macOS binaries and pushing
  GHCR images.

Three MCP writes remain deferred — same class of blocker as prior
sessions: the Claude Code harness only re-reads `.mcp.json` at
session start, so even though the disk state is correct, the
in-session tool surface is still the 4 pre-restart servers.
Restart + file the deferred entities is the opening sequence for
the next session.

GitHub release: https://github.com/projectious-work/aibox/releases/tag/v0.18.7
Docs: https://projectious-work.github.io/aibox/
