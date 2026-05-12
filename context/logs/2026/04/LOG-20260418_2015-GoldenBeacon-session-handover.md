---
apiVersion: processkit.projectious.work/v2
kind: LogEntry
metadata:
  id: LOG-20260418_2015-GoldenBeacon-session-handover
  created: '2026-04-18T20:15:52Z'
spec:
  event_type: session.handover
  timestamp: '2026-04-18T20:15:52Z'
  actor: claude-opus-4-7
  summary: "v0.18.7 shipped \u2014 processkit v0.18.2 integrated, Linux Phase 1 + macOS Phase 2 complete, GitHub release and docs deployed; 3 MCP writes remain deferred pending harness restart."
  details:
    session_date: '2026-04-18'
    current_state: "**v0.18.7 is live.** Tag `v0.18.7` pushed; GitHub release\ncreated with both Linux binaries\n(`aibox-v0.18.7-aarch64-unknown-linux-gnu.tar.gz`,\n`aibox-v0.18.7-x86_64-unknown-linux-gnu.tar.gz`); docs\ndeployed to gh-pages (clean skip on Pages auto-config \u2014 the\nv0.18.7 probe-first fix worked); macOS Phase 2 confirmed\ndone by the owner on host.\n\n**Main is clean at `5db8a34`.** Session landed three commits\non top of the prior v0.18.7 code-complete commit (`26d3b0e`):\n1. `b010680` feat(v0.18.7): integrate processkit v0.18.2\n   \u2014 bumped `PROCESSKIT_DEFAULT_VERSION` v0.18.1\u2192v0.18.2 in\n   `cli/src/processkit_vocab.rs`, bumped v0.18.7\n   COMPAT_TABLE entry's `processkit_version` in\n   `cli/src/compat.rs`, refreshed `aibox.lock` +\n   `.devcontainer/*` against v0.18.7, absorbed 5 in-place\n   processkit v0.18.2 skill updates (skill-gate server/hooks,\n   session-handover SKILL, standup-context command,\n   compliance-contract asset).\n2. `044832f` chore:\
      \ track carried session artifacts +\n   v0.18.2 template snapshot \u2014 committed the two prior\n   session handovers (BrightSummit, QuietForge), the\n   applied v0.18.5\u2192v0.18.6 migration doc, three pending\n   migration docs, and the v0.18.6 aibox-home +\n   v0.18.2 processkit template snapshots. Added\n   `__pycache__/` and `*.pyc` to `.gitignore` (processkit\n   MCP helper libs generate these). Required to pass the\n   release preflight cleanliness check.\n3. `5db8a34` style: cargo fmt \u2014 collapse two-line assertion\n   \u2014 caught by `maintain.sh release`'s fmt preflight; no\n   behavior change.\n\n**`.mcp.json` is now valid end-to-end.** 16/16 server\nscript paths exist on disk (verified after sync). The\nprocesskit#8 upstream fix landed in v0.18.2, so all\nper-skill `mcp-config.json` files ship with the correct\n`processkit/` category prefix from upstream and the\nlocal hotpatch is no longer needed. Claude Code's\nnext handshake should load all 16 servers; in-session\n\
      the harness still has only the 4 pre-restart servers\n(skill-finder, skill-gate, task-router, artifact-management),\nwhich is why the three deferred writes (see below) could not\nbe filed this session.\n\n**Release flow actually executed:**\nPhase 1 (Linux + docs, from inside the devcontainer):\npreflight OK \u2192 cargo fmt check OK (after `5db8a34`) \u2192\n617/617 tests pass \u2192 cargo audit clean \u2192 built aarch64 +\nx86_64 binaries \u2192 `aibox --version` = 0.18.7 \u2192 tag pushed \u2192\nGitHub release created with both binaries \u2192 docs built by\nDocusaurus \u2192 gh-pages force-pushed \u2192 GitHub Pages\nauto-config correctly skipped (no spurious warning \u2014\nv0.18.7 probe-first worked).\nPhase 2 (macOS + GHCR, from host):\n`./scripts/maintain.sh release-host 0.18.7` \u2014 confirmed\ndone by owner.\n\n**Addon definitions workaround applied:**\n`aibox sync` in the devcontainer failed on first run with\n\"Addon definitions not found at\n/home/aibox/.config/aibox/addons\"\
      . Worked around by\n`cp -r /workspace/addons /home/aibox/.config/aibox/addons`.\nSame dev-env limitation noted in the prior handover. Worth\nfiling as a real issue: `aibox init` (host) and `aibox sync`\n(container) should find addon definitions without requiring\nthe install script \u2014 or the devcontainer image should\ninclude them at a predictable path. Low priority but aged.\n"
    issues_resolved:
    - "processkit v0.18.2 pulled and integrated \u2014 resolves processkit#8 upstream-side; aibox`s safety rail in v0.18.7 ensures any future regression of the same class is caught at sync time instead of silently emitting broken config."
    - "`.mcp.json` regenerated with all 16 server paths valid on disk. The drift diagnosed in the prior session (`/workspace/.mcp.json` had 12 broken paths despite per-skill `mcp-config.json` hotpatch) is resolved by the upstream fix \u2014 no more local hotpatch needed going forward."
    - "cargo fmt drift in `cli/src/mcp_registration.rs` test \u2014 caught by release preflight, fixed in `5db8a34`. Not a blocker but a reminder that pre-commit `cargo fmt` is an unwritten norm here."
    - "Carried session artifacts committed to git \u2014 the accumulation of handover logs, migration docs, and template snapshots that had been untracked across multiple sessions (because the compliance gate was shut) are now properly tracked. Matches the prior pattern of tracked siblings (context/logs/, context/migrations/applied/, context/templates/aibox-home/0.17.20/)."
    - 'v0.18.7 released end-to-end: Linux Phase 1 and macOS Phase 2 both complete; GitHub release live; docs deployed; Pages auto-config warning no longer emitted.'
    issues_remaining:
    - "Three MCP writes still deferred, one more than inherited \u2014 CarefulFalcon DecisionRecord (unchanged from prior), MIG-20260418T111856 apply (unchanged), MIG-20260418T195315 apply (NEW \u2014 the processkit v0.18.1\u2192v0.18.2 migration generated this session; 18 conflicting files recorded in `context/migrations/pending/`), and either a release.prepared or release.shipped event log for tag v0.18.7 / commit 5db8a34. All blocked on Claude Code harness re-handshaking `.mcp.json` (needs a restart)."
    - "MIG-RUNTIME-20260418T175825 runtime migration doc also pending in `context/migrations/pending/` \u2014 origin unclear (possibly from the prior devcontainer rebuild). Worth an `apply_migration` check in the next session."
    - "Addon-definitions-not-found on sync inside the devcontainer \u2014 had to manually `cp /workspace/addons \u2192 /home/aibox/.config/aibox/addons`. Worth a real fix (bake into image, or fall back to the in-repo copy), but not pressing."
    - "Compliance contract drift between AGENTS.md and canonical at `context/skills/processkit/skill-gate/assets/compliance-contract.md` \u2014 sync warned and suggested `aibox sync --fix-compliance-contract`, but that flag is not implemented yet (only referenced in the warning message). Prospective feature for a future release."
    - 'Decision-language capture: the owner said "Option B" / "Let''s go" this session. Per the v2 compliance contract, those moments should either produce a `record_decision` or a `skip_decision_record(reason=...)`. Neither tool was loaded (decision-record MCP server offline; `skip_decision_record` not yet shipped upstream). The sequencing decision ("release before MCP writes") is operational rather than cross-cutting; likely skippable, but worth surfacing to the next session for a formal acknowledgement.'
    - "BACK-20260411_0000-SoundRabbit (critical, self-hosted deployment) \u2014 unchanged across many sessions. Still needs a grooming session."
    - "BACK-AmberWren, BACK-CoolBear, BACK-JollyWren, BACK-LoyalSeal \u2014 4 high-priority backlog items, still unstarted."
    - "Yazi Tier-3 interactive verification \u2014 untouched this session; still waiting on a host-side run with DISPLAY/TTY."
    open_threads:
    - "AFTER SESSION RESTART (first 5 minutes): verify that all 16 processkit MCP tool surfaces are reachable. Start with `check_contract_acknowledged` \u2192 `acknowledge_contract(version=\"v2\")` (contract is now v2 in-session as of this session). Then `query_entities(kind=\"LogEntry\", event_type=\"session.handover\", limit=1)` as a smoke test that index-management and event-log are live."
    - "AFTER RESTART \u2014 file the deferred entities in one pass: (a) `record_decision` for CarefulFalcon (warn-and-continue + bare-name keys; close `BACK-20260418_1145-CarefulFalcon`); (b) `apply_migration(\"MIG-20260418T111856\")`; (c) `apply_migration(\"MIG-20260418T195315\")` \u2014 this one is substantive (18 conflicts from the processkit v0.18.1\u2192v0.18.2 delta); (d) `apply_migration(\"MIG-RUNTIME-20260418T175825\")` if applicable; (e) `create_log_entry(event_type=\"release.shipped\", ...)` for tag v0.18.7 / commit 5db8a34 (release is already live, so \"prepared\" is past tense \u2014 \"shipped\" is accurate)."
    - "AFTER RESTART \u2014 retro the decision-language moments: `record_decision` or `skip_decision_record(reason=\"operational sequencing, not cross-cutting\")` for the \"release before MCP writes\" choice the owner made this session (\"Option B\")."
    - "MIG-20260418T195315 is not trivial \u2014 18 conflicts across AGENTS and skills/processkit. The migration doc at `context/migrations/pending/MIG-20260418T195315.md` lists them. Sync decided to force-apply 5 of them (skill-gate server/hooks + a few others \u2014 those are in `b010680`); the remaining 13 are conflicts that require a human decide-keep-local-or-take-upstream pass. Apply_migration will walk the conflict list interactively or auto-resolve per configured policy."
    - "Compliance contract v1 \u2192 v2 transition is complete enough for this session (the contract text in SessionStart/UserPromptSubmit hooks shifted to v2 mid-session and the v0.18.7 aibox drift checker tolerates both). But the canonical AGENTS.md at project root still differs from the canonical source \u2014 see `aibox sync --fix-compliance-contract` warning above. When the flag is actually implemented upstream, run it."
    - "Addon-definitions-not-found workaround is not committed anywhere \u2014 lives in `/home/aibox/.config/aibox/addons/` which is outside the repo. If the devcontainer is rebuilt again without an updated image that bakes it in, the same workaround is needed. File a proper fix if the pattern recurs."
    - BACK-20260411_0000-SoundRabbit still needs a grooming session; 4 other high-priority backlog items (AmberWren, CoolBear, JollyWren, LoyalSeal) still unstarted.
    next_recommended_action: "**Restart Claude Code first, before anything else.** The\nin-session MCP tool surface is still the 4 pre-restart\nservers; the 16-server config on disk is correct and will\nload on the next handshake.\n\nThen in the opening minutes of the next session, in this\norder:\n\n1. Smoke-test MCP: `check_contract_acknowledged`,\n   `acknowledge_contract(\"v2\")`, `query_entities(kind=LogEntry, limit=1)`.\n   If all 16 servers are up, proceed; if any are still\n   missing, diagnose before writing anything.\n\n2. File the deferred entities as a single opening pass:\n   - `record_decision` CarefulFalcon \u2192 close\n     BACK-20260418_1145-CarefulFalcon.\n   - `apply_migration(\"MIG-20260418T111856\")` (trivial \u2014\n     AGENTS.md v1\u2192v2 marker; drift checker already tolerates\n     both).\n   - `apply_migration(\"MIG-20260418T195315\")` (18 conflicts\n     from processkit v0.18.1\u2192v0.18.2; walk them).\n   - `apply_migration(\"MIG-RUNTIME-20260418T175825\"\
      )` if\n     index-management still lists it as pending.\n   - `create_log_entry(event_type=\"release.shipped\", ...)`\n     for commit `5db8a34` / tag `v0.18.7`.\n   - Either `record_decision` or `skip_decision_record` for\n     the owner's \"release before MCP writes\" call this\n     session.\n\n3. Once the context is compliance-clean, pick from the\n   backlog. BACK-SoundRabbit is the most overdue; the 4\n   high-priority reviews (AmberWren / CoolBear / JollyWren\n   / LoyalSeal) are next tier.\n"
    branch: main
    commit: 5db8a34
    tag: v0.18.7
    uncommitted_changes: []
    stashes: []
    releases:
    - 'v0.18.7: shipped this session. Phase 1 (Linux binaries, GitHub release, docs deploy) completed by claude-opus-4-7 inside the devcontainer. Phase 2 (macOS binaries + GHCR push) confirmed done by owner. GitHub release live: https://github.com/projectious-work/aibox/releases/tag/v0.18.7'
    behavioral_retrospective:
    - "Committed work before running `cargo fmt` \u2192 release preflight caught the drift. Cost one extra commit (`5db8a34`). Encoded for next session: when preparing to ship, run `cd cli && cargo fmt --check` BEFORE the first integration commit, not as a mid-release patch. Not a process gap \u2014 the safety net worked as designed \u2014 but an avoidable retry."
    - "Delegated pre-integration research to an Explore agent (processkit_vocab.rs default, compat.rs entry, aibox.toml/lock state, release script phases, sync semantics). Under 300 words, structured report, one turn, zero main-thread context spent on filesystem walking. This is the pattern the prior handover encoded (\"agents are great for research, diff drafting, and read-only investigation; main thread does the actual edits\") \u2014 confirmed useful in practice this session."
    - "Distinguished \"edit a file under `context/templates/`\" (forbidden by contract) from \"commit an already-present file under `context/templates/`\" (correct behavior \u2014 those template snapshots are meant to be tracked; siblings like the v0.17.20 snapshot are already in git). The contract text reads as a blanket prohibition; the nuance is that the snapshots are WRITTEN by aibox sync automatically and should simply be committed without hand editing. Worth surfacing upstream if the contract wording drifts this rule."
    - "Decision-language compliance gap \u2014 the v2 contract introduced a new rule: when user messages contain approval language (\"ok\", \"yes\", \"Option B\", \"let's go\"), either `record_decision` or `skip_decision_record` in the same turn. Both tools were unavailable this session (server offline / tool not yet shipped upstream). Transparent in-text acknowledgement is the honest fallback, and next session can retroactively file or skip. Encoded as an open_thread so it doesn't get forgotten."
    - "Addon-definitions workaround (`cp -r /workspace/addons \u2192 /home/aibox/.config/aibox/addons`) is identical to a workaround pattern noted in the prior handover. Two sessions in a row with the same manual step; worth filing as a real aibox-side issue so the third session doesn't repeat."
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
