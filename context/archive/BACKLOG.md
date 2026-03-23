# Backlog Archive

Completed, merged, and archived items. Active backlog: [../BACKLOG.md](../BACKLOG.md)

---

| ID | Title | Status | Priority | Notes |
|----|-------|--------|----------|-------|
| BACK-001 | Theming screenshots gallery | done | — | #14 — completed in session 2026-03-23 |
| BACK-003 | `aibox skill install` command | archived | — | Split into BACK-023 (skill command) and BACK-024 (external skills) |
| BACK-005 | CLI simplification | archived | — | Merged into BACK-028 (CLI/UX overhaul) |
| BACK-009 | Automated context migration | archived | — | Merged into BACK-022 (items 9–10) |
| BACK-013 | TeX Live builder deduplication | done | — | Resolved by BACK-022 — LaTeX is now a single add-on |
| BACK-021 | Investigate migration from Zensical to Docusaurus | done | — | Migration complete — docs-site/ has Docusaurus content. Old docs/ and zensical.toml cleanup done in BACK-038 Phase 0 |
| BACK-022 | Declarative config + minimal base images | done | — | DEC-016. 5 phases: addon registry, single base image, process packages, sync expansion, migration system |
| BACK-023 | `aibox skill` command | archived | — | Merged into BACK-028 (CLI/UX overhaul) |
| BACK-036 | Bug: Yazi "e" key does not open files in vim | done | — | Fixed in 06d9505 — corrected $0→$1 variable syntax in keymap.toml, added EDITOR/VISUAL exports |
| BACK-037 | Bug: Yazi preview broken for images/PDF/GIF | done | — | Fixed in 06d9505 — added chafa, poppler-utils, ffmpeg, imagemagick to base image |
| BACK-038 | Rename project to "aibox" | done | — | 5 phases: Rust core, config rename, scripts/docs, meta-docs, GitHub rename. Completed 2026-03-23 |
| BACK-041 | Backlog structure: separate active from archive | done | — | Split BACKLOG.md into active + BACKLOG-ARCHIVE.md |
| — | aibox sync | done | — | #25 — theme switching without manual file deletion |
| — | Shell enhancement tools | done | — | ripgrep, fd, bat, eza, zoxide, fzf, delta + aliases |
| — | Starship prompt | done | — | #28 — installed in base image |
| — | Keyboard shortcuts cheatsheet | done | — | #16 — docs page |
| — | generate deprecated | done | — | Replaced by sync (#25) |
| — | AI provider flexibility | done | — | #19 — Claude/Aider/Gemini, dynamic layouts, optional |
| — | Process templates | done | — | #29 — release, code-review, feature-dev, bug-fix |
| — | SKILL.md support | done | — | #30 — 3 example skills scaffolded |
| — | Addon packages | done | — | #18 — infrastructure, kubernetes, cloud, 6 docs addons |
| — | Security audit command | done | — | #24 — aibox audit: cargo audit, pip-audit, trivy |
| — | Zensical migration | done | — | #26 — config + maintain.sh support |
| — | Dockerfile optimization | done | — | #27 — pinned versions, fontconfig fix, COPY consolidation |
| — | Version in docs header | done | — | #12 — site_name includes version |
| — | Starship prompt presets | done | — | #17 — 6 presets with theme-aware colors |
| — | Additional image flavors | done | — | Node.js + Go (10 images total) |
| — | Code review for simplification | done | — | #22 — dedup, test helpers, dead code removal |
| — | Curated skill library | done | — | #30 — 83 skills, 14 categories, 57 reference files |
| — | Skills Library docs | done | — | 15 category pages, search with autocomplete |
| — | assist→cowork cleanup | done | — | Removed all stale assist.kdl references |
| — | Vim colorscheme fix | done | — | Granular .vim mounts so image colors survive |
| — | Remove mkdocs.yml | done | — | Fully migrated to zensical.toml |
| — | Dogfood product process | done | — | Adopt own product template, migrate GitHub issues |
