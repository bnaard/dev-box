# Source repositories

This project has no single home repository. It audits a theme system against
several upstream sources, all read read-only.

## Primary source of the theme set

repo: shikijs/textmate-grammars-themes
branch: main
path: packages/tm-themes/themes

The 31 aibox theme families are exactly the Shiki bundled theme set, which is
what made per-family verification possible: for most families the upstream
artifact is a single theme JSON carrying both colours and per-scope `fontStyle`.

## Additional sources read

- rebelot/kanagawa.nvim @ master — `lua/kanagawa/colors.lua` (wave / dragon / lotus palettes)
- sainnhe/everforest @ master — `palette.md` (dark + light, role semantics)

## Last sync

date: 2026-08-20T09:25:40Z
commit: (not recorded — reads were by branch ref, no commit sha resolved)

### Updated in this project

- Corrected 53 slot values that were not the upstream value across 31 families.
- Closed 145 foreground/background pairs that sat below their per-role contrast floor.
- Rebuilt the projectious theme from brand v2.1.1 tokens as five variants.
- Added an emphasis (bold/italic/dim) spec, recovering the `fontStyle` data aibox discards.

## Screen map

| Deliverable | Built from |
|---|---|
| `Theme Audit.dc.html` | `theme-data.js` (all upstream reads), projectious brand tokens in `_ds/` |
| `AIBOX-THEME-CORRECTIONS.md` | same, generated |
| `aibox-theme-corrections.toml` | same, generated |
| `theme-data.js` — CANON entries keyed `shiki` | `packages/tm-themes/themes/{andromeeda,laserwave,vesper,min-dark}.json` |
| `theme-data.js` — CANON entries keyed `kana` | kanagawa.nvim `lua/kanagawa/colors.lua` |
| `theme-data.js` — CANON entries keyed `ever` | everforest `palette.md` |
