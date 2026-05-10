---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260426_1636-MightySky-codex-slash-commands-shipped
  created: '2026-04-26T16:36:26+00:00'
spec:
  title: Codex slash commands shipped as Codex Skills, not legacy ~/.codex/prompts/
    files
  state: accepted
  decision: |
    For the Codex harness, aibox will scaffold each `/pk-*` workflow as a **Codex Skill** at `.agents/skills/<name>/SKILL.md` rather than as a legacy `~/.codex/prompts/<name>.md` prompt file. The current Codex profile in `cli/src/harness_commands.rs` (which writes to `.aibox-home/.codex/prompts/`) is wrong-target and must be replaced.

    Invocation surface in Codex CLI 0.125.0 is `$pk-resume` (dollar-sign prefix) plus `/skills` for the picker; the slash-prefixed `/pk-resume` form only appears in the Codex App composer when a skill is enabled there. Aibox documentation must reflect that asymmetry — `/pk-resume` works in Claude Code, `$pk-resume` works in Codex CLI.
  context: |
    aibox v0.21.1 commit 5204969 ("feat(harness): scaffold processkit slash commands beyond Claude Code") generalized the per-harness scaffolding engine and added a Codex profile that writes Claude-Code-style markdown prompt files (front-matter `argument-hint`, `allowed-tools`) to `~/.codex/prompts/<name>.md` (mounted as `.aibox-home/.codex/prompts/`). Files are present in the running container, but `/pk-resume` does not surface in Codex CLI 0.125.0.

    Authoritative answer from Codex itself (relayed by user, 2026-04-26): Codex CLI does not register custom slash commands through `~/.codex/prompts/` or any analogous path. The supported customization mechanism is Codex Skills, documented at https://developers.openai.com/codex/skills, with skills laid out as `<workspace>/.agents/skills/<name>/SKILL.md`. CLI slash commands at https://developers.openai.com/codex/cli/slash-commands are exclusively built-ins. The Codex App separately surfaces enabled skills in its slash-command list (https://developers.openai.com/codex/app/commands), which is why prior generations of this scaffolder appeared to "work" in some surfaces.
  rationale: |
    Three reasons we're picking Codex Skills over alternatives:

    1. **Authoritatively supported.** Codex docs name `.agents/skills/<name>/SKILL.md` as THE customization path; it's not a hack or undocumented surface. The legacy prompts-directory we were targeting is unsupported (and silently ignored) in 0.125.0.
    2. **Format proximity.** Codex Skills use YAML front-matter with `name` + `description` and a markdown body — this is structurally compatible with Claude Code's command files and processkit's existing SKILL.md, so the scaffolder change is mostly a path/wrapper change rather than a content rewrite.
    3. **Single source of truth path forward.** processkit already authors SKILL.md files for every skill. Once we ship pk-* as Codex Skills, we've established the precedent that Codex sees the same skill semantics as Claude — easier to keep parity going forward, easier to add new harnesses (Cursor's `.cursor/skills/`, etc.) by analogy.
  alternatives:
  - option: Keep the ~/.codex/prompts/ scaffolding and lobby OpenAI to re-enable it
    reason_rejected: Unsupported surface, no upstream documentation, no leverage.
      Even if OpenAI restored it, every aibox release between now and then ships broken
      Codex commands.
  - option: Tell users to type built-in /skills and pick manually
    reason_rejected: Defeats the whole purpose of the per-harness scaffolder, which
      is to make the same /pk-resume invocation work everywhere with zero per-harness
      friction.
  - option: Generate a single Codex skill that dispatches to all pk-* commands by
      argument
    reason_rejected: Loses tab-completion and discoverability — operator has to remember
      the dispatcher syntax. The scaffolder generating one skill per command preserves
      the pk-doctor / pk-resume / pk-work mental model.
  - option: Ship Codex Skills in addition to the legacy prompts/ files
    reason_rejected: Belt-and-braces would be reasonable if the legacy path were still
      partially functional, but it isn't — we'd just be writing dead files. Cleaner
      to remove the legacy emit entirely.
  consequences: |
    ## Implementation impact

    - `cli/src/harness_commands.rs::profile_for(AiHarness::Codex, ...)` must change: target dir becomes `.agents/skills/`, layout becomes one subdirectory per command, file is `SKILL.md`, front-matter is reduced to `name` + `description` (drop `argument-hint`, `allowed-tools` — those are Claude conventions).
    - `cli/src/sync_perimeter.rs::SYNC_PERIMETER` must add `.agents/skills/` and may need to remove `.aibox-home/.codex/prompts/` (TBD: whether to leave it for backwards-compat with users still on older codex).
    - `cli/src/doctor.rs::check_command_registrations` per-Codex check must validate the new path.
    - A migration step in `aibox sync` (or doctor remediation) should remove orphaned `.aibox-home/.codex/prompts/pk-*.md` files left over from v0.21.1.
    - All existing Codex-profile unit tests in `cli/src/harness_commands.rs` (`codex_profile_writes_md_to_aibox_home_codex_prompts`, `codex_profile_skipped_when_not_enabled`) need rewriting to assert against the new layout.
    - AGENTS.md / docs note the invocation asymmetry: `/pk-resume` in Claude Code, `$pk-resume` (or `/skills` picker) in Codex CLI, `/pk-resume` in Codex App.

    ## Release impact

    - Ships in the next aibox patch release (v0.21.2) — fixes a bug landed in v0.21.1.
    - No processkit upstream change required: SKILL.md format already aligns; aibox does the wrapping. (No upstream issue to file.)
    - Anyone who upgraded to v0.21.1 inside a Codex-enabled project has stale `pk-*.md` files in `~/.codex/prompts/` that do nothing. Sync in v0.21.2 should clean them up.

    ## Future-proofing

    - This decision establishes Codex Skills as the canonical Codex integration surface. If OpenAI later adds a separate slash-command registry, we revisit; until then, all per-harness Codex work goes through `.agents/skills/`.
  related_workitems:
  - BACK-20260426_1627-StrongHawk-codex-cli-0-125
  decided_at: '2026-04-26T16:36:26+00:00'
---
