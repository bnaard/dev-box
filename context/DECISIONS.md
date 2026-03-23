# Decisions Log

Inverse chronological. Each decision has a rationale and alternatives considered.

## DEC-016 — Declarative config + minimal base images (2026-03-23)

**Decision:** Redesign aibox around a single published base image (base-debian), unified add-on system with per-tool version selection, 13 composable process packages replacing 4 monolithic levels, and declarative skill management. No backward compatibility — clean break.

**Rationale:** The 10 pre-compiled image architecture creates maintenance burden (TeX Live duplicated 3x across 3 Dockerfiles), limits composability (can't combine Node+Go without a dedicated image), and gives users no control over which skills they deploy. The 4 monolithic process levels (minimal/managed/research/product) don't fit non-software projects (document, research, data). Moving everything to add-ons + composable process packages gives users full control while reducing our maintenance surface from 10 images to 1.

**Key decisions within:**
- Abstract base contract (Debian now, Alpine later) — not tied to specific distro
- LaTeX becomes an add-on with multi-stage builder (no dedicated base-latex image)
- Add-ons have internal recipe versioning; users select per-tool versions from curated lists
- 13 atomic process packages + 4 convenience presets, freely composable
- Core package (always present): agent-management + owner-profile skills, AIBOX.md + OWNER.md
- Content-addressed skill updates on sync
- AI providers: Claude, Aider, Gemini, Mistral (bring-your-own-model deferred)

**Alternatives:** Keep base-latex image for build speed (rejected — Docker layer caching + future GHCR cache image sufficient), keep 4 monolithic processes (rejected — too rigid for non-software projects), maintain backward compat (rejected — too few users, baggage not worth carrying).

## DEC-015 — Dogfood the product process template (2026-03-23)

**Decision:** Align aibox's own `context/` with the product process template it ships to users. Adopt BACK-NNN IDs in BACKLOG.md, add PROJECTS.md and PRD.md, install 8 product-relevant skills in `.claude/skills/`, close 13 completed GitHub issues, and update the public roadmap.

**Rationale:** aibox promotes structured work processes but wasn't fully following its own product template. Eating our own dogfood validates the template and reveals friction. The existing context/ was close but used a different backlog format (checkboxes vs BACK-NNN table) and lacked structured project tracking. GitHub had 16 open issues, 13 of which were already done — creating a false impression of outstanding work.

**Deviations from template:** STANDUPS.md omitted — session handovers in `project-notes/session-*.md` are more detailed and serve the same purpose. OWNER.md kept (not in product template but useful). Extra work-instructions kept (DOCKERFILE-PRACTICES.md, SCREENCASTS.md) as project-specific extensions. `backlog-context` skill customized for table format with BACK-NNN IDs.

**Alternatives:** Full template adoption including STANDUPS.md (redundant with session handovers), keep current format (misses dogfooding opportunity), automated migration tool (over-engineering for a one-time task).

## DEC-014 — Skills Library: curated quality over marketplace quantity (2026-03-22)

**Decision:** Ship 83 curated skills with reference files rather than providing a marketplace integration or a smaller "starter" set. Skills are embedded in the binary via `include_str!` and scaffolded on `aibox init`. No external download step.

**Rationale:** Marketplace research (SkillsMP: 97K skills, Skills.sh: 40K, ClawHub: 13.7K) revealed that 46.3% of publicly available skills are duplicates or near-duplicates (HuggingFace analysis). The ecosystem's #1 problem is quality, not quantity. A curated library with progressive disclosure (SKILL.md < 150 lines, reference files for depth) differentiates aibox from "skill slop." Embedding in the binary ensures skills work offline and are version-locked to the CLI.

**Categories chosen (14):** Process, Development, Language, Infrastructure, Architecture, Design & Visual, Data & Analytics, AI & ML, API & Integration, Security, Observability, Database, Performance, Framework & SEO. Based on marketplace demand analysis: infrastructure and data skills are vastly underserved relative to frontend/framework skills.

**Alternatives:** Marketplace-first (ClawHub/Skills.sh integration — deferred to backlog), smaller starter set with `aibox skill install` (adds complexity, network dependency), external file download during init (fragile, no offline support).

## DEC-013 — Granular vim mounts preserve image colorschemes (2026-03-22)

**Decision:** The docker-compose template mounts `.vim/vimrc` and `.vim/undo` individually instead of the entire `.vim/` directory. This preserves the image-baked `~/.vim/colors/` and `~/.vim/pack/` directories.

**Rationale:** The base image downloads 6 vim colorscheme files (gruvbox, catppuccin-mocha, catppuccin-latte, tokyonight, nord, dracula) into `/root/.vim/colors/` during Docker build. When the entire `.vim/` was mounted from the host (`.aibox-home/.vim/`), the image's `colors/` directory was shadowed. Result: `E185: Cannot find color scheme 'gruvbox'` in derived projects. Mounting only the two files we actually persist (vimrc and undo/) leaves the image's baked-in directories visible.

**Alternatives:** Copy colorschemes into `.aibox-home/.vim/colors/` during seed (duplicates files, maintenance burden), embed colorschemes via `include_str!` in the binary (bloat, version drift), post-create command to copy (fragile).

## DEC-012 — Reference file scaffolding via SkillDef type (2026-03-22)

**Decision:** Extend `scaffold_skills()` to deploy reference files alongside SKILL.md. Changed the skills data structure from `(&str, &str)` to a `SkillDef` type alias: `(&str, &str, &[(&str, &str)])` = `(name, content, [(ref_filename, ref_content)])`. Reference files go in `.claude/skills/<name>/references/`.

**Rationale:** 8 of the original 26 skills had reference files on disk (11 files total) but they were never deployed to derived projects — `scaffold_skills()` only wrote SKILL.md. With the expansion to 83 skills and 57 reference files, fixing this was a prerequisite. The `SkillDef` type alias satisfies clippy's type_complexity lint while keeping the flat `include_str!` embedding pattern.

**Alternatives:** Struct-based SkillDef (heavier, overkill for static data), dynamic file discovery at runtime (fragile, no compile-time guarantees), separate scaffolding function for references (unnecessary split).

## DEC-011 — Skills + Processes architecture: separate WHAT from HOW (2026-03-22)

**Decision:** Process declarations in context/ define WHAT processes exist ("there shall be backlog management"). Skills (SKILL.md standard) define HOW they're executed. Context stores the resulting artifacts (BACKLOG.md, DECISIONS.md, etc.). Skills come in flavors (e.g., backlog-context vs backlog-github) that users choose.

**Rationale:** Today's process presets bake both "what" and "how" into context template files. This makes them rigid — you can't swap from a context-file backlog to GitHub Issues without restructuring. By separating concerns: process declarations become thin ("there shall be X"), skills become the executable implementation, and artifacts remain in context/. This enables: swappable implementations, testable skills (via SKILL.md eval framework), thinner aibox scaffolding, and a clear boundary between aibox (infrastructure + curated skills) and derived projects (tailoring + execution).

**Relationship to SKILL.md standard:** The open standard at agentskills.io/specification provides the skill format. aibox provides curated, vetted skills. External marketplaces (ClawHub) are user responsibility.

**Implications:** Process presets (minimal/managed/research/product) become skill compositions. aibox.toml gains a [skills] section mapping processes to skill flavors. aibox doctor checks consistency between declared processes and installed skills.

**Alternatives:** Keep current monolithic process templates (simpler but rigid). Full framework integration like SAFe/PMBOK (too heavy for aibox scope — that's kaits territory).

---

Older decisions (DEC-001 through DEC-010): [archive/DECISIONS.md](archive/DECISIONS.md)
