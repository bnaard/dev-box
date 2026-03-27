# Skill Customization Design — Organizational Learning via Skills

**Date:** 2026-03-26
**Task:** BACK-051
**Status:** Draft

---

## 1. Problem Statement

aibox ships 84 curated skills that encode generic best practices (e.g.,
`rust-conventions` teaches `anyhow::Result` for app code, `thiserror` for
libraries). These are valuable starting points, but real teams accumulate
specific conventions that are equally important:

- **Library choices:** "prefer clap over structopt for CLI args"
- **Error handling patterns:** "always wrap errors in our `AppError` enum"
- **Naming conventions:** "REST endpoints use plural nouns, database tables use singular"
- **Architectural rules:** "no direct database access from HTTP handlers — always go through a service layer"
- **CI/CD patterns:** "all PRs must have a `Test plan` section"

Today, teams repeat these rules every session (in prompts or CLAUDE.md) or
hope the agent infers them from code. Neither scales. Conventions should
accumulate over time, attach to the relevant skill, and persist across
sessions, developers, and machines.

**Core tension:** Skills must remain curated and updatable (aibox owns the
base content), while organizations and projects must be able to extend them
with their own rules without forking.

---

## 2. Design Options

### Option A: Skill Overlays (per-project files)

A `CUSTOM.md` file alongside the curated `SKILL.md`, concatenated at
deployment time.

**File structure:**
```
.claude/skills/rust-conventions/
  SKILL.md          # curated (owned by aibox, overwritten on sync)
  CUSTOM.md         # project-specific (owned by team, never overwritten)
  references/       # curated reference files
```

**CLI commands:**
```bash
aibox skill customize rust-conventions   # opens $EDITOR on CUSTOM.md
aibox skill customize rust-conventions --show  # prints current overlay
```

**Deployment flow (`aibox sync`):**
1. Deploy curated `SKILL.md` (overwrite if content-hash changed).
2. If `CUSTOM.md` exists, append its content to `SKILL.md` under a
   `## Project Conventions` heading.
3. Alternatively, keep them as separate files — Claude Code reads all
   `.md` files in a skill directory, so `CUSTOM.md` is automatically
   included in the skill's context.

**Pros:**
- Simple implementation — one new CLI command, minor sync change.
- Git-committable — team conventions travel with the repo.
- No new infrastructure — works with existing file-based skill system.
- Non-destructive — `CUSTOM.md` is never touched by `aibox sync`.

**Cons:**
- Per-project only — no org-wide sharing without copy-paste.
- No structure enforcement — `CUSTOM.md` is freeform markdown.
- Duplication across repos in the same organization.

### Option B: Layered Skill System (org + project)

Multiple skill sources with merge priority: `curated < org < project`.

**Config (`aibox.toml`):**
```toml
[skills]
org_source = "git@github.com:myorg/aibox-skills.git"
# or: org_source = "https://github.com/myorg/aibox-skills.git"
# or: org_source = "/path/to/local/skills"  # for monorepos
```

**Org skill repo structure:**
```
myorg-aibox-skills/
  rust-conventions/
    CUSTOM.md       # org-wide Rust conventions
  python-best-practices/
    CUSTOM.md       # org-wide Python conventions
  our-api-style/
    SKILL.md        # entirely new org skill (not a curated overlay)
```

**Resolution order:**
1. Curated skill base (`templates/skills/`)
2. Org-level overlay (`org_source` repo, `<skill-name>/CUSTOM.md`)
3. Project-level overlay (`.claude/skills/<skill-name>/CUSTOM.md`)

**Sync flow:**
1. Clone/pull org skill repo to a cache directory (`~/.aibox/org-skills/`
   or `.aibox-home/org-skills/`).
2. For each active skill: merge curated + org overlay + project overlay.
3. Org-only skills (no curated base) are deployed as standalone skills.

**Pros:**
- Org-wide conventions defined once, inherited by all projects.
- Supports entirely new org-specific skills.
- Git-based distribution — familiar, auditable, versioned.
- Project overlays can still override org conventions.

**Cons:**
- Significantly more complex — git clone/pull, cache management, merge logic.
- Needs authentication handling for private org repos.
- Version pinning question — should orgs pin to a tag/branch?
- New concept for users to learn (three-layer merge).

### Option C: CLAUDE.md Integration (no new system)

Conventions go in `CLAUDE.md`, skills stay generic and read-only.

**Usage:**
```markdown
# CLAUDE.md

## Rust Conventions
- Prefer clap over structopt for CLI argument parsing
- Always wrap errors in our AppError enum defined in src/error.rs
- Use tracing instead of log for all logging
```

**Pros:**
- Zero implementation cost — already works today.
- Single source of truth for project-specific rules.
- Familiar to Claude Code users.

**Cons:**
- Does not scale — CLAUDE.md becomes a dumping ground for everything.
- No structure — conventions are not linked to the skill they extend.
- Cannot share across projects without copy-paste.
- No tooling — no `aibox skill learn`, no validation, no diffing.
- Conflates freeform instructions with structured conventions.

### Option D: `aibox skill learn` Command (interactive append)

A quick-fire command to append rules to an overlay file.

**CLI:**
```bash
aibox skill learn rust-conventions "prefer clap over structopt"
aibox skill learn rust-conventions "use tracing, not log"
aibox skill learn python-best-practices "always use pydantic for data models"
```

**Storage:** Appends to `.claude/skills/<name>/CUSTOM.md` as bullet points
under a `## Learned Rules` heading:

```markdown
## Learned Rules

- prefer clap over structopt
- use tracing, not log
```

**Additional commands:**
```bash
aibox skill learned rust-conventions     # list current rules
aibox skill unlearn rust-conventions 1   # remove rule by index
```

**Pros:**
- Lowest friction — one command to capture a convention mid-session.
- Git-committable — same file as Option A.
- Complements Option A (structured editor) with a quick-fire alternative.

**Cons:**
- Per-project unless combined with Option B.
- Rules are terse — may lack context for why a convention exists.
- Index-based deletion is fragile.

---

## 3. Interaction with Existing Systems

### Skills vs. CLAUDE.md

**Complementary, not competing.** Skills are the structured channel —
categorized, named, deployed per skill domain. CLAUDE.md is the freeform
channel — project-wide rules, workflow instructions, meta-instructions.

A convention like "prefer clap over structopt" belongs in the
`rust-conventions` skill overlay because it is scoped to Rust work. A
rule like "always run tests before committing" belongs in CLAUDE.md because
it is cross-cutting.

Guideline for users: if a convention is domain-specific (language, framework,
infrastructure), put it in a skill overlay. If it is project-wide workflow,
put it in CLAUDE.md.

### Skills vs. Memory System

The memory system (Claude Code's `~/.claude/` memory, session history) is
**per-user and per-session**. Skill customizations are **per-project and
per-team**. They serve different purposes:

- Memory: "the user prefers concise responses" (personal preference)
- Skill overlay: "this project uses clap for CLI args" (team convention)

No conflict — memory is implicit and personal; skill overlays are explicit
and shared.

### Skills vs. BACK-007 (Plugin System)

The plugin system (BACK-007) is about extending aibox's CLI functionality —
new commands, hooks, template overrides. Skill customization is about
extending skill content. They are orthogonal but can compose:

- A plugin could provide an entirely new skill (already partially covered
  by BACK-024, external skill installation).
- A plugin could provide org-level skill overlays (this is essentially
  Option B implemented as a plugin).
- The `aibox skill learn` command (Option D) is a CLI feature, not a plugin.

**Recommendation:** Implement skill customization as a core feature (Options
A+D), not as a plugin. Org-level distribution (Option B) could later be
implemented as a plugin or core feature.

### Skills vs. BACK-024 (External Skill Installation)

BACK-024 covers installing entirely new skills from external sources.
BACK-051 covers augmenting existing curated skills with custom rules.
They are complementary:

- BACK-024: `aibox skill install https://github.com/someone/cool-skill`
- BACK-051: `aibox skill customize rust-conventions`

Both produce files in `.claude/skills/` but with different ownership models.

---

## 4. Recommendation

**Phase 1 (implement now): Options A + D combined.**

This gives teams two ways to customize skills:

1. `aibox skill customize <name>` — opens `CUSTOM.md` in `$EDITOR` for
   structured, multi-paragraph conventions with rationale.
2. `aibox skill learn <name> "<rule>"` — quick one-liner append for
   capturing conventions mid-session.

Both write to the same file (`.claude/skills/<name>/CUSTOM.md`), which is
git-committed and never overwritten by `aibox sync`. This covers the core
use case with minimal implementation effort.

**Phase 2 (implement later, if demand exists): Option B.**

Org-level skill distribution adds significant complexity (git operations,
caching, auth, merge logic). Defer until multiple organizations request it.
When the time comes, it can build on Phase 1's `CUSTOM.md` convention —
the org repo just provides `CUSTOM.md` files that are merged before
project-level ones.

**Do not implement Option C as the solution.** CLAUDE.md already works for
freeform rules; the point of BACK-051 is to provide something more
structured. However, documentation should guide users on when to use
CLAUDE.md vs. skill overlays.

---

## 5. Implementation Sketch (Phase 1)

### 5.1 File Convention

Skill directories gain an optional `CUSTOM.md`:

```
.claude/skills/rust-conventions/
  SKILL.md        # curated, owned by aibox (content-hash updated on sync)
  CUSTOM.md       # project-owned, never overwritten
  references/     # curated references
```

Claude Code already reads all `.md` files in a skill directory when the
skill triggers. No concatenation is needed — `CUSTOM.md` as a separate
file is automatically included in the skill's context window. This is
the simplest approach and avoids any merge logic in `aibox sync`.

### 5.2 CLI Commands

**New subcommands under `aibox skill`:**

```
aibox skill customize <name>           # open CUSTOM.md in $EDITOR
aibox skill customize <name> --show    # print current CUSTOM.md to stdout
aibox skill learn <name> "<rule>"      # append a rule to CUSTOM.md
aibox skill learned <name>             # list rules in CUSTOM.md
aibox skill unlearn <name> "<rule>"    # remove a rule (exact match)
```

**`aibox skill customize`** implementation:
1. Verify `<name>` is a known skill and is in the active set.
2. Create `.claude/skills/<name>/CUSTOM.md` if it does not exist, with a
   template header:
   ```markdown
   # Project Conventions — rust-conventions

   > Custom rules for this project. This file is never overwritten by
   > aibox sync. Commit it to version control to share with your team.

   ## Rules

   <!-- Add your project-specific conventions below -->
   ```
3. Open in `$EDITOR` (fall back to `vi`).

**`aibox skill learn`** implementation:
1. Verify `<name>` is a known skill.
2. Create `CUSTOM.md` with template if it does not exist.
3. Append `- <rule>` under the `## Rules` heading.
4. Print confirmation: `Learned: "prefer clap over structopt" (rust-conventions)`

**`aibox skill learned`** implementation:
1. Read `CUSTOM.md`, extract lines starting with `- ` under `## Rules`.
2. Print numbered list.

**`aibox skill unlearn`** implementation:
1. Read `CUSTOM.md`, find and remove the matching rule line.
2. Print confirmation.

### 5.3 Changes to `aibox sync`

The `reconcile_skills` function in `context.rs` currently deploys
`SKILL.md` and `references/` using `write_if_missing`. Changes needed:

1. **Preserve `CUSTOM.md`:** The existing logic already does not touch
   unknown files in skill directories. Verify this — `reconcile_skills`
   only writes `SKILL.md` and reference files, and only if missing. No
   change needed for preservation.

2. **Content-hash updates for curated content:** When the curated
   `SKILL.md` changes between aibox versions, the current `write_if_missing`
   approach will NOT update it. This is an existing limitation (not
   introduced by this feature). A future change should compare content
   hashes and update curated files when the base changes — but that is
   separate from BACK-051.

3. **Report customized skills:** During sync output, note which skills
   have a `CUSTOM.md` overlay:
   ```
   [ok] Deployed 3 missing skills
   [info] 2 skills have project customizations (rust-conventions, git-workflow)
   ```

### 5.4 Changes to `aibox skill info`

When displaying skill info, also show whether a `CUSTOM.md` exists and
its rule count:

```
Skill: rust-conventions
Package: development
Customized: yes (3 project rules)

  ---
  name: rust-conventions
  description: Rust patterns and conventions...
  ---
  ...
```

### 5.5 Changes to `aibox skill list`

Add a marker column or symbol for customized skills:

```
  SKILL                    PACKAGE        STATUS
  rust-conventions    *    development    active
  python-best-practices    development    active
```

Where `*` indicates a `CUSTOM.md` exists.

### 5.6 Rust Implementation Outline

**New file: `cli/src/skill_customize.rs`** (or extend `skill_cmd.rs`)

```rust
// Key functions:

/// Path to CUSTOM.md for a skill
fn custom_md_path(skill_name: &str) -> PathBuf {
    Path::new(".claude/skills").join(skill_name).join("CUSTOM.md")
}

/// Create CUSTOM.md with template header if it doesn't exist
fn ensure_custom_md(skill_name: &str) -> Result<PathBuf>

/// Open CUSTOM.md in $EDITOR
pub fn cmd_skill_customize(name: &str, show: bool) -> Result<()>

/// Append a rule to CUSTOM.md
pub fn cmd_skill_learn(name: &str, rule: &str) -> Result<()>

/// List rules from CUSTOM.md
pub fn cmd_skill_learned(name: &str) -> Result<()>

/// Remove a rule from CUSTOM.md
pub fn cmd_skill_unlearn(name: &str, rule: &str) -> Result<()>

/// Check if a skill has customizations
pub fn has_custom_overlay(skill_name: &str) -> bool
```

**CLI registration in `cli.rs`:**

```rust
#[derive(Subcommand)]
enum SkillCommand {
    List,
    Add { name: String },
    Remove { name: String },
    Info { name: String },
    Customize {
        name: String,
        #[arg(long)]
        show: bool,
    },
    Learn {
        name: String,
        rule: String,
    },
    Learned {
        name: String,
    },
    Unlearn {
        name: String,
        rule: String,
    },
}
```

### 5.7 Testing Plan

1. **Unit tests:**
   - `ensure_custom_md` creates file with correct template.
   - `cmd_skill_learn` appends rules correctly.
   - `cmd_skill_unlearn` removes the correct rule.
   - `cmd_skill_learned` parses rules from `CUSTOM.md`.
   - `has_custom_overlay` returns correct boolean.

2. **Integration tests:**
   - Full cycle: learn a rule, verify it appears in `learned`, unlearn it.
   - `aibox sync` preserves `CUSTOM.md` when updating curated content.
   - `aibox skill info` shows customization status.

3. **Edge cases:**
   - Learn a rule for a skill that is not in the active set (should warn).
   - Learn a duplicate rule (should warn, not duplicate).
   - Unlearn a rule that does not exist (should warn).
   - `CUSTOM.md` with manual edits (non-bullet content) — `learned` should
     only list bullet items under `## Rules`.

---

## 6. Future Considerations

### Phase 2: Org-Level Distribution

When implementing Option B, the design should:
- Use the same `CUSTOM.md` file convention (org repo provides `CUSTOM.md`
  files per skill).
- Cache org skills in `~/.aibox/cache/org-skills/` with a TTL or manual
  `aibox skill sync-org` command.
- Support both SSH and HTTPS git URLs.
- Support local paths for monorepo setups.
- Merge order: curated `SKILL.md` + org `CUSTOM.md` + project `CUSTOM.md`
  (or keep all three as separate files in the skill directory — Claude Code
  reads them all).

### Skill Versioning Interaction (BACK-069)

When skills gain version metadata (BACK-069), `CUSTOM.md` files should
not have versions — they are always current. If a curated skill's major
version changes (e.g., restructured conventions), `aibox sync` should warn
that the project's `CUSTOM.md` may need review.

### Plugin System Interaction (BACK-007)

A plugin could register as a skill source, providing both new skills and
overlays for existing skills. This would be a natural extension of Option B
where the "org repo" is replaced by a plugin registry.
