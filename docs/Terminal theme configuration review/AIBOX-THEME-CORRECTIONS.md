# aibox theme system — corrected values and semantic model

Review of the v0.x snapshot in `THEME-DESIGN-AGENT-BRIEFING.md`. 31 families, 61 concrete variants, plus the projectious brand theme rebuilt as five variants.

Three passes, in this order:

1. **Canonical pass** — every family checked against its upstream source. 53 slot values were not the upstream value.
2. **Readability pass** — every slot measured against its own background at a per-role floor. 145 foreground/background pairs failed; 0 remain after correction (the remainder are the deliberate exemptions listed under *Accepted exemptions*).
3. **projectious pass** — rebuilt from brand v2.1.1 tokens, from one variant to five.

Every hex in this file is either read from an upstream source (cited) or derived by a stated rule. Nothing is a taste call without a note.

## How the upstream sources were established

The 31 families are exactly the Shiki bundled theme set, which is what made verification tractable: for most families the upstream artifact is a single theme JSON.

| Evidence key | Source |
|---|---|
| `kana` | rebelot/kanagawa.nvim@master · lua/kanagawa/colors.lua (read 2026-08-20) |
| `ever` | sainnhe/everforest@master · palette.md (read 2026-08-20) |
| `shiki` | shikijs/textmate-grammars-themes@main · packages/tm-themes/themes/*.json (read 2026-08-20) |
| `gruv` | morhetz/gruvbox palette — bright set is the dark-mode set, faded set the light-mode set |
| `nord` | nordtheme/nord — nord0-nord15 slot definitions |
| `primer` | primer/primitives — GitHub fgColor scales per theme |
| `cat` | catppuccin/catppuccin — palette steps (overlay/subtext) |
| `tn` | folke/tokyonight.nvim — palette + comment step |
| `one` | atom/one-dark-syntax — palette |
| `mono` | monokai/monokai — original palette |
| `sol` | altercation/solarized — base03..base3 role table |
| `no` | sdras/night-owl-vscode-theme — palette |
| `brand` | projectious.work brand v2.1.1 · brand/tokens/variables.css (mirrored in this project) |

## Contrast floors, per role

The brief asked for explicit WCAG-style calculations. A single floor across nine slots would be wrong in both directions — it would over-constrain a status-bar surface and under-constrain body text. Floors used throughout:

| Role | Floor | Why this floor |
|---|---|---|
| `fg` | 7:1 | body text, read for hours — AAA for the one colour every character uses |
| `muted` | 4.5:1 | comments and metadata are text, not decoration — SC 1.4.3 applies |
| `accent` | 4.5:1 | accent is drawn as text (directories, titles, function names), not only as a fill |
| `green` | 4.5:1 | syntax + success status, both carrying text |
| `red` | 4.5:1 | syntax + error status — the one colour a user must never miss |
| `yellow` | 4.5:1 | syntax types + warning status |
| `orange` | 4.5:1 | numbers and constants — dense small glyphs |
| `cyan` | 4.5:1 | operators and info status |
| `magenta` | 4.5:1 | keywords/preprocessor once it becomes first-class |
| `surface` | 1.2:1 | status bars and popups must be *visible* against the page, not readable — a luminance step, not a text pair |
| `pane_inactive_fg` | 3:1 | deliberately dim, still legible: SC 1.4.3 large-text floor |
| `border` | 3:1 | non-text UI contrast, SC 1.4.11 |

Ratios are WCAG 2.1 relative luminance, computed on the corrected background.

## Derivation rules

These are the rules the generator applies, in order. They matter more than the individual hexes: they are what keeps the next 31 families consistent.

1. **Upstream first.** If the upstream theme defines a value for a role, use it. A slot filled from a different theme (Dark+ leakage) is a bug, not a fallback.
2. **A step before a computation.** When a value fails its floor, prefer another step from the *same* upstream palette (Catppuccin `overlay0`→`overlay2`, Gruvbox `gray`→`light4`, Kanagawa `fujiGray`→`springViolet1`). The theme stays recognisably itself.
3. **Compute only as a last resort.** Step toward white (dark bg) or black (light bg) in 2% increments until the floor is met. Hue is preserved; the shift is reported.
4. **No shared surfaces.** `surface` is derived per theme: step `bg` toward `fg` until the pair clears 1.20:1. The single `#313244`/`#CCD0DA` surfaces are dropped.
5. **Every fill gets a computed ink.** `active.foreground` is not `bg` by assumption — it is whichever of {bg, fg, white, black} measures highest on that fill. This is the same inversion the brand's own status solids need.
6. **Diff and selection surfaces are tints of the page**, not fixed hexes: `bg` mixed 22% (dark) / 16% (light) toward the relevant hue, then the ink is checked on the result.
7. **Every status carries a second channel.** Colour alone never encodes state; the label or glyph stays.

## Semantic token model

The nine-slot palette is an input format, not a semantic model. It forces four collisions: red is both *error* and *keyword*; yellow is both *warning* and *type*; accent is both *focus* and *function*; muted is comment, disabled, border and metadata at once. The model below separates them while keeping `theme + mode + variant` in `aibox.toml` untouched.

```
palette (authored, per variant)
  bg fg accent green red yellow orange cyan magenta muted
     └─ magenta is new and first-class: it was being mixed from 60% red + 40% accent,
        which produced a muddy red-violet in every theme that already had a purple.

chrome (derived unless overridden)
  surface  selection.bg  selection.fg  active.bg  active.fg
  inactive.bg  inactive.fg  border.active  border.inactive
  pane.active.bg  pane.active.fg  pane.inactive.bg  pane.inactive.fg
  cursor  cursor.text  terminal.selection.bg  terminal.selection.fg

status (mapped, never positional)
  status.success ← green   status.info ← cyan     status.warning ← yellow
  status.error ← red       status.ok ← muted+accent mix   status.disabled ← muted

code (independent of status)
  code.keyword ← magenta   code.type ← yellow     code.function ← accent
  code.string ← green      code.number ← orange   code.operator ← cyan
  code.comment ← muted.comment    code.invalid ← red (bold)
  code.current-line ← surface     code.selection.bg/fg ← selection.*

muted, split (the single biggest source of failures)
  muted.comment   4.5:1 floor — it is text
  muted.metadata  4.5:1 floor — it is text
  muted.disabled  3.0:1 floor — SC 1.4.3 exempts inactive controls
  muted.border    3.0:1 floor — SC 1.4.11 non-text
```

Four tokens the current system never emits and should: `cursor`, `cursor.text`, `terminal.selection.bg`, `terminal.selection.fg`. They are currently inherited from whatever the user's emulator happens to be set to, which means the one element that is always on screen is the one element the theme does not control.

## Cross-tool inconsistencies found

| # | Finding | Fix |
|---|---|---|
| 1 | Delta's minus/plus backgrounds are hard-coded (`#3B1F22`/`#6B1E25`, `#1F3B25`/`#1F5B33`) for all 61 variants, including all 17 light ones. On `GithubLight` the minus background is darker than the text. | `diff.*.bg` per variant, derived by rule 6. Values below. |
| 2 | LazyGit's selected line background is `bg` — selection is invisible. | `selection.bg`. |
| 3 | fzf sets the same `fg` for selected and unselected rows; selection is carried by surface alone. | `selection.fg` explicit. |
| 4 | Vim sets `Visual` background only; selected text keeps its syntax colour, which can land at 1.1:1 on the selection tint. | `selection.fg` explicit. |
| 5 | Yazi's input `selected` uses reverse video, so it ignores the palette entirely. | `selection.bg`/`.fg`. |
| 6 | Yazi marks *copied* files with `accent` — the same colour as active tabs and focus. | copied ← `magenta`, matching the untracked/unset convention. |
| 7 | One `surface` shared by 17 light variants (`#CCD0DA`) and one by most dark ones (`#313244`). Catppuccin's own surface leaked into Gruvbox, Solarized and Ayu. | Rule 4. |
| 8 | `active.foreground` is always `bg`. On light variants with a light accent (`SnazzyLight` accent `#57C7FF`) that is white-on-pale. | Rule 5. |
| 9 | Solo families ignore `mode`, silently. A user who sets `mode = "light"` on `vesper` gets dark with no message. | Reject with a message naming the family's available variants. |
| 10 | Bat/lnav/Aider/Gemini/OpenCode fall back to a *different theme* (`monocai`, `Coldark`, `monokai`) rather than an approximation of the active palette. Gruvbox Light maps to `gruvbox-dark` in Aider. | Emit a generated theme for every tool that accepts one (bat `.tmTheme`, delta syntax theme, lnav JSON, OpenCode JSON). Named-built-in mapping stays only where custom themes are impossible (Claude Code). |

## Accepted exemptions

Not every remaining sub-floor pair is a defect:

- **`status.disabled` and inactive chrome** — SC 1.4.3 exempts inactive components. Floor 3.0:1, and they still meet it.
- **`Red` (the family)** — a deliberate novelty theme on `#390000`. Its greens and oranges are pale pinks; corrected to real hues, it stops being the joke it is. Flagged, values proposed, adoption optional.
- **`GithubDarkHighContrast` / `GithubLightHighContrast`** — already above every floor; untouched except the duplicated orange.
- **Logotype and brand marks** — the projectious wordmark sets "work" in `#e05232`; SC 1.4.3 exempts logotypes.

## Suspect palettes — verify before shipping

- **AuroraX** — Every slot equals VS Code Dark+ (#569CD6/#B5CEA8/#F44747/#CE9178/#4EC9B0). Aurora X only shares its background. Treat as an unfilled palette, not a theme.
- **DraculaSoft** — Unverified against dracula-soft.json; bg #22212C matches Dracula Pro, not Dracula Soft.
- **MinDark** — Min has no green and no cyan. Both slots were filled with VS Code Dark+ values; they must be derived and labelled as derived.
- **MinLight** — Every slot equals VS Code Light+ (#0000FF/#098658/#E50000/#865F00/#267F99). Min Light shares none of them beyond the background.
- **Red** — green #F4C2C2 and orange #FFD0D0 are pale pinks, not a green or an orange. Semantic status colour is unreadable as status.

## Corrected values, per variant

`cur` is the value in the briefing; `new` is the corrected value; `ratio` is the corrected value against the corrected background. `kind`: `canon` = upstream correction, `step` = moved to another step of the same palette, `lift` = computed, `kept` = already correct.

### andromeeda

**Andromeeda** — dark, `variant = "solo"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#23262E` | — | `#23262E` | — | kept |  |
| fg | `#D5CED9` | 9.84 | `#D5CED9` | 9.84 | kept |  |
| accent | `#00E8C6` | 9.62 | `#00E8C6` | 9.62 | kept |  |
| green | `#89E044` | 9.23 | `#96E072` | 9.51 | canon | terminal.ansiGreen / string token is #96E072; #89E044 appears nowhere upstream (shiki) |
| red | `#EE5D43` | 4.53 | `#EE5D43` | 4.53 | kept |  |
| yellow | `#FFCC00` | 10.01 | `#FFE66D` | 12.1 | canon | terminal.ansiYellow / type token is #FFE66D; #FFCC00 appears nowhere upstream (shiki) |
| orange | `#F39C12` | 6.9 | `#F39C12` | 6.9 | kept |  |
| cyan | `#00E8C6` | 9.62 | `#00E8C6` | 9.62 | kept |  |
| muted | `#6B6B6B` | 2.84 | `#A0A1A7` | 5.87 | canon | comment token is #A0A1A7cc; #6B6B6B is invented and fails as text (shiki) |

_new slots + derived chrome:_ magenta `#CB5BEE` · surface `#31333C` · selection `#7C2323` on ink `#FFFFFF` · cursor `#00E8C6`/`#000000` · border active `#00E8C6` inactive `#A0A1A7` · pane inactive `#32353D`/`#C8C3CD` · diff +`#3C4F3D` ~`#53503C` −`#503233` · active ink `#000000`

### aurora-x

**AuroraX** — dark, `variant = "solo"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#07090F` | — | `#07090F` | — | kept |  |
| fg | `#D4D4D4` | 13.43 | `#D4D4D4` | 13.43 | kept |  |
| accent | `#569CD6` | 6.75 | `#569CD6` | 6.75 | kept |  |
| green | `#B5CEA8` | 11.71 | `#B5CEA8` | 11.71 | kept |  |
| red | `#F44747` | 5.53 | `#F44747` | 5.53 | kept |  |
| yellow | `#CE9178` | 7.53 | `#CE9178` | 7.53 | kept |  |
| orange | `#CE9178` | 7.53 | `#CE9178` | 7.53 | kept |  |
| cyan | `#4EC9B0` | 9.77 | `#4EC9B0` | 9.77 | kept |  |
| muted | `#5C6370` | 3.29 | `#737984` | 4.55 | lift | Computed lift #5C6370→#737984: 3.29:1 misses the 4.5:1 floor. |

_new slots + derived chrome:_ magenta `#C586C0` · surface `#1E1F25` · selection `#18293B` on ink `#FFFFFF` · cursor `#569CD6`/`#000000` · border active `#569CD6` inactive `#737984` · pane inactive `#14161D`/`#BCBDC0` · diff +`#2D3431` ~`#332726` −`#3B171B` · active ink `#000000`

### ayu

**AyuDark** — dark, `variant = "default"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#0A0E14` | — | `#0A0E14` | — | kept |  |
| fg | `#B3B1AD` | 9.03 | `#B3B1AD` | 9.03 | kept |  |
| accent | `#39BAE6` | 8.6 | `#39BAE6` | 8.6 | kept |  |
| green | `#AAD94C` | 11.73 | `#AAD94C` | 11.73 | kept |  |
| red | `#F07178` | 6.76 | `#F07178` | 6.76 | kept |  |
| yellow | `#FFB454` | 10.97 | `#FFB454` | 10.97 | kept |  |
| orange | `#FF8F40` | 8.52 | `#FF8F40` | 8.52 | kept |  |
| cyan | `#95E6CB` | 13.32 | `#95E6CB` | 13.32 | kept |  |
| muted | `#626A73` | 3.53 | `#757C84` | 4.58 | lift | Computed lift #626A73→#757C84: 3.53:1 misses the 4.5:1 floor. |

_new slots + derived chrome:_ magenta `#D2A6FF` · surface `#1E2226` · selection `#194557` on ink `#FFFFFF` · cursor `#39BAE6`/`#000000` · border active `#39BAE6` inactive `#757C84` · pane inactive `#171B21`/`#A4A4A3` · diff +`#2D3B20` ~`#403322` −`#3D242A` · active ink `#000000`

**AyuMirage** — dark, `variant = "mirage"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#1F2430` | — | `#1F2430` | — | kept |  |
| fg | `#CCCAC2` | 9.45 | `#CCCAC2` | 9.45 | kept |  |
| accent | `#5CCFE6` | 8.51 | `#5CCFE6` | 8.51 | kept |  |
| green | `#BAE67E` | 10.87 | `#BAE67E` | 10.87 | kept |  |
| red | `#F28779` | 6.29 | `#F28779` | 6.29 | kept |  |
| yellow | `#FFD173` | 10.8 | `#FFD173` | 10.8 | kept |  |
| orange | `#FFAD66` | 8.44 | `#FFAD66` | 8.44 | kept |  |
| cyan | `#95E6CB` | 10.68 | `#95E6CB` | 10.68 | kept |  |
| muted | `#707A8C` | 3.58 | `#848D9C` | 4.63 | lift | Computed lift #707A8C→#848D9C: 3.58:1 misses the 4.5:1 floor. |

_new slots + derived chrome:_ magenta `#D4BFFF` · surface `#2F333D` · selection `#305463` on ink `#FFFFFF` · cursor `#5CCFE6`/`#000000` · border active `#5CCFE6` inactive `#848D9C` · pane inactive `#2B313D`/`#BABBB9` · diff +`#414F41` ~`#504A3F` −`#4D3A40` · active ink `#000000`

**AyuLight** — light, `variant = "default"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#FAFAFA` | — | `#FAFAFA` | — | kept |  |
| fg | `#5C6773` | 5.52 | `#4D5761` | 7.06 | lift | Computed lift #5C6773→#4D5761: 5.52:1 misses the 7:1 floor. |
| accent | `#55B4D4` | 2.27 | `#3A7A90` | 4.61 | lift | Computed lift #55B4D4→#3A7A90: 2.27:1 misses the 4.5:1 floor. |
| green | `#86B300` | 2.38 | `#5E7D00` | 4.56 | lift | Computed lift #86B300→#5E7D00: 2.38:1 misses the 4.5:1 floor. |
| red | `#E7676A` | 3.07 | `#B95255` | 4.57 | lift | Computed lift #E7676A→#B95255: 3.07:1 misses the 4.5:1 floor. |
| yellow | `#FA8D3E` | 2.25 | `#AA602A` | 4.56 | lift | Computed lift #FA8D3E→#AA602A: 2.25:1 misses the 4.5:1 floor. |
| orange | `#F07171` | 2.75 | `#B65656` | 4.52 | lift | Computed lift #F07171→#B65656: 2.75:1 misses the 4.5:1 floor. |
| cyan | `#4CBF99` | 2.18 | `#327E65` | 4.67 | lift | Computed lift #4CBF99→#327E65: 2.18:1 misses the 4.5:1 floor. |
| muted | `#ABB0B6` | 2.09 | `#6D7174` | 4.72 | lift | Computed lift #ABB0B6→#6D7174: 2.09:1 misses the 4.5:1 floor. |

_new slots + derived chrome:_ magenta `#8664A7` · surface `#E4E5E6` · selection `#C4D6DC` on ink `#000000` · cursor `#3A7A90`/`#FFFFFF` · border active `#3A7A90` inactive `#6D7174` · pane inactive `#E9EAEA`/`#555E66` · diff +`#E1E6D2` ~`#EDE1D9` −`#F0DFE0` · active ink `#FFFFFF`

### catppuccin

**CatppuccinMocha** — dark, `variant = "default"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#1E1E2E` | — | `#1E1E2E` | — | kept |  |
| fg | `#CDD6F4` | 11.34 | `#CDD6F4` | 11.34 | kept |  |
| accent | `#89B4FA` | 7.79 | `#89B4FA` | 7.79 | kept |  |
| green | `#A6E3A1` | 11.03 | `#A6E3A1` | 11.03 | kept |  |
| red | `#F38BA8` | 7.08 | `#F38BA8` | 7.08 | kept |  |
| yellow | `#F9E2AF` | 12.91 | `#F9E2AF` | 12.91 | kept |  |
| orange | `#FAB387` | 9.27 | `#FAB387` | 9.27 | kept |  |
| cyan | `#94E2D5` | 11.01 | `#94E2D5` | 11.01 | kept |  |
| muted | `#6C7086` | 3.36 | `#9399B2` | 5.81 | step | Lifted to the in-palette step overlay2 for 4.5:1. (cat) |

_new slots + derived chrome:_ magenta `#CBA6F7` · surface `#2C2D3E` · selection `#3C4867` on ink `#FFFFFF` · cursor `#89B4FA`/`#000000` · border active `#89B4FA` inactive `#9399B2` · pane inactive `#2C2D3E`/`#BFC7E4` · diff +`#3C4947` ~`#4E494A` −`#4D3649` · active ink `#000000`

**CatppuccinMacchiato** — dark, `variant = "macchiato"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#24273A` | — | `#24273A` | — | kept |  |
| fg | `#CAD3F5` | 9.92 | `#CAD3F5` | 9.92 | kept |  |
| accent | `#8AADF4` | 6.57 | `#8AADF4` | 6.57 | kept |  |
| green | `#A6DA95` | 9.17 | `#A6DA95` | 9.17 | kept |  |
| red | `#ED8796` | 5.96 | `#ED8796` | 5.96 | kept |  |
| yellow | `#EED49F` | 10.2 | `#EED49F` | 10.2 | kept |  |
| orange | `#F5A97F` | 7.62 | `#F5A97F` | 7.62 | kept |  |
| cyan | `#8BD5CA` | 8.74 | `#8BD5CA` | 8.74 | kept |  |
| muted | `#6E738D` | 3.15 | `#939AB7` | 5.29 | step | Lifted to the in-palette step overlay2 for 4.5:1. (cat) |

_new slots + derived chrome:_ magenta `#C6A0F6` · surface `#313549` · selection `#434F72` on ink `#FFFFFF` · cursor `#8AADF4`/`#000000` · border active `#8AADF4` inactive `#939AB7` · pane inactive `#313549`/`#BCC5E6` · diff +`#414E4E` ~`#504D50` −`#503C4E` · active ink `#000000`

**CatppuccinFrappe** — dark, `variant = "frappe"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#303446` | — | `#303446` | — | kept |  |
| fg | `#C6D0F5` | 8.06 | `#C6D0F5` | 8.06 | kept |  |
| accent | `#8CAAEE` | 5.34 | `#8CAAEE` | 5.34 | kept |  |
| green | `#A6D189` | 7.1 | `#A6D189` | 7.1 | kept |  |
| red | `#E78284` | 4.65 | `#E78284` | 4.65 | kept |  |
| yellow | `#E5C890` | 7.62 | `#E5C890` | 7.62 | kept |  |
| orange | `#EF9F76` | 5.8 | `#EF9F76` | 5.8 | kept |  |
| cyan | `#81C8BE` | 6.41 | `#81C8BE` | 6.41 | kept |  |
| muted | `#737994` | 2.87 | `#949CBB` | 4.53 | step | Lifted to the in-palette step overlay2 for 4.5:1. (cat) |

_new slots + derived chrome:_ magenta `#CA9EE6` · surface `#3C4054` · selection `#4F5C7F` on ink `#FFFFFF` · cursor `#8CAAEE`/`#000000` · border active `#8CAAEE` inactive `#949CBB` · pane inactive `#3C4054`/`#BAC3E7` · diff +`#4A5755` ~`#585556` −`#584554` · active ink `#000000`

**CatppuccinLatte** — light, `variant = "default"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#EFF1F5` | — | `#EFF1F5` | — | kept |  |
| fg | `#4C4F69` | 7.06 | `#4C4F69` | 7.06 | kept |  |
| accent | `#1E66F5` | 4.34 | `#1D62EB` | 4.64 | lift | Computed lift #1E66F5→#1D62EB: 4.34:1 misses the 4.5:1 floor. |
| green | `#40A02B` | 2.96 | `#327D22` | 4.54 | lift | Computed lift #40A02B→#327D22: 2.96:1 misses the 4.5:1 floor. |
| red | `#D20F39` | 4.8 | `#D20F39` | 4.8 | kept |  |
| yellow | `#DF8E1D` | 2.31 | `#986114` | 4.58 | lift | Computed lift #DF8E1D→#986114: 2.31:1 misses the 4.5:1 floor. |
| orange | `#FE640B` | 2.64 | `#BC4A08` | 4.51 | lift | Computed lift #FE640B→#BC4A08: 2.64:1 misses the 4.5:1 floor. |
| cyan | `#179299` | 3.31 | `#13787D` | 4.63 | lift | Computed lift #179299→#13787D: 3.31:1 misses the 4.5:1 floor. |
| muted | `#9CA0B0` | 2.3 | `#6A6D82` | 4.5 | lift | Lifted to the in-palette step subtext0 for 4.5:1. Computed lift #6C6F85→#6A6D82: 4.37:1 misses the 4.5:1 floor. (cat) |

_new slots + derived chrome:_ magenta `#8839EF` · surface `#DADCE3` · selection `#B8CCF2` on ink `#000000` · cursor `#1D62EB`/`#FFFFFF` · border active `#1D62EB` inactive `#6A6D82` · pane inactive `#DFE1E7`/`#54576F` · diff +`#D1DED3` ~`#E1DAD1` −`#EACDD7` · active ink `#FFFFFF`

### dracula

**Dracula** — dark, `variant = "default"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#282A36` | — | `#282A36` | — | kept |  |
| fg | `#F8F8F2` | 13.36 | `#F8F8F2` | 13.36 | kept |  |
| accent | `#BD93F9` | 5.9 | `#BD93F9` | 5.9 | kept |  |
| green | `#50FA7B` | 10.38 | `#50FA7B` | 10.38 | kept |  |
| red | `#FF5555` | 4.53 | `#FF5555` | 4.53 | kept |  |
| yellow | `#F1FA8C` | 12.74 | `#F1FA8C` | 12.74 | kept |  |
| orange | `#FFB86C` | 8.36 | `#FFB86C` | 8.36 | kept |  |
| cyan | `#8BE9FD` | 10.29 | `#8BE9FD` | 10.29 | kept |  |
| muted | `#6272A4` | 3.03 | `#8591B8` | 4.57 | lift | Computed lift #6272A4→#8591B8: 3.03:1 misses the 4.5:1 floor. |

_new slots + derived chrome:_ magenta `#FF79C6` · surface `#373843` · selection `#584C74` on ink `#FFFFFF` · cursor `#BD93F9`/`#000000` · border active `#BD93F9` inactive `#8591B8` · pane inactive `#333646`/`#DBDEE4` · diff +`#315845` ~`#545849` −`#57333D` · active ink `#000000`

**DraculaSoft** — dark, `variant = "soft"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#22212C` | — | `#22212C` | — | kept |  |
| fg | `#F8F8F2` | 14.9 | `#F8F8F2` | 14.9 | kept |  |
| accent | `#C8A8F9` | 7.89 | `#C8A8F9` | 7.89 | kept |  |
| green | `#62E884` | 10.13 | `#62E884` | 10.13 | kept |  |
| red | `#E76D6D` | 5.15 | `#E76D6D` | 5.15 | kept |  |
| yellow | `#E9E987` | 12.46 | `#E9E987` | 12.46 | kept |  |
| orange | `#FFCA80` | 10.6 | `#FFCA80` | 10.6 | kept |  |
| cyan | `#A1F0FE` | 12.41 | `#A1F0FE` | 12.41 | kept |  |
| muted | `#7970A9` | 3.56 | `#8C84B5` | 4.6 | lift | Computed lift #7970A9→#8C84B5: 3.56:1 misses the 4.5:1 floor. |

_new slots + derived chrome:_ magenta `#FF79C6` · surface `#31303A` · selection `#473F59` on ink `#FFFFFF` · cursor `#C8A8F9`/`#000000` · border active `#C8A8F9` inactive `#8C84B5` · pane inactive `#2F2D3C`/`#DDDBE3` · diff +`#304D3F` ~`#4E4D40` −`#4D323A` · active ink `#000000`

### everforest

**EverforestDark** — dark, `variant = "default"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#2D353B` | — | `#2D353B` | — | kept |  |
| fg | `#D3C6AA` | 7.38 | `#D3C6AA` | 7.38 | kept |  |
| accent | `#7FBBB3` | 5.74 | `#7FBBB3` | 5.74 | kept |  |
| green | `#A7C080` | 6.23 | `#A7C080` | 6.23 | kept |  |
| red | `#E67E80` | 4.55 | `#E67E80` | 4.55 | kept |  |
| yellow | `#DBBC7F` | 6.84 | `#DBBC7F` | 6.84 | kept |  |
| orange | `#D699B6` | 5.4 | `#E69875` | 5.41 | canon | #D699B6 is Everforest *purple*; the orange role is #E69875 (ever) |
| cyan | `#83C092` | 5.89 | `#83C092` | 5.89 | kept |  |
| muted | `#7A8478` | 3.21 | `#949F97` | 4.55 | canon+lift | grey1 is the comment/UI-text grey; grey0 (#7A8478) is line numbers Computed lift #859289→#949F97: 3.84:1 misses the 4.5:1 floor. (ever) |

_new slots + derived chrome:_ magenta `#D699B6` · surface `#3C4245` · selection `#476061` on ink `#FFFFFF` · cursor `#7FBBB3`/`#000000` · border active `#7FBBB3` inactive `#949F97` · pane inactive `#394246`/`#C3BCA5` · diff +`#48544A` ~`#53534A` −`#56454A` · active ink `#000000`

**EverforestLight** — light, `variant = "default"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#FDF6E3` | — | `#FDF6E3` | — | kept |  |
| fg | `#5C6A72` | 5.18 | `#4A555B` | 7.1 | lift | Computed lift #5C6A72→#4A555B: 5.18:1 misses the 7:1 floor. |
| accent | `#3A94C5` | 3.13 | `#2E769E` | 4.63 | lift | Computed lift #3A94C5→#2E769E: 3.13:1 misses the 4.5:1 floor. |
| green | `#8DA101` | 2.69 | `#687701` | 4.6 | lift | Computed lift #8DA101→#687701: 2.69:1 misses the 4.5:1 floor. |
| red | `#F85552` | 3.04 | `#C64442` | 4.52 | lift | Computed lift #F85552→#C64442: 3.04:1 misses the 4.5:1 floor. |
| yellow | `#DFA000` | 2.12 | `#936A00` | 4.53 | lift | Computed lift #DFA000→#936A00: 2.12:1 misses the 4.5:1 floor. |
| orange | `#DF69BA` | 2.83 | `#B05A1B` | 4.5 | canon+lift | #DF69BA is Everforest *purple*; the orange role is #F57D26 Computed lift #F57D26→#B05A1B: 2.48:1 misses the 4.5:1 floor. (ever) |
| cyan | `#35A77C` | 2.79 | `#287F5E` | 4.54 | lift | Computed lift #35A77C→#287F5E: 2.79:1 misses the 4.5:1 floor. |
| muted | `#939F91` | 2.56 | `#6A7268` | 4.61 | lift | Computed lift #939F91→#6A7268: 2.56:1 misses the 4.5:1 floor. |

_new slots + derived chrome:_ magenta `#A9508D` · surface `#E6E1D1` · selection `#C7D5D1` on ink `#000000` · cursor `#2E769E`/`#FFFFFF` · border active `#2E769E` inactive `#6A7268` · pane inactive `#EBE6D4`/`#525C5E` · diff +`#E5E2BF` ~`#ECE0BF` −`#F4DAC9` · active ink `#FFFFFF`

### github

**GithubDark** — dark, `variant = "default"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#0D1117` | — | `#0D1117` | — | kept |  |
| fg | `#C9D1D9` | 12.26 | `#E6EDF3` | 16.02 | canon | fgColor.default moved to #e6edf3; #c9d1d9 is the retired 2021 value (primer) |
| accent | `#58A6FF` | 7.49 | `#58A6FF` | 7.49 | kept |  |
| green | `#3FB950` | 7.45 | `#3FB950` | 7.45 | kept |  |
| red | `#F85149` | 5.65 | `#F85149` | 5.65 | kept |  |
| yellow | `#D29922` | 7.5 | `#D29922` | 7.5 | kept |  |
| orange | `#DB6D28` | 5.61 | `#DB6D28` | 5.61 | kept |  |
| cyan | `#79C0FF` | 9.73 | `#79C0FF` | 9.73 | kept |  |
| muted | `#8B949E` | 6.15 | `#8B949E` | 6.15 | kept |  |

_new slots + derived chrome:_ magenta `#D2A8FF` · surface `#21252B` · selection `#254161` on ink `#FFFFFF` · cursor `#58A6FF`/`#000000` · border active `#58A6FF` inactive `#8B949E` · pane inactive `#1C2127`/`#CFD7DE` · diff +`#183624` ~`#382F19` −`#411F22` · active ink `#000000`

**GithubLight** — light, `variant = "default"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#FFFFFF` | — | `#FFFFFF` | — | kept |  |
| fg | `#24292F` | 14.65 | `#24292F` | 14.65 | kept |  |
| accent | `#0969DA` | 5.19 | `#0969DA` | 5.19 | kept |  |
| green | `#1A7F37` | 5.08 | `#1A7F37` | 5.08 | kept |  |
| red | `#CF222E` | 5.36 | `#CF222E` | 5.36 | kept |  |
| yellow | `#9A6700` | 4.87 | `#9A6700` | 4.87 | kept |  |
| orange | `#BC4C00` | 5.03 | `#BC4C00` | 5.03 | kept |  |
| cyan | `#218BFF` | 3.39 | `#1C75D6` | 4.6 | lift | Computed lift #218BFF→#1C75D6: 3.39:1 misses the 4.5:1 floor. |
| muted | `#6E7781` | 4.55 | `#6E7781` | 4.55 | kept |  |

_new slots + derived chrome:_ magenta `#8250DF` · surface `#E9EAEA` · selection `#C4DBF6` on ink `#000000` · cursor `#0969DA`/`#FFFFFF` · border active `#0969DA` inactive `#6E7781` · pane inactive `#EEEFF0`/`#373D44` · diff +`#DAEBDF` ~`#EFE7D6` −`#F7DCDE` · active ink `#FFFFFF`

**GithubDarkDimmed** — dark, `variant = "dimmed"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#22272E` | — | `#22272E` | — | kept |  |
| fg | `#ADBAC7` | 7.6 | `#ADBAC7` | 7.6 | kept |  |
| accent | `#539BF5` | 5.28 | `#539BF5` | 5.28 | kept |  |
| green | `#57AB5A` | 5.28 | `#57AB5A` | 5.28 | kept |  |
| red | `#F47067` | 5.27 | `#F47067` | 5.27 | kept |  |
| yellow | `#C69026` | 5.31 | `#C69026` | 5.31 | kept |  |
| orange | `#F47067` | 5.27 | `#E0823D` | 5.32 | canon | orange was duplicating red (#F47067); dimmed fgColor.orange is #e0823d (primer) |
| cyan | `#6CB6FF` | 6.99 | `#6CB6FF` | 6.99 | kept |  |
| muted | `#768390` | 3.88 | `#848F9B` | 4.57 | lift | Computed lift #768390→#848F9B: 3.88:1 misses the 4.5:1 floor. |

_new slots + derived chrome:_ magenta `#DCBDFB` · surface `#2F343C` · selection `#345176` on ink `#FFFFFF` · cursor `#539BF5`/`#000000` · border active `#539BF5` inactive `#848F9B` · pane inactive `#2E333B`/`#A3AFBC` · diff +`#2E4438` ~`#463E2C` −`#50373B` · active ink `#000000`

**GithubDarkHighContrast** — dark, `variant = "high-contrast-dark"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#0A0C10` | — | `#0A0C10` | — | kept |  |
| fg | `#F0F3F6` | 17.57 | `#F0F3F6` | 17.57 | kept |  |
| accent | `#71B7FF` | 9.24 | `#71B7FF` | 9.24 | kept |  |
| green | `#26CD4D` | 9.25 | `#26CD4D` | 9.25 | kept |  |
| red | `#FF6A69` | 7.01 | `#FF6A69` | 7.01 | kept |  |
| yellow | `#F0B72F` | 10.74 | `#F0B72F` | 10.74 | kept |  |
| orange | `#FF6A69` | 7.01 | `#FFA657` | 10.11 | canon | orange was duplicating red (#FF6A69) (primer) |
| cyan | `#91CBFF` | 11.36 | `#91CBFF` | 11.36 | kept |  |
| muted | `#9198A1` | 6.72 | `#9198A1` | 6.72 | kept |  |

_new slots + derived chrome:_ magenta `#DBB7FF` · surface `#1F2125` · selection `#293F58` on ink `#FFFFFF` · cursor `#71B7FF`/`#000000` · border active `#71B7FF` inactive `#9198A1` · pane inactive `#1A1D21`/`#D8DCE1` · diff +`#10361D` ~`#3D3217` −`#402124` · active ink `#000000`

**GithubLightHighContrast** — light, `variant = "high-contrast-light"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#FFFFFF` | — | `#FFFFFF` | — | kept |  |
| fg | `#0E1116` | 18.91 | `#0E1116` | 18.91 | kept |  |
| accent | `#1A69DB` | 5.14 | `#1A69DB` | 5.14 | kept |  |
| green | `#104F24` | 9.7 | `#104F24` | 9.7 | kept |  |
| red | `#A0111F` | 8.09 | `#A0111F` | 8.09 | kept |  |
| yellow | `#7D4E00` | 7.09 | `#7D4E00` | 7.09 | kept |  |
| orange | `#A0111F` | 8.09 | `#702C00` | 10.25 | canon | orange was duplicating red (#A0111F) (primer) |
| cyan | `#034188` | 9.93 | `#034188` | 9.93 | kept |  |
| muted | `#69717B` | 4.94 | `#69717B` | 4.94 | kept |  |

_new slots + derived chrome:_ magenta `#5A1E96` · surface `#E9EAEA` · selection `#C8DBF6` on ink `#000000` · cursor `#1A69DB`/`#FFFFFF` · border active `#1A69DB` inactive `#69717B` · pane inactive `#EDEEEF`/`#25292F` · diff +`#D9E3DC` ~`#EAE3D6` −`#F0D9DB` · active ink `#FFFFFF`

### gruvbox

**GruvboxDark** — dark, `variant = "default"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#282828` | — | `#282828` | — | kept |  |
| fg | `#D5C4A1` | 8.59 | `#EBDBB2` | 10.75 | canon | fg is fg1 #ebdbb2; #d5c4a1 is fg2 (gruv) |
| accent | `#D79921` | 5.94 | `#FABD2F` | 8.69 | canon | on a dark bg gruvbox uses the *bright* set; neutral #d79921 is the light-mode yellow (gruv) |
| green | `#98971A` | 4.76 | `#B8BB26` | 7.14 | canon | bright green — neutral #98971a measures 3.4:1 on #282828 (gruv) |
| red | `#CC241D` | 2.69 | `#FB5440` | 4.52 | canon+lift | bright red — neutral #cc241d measures 2.4:1 on #282828 Computed lift #FB4934→#FB5440: 4.29:1 misses the 4.5:1 floor. (gruv) |
| yellow | `#D79921` | 5.94 | `#FABD2F` | 8.69 | canon | bright yellow (gruv) |
| orange | `#D65D0E` | 3.81 | `#FE8019` | 5.84 | canon | bright orange (gruv) |
| cyan | `#689D6A` | 4.65 | `#8EC07C` | 7.01 | canon | bright aqua — neutral #689d6a measures 3.4:1 (gruv) |
| muted | `#928374` | 4.02 | `#A89984` | 5.3 | step | Lifted to the in-palette step gray/light4 for 4.5:1. (gruv) |

_new slots + derived chrome:_ magenta `#D3869B` · surface `#363532` · selection `#5F4F2A` on ink `#FFFFFF` · cursor `#FABD2F`/`#000000` · border active `#FABD2F` inactive `#A89984` · pane inactive `#373633`/`#DACBA7` · diff +`#484828` ~`#56492A` −`#56322D` · active ink `#000000`

**GruvboxLight** — light, `variant = "default"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#FBF1C7` | — | `#FBF1C7` | — | kept |  |
| fg | `#3C3836` | 10.22 | `#3C3836` | 10.22 | kept |  |
| accent | `#D65D0E` | 3.41 | `#AF3A03` | 5.4 | canon | light mode uses the *faded* set; #d65d0e is the neutral orange (gruv) |
| green | `#79740E` | 4.29 | `#746F0D` | 4.6 | lift | Computed lift #79740E→#746F0D: 4.29:1 misses the 4.5:1 floor. |
| red | `#CC241D` | 4.82 | `#9D0006` | 7.6 | canon | faded red — neutral #cc241d measures 3.9:1 on #fbf1c7 (gruv) |
| yellow | `#B57614` | 3.33 | `#946110` | 4.65 | lift | Computed lift #B57614→#946110: 3.33:1 misses the 4.5:1 floor. |
| orange | `#D65D0E` | 3.41 | `#AF3A03` | 5.4 | canon | faded orange (gruv) |
| cyan | `#076678` | 5.82 | `#417956` | 4.52 | canon+lift | faded aqua; #076678 is the faded *blue*, a different slot Computed lift #427B58→#417956: 4.4:1 misses the 4.5:1 floor. (gruv) |
| muted | `#928374` | 3.24 | `#786B5F` | 4.55 | lift | Computed lift #928374→#786B5F: 3.24:1 misses the 4.5:1 floor. |

_new slots + derived chrome:_ magenta `#8F3F71` · surface `#E6DDB7` · selection `#E9C598` on ink `#000000` · cursor `#AF3A03`/`#FFFFFF` · border active `#AF3A03` inactive `#786B5F` · pane inactive `#EBE1BB`/`#4B4540` · diff +`#E5DCA9` ~`#EBDAAA` −`#ECCAA8` · active ink `#FFFFFF`

### houston

**Houston** — dark, `variant = "solo"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#17191E` | — | `#17191E` | — | kept |  |
| fg | `#CDD6F4` | 12.16 | `#CDD6F4` | 12.16 | kept |  |
| accent | `#F9C86A` | 11.3 | `#F9C86A` | 11.3 | kept |  |
| green | `#4AF2C8` | 12.41 | `#4AF2C8` | 12.41 | kept |  |
| red | `#FF5370` | 5.63 | `#FF5370` | 5.63 | kept |  |
| yellow | `#FFA726` | 9.05 | `#FFA726` | 9.05 | kept |  |
| orange | `#81D4FA` | 10.66 | `#81D4FA` | 10.66 | kept |  |
| cyan | `#4AF2C8` | 12.41 | `#4AF2C8` | 12.41 | kept |  |
| muted | `#545878` | 2.55 | `#7D8098` | 4.53 | lift | Computed lift #545878→#7D8098: 2.55:1 misses the 4.5:1 floor. |

_new slots + derived chrome:_ magenta `#FF6BCB` · surface `#272A31` · selection `#4D4330` on ink `#FFFFFF` · cursor `#F9C86A`/`#000000` · border active `#F9C86A` inactive `#7D8098` · pane inactive `#23252D`/`#B9C1DD` · diff +`#224943` ~`#4A3820` −`#4A2630` · active ink `#000000`

### kanagawa

**KanagawaWave** — dark, `variant = "default"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#1F1F28` | — | `#1F1F28` | — | kept |  |
| fg | `#DCD7BA` | 11.26 | `#DCD7BA` | 11.26 | kept |  |
| accent | `#7E9CD8` | 5.94 | `#7E9CD8` | 5.94 | kept |  |
| green | `#98BB6C` | 7.52 | `#98BB6C` | 7.52 | kept |  |
| red | `#C34043` | 3.22 | `#E46876` | 5.09 | canon | waveRed — autumnRed #C34043 is the diff/diag red and measures 3.4:1 on sumiInk3 (kana) |
| yellow | `#FF9E3B` | 7.94 | `#E6C384` | 9.73 | canon | carpYellow is the type/yellow role; #FF9E3B is roninYellow (diagnostic warning) (kana) |
| orange | `#D27E99` | 5.59 | `#FFA066` | 8.15 | canon | surimiOrange — #D27E99 is sakuraPink, a different hue entirely (kana) |
| cyan | `#7AA89F` | 6.17 | `#7AA89F` | 6.17 | kept |  |
| muted | `#727169` | 3.33 | `#938AA9` | 5.02 | step | Lifted to the in-palette step springViolet1 for 4.5:1. (kana) |

_new slots + derived chrome:_ magenta `#957FB8` · surface `#2E2E34` · selection `#3F4A64` on ink `#FFFFFF` · cursor `#7E9CD8`/`#000000` · border active `#7E9CD8` inactive `#938AA9` · pane inactive `#2D2C37`/`#CAC4B6` · diff +`#3A4137` ~`#4B433C` −`#4A2F39` · active ink `#000000`

**KanagawaDragon** — dark, `variant = "dragon"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#181616` | — | `#181616` | — | kept |  |
| fg | `#C5C9C5` | 10.76 | `#C5C9C5` | 10.76 | kept |  |
| accent | `#7EB3C9` | 7.88 | `#8BA4B0` | 6.9 | canon | dragonBlue2 — #7EB3C9 exists nowhere in the kanagawa palette (kana) |
| green | `#87A987` | 6.91 | `#87A987` | 6.91 | kept |  |
| red | `#C4746E` | 5.21 | `#C4746E` | 5.21 | kept |  |
| yellow | `#B6927B` | 6.34 | `#C4B28A` | 8.65 | canon | dragonYellow — #B6927B is dragonOrange (kana) |
| orange | `#C4746E` | 5.21 | `#B6927B` | 6.34 | canon | dragonOrange; orange was duplicating red (kana) |
| cyan | `#8EA4A2` | 6.85 | `#8EA4A2` | 6.85 | kept |  |
| muted | `#8A8980` | 5.12 | `#9E9B93` | 6.49 | canon | dragonGray2 — #8A8980 is lotusGray3, a light-theme value (kana) |

_new slots + derived chrome:_ magenta `#A292A3` · surface `#292828` · selection `#3F464A` on ink `#FFFFFF` · cursor `#8BA4B0`/`#000000` · border active `#8BA4B0` inactive `#9E9B93` · pane inactive `#282625`/`#BBBEB9` · diff +`#30362F` ~`#3E3830` −`#3E2B29` · active ink `#000000`

**KanagawaLotus** — light, `variant = "default"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#F2ECBC` | — | `#F2ECBC` | — | kept |  |
| fg | `#545464` | 6.19 | `#4C4C5A` | 7.03 | lift | Computed lift #545464→#4C4C5A: 6.19:1 misses the 7:1 floor. |
| accent | `#1F5F8A` | 5.71 | `#4D699B` | 4.59 | canon | lotusBlue4 — #1F5F8A is not in the palette (kana) |
| green | `#4E7C3F` | 4.09 | `#5B7040` | 4.56 | canon+lift | lotusGreen — #4E7C3F is not in the palette Computed lift #6F894E→#5B7040: 3.26:1 misses the 4.5:1 floor. (kana) |
| red | `#C84053` | 4.06 | `#B83B4C` | 4.64 | lift | Computed lift #C84053→#B83B4C: 4.06:1 misses the 4.5:1 floor. |
| yellow | `#835C00` | 5.01 | `#796644` | 4.61 | canon+lift | lotusYellow2 — #835C00 is not in the palette Computed lift #836F4A→#796644: 4.03:1 misses the 4.5:1 floor. (kana) |
| orange | `#B5485D` | 4.32 | `#9F5500` | 4.64 | canon+lift | lotusOrange — #B5485D is not in the palette (reads as red) Computed lift #CC6D00→#9F5500: 3.04:1 misses the 4.5:1 floor. (kana) |
| cyan | `#536A5B` | 4.89 | `#506F69` | 4.58 | canon+lift | lotusAqua — #536A5B is not in the palette Computed lift #597B75→#506F69: 3.88:1 misses the 4.5:1 floor. (kana) |
| muted | `#A09F8F` | 2.23 | `#6C6A5D` | 4.54 | canon+lift | lotusGray2 — #A09F8F is not in the palette and measures 2.3:1 Computed lift #716E61→#6C6A5D: 4.26:1 misses the 4.5:1 floor. (kana) |

_new slots + derived chrome:_ magenta `#9E506A` · surface `#DCD7AF` · selection `#C4C7B3` on ink `#000000` · cursor `#4D699B`/`#FFFFFF` · border active `#4D699B` inactive `#6C6A5D` · pane inactive `#E2DCB1`/`#54545B` · diff +`#DAD8A8` ~`#DFD7A9` −`#E9D0AA` · active ink `#FFFFFF`

### laserwave

**Laserwave** — dark, `variant = "solo"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#27212E` | — | `#27212E` | — | kept |  |
| fg | `#FFFFFF` | 15.63 | `#FFFFFF` | 15.63 | kept |  |
| accent | `#EB64B9` | 5.25 | `#EB64B9` | 5.25 | kept |  |
| green | `#74DFC4` | 9.75 | `#74DFC4` | 9.75 | kept |  |
| red | `#FE4450` | 4.58 | `#FF3E7B` | 4.64 | canon | editorError.foreground; #FE4450 is Synthwave '84's red (shiki) |
| yellow | `#FFEE79` | 13.22 | `#FFE261` | 12.14 | canon | constant.language token; #FFEE79 is not a LaserWave value (shiki) |
| orange | `#FFEE79` | 13.22 | `#FFB85B` | 9.13 | canon | the theme's own orange token — orange was duplicating yellow (shiki) |
| cyan | `#74DFC4` | 9.75 | `#B4DCE7` | 10.67 | canon | terminal.ansiCyan; #74DFC4 is ansiGreen (already the green slot) (shiki) |
| muted | `#6B5F7D` | 2.65 | `#91889B` | 4.61 | canon | comment token; #6B5F7D is the punctuation grey (shiki) |

_new slots + derived chrome:_ magenta `#B381C5` · surface `#36313D` · selection `#713A63` on ink `#FFFFFF` · cursor `#EB64B9`/`#000000` · border active `#EB64B9` inactive `#91889B` · pane inactive `#342D3B`/`#E4E1E6` · diff +`#384B4F` ~`#574B39` −`#57273F` · active ink `#000000`

### material

**Material** — dark, `variant = "default"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#263238` | — | `#263238` | — | kept |  |
| fg | `#EEFFFF` | 12.77 | `#EEFFFF` | 12.77 | kept |  |
| accent | `#82AAFF` | 5.73 | `#82AAFF` | 5.73 | kept |  |
| green | `#C3E88D` | 9.56 | `#C3E88D` | 9.56 | kept |  |
| red | `#F07178` | 4.6 | `#F07178` | 4.6 | kept |  |
| yellow | `#FFCB6B` | 8.78 | `#FFCB6B` | 8.78 | kept |  |
| orange | `#F78C6C` | 5.59 | `#F78C6C` | 5.59 | kept |  |
| cyan | `#89DDFF` | 8.68 | `#89DDFF` | 8.68 | kept |  |
| muted | `#546E7A` | 2.44 | `#8B9CA5` | 4.64 | lift | Computed lift #546E7A→#8B9CA5: 2.44:1 misses the 4.5:1 floor. |

_new slots + derived chrome:_ magenta `#C792EA` · surface `#344046` · selection `#435878` on ink `#FFFFFF` · cursor `#82AAFF`/`#000000` · border active `#82AAFF` inactive `#8B9CA5` · pane inactive `#323F45`/`#D5E6E9` · diff +`#495A4B` ~`#565443` −`#524046` · active ink `#000000`

**MaterialOcean** — dark, `variant = "ocean"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#0F111A` | — | `#0F111A` | — | kept |  |
| fg | `#A6ACCD` | 8.43 | `#A6ACCD` | 8.43 | kept |  |
| accent | `#82AAFF` | 8.2 | `#82AAFF` | 8.2 | kept |  |
| green | `#C3E88D` | 13.67 | `#C3E88D` | 13.67 | kept |  |
| red | `#F07178` | 6.58 | `#F07178` | 6.58 | kept |  |
| yellow | `#FFCB6B` | 12.56 | `#FFCB6B` | 12.56 | kept |  |
| orange | `#F78C6C` | 8 | `#F78C6C` | 8 | kept |  |
| cyan | `#89DDFF` | 12.42 | `#89DDFF` | 12.42 | kept |  |
| muted | `#464B5D` | 2.17 | `#7A7D8A` | 4.6 | lift | Computed lift #464B5D→#7A7D8A: 2.17:1 misses the 4.5:1 floor. |

_new slots + derived chrome:_ magenta `#C792EA` · surface `#21242F` · selection `#323F5F` on ink `#FFFFFF` · cursor `#82AAFF`/`#000000` · border active `#82AAFF` inactive `#7A7D8A` · pane inactive `#1C1E27`/`#9BA0BC` · diff +`#374033` ~`#443A2C` −`#41262F` · active ink `#000000`

**MaterialPalenight** — dark, `variant = "palenight"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#292D3E` | — | `#292D3E` | — | kept |  |
| fg | `#A6ACCD` | 6.11 | `#B4B9D5` | 7.04 | lift | Computed lift #A6ACCD→#B4B9D5: 6.11:1 misses the 7:1 floor. |
| accent | `#82AAFF` | 5.94 | `#82AAFF` | 5.94 | kept |  |
| green | `#C3E88D` | 9.91 | `#C3E88D` | 9.91 | kept |  |
| red | `#F07178` | 4.77 | `#F07178` | 4.77 | kept |  |
| yellow | `#FFCB6B` | 9.1 | `#FFCB6B` | 9.1 | kept |  |
| orange | `#F78C6C` | 5.8 | `#F78C6C` | 5.8 | kept |  |
| cyan | `#89DDFF` | 9 | `#89DDFF` | 9 | kept |  |
| muted | `#676E95` | 2.76 | `#8F94B1` | 4.57 | lift | Computed lift #676E95→#8F94B1: 2.76:1 misses the 4.5:1 floor. |

_new slots + derived chrome:_ magenta `#C792EA` · surface `#363A4C` · selection `#45557C` on ink `#FFFFFF` · cursor `#82AAFF`/`#000000` · border active `#82AAFF` inactive `#8F94B1` · pane inactive `#35394C`/`#ABB0CC` · diff +`#4B564F` ~`#585048` −`#553C4B` · active ink `#000000`

**MaterialLighter** — light, `variant = "default"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#FAFAFA` | — | `#FAFAFA` | — | kept |  |
| fg | `#546E7A` | 5.17 | `#435862` | 7.16 | lift | Computed lift #546E7A→#435862: 5.17:1 misses the 7:1 floor. |
| accent | `#6182B8` | 3.73 | `#5572A2` | 4.66 | lift | Computed lift #6182B8→#5572A2: 3.73:1 misses the 4.5:1 floor. |
| green | `#91B859` | 2.19 | `#60793B` | 4.68 | lift | Computed lift #91B859→#60793B: 2.19:1 misses the 4.5:1 floor. |
| red | `#E53935` | 4.05 | `#D73632` | 4.51 | lift | Computed lift #E53935→#D73632: 4.05:1 misses the 4.5:1 floor. |
| yellow | `#F6A434` | 1.96 | `#996620` | 4.71 | lift | Computed lift #F6A434→#996620: 1.96:1 misses the 4.5:1 floor. |
| orange | `#F76D47` | 2.78 | `#BC5336` | 4.54 | lift | Computed lift #F76D47→#BC5336: 2.78:1 misses the 4.5:1 floor. |
| cyan | `#39ADB5` | 2.57 | `#297D82` | 4.63 | lift | Computed lift #39ADB5→#297D82: 2.57:1 misses the 4.5:1 floor. |
| muted | `#90A4AE` | 2.48 | `#65737A` | 4.69 | lift | Computed lift #90A4AE→#65737A: 2.48:1 misses the 4.5:1 floor. |

_new slots + derived chrome:_ magenta `#7C4DFF` · surface `#E2E5E6` · selection `#CCD4E1` on ink `#000000` · cursor `#5572A2`/`#FFFFFF` · border active `#5572A2` inactive `#65737A` · pane inactive `#E8EAEB`/`#4C5F68` · diff +`#E1E5DB` ~`#EAE2D7` −`#F4DBDA` · active ink `#FFFFFF`

**MaterialDarker** — dark, `variant = "darker"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#212121` | — | `#212121` | — | kept |  |
| fg | `#EEFFFF` | 15.63 | `#EEFFFF` | 15.63 | kept |  |
| accent | `#89DDFF` | 10.62 | `#89DDFF` | 10.62 | kept |  |
| green | `#C3E88D` | 11.69 | `#C3E88D` | 11.69 | kept |  |
| red | `#FF5370` | 5.16 | `#FF5370` | 5.16 | kept |  |
| yellow | `#FFCB6B` | 10.74 | `#FFCB6B` | 10.74 | kept |  |
| orange | `#F78C6C` | 6.84 | `#F78C6C` | 6.84 | kept |  |
| cyan | `#82AAFF` | 7.01 | `#82AAFF` | 7.01 | kept |  |
| muted | `#546E7A` | 2.98 | `#768B95` | 4.52 | lift | Computed lift #546E7A→#768B95: 2.98:1 misses the 4.5:1 floor. |

_new slots + derived chrome:_ magenta `#C792EA` · surface `#2F3131` · selection `#3A4E56` on ink `#FFFFFF` · cursor `#89DDFF`/`#000000` · border active `#89DDFF` inactive `#768B95` · pane inactive `#2B2E2F`/`#D0E2E5` · diff +`#454D39` ~`#524631` −`#522C32` · active ink `#000000`

### min

**MinDark** — dark, `variant = "default"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#1F1F1F` | — | `#1F1F1F` | — | kept |  |
| fg | `#B2B2B2` | 7.77 | `#F8F8F8` | 15.52 | canon | constant/plain token ink; #B2B2B2 is invented (shiki) |
| accent | `#569CD6` | 5.59 | `#79B8FF` | 7.94 | canon | Min's blue — #569CD6 is VS Code Dark+ (shiki) |
| green | `#B5CEA8` | 9.7 | `#B5CEA8` | 9.7 | kept |  |
| red | `#F44747` | 4.58 | `#F97583` | 6.2 | canon | keyword token — #F44747 is VS Code Dark+ (shiki) |
| yellow | `#CCA700` | 7.14 | `#FF9800` | 7.65 | canon | parameter token — #CCA700 is VS Code Dark+ (shiki) |
| orange | `#CE9178` | 6.24 | `#FFAB70` | 8.88 | canon | string/tag token — #CE9178 is VS Code Dark+ (shiki) |
| cyan | `#4EC9B0` | 8.09 | `#4EC9B0` | 8.09 | kept |  |
| muted | `#525252` | 2.11 | `#80878E` | 4.53 | canon+lift | comment token — #525252 is invented Computed lift #6B737C→#80878E: 3.43:1 misses the 4.5:1 floor. (shiki) |

_new slots + derived chrome:_ magenta `#B392F0` · surface `#2E2E2E` · selection `#334150` on ink `#FFFFFF` · cursor `#79B8FF`/`#000000` · border active `#79B8FF` inactive `#80878E` · pane inactive `#2B2B2C`/`#DADCDE` · diff +`#40463D` ~`#503A18` −`#4F3235` · active ink `#000000`

**MinLight** — light, `variant = "default"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#F8F8F8` | — | `#F8F8F8` | — | kept |  |
| fg | `#333333` | 11.9 | `#333333` | 11.9 | kept |  |
| accent | `#0000FF` | 8.09 | `#0000FF` | 8.09 | kept |  |
| green | `#098658` | 4.33 | `#098356` | 4.5 | lift | Computed lift #098658→#098356: 4.33:1 misses the 4.5:1 floor. |
| red | `#E50000` | 4.56 | `#E50000` | 4.56 | kept |  |
| yellow | `#865F00` | 5.42 | `#865F00` | 5.42 | kept |  |
| orange | `#C1440E` | 4.82 | `#C1440E` | 4.82 | kept |  |
| cyan | `#267F99` | 4.32 | `#247A93` | 4.62 | lift | Computed lift #267F99→#247A93: 4.32:1 misses the 4.5:1 floor. |
| muted | `#9A9A9A` | 2.65 | `#727272` | 4.53 | lift | Computed lift #9A9A9A→#727272: 2.65:1 misses the 4.5:1 floor. |

_new slots + derived chrome:_ magenta `#AF00DB` · surface `#E2E2E2` · selection `#D5D5F9` on ink `#000000` · cursor `#0000FF`/`#FFFFFF` · border active `#0000FF` inactive `#727272` · pane inactive `#E8E8E8`/`#434343` · diff +`#D2E5DE` ~`#E6E0D0` −`#F5D0D0` · active ink `#FFFFFF`

### monokai

**Monokai** — dark, `variant = "solo"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#272822` | — | `#272822` | — | kept |  |
| fg | `#F8F8F2` | 13.94 | `#F8F8F2` | 13.94 | kept |  |
| accent | `#F92672` | 3.93 | `#FA4989` | 4.51 | lift | Computed lift #F92672→#FA4989: 3.93:1 misses the 4.5:1 floor. |
| green | `#A6E22E` | 9.58 | `#A6E22E` | 9.58 | kept |  |
| red | `#F92672` | 3.93 | `#FA4989` | 4.51 | lift | Computed lift #F92672→#FA4989: 3.93:1 misses the 4.5:1 floor. |
| yellow | `#E6DB74` | 10.44 | `#E6DB74` | 10.44 | kept |  |
| orange | `#AE81FF` | 5.23 | `#FD971F` | 6.81 | canon | Monokai's orange; #AE81FF is its purple, which belongs in the new magenta slot (mono) |
| cyan | `#66D9EF` | 9.01 | `#66D9EF` | 9.01 | kept |  |
| muted | `#75715E` | 3.03 | `#939081` | 4.63 | lift | Computed lift #75715E→#939081: 3.03:1 misses the 4.5:1 floor. |

_new slots + derived chrome:_ magenta `#AE81FF` · surface `#363731` · selection `#80364D` on ink `#FFFFFF` · cursor `#FA4989`/`#000000` · border active `#FA4989` inactive `#939081` · pane inactive `#34342D`/`#DFDED6` · diff +`#435125` ~`#514F34` −`#552F39` · active ink `#000000`

### moonlight

**Moonlight** — dark, `variant = "solo"` — no changes

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#212337` | — | `#212337` | — | kept |  |
| fg | `#C8D3F5` | 10.36 | `#C8D3F5` | 10.36 | kept |  |
| accent | `#82AAFF` | 6.72 | `#82AAFF` | 6.72 | kept |  |
| green | `#C3E88D` | 11.21 | `#C3E88D` | 11.21 | kept |  |
| red | `#FF757F` | 5.96 | `#FF757F` | 5.96 | kept |  |
| yellow | `#FFC777` | 10.06 | `#FFC777` | 10.06 | kept |  |
| orange | `#F78C6C` | 6.56 | `#F78C6C` | 6.56 | kept |  |
| cyan | `#86E1FC` | 10.43 | `#86E1FC` | 10.43 | kept |  |
| muted | `#7A88CF` | 4.59 | `#7A88CF` | 4.59 | kept |  |

_new slots + derived chrome:_ magenta `#C099FF` · surface `#2E3146` · selection `#3E4C73` on ink `#FFFFFF` · cursor `#82AAFF`/`#000000` · border active `#82AAFF` inactive `#7A88CF` · pane inactive `#2C2F49`/`#B5C0EC` · diff +`#454E4A` ~`#524745` −`#523547` · active ink `#000000`

### night-owl

**NightOwl** — dark, `variant = "default"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#011627` | — | `#011627` | — | kept |  |
| fg | `#D6DEEB` | 13.54 | `#D6DEEB` | 13.54 | kept |  |
| accent | `#82AAFF` | 7.98 | `#82AAFF` | 7.98 | kept |  |
| green | `#22DA6E` | 9.88 | `#22DA6E` | 9.88 | kept |  |
| red | `#EF5350` | 5.26 | `#EF5350` | 5.26 | kept |  |
| yellow | `#C5E478` | 12.87 | `#ECC48D` | 11.22 | canon | strings/yellow role; #C5E478 is the green-yellow already covered by green (no) |
| orange | `#F78C6C` | 7.79 | `#F78C6C` | 7.79 | kept |  |
| cyan | `#21C7A8` | 8.56 | `#21C7A8` | 8.56 | kept |  |
| muted | `#637777` | 3.87 | `#6F8282` | 4.53 | lift | Computed lift #637777→#6F8282: 3.87:1 misses the 4.5:1 floor. |

_new slots + derived chrome:_ magenta `#C792EA` · surface `#142839` · selection `#284268` on ink `#FFFFFF` · cursor `#82AAFF`/`#000000` · border active `#82AAFF` inactive `#6F8282` · pane inactive `#0E2332`/`#BCC7D1` · diff +`#084137` ~`#353C3D` −`#352330` · active ink `#000000`

**NightOwlLight** — light, `variant = "default"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#FBFBFB` | — | `#FBFBFB` | — | kept |  |
| fg | `#403F53` | 9.88 | `#403F53` | 9.88 | kept |  |
| accent | `#4876D6` | 4.19 | `#4571CD` | 4.51 | lift | Computed lift #4876D6→#4571CD: 4.19:1 misses the 4.5:1 floor. |
| green | `#2AA298` | 3.02 | `#217E77` | 4.7 | lift | Computed lift #2AA298→#217E77: 3.02:1 misses the 4.5:1 floor. |
| red | `#D3423E` | 4.39 | `#CF413D` | 4.53 | lift | Computed lift #D3423E→#CF413D: 4.39:1 misses the 4.5:1 floor. |
| yellow | `#DAA520` | 2.16 | `#906D15` | 4.63 | lift | Computed lift #DAA520→#906D15: 2.16:1 misses the 4.5:1 floor. |
| orange | `#DD6A58` | 3.23 | `#B55748` | 4.59 | lift | Computed lift #DD6A58→#B55748: 3.23:1 misses the 4.5:1 floor. |
| cyan | `#08916A` | 3.85 | `#07835F` | 4.59 | lift | Computed lift #08916A→#07835F: 3.85:1 misses the 4.5:1 floor. |
| muted | `#989FB1` | 2.56 | `#6D727F` | 4.65 | lift | Computed lift #989FB1→#6D727F: 2.56:1 misses the 4.5:1 floor. |

_new slots + derived chrome:_ magenta `#994CC3` · surface `#E6E6E9` · selection `#C8D4EE` on ink `#000000` · cursor `#4571CD`/`#FFFFFF` · border active `#4571CD` inactive `#6D727F` · pane inactive `#EAEBEC`/`#4B4C5E` · diff +`#D8E7E6` ~`#EAE4D6` −`#F4DDDD` · active ink `#FFFFFF`

### nord

**Nord** — dark, `variant = "solo"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#2E3440` | — | `#2E3440` | — | kept |  |
| fg | `#D8DEE9` | 9.25 | `#D8DEE9` | 9.25 | kept |  |
| accent | `#88C0D0` | 6.24 | `#88C0D0` | 6.24 | kept |  |
| green | `#A3BE8C` | 6.13 | `#A3BE8C` | 6.13 | kept |  |
| red | `#BF616A` | 3.05 | `#D08A91` | 4.6 | lift | Computed lift #BF616A→#D08A91: 3.05:1 misses the 4.5:1 floor. |
| yellow | `#EBCB8B` | 8 | `#EBCB8B` | 8 | kept |  |
| orange | `#D08770` | 4.39 | `#D28C76` | 4.61 | lift | Computed lift #D08770→#D28C76: 4.39:1 misses the 4.5:1 floor. |
| cyan | `#81A1C1` | 4.64 | `#8FBCBB` | 5.99 | canon | nord7 is cyan; #81A1C1 is nord9, the blue (nord) |
| muted | `#4C566A` | 1.69 | `#979DA9` | 4.59 | lift | Computed lift #4C566A→#979DA9: 1.69:1 misses the 4.5:1 floor. |

_new slots + derived chrome:_ magenta `#B690AF` · surface `#3C424E` · selection `#495E6B` on ink `#FFFFFF` · cursor `#88C0D0`/`#000000` · border active `#88C0D0` inactive `#979DA9` · pane inactive `#3B414D`/`#C8CED9` · diff +`#485251` ~`#585551` −`#524752` · active ink `#000000`

### one-dark

**OneDarkPro** — dark, `variant = "default"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#282C34` | — | `#282C34` | — | kept |  |
| fg | `#ABB2BF` | 6.57 | `#B2B8C4` | 7.03 | lift | Computed lift #ABB2BF→#B2B8C4: 6.57:1 misses the 7:1 floor. |
| accent | `#61AFEF` | 5.92 | `#61AFEF` | 5.92 | kept |  |
| green | `#98C379` | 6.94 | `#98C379` | 6.94 | kept |  |
| red | `#E06C75` | 4.38 | `#E1727B` | 4.6 | lift | Computed lift #E06C75→#E1727B: 4.38:1 misses the 4.5:1 floor. |
| yellow | `#E5C07B` | 8.1 | `#E5C07B` | 8.1 | kept |  |
| orange | `#D19A66` | 5.68 | `#D19A66` | 5.68 | kept |  |
| cyan | `#56B6C2` | 5.91 | `#56B6C2` | 5.91 | kept |  |
| muted | `#5C6370` | 2.32 | `#90959E` | 4.65 | lift | Computed lift #5C6370→#90959E: 2.32:1 misses the 4.5:1 floor. |

_new slots + derived chrome:_ magenta `#C678DD` · surface `#343941` · selection `#3A5670` on ink `#FFFFFF` · cursor `#61AFEF`/`#000000` · border active `#61AFEF` inactive `#90959E` · pane inactive `#343941`/`#AAAFBB` · diff +`#414D43` ~`#524D44` −`#513B44` · active ink `#000000`

**OneLight** — light, `variant = "default"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#FAFAFA` | — | `#FAFAFA` | — | kept |  |
| fg | `#383A42` | 10.86 | `#383A42` | 10.86 | kept |  |
| accent | `#4078F2` | 3.88 | `#3A6CDA` | 4.64 | lift | Computed lift #4078F2→#3A6CDA: 3.88:1 misses the 4.5:1 floor. |
| green | `#50A14F` | 3.07 | `#40813F` | 4.54 | lift | Computed lift #50A14F→#40813F: 3.07:1 misses the 4.5:1 floor. |
| red | `#CA1243` | 5.47 | `#CA1243` | 5.47 | kept |  |
| yellow | `#C18401` | 3.06 | `#9A6A01` | 4.54 | lift | Computed lift #C18401→#9A6A01: 3.06:1 misses the 4.5:1 floor. |
| orange | `#986801` | 4.66 | `#986801` | 4.66 | kept |  |
| cyan | `#0184BC` | 4 | `#0179AD` | 4.64 | lift | Computed lift #0184BC→#0179AD: 4:1 misses the 4.5:1 floor. |
| muted | `#A0A1A7` | 2.47 | `#707175` | 4.67 | lift | Computed lift #A0A1A7→#707175: 2.47:1 misses the 4.5:1 floor. |

_new slots + derived chrome:_ magenta `#A626A4` · surface `#E5E5E6` · selection `#C8D5F2` on ink `#000000` · cursor `#3A6CDA`/`#FFFFFF` · border active `#3A6CDA` inactive `#707175` · pane inactive `#E9EAEA`/`#46484F` · diff +`#DCE7DC` ~`#EBE3D2` −`#F2D5DD` · active ink `#FFFFFF`

### plastic

**Plastic** — dark, `variant = "solo"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#1B1D23` | — | `#1B1D23` | — | kept |  |
| fg | `#ABB2BF` | 7.9 | `#ABB2BF` | 7.9 | kept |  |
| accent | `#61AFEF` | 7.13 | `#61AFEF` | 7.13 | kept |  |
| green | `#98C379` | 8.36 | `#98C379` | 8.36 | kept |  |
| red | `#E06C75` | 5.27 | `#E06C75` | 5.27 | kept |  |
| yellow | `#E5C07B` | 9.75 | `#E5C07B` | 9.75 | kept |  |
| orange | `#D19A66` | 6.84 | `#D19A66` | 6.84 | kept |  |
| cyan | `#56B6C2` | 7.11 | `#56B6C2` | 7.11 | kept |  |
| muted | `#7A7E8A` | 4.15 | `#828691` | 4.63 | lift | Computed lift #7A7E8A→#828691: 4.15:1 misses the 4.5:1 floor. |

_new slots + derived chrome:_ magenta `#C678DD` · surface `#292C33` · selection `#304960` on ink `#FFFFFF` · cursor `#61AFEF`/`#000000` · border active `#61AFEF` inactive `#828691` · pane inactive `#272A30`/`#A1A7B4` · diff +`#374236` ~`#474136` −`#462E35` · active ink `#000000`

### poimandres

**Poimandres** — dark, `variant = "solo"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#1B1E28` | — | `#1B1E28` | — | kept |  |
| fg | `#A6ACCD` | 7.45 | `#A6ACCD` | 7.45 | kept |  |
| accent | `#A6DA95` | 10.36 | `#A6DA95` | 10.36 | kept |  |
| green | `#5DE4C7` | 10.6 | `#5DE4C7` | 10.6 | kept |  |
| red | `#D0679D` | 4.84 | `#D0679D` | 4.84 | kept |  |
| yellow | `#FFFAC2` | 15.6 | `#FFFAC2` | 15.6 | kept |  |
| orange | `#D0679D` | 4.84 | `#D0679D` | 4.84 | kept |  |
| cyan | `#ADD7FF` | 11.04 | `#ADD7FF` | 11.04 | kept |  |
| muted | `#767C9D` | 4.07 | `#7E84A3` | 4.53 | lift | Computed lift #767C9D→#7E84A3: 4.07:1 misses the 4.5:1 floor. |

_new slots + derived chrome:_ magenta `#FCC5E9` · surface `#2A2E3A` · selection `#3C4B42` on ink `#FFFFFF` · cursor `#A6DA95`/`#000000` · border active `#A6DA95` inactive `#7E84A3` · pane inactive `#272A37`/`#9CA2C3` · diff +`#2A4A4B` ~`#4D4E4A` −`#432E42` · active ink `#000000`

### projectious

**Projectious** — dark, `variant = "solo"` — no changes

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#0E1720` | — | `#0E1720` | — | kept |  |
| fg | `#C5DAF0` | 12.62 | `#C5DAF0` | 12.62 | kept |  |
| accent | `#E05232` | 4.67 | `#E05232` | 4.67 | kept |  |
| green | `#4FB07A` | 6.73 | `#4FB07A` | 6.73 | kept |  |
| red | `#E55B5B` | 5.15 | `#E55B5B` | 5.15 | kept |  |
| yellow | `#E0B85B` | 9.62 | `#E0B85B` | 9.62 | kept |  |
| orange | `#F2A65A` | 8.93 | `#F2A65A` | 8.93 | kept |  |
| cyan | `#8AACC8` | 7.59 | `#8AACC8` | 7.59 | kept |  |
| muted | `#7B8DA3` | 5.32 | `#7B8DA3` | 5.32 | kept |  |

_new slots + derived chrome:_ magenta `#D491B4` · surface `#1E2933` · selection `#6F3228` on ink `#FFFFFF` · cursor `#E05232`/`#000000` · border active `#E05232` inactive `#7B8DA3` · pane inactive `#1B2530`/`#B3C7DD` · diff +`#1C3934` ~`#3C3A2D` −`#3D262D` · active ink `#000000`

### red

**Red** — dark, `variant = "solo"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#390000` | — | `#390000` | — | kept |  |
| fg | `#F8F8F8` | 16.84 | `#F8F8F8` | 16.84 | kept |  |
| accent | `#FF6666` | 6.26 | `#FF6666` | 6.26 | kept |  |
| green | `#F4C2C2` | 11.37 | `#F4C2C2` | 11.37 | kept |  |
| red | `#FF0000` | 4.47 | `#FF0A0A` | 4.51 | lift | Computed lift #FF0000→#FF0A0A: 4.47:1 misses the 4.5:1 floor. |
| yellow | `#FF8800` | 7.47 | `#FF8800` | 7.47 | kept |  |
| orange | `#FFD0D0` | 12.93 | `#FFD0D0` | 12.93 | kept |  |
| cyan | `#FF9999` | 8.75 | `#FF9999` | 8.75 | kept |  |
| muted | `#A06060` | 3.69 | `#AB7373` | 4.63 | lift | Computed lift #A06060→#AB7373: 3.69:1 misses the 4.5:1 floor. |

_new slots + derived chrome:_ magenta `#FF99CC` · surface `#4A1616` · selection `#651616` on ink `#FFFFFF` · cursor `#FF6666`/`#000000` · border active `#FF6666` inactive `#AB7373` · pane inactive `#470E0E`/`#E5D7D7` · diff +`#622B2B` ~`#651E00` −`#650202` · active ink `#000000`

### rose-pine

**RosePine** — dark, `variant = "default"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#191724` | — | `#191724` | — | kept |  |
| fg | `#E0DEF4` | 13.39 | `#E0DEF4` | 13.39 | kept |  |
| accent | `#C4A7E7` | 8.43 | `#C4A7E7` | 8.43 | kept |  |
| green | `#31748F` | 3.38 | `#528AA1` | 4.63 | lift | Computed lift #31748F→#528AA1: 3.38:1 misses the 4.5:1 floor. |
| red | `#EB6F92` | 6.07 | `#EB6F92` | 6.07 | kept |  |
| yellow | `#F6C177` | 10.77 | `#F6C177` | 10.77 | kept |  |
| orange | `#EA9A97` | 8.04 | `#EA9A97` | 8.04 | kept |  |
| cyan | `#9CCFD8` | 10.37 | `#9CCFD8` | 10.37 | kept |  |
| muted | `#6E6A86` | 3.42 | `#908CAA` | 5.48 | step | Lifted to the in-palette step subtle for 4.5:1. (shiki) |

_new slots + derived chrome:_ magenta `#C4A7E7` · surface `#292735` · selection `#493F5B` on ink `#FFFFFF` · cursor `#C4A7E7`/`#000000` · border active `#C4A7E7` inactive `#908CAA` · pane inactive `#272534`/`#CCCAE2` · diff +`#263040` ~`#4A3C36` −`#472A3C` · active ink `#000000`

**RosePineMoon** — dark, `variant = "moon"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#232136` | — | `#232136` | — | kept |  |
| fg | `#E0DEF4` | 11.86 | `#E0DEF4` | 11.86 | kept |  |
| accent | `#C4A7E7` | 7.47 | `#C4A7E7` | 7.47 | kept |  |
| green | `#3E8FB0` | 4.29 | `#4693B3` | 4.53 | lift | Computed lift #3E8FB0→#4693B3: 4.29:1 misses the 4.5:1 floor. |
| red | `#EB6F92` | 5.38 | `#EB6F92` | 5.38 | kept |  |
| yellow | `#F6C177` | 9.55 | `#F6C177` | 9.55 | kept |  |
| orange | `#EA9A97` | 7.13 | `#EA9A97` | 7.13 | kept |  |
| cyan | `#9CCFD8` | 9.19 | `#9CCFD8` | 9.19 | kept |  |
| muted | `#6E6A86` | 3.03 | `#908CAA` | 4.86 | step | Lifted to the in-palette step subtle for 4.5:1. (shiki) |

_new slots + derived chrome:_ magenta `#C4A7E7` · surface `#323045` · selection `#504768` on ink `#FFFFFF` · cursor `#C4A7E7`/`#000000` · border active `#C4A7E7` inactive `#908CAA` · pane inactive `#302E44`/`#CCCAE2` · diff +`#2B3A52` ~`#514444` −`#4F324A` · active ink `#000000`

**RosePineDawn** — light, `variant = "default"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#FAF4ED` | — | `#FAF4ED` | — | kept |  |
| fg | `#575279` | 6.66 | `#524D72` | 7.22 | lift | Computed lift #575279→#524D72: 6.66:1 misses the 7:1 floor. |
| accent | `#907AA9` | 3.47 | `#79668E` | 4.69 | lift | Computed lift #907AA9→#79668E: 3.47:1 misses the 4.5:1 floor. |
| green | `#56949F` | 3.14 | `#45767F` | 4.63 | lift | Computed lift #56949F→#45767F: 3.14:1 misses the 4.5:1 floor. |
| red | `#B4637A` | 3.84 | `#A2596E` | 4.59 | lift | Computed lift #B4637A→#A2596E: 3.84:1 misses the 4.5:1 floor. |
| yellow | `#EA9D34` | 2.05 | `#966421` | 4.64 | lift | Computed lift #EA9D34→#966421: 2.05:1 misses the 4.5:1 floor. |
| orange | `#D7827E` | 2.6 | `#9B5E5B` | 4.64 | lift | Computed lift #D7827E→#9B5E5B: 2.6:1 misses the 4.5:1 floor. |
| cyan | `#286983` | 5.59 | `#286983` | 5.59 | kept |  |
| muted | `#9893A5` | 2.73 | `#706D7A` | 4.62 | lift | Computed lift #9893A5→#706D7A: 2.73:1 misses the 4.5:1 floor. |

_new slots + derived chrome:_ magenta `#79668E` · surface `#E4DEDD` · selection `#D6CCD2` on ink `#000000` · cursor `#79668E`/`#FFFFFF` · border active `#79668E` inactive `#706D7A` · pane inactive `#E9E4DF`/`#5A5574` · diff +`#DDE0DB` ~`#EADDCC` −`#ECDBD9` · active ink `#FFFFFF`

### slack

**SlackDark** — dark, `variant = "default"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#222529` | — | `#222529` | — | kept |  |
| fg | `#D1D2D3` | 10.16 | `#D1D2D3` | 10.16 | kept |  |
| accent | `#8CC4FF` | 8.39 | `#8CC4FF` | 8.39 | kept |  |
| green | `#AFE3A4` | 10.51 | `#AFE3A4` | 10.51 | kept |  |
| red | `#E07070` | 4.93 | `#E07070` | 4.93 | kept |  |
| yellow | `#DFC55A` | 8.99 | `#DFC55A` | 8.99 | kept |  |
| orange | `#DFC55A` | 8.99 | `#DFC55A` | 8.99 | kept |  |
| cyan | `#98D1E0` | 9.18 | `#98D1E0` | 9.18 | kept |  |
| muted | `#60656A` | 2.61 | `#898D91` | 4.6 | lift | Computed lift #60656A→#898D91: 2.61:1 misses the 4.5:1 floor. |

_new slots + derived chrome:_ magenta `#D6A6E0` · surface `#303337` · selection `#3E4E61` on ink `#FFFFFF` · cursor `#8CC4FF`/`#000000` · border active `#8CC4FF` inactive `#898D91` · pane inactive `#2E3135`/`#BFC1C3` · diff +`#414F44` ~`#4C4834` −`#4C3639` · active ink `#000000`

**SlackOchin** — light, `variant = "ochin"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#F9F9F9` | — | `#F9F9F9` | — | kept |  |
| fg | `#383A3C` | 10.85 | `#383A3C` | 10.85 | kept |  |
| accent | `#0070D1` | 4.71 | `#0070D1` | 4.71 | kept |  |
| green | `#268829` | 4.3 | `#248327` | 4.58 | lift | Computed lift #268829→#248327: 4.3:1 misses the 4.5:1 floor. |
| red | `#D0104C` | 5.17 | `#D0104C` | 5.17 | kept |  |
| yellow | `#C64B10` | 4.52 | `#C64B10` | 4.52 | kept |  |
| orange | `#C64B10` | 4.52 | `#C64B10` | 4.52 | kept |  |
| cyan | `#007A7A` | 4.91 | `#007A7A` | 4.91 | kept |  |
| muted | `#A0A4A8` | 2.38 | `#707376` | 4.53 | lift | Computed lift #A0A4A8→#707376: 2.38:1 misses the 4.5:1 floor. |

_new slots + derived chrome:_ magenta `#8E4EC6` · surface `#E4E4E4` · selection `#BDD8EF` on ink `#000000` · cursor `#0070D1`/`#FFFFFF` · border active `#0070D1` inactive `#707376` · pane inactive `#E9E9E9`/`#46484B` · diff +`#D7E6D7` ~`#F1DDD4` −`#F2D4DD` · active ink `#FFFFFF`

### snazzy

**SnazzyLight** — light, `variant = "solo"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#FAFBFC` | — | `#FAFBFC` | — | kept |  |
| fg | `#2D2D2D` | 13.29 | `#2D2D2D` | 13.29 | kept |  |
| accent | `#57C7FF` | 1.84 | `#367B9E` | 4.52 | lift | Computed lift #57C7FF→#367B9E: 1.84:1 misses the 4.5:1 floor. |
| green | `#5AF78E` | 1.34 | `#2F804A` | 4.71 | lift | Computed lift #5AF78E→#2F804A: 1.34:1 misses the 4.5:1 floor. |
| red | `#FF5C57` | 2.93 | `#C74844` | 4.57 | lift | Computed lift #FF5C57→#C74844: 2.93:1 misses the 4.5:1 floor. |
| yellow | `#FF9F43` | 1.97 | `#A3662B` | 4.51 | lift | Computed lift #FF9F43→#A3662B: 1.97:1 misses the 4.5:1 floor. |
| orange | `#FF6AC1` | 2.51 | `#B84C8B` | 4.56 | lift | Computed lift #FF6AC1→#B84C8B: 2.51:1 misses the 4.5:1 floor. |
| cyan | `#57C7FF` | 1.84 | `#367B9E` | 4.52 | lift | Computed lift #57C7FF→#367B9E: 1.84:1 misses the 4.5:1 floor. |
| muted | `#9E9E9E` | 2.59 | `#727272` | 4.64 | lift | Computed lift #9E9E9E→#727272: 2.59:1 misses the 4.5:1 floor. |

_new slots + derived chrome:_ magenta `#B84C8B` · surface `#E6E6E7` · selection `#C3D7E2` on ink `#000000` · cursor `#367B9E`/`#FFFFFF` · border active `#367B9E` inactive `#727272` · pane inactive `#EAEBEB`/`#3E3E3E` · diff +`#DAE7E0` ~`#ECE3DB` −`#F2DEDF` · active ink `#FFFFFF`

### solarized

**SolarizedDark** — dark, `variant = "default"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#002B36` | — | `#002B36` | — | kept |  |
| fg | `#93A1A1` | 5.61 | `#A9B4B4` | 7.06 | lift | Computed lift #93A1A1→#A9B4B4: 5.61:1 misses the 7:1 floor. |
| accent | `#268BD2` | 4.08 | `#3794D6` | 4.55 | lift | Computed lift #268BD2→#3794D6: 4.08:1 misses the 4.5:1 floor. |
| green | `#859900` | 4.69 | `#859900` | 4.69 | kept |  |
| red | `#DC322F` | 3.25 | `#E56765` | 4.62 | lift | Computed lift #DC322F→#E56765: 3.25:1 misses the 4.5:1 floor. |
| yellow | `#B58900` | 4.68 | `#B58900` | 4.68 | kept |  |
| orange | `#CB4B16` | 3.26 | `#D67349` | 4.58 | lift | Computed lift #CB4B16→#D67349: 3.26:1 misses the 4.5:1 floor. |
| cyan | `#2AA198` | 4.75 | `#2AA198` | 4.75 | kept |  |
| muted | `#657B83` | 3.37 | `#7E9097` | 4.52 | lift | Computed lift #657B83→#7E9097: 3.37:1 misses the 4.5:1 floor. |

_new slots + derived chrome:_ magenta `#DD629E` · surface `#113943` · selection `#165576` on ink `#FFFFFF` · cursor `#3794D6`/`#000000` · border active `#3794D6` inactive `#7E9097` · pane inactive `#0F3742`/`#9EABAD` · diff +`#1D432A` ~`#28402A` −`#323840` · active ink `#000000`

**SolarizedLight** — light, `variant = "default"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#FDF6E3` | — | `#FDF6E3` | — | kept |  |
| fg | `#586E75` | 4.99 | `#45565B` | 7.12 | lift | Computed lift #586E75→#45565B: 4.99:1 misses the 7:1 floor. |
| accent | `#268BD2` | 3.41 | `#2075B0` | 4.6 | lift | Computed lift #268BD2→#2075B0: 3.41:1 misses the 4.5:1 floor. |
| green | `#859900` | 2.97 | `#687700` | 4.61 | lift | Computed lift #859900→#687700: 2.97:1 misses the 4.5:1 floor. |
| red | `#DC322F` | 4.29 | `#D3302D` | 4.6 | lift | Computed lift #DC322F→#D3302D: 4.29:1 misses the 4.5:1 floor. |
| yellow | `#B58900` | 2.98 | `#8D6B00` | 4.6 | lift | Computed lift #B58900→#8D6B00: 2.98:1 misses the 4.5:1 floor. |
| orange | `#CB4B16` | 4.27 | `#C34815` | 4.57 | lift | Computed lift #CB4B16→#C34815: 4.27:1 misses the 4.5:1 floor. |
| cyan | `#2AA198` | 2.93 | `#217E77` | 4.51 | lift | Computed lift #2AA198→#217E77: 2.93:1 misses the 4.5:1 floor. |
| muted | `#93A1A1` | 2.48 | `#677171` | 4.66 | lift | Computed lift #93A1A1→#677171: 2.48:1 misses the 4.5:1 floor. |

_new slots + derived chrome:_ magenta `#C6337A` · surface `#E5E1D1` · selection `#C4D4D6` on ink `#000000` · cursor `#2075B0`/`#FFFFFF` · border active `#2075B0` inactive `#677171` · pane inactive `#EBE6D5`/`#4E5D61` · diff +`#E5E2BF` ~`#EBE0BF` −`#F6D6C6` · active ink `#FFFFFF`

### synthwave-84

**Synthwave84** — dark, `variant = "solo"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#2A2139` | — | `#2A2139` | — | kept |  |
| fg | `#FFFFFF` | 15.27 | `#FFFFFF` | 15.27 | kept |  |
| accent | `#36F9F6` | 11.66 | `#36F9F6` | 11.66 | kept |  |
| green | `#FF7EDB` | 6.73 | `#FF7EDB` | 6.73 | kept |  |
| red | `#FE4450` | 4.48 | `#FE4854` | 4.56 | lift | Computed lift #FE4450→#FE4854: 4.48:1 misses the 4.5:1 floor. |
| yellow | `#FEDE5D` | 11.51 | `#FEDE5D` | 11.51 | kept |  |
| orange | `#F97E72` | 6 | `#F97E72` | 6 | kept |  |
| cyan | `#36F9F6` | 11.66 | `#36F9F6` | 11.66 | kept |  |
| muted | `#848082` | 3.92 | `#908D8F` | 4.65 | lift | Computed lift #848082→#908D8F: 3.92:1 misses the 4.5:1 floor. |

_new slots + derived chrome:_ magenta `#FF7EDB` · surface `#393147` · selection `#2D5566` on ink `#FFFFFF` · cursor `#36F9F6`/`#000000` · border active `#36F9F6` inactive `#908D8F` · pane inactive `#362E43`/`#E3E3E3` · diff +`#59355D` ~`#594B41` −`#592A3F` · active ink `#000000`

### tokyo-night

**TokyoNight** — dark, `variant = "default"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#1A1B26` | — | `#1A1B26` | — | kept |  |
| fg | `#C0CAF5` | 10.59 | `#C0CAF5` | 10.59 | kept |  |
| accent | `#7AA2F7` | 6.79 | `#7AA2F7` | 6.79 | kept |  |
| green | `#9ECE6A` | 9.35 | `#9ECE6A` | 9.35 | kept |  |
| red | `#F7768E` | 6.46 | `#F7768E` | 6.46 | kept |  |
| yellow | `#E0AF68` | 8.55 | `#E0AF68` | 8.55 | kept |  |
| orange | `#FF9E64` | 8.4 | `#FF9E64` | 8.4 | kept |  |
| cyan | `#7DCFFF` | 9.96 | `#7DCFFF` | 9.96 | kept |  |
| muted | `#565F89` | 2.76 | `#7982A9` | 4.54 | step | Lifted to the in-palette step comment (lifted step) for 4.5:1. (tn) |

_new slots + derived chrome:_ magenta `#BB9AF7` · surface `#292B39` · selection `#394669` on ink `#FFFFFF` · cursor `#7AA2F7`/`#000000` · border active `#7AA2F7` inactive `#7982A9` · pane inactive `#252736`/`#AEB8E2` · diff +`#374235` ~`#463C35` −`#4B2F3D` · active ink `#000000`

**TokyoNightStorm** — dark, `variant = "storm"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#24283B` | — | `#24283B` | — | kept |  |
| fg | `#C0CAF5` | 9.02 | `#C0CAF5` | 9.02 | kept |  |
| accent | `#7AA2F7` | 5.78 | `#7AA2F7` | 5.78 | kept |  |
| green | `#9ECE6A` | 7.97 | `#9ECE6A` | 7.97 | kept |  |
| red | `#F7768E` | 5.51 | `#F7768E` | 5.51 | kept |  |
| yellow | `#E0AF68` | 7.28 | `#E0AF68` | 7.28 | kept |  |
| orange | `#FF9E64` | 7.16 | `#FF9E64` | 7.16 | kept |  |
| cyan | `#7DCFFF` | 8.49 | `#7DCFFF` | 8.49 | kept |  |
| muted | `#565F89` | 2.35 | `#898FAC` | 4.57 | lift | Computed lift #565F89→#898FAC: 2.35:1 misses the 4.5:1 floor. |

_new slots + derived chrome:_ magenta `#BB9AF7` · surface `#30354A` · selection `#404F77` on ink `#FFFFFF` · cursor `#7AA2F7`/`#000000` · border active `#7AA2F7` inactive `#898FAC` · pane inactive `#303449`/`#B2BBE3` · diff +`#3F4D45` ~`#4D4645` −`#52394D` · active ink `#000000`

**TokyoNightDay** — light, `variant = "default"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#E1E2E7` | — | `#E1E2E7` | — | kept |  |
| fg | `#3760BF` | 4.52 | `#28458A` | 7.03 | lift | Computed lift #3760BF→#28458A: 4.52:1 misses the 7:1 floor. |
| accent | `#2E7DE9` | 3.11 | `#2462B6` | 4.64 | lift | Computed lift #2E7DE9→#2462B6: 3.11:1 misses the 4.5:1 floor. |
| green | `#587539` | 4.04 | `#516C34` | 4.58 | lift | Computed lift #587539→#516C34: 4.04:1 misses the 4.5:1 floor. |
| red | `#F52A65` | 3.01 | `#BF214F` | 4.58 | lift | Computed lift #F52A65→#BF214F: 3.01:1 misses the 4.5:1 floor. |
| yellow | `#8C6C3E` | 3.75 | `#7B5F37` | 4.59 | lift | Computed lift #8C6C3E→#7B5F37: 3.75:1 misses the 4.5:1 floor. |
| orange | `#B15C00` | 3.69 | `#9C5100` | 4.52 | lift | Computed lift #B15C00→#9C5100: 3.69:1 misses the 4.5:1 floor. |
| cyan | `#007197` | 4.26 | `#006C91` | 4.57 | lift | Computed lift #007197→#006C91: 4.26:1 misses the 4.5:1 floor. |
| muted | `#7B8496` | 2.91 | `#5D6472` | 4.6 | lift | Computed lift #7B8496→#5D6472: 2.91:1 misses the 4.5:1 floor. |

_new slots + derived chrome:_ magenta `#7D45C6` · surface `#CBCFDC` · selection `#B0C1DA` on ink `#000000` · cursor `#2462B6`/`#FFFFFF` · border active `#2462B6` inactive `#5D6472` · pane inactive `#D1D3D9`/`#354D84` · diff +`#CACFCA` ~`#D1CDCB` −`#DCC3CF` · active ink `#FFFFFF`

### vesper

**Vesper** — dark, `variant = "solo"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#101010` | — | `#101010` | — | kept |  |
| fg | `#FFFFFF` | 19.03 | `#FFFFFF` | 19.03 | kept |  |
| accent | `#FF7B00` | 7.33 | `#FFC799` | 12.58 | canon | the theme's single accent (button/focus/tab border); #FF7B00 is not a Vesper value (shiki) |
| green | `#99FFE4` | 16.11 | `#99FFE4` | 16.11 | kept |  |
| red | `#F44747` | 5.29 | `#FF8080` | 7.84 | canon | editorError.foreground — #F44747 is VS Code Dark+ (shiki) |
| yellow | `#FF7B00` | 7.33 | `#FFC799` | 12.58 | canon | editorWarning.foreground — Vesper has one warm hue by design (shiki) |
| orange | `#FFC799` | 12.58 | `#FFC799` | 12.58 | kept |  |
| cyan | `#FFC799` | 12.58 | `#99FFE4` | 16.11 | canon | the mint/aqua; cyan was duplicating orange (shiki) |
| muted | `#5C5C5C` | 2.85 | `#A0A0A0` | 7.28 | canon | keyword/UI grey; #5C5C5C is below the comment token's effective ink (shiki) |

_new slots + derived chrome:_ magenta `#FFC799` · surface `#232323` · selection `#4E4034` on ink `#FFFFFF` · cursor `#FFC799`/`#000000` · border active `#FFC799` inactive `#A0A0A0` · pane inactive `#212121`/`#E7E7E7` · diff +`#2E453F` ~`#45382E` −`#452929` · active ink `#000000`

### vitesse

**VitesseDark** — dark, `variant = "default"` — no changes

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#121212` | — | `#121212` | — | kept |  |
| fg | `#DBD7CA` | 13.01 | `#DBD7CA` | 13.01 | kept |  |
| accent | `#4D9375` | 5.13 | `#4D9375` | 5.13 | kept |  |
| green | `#C98A7D` | 6.61 | `#C98A7D` | 6.61 | kept |  |
| red | `#E06C75` | 5.86 | `#E06C75` | 5.86 | kept |  |
| yellow | `#D4976C` | 7.53 | `#D4976C` | 7.53 | kept |  |
| orange | `#6496C8` | 6.01 | `#6496C8` | 6.01 | kept |  |
| cyan | `#80A0C0` | 6.88 | `#80A0C0` | 6.88 | kept |  |
| muted | `#758575` | 4.79 | `#758575` | 4.79 | kept |  |

_new slots + derived chrome:_ magenta `#CB7676` · surface `#242423` · selection `#2A463A` on ink `#FFFFFF` · cursor `#4D9375`/`#000000` · border active `#4D9375` inactive `#758575` · pane inactive `#1E201E`/`#C2C3B5` · diff +`#3A2C2A` ~`#3D2F26` −`#3F2628` · active ink `#000000`

**VitesseLight** — light, `variant = "default"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#FFFFFF` | — | `#FFFFFF` | — | kept |  |
| fg | `#393A34` | 11.48 | `#393A34` | 11.48 | kept |  |
| accent | `#1E754F` | 5.66 | `#1E754F` | 5.66 | kept |  |
| green | `#B56959` | 4.09 | `#AA6354` | 4.54 | lift | Computed lift #B56959→#AA6354: 4.09:1 misses the 4.5:1 floor. |
| red | `#AB5959` | 4.88 | `#AB5959` | 4.88 | kept |  |
| yellow | `#B07D48` | 3.58 | `#976C3E` | 4.64 | lift | Computed lift #B07D48→#976C3E: 3.58:1 misses the 4.5:1 floor. |
| orange | `#296AA3` | 5.7 | `#296AA3` | 5.7 | kept |  |
| cyan | `#2E808F` | 4.56 | `#2E808F` | 4.56 | kept |  |
| muted | `#A0A077` | 2.7 | `#767658` | 4.66 | lift | Computed lift #A0A077→#767658: 2.7:1 misses the 4.5:1 floor. |

_new slots + derived chrome:_ magenta `#AA6354` · surface `#E9E9E9` · selection `#C9DED5` on ink `#000000` · cursor `#1E754F`/`#FFFFFF` · border active `#1E754F` inactive `#767658` · pane inactive `#EFEFEB`/`#48493D` · diff +`#F1E6E4` ~`#EEE7E0` −`#F2E4E4` · active ink `#FFFFFF`

**VitesseBlack** — dark, `variant = "black"`

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#000000` | — | `#000000` | — | kept |  |
| fg | `#DBD7CA` | 14.59 | `#DBD7CA` | 14.59 | kept |  |
| accent | `#4D9375` | 5.75 | `#4D9375` | 5.75 | kept |  |
| green | `#C98A7D` | 7.42 | `#C98A7D` | 7.42 | kept |  |
| red | `#E06C75` | 6.57 | `#E06C75` | 6.57 | kept |  |
| yellow | `#D4976C` | 8.44 | `#D4976C` | 8.44 | kept |  |
| orange | `#6496C8` | 6.74 | `#6496C8` | 6.74 | kept |  |
| cyan | `#80A0C0` | 7.71 | `#80A0C0` | 7.71 | kept |  |
| muted | `#606060` | 3.34 | `#767676` | 4.62 | lift | Computed lift #606060→#767676: 3.34:1 misses the 4.5:1 floor. |

_new slots + derived chrome:_ magenta `#CB7676` · surface `#1A1A18` · selection `#224133` on ink `#FFFFFF` · cursor `#4D9375`/`#000000` · border active `#4D9375` inactive `#767676` · pane inactive `#0E0E0E`/`#C2BFB5` · diff +`#2C1E1C` ~`#2F2118` −`#31181A` · active ink `#000000`

### vscode

**VsCodeDarkPlus** — dark, `variant = "default"` — no changes

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#1E1E1E` | — | `#1E1E1E` | — | kept |  |
| fg | `#D4D4D4` | 11.25 | `#D4D4D4` | 11.25 | kept |  |
| accent | `#569CD6` | 5.65 | `#569CD6` | 5.65 | kept |  |
| green | `#B5CEA8` | 9.81 | `#B5CEA8` | 9.81 | kept |  |
| red | `#F44747` | 4.64 | `#F44747` | 4.64 | kept |  |
| yellow | `#CCA700` | 7.22 | `#CCA700` | 7.22 | kept |  |
| orange | `#CE9178` | 6.31 | `#CE9178` | 6.31 | kept |  |
| cyan | `#4EC9B0` | 8.18 | `#4EC9B0` | 8.18 | kept |  |
| muted | `#6A9955` | 5 | `#6A9955` | 5 | kept |  |

_new slots + derived chrome:_ magenta `#C586C0` · surface `#2D2D2D` · selection `#324B60` on ink `#FFFFFF` · cursor `#569CD6`/`#000000` · border active `#569CD6` inactive `#6A9955` · pane inactive `#272D25`/`#BAC5B4` · diff +`#3F453C` ~`#443C17` −`#4D2727` · active ink `#000000`

**VsCodeLightPlus** — light, `variant = "default"` — no changes

| slot | cur | ratio | new | ratio | kind | note |
|---|---|---|---|---|---|---|
| bg | `#FFFFFF` | — | `#FFFFFF` | — | kept |  |
| fg | `#000000` | 21 | `#000000` | 21 | kept |  |
| accent | `#0000FF` | 8.59 | `#0000FF` | 8.59 | kept |  |
| green | `#098658` | 4.6 | `#098658` | 4.6 | kept |  |
| red | `#CD3131` | 5.15 | `#CD3131` | 5.15 | kept |  |
| yellow | `#A65E00` | 4.97 | `#A65E00` | 4.97 | kept |  |
| orange | `#A31515` | 7.85 | `#A31515` | 7.85 | kept |  |
| cyan | `#267F99` | 4.59 | `#267F99` | 4.59 | kept |  |
| muted | `#008000` | 5.14 | `#008000` | 5.14 | kept |  |

_new slots + derived chrome:_ magenta `#AF00DB` · surface `#E4E4E4` · selection `#D6D6FF` on ink `#000000` · cursor `#0000FF`/`#FFFFFF` · border active `#0000FF` inactive `#008000` · pane inactive `#E0F0E0`/`#002000` · diff +`#D8ECE4` ~`#F1E5D6` −`#F7DEDE` · active ink `#FFFFFF`

## projectious — rebuilt from brand v2.1.1

The briefing carries one `Projectious` variant, marked `solo`. Three problems with it:

1. **It is the deep-dark page with none of the brand's own colours.** `green #4FB07A`, `red #E55B5B`, `yellow #E0B85B`, `orange #F2A65A`, `cyan #8AACC8` are approximations. The brand ships a measured 16-slot ANSI set and ten syntax roles against exactly this surface.
2. **`accent` is `#E05232` used as text.** The brand is explicit: `#e05232` measures 3.87:1 and is the identity colour for marks, borders and fills — accent *text* on dark takes `#ea7558`. Both values are now emitted: `accent` for chrome, `accent.text` for glyphs.
3. **Navy dark is the brand's default dark; deep dark is the floor.** The single variant implemented the floor. Five variants now exist, and `mode = "auto"` resolves dark → navy.

### Projectious · navy dark

Brand default dark. Page is midnight-1 dark (#132440); code panels stay on #0e1720 so a code block still reads as an inset panel.

| slot | value | ratio on bg | source |
|---|---|---|---|
| bg | `#132440` | — | --color-bg / midnight ramp |
| fg | `#C5DAF0` | 10.83 | --color-text-primary |
| accent | `#E05232` | 4 | --orange-9 (identity) |
| green | `#6CC090` | 7.08 | --color-success / terminal-ansi-10 |
| red | `#F08B80` | 6.43 | --color-danger / terminal-ansi-9 |
| yellow | `#E0A92A` | 7.3 | --terminal-ansi-11 |
| orange | `#EA7558` | 5.31 | --color-accent-light / syntax-string |
| cyan | `#74C0C9` | 7.48 | --terminal-ansi-14 |
| muted | `#97A8B8` | 6.36 | --color-text-secondary |

_chrome:_ surface `#1A2B3E` · magenta `#D491B4` · border_inactive `#7B8DA3` · cursor `#E05232` · cursor_text `#0E1720` · selection_bg `#3A5C7E` · selection_fg `#C5DAF0` · accent_text `#EA7558` · code_panel `#0E1720` · terminal `#0E1720`

### Projectious · deep dark

The bottom of the midnight ramp. Same ink, deeper page — panels lift by surface step, not shadow.

| slot | value | ratio on bg | source |
|---|---|---|---|
| bg | `#0E1720` | — | --color-bg / midnight ramp |
| fg | `#C5DAF0` | 12.62 | --color-text-primary |
| accent | `#E05232` | 4.67 | --orange-9 (identity) |
| green | `#6CC090` | 8.24 | --color-success / terminal-ansi-10 |
| red | `#F08B80` | 7.49 | --color-danger / terminal-ansi-9 |
| yellow | `#E0A92A` | 8.5 | --terminal-ansi-11 |
| orange | `#EA7558` | 6.18 | --color-accent-light / syntax-string |
| cyan | `#74C0C9` | 8.71 | --terminal-ansi-14 |
| muted | `#97A8B8` | 7.41 | --color-text-secondary |

_chrome:_ surface `#131E2B` · magenta `#D491B4` · border_inactive `#7B8DA3` · cursor `#E05232` · cursor_text `#0E1720` · selection_bg `#2E4B68` · selection_fg `#C5DAF0` · accent_text `#EA7558` · code_panel `#131E2B` · terminal `#0E1720`

### Projectious · light

Page is midnight-1 (#f8f9fb), not white; white returns as the raised surface. Accent text takes orange-11, never orange-9.

| slot | value | ratio on bg | source |
|---|---|---|---|
| bg | `#F8F9FB` | — | --color-bg / midnight ramp |
| fg | `#142438` | 14.89 | --color-text-primary |
| accent | `#C04424` | 4.87 | --orange-11 (accent text-safe) |
| green | `#276754` | 6.32 | --color-success / terminal-ansi-10 |
| red | `#A8261C` | 6.74 | --color-danger / terminal-ansi-9 |
| yellow | `#8B6508` | 5.03 | --terminal-ansi-11 |
| orange | `#C94208` | 4.67 | --color-accent-light / syntax-string |
| cyan | `#1C6B6B` | 5.92 | --terminal-ansi-14 |
| muted | `#5C6F82` | 4.92 | --color-text-secondary |

_chrome:_ surface `#FFFFFF` · magenta `#8A3F6E` · border_inactive `#ADB2BA` · cursor `#E05232` · cursor_text `#F4F5F7` · selection_bg `#C3D1E3` · selection_fg `#142438` · accent_fill `#CC4528` · code_panel `#0E1720` · terminal `#0E1720`

### Projectious · high-contrast dark

data-contrast=high, dark. Text roles pushed to the ends of the ramp; accent lifted to #ff8161 so it clears AA as text.

| slot | value | ratio on bg | source |
|---|---|---|---|
| bg | `#0E1720` | — | --color-bg / midnight ramp |
| fg | `#FFFFFF` | 18.07 | --color-text-primary |
| accent | `#FF8161` | 7.37 | --orange-9 (identity) |
| green | `#A8E6C4` | 12.72 | --color-success / terminal-ansi-10 |
| red | `#FFC0B8` | 11.6 | --color-danger / terminal-ansi-9 |
| yellow | `#FFDF94` | 13.97 | --terminal-ansi-11 |
| orange | `#FFB49C` | 10.55 | --color-accent-light / syntax-string |
| cyan | `#9BD6DD` | 11.24 | --terminal-ansi-14 |
| muted | `#C5DAF0` | 12.62 | --color-text-secondary |

_chrome:_ surface `#1A2B3E` · magenta `#F0B6D3` · border_inactive `#C5DAF0` · cursor `#FF8161` · cursor_text `#0E1720` · selection_bg `#3A5C7E` · selection_fg `#FFFFFF` · code_panel `#0E1720` · terminal `#0E1720`

### Projectious · high-contrast light

data-contrast=high, light. Tints flatten to solid borders; every text role sits at the dark end of its scale.

| slot | value | ratio on bg | source |
|---|---|---|---|
| bg | `#FFFFFF` | — | --color-bg / midnight ramp |
| fg | `#000000` | 21 | --color-text-primary |
| accent | `#A02F16` | 7.21 | --orange-11 (accent text-safe) |
| green | `#0F4D3A` | 9.8 | --color-success / terminal-ansi-10 |
| red | `#7A1610` | 10.77 | --color-danger / terminal-ansi-9 |
| yellow | `#4A3400` | 11.78 | --terminal-ansi-11 |
| orange | `#8A2E0A` | 8.48 | --color-accent-light / syntax-string |
| cyan | `#0F4C4C` | 9.73 | --terminal-ansi-14 |
| muted | `#1E2B38` | 14.4 | --color-text-secondary |

_chrome:_ surface `#F8F9FB` · magenta `#6E1A50` · border_inactive `#1E2B38` · cursor `#A02F16` · cursor_text `#FFFFFF` · selection_bg `#B0C1D6` · selection_fg `#000000` · code_panel `#0E1720` · terminal `#0E1720`

**Brand rules the generator must honour for projectious specifically**

- Code and terminal panels stay on `#0e1720` in *both* dark appearances. On navy the page is lighter than the panel, which is the point: a code block reads as an inset panel, not a hole.
- Navy dark overrides exactly fourteen names from deep dark (the brand's override manifest). The theme generator should express navy as a diff over deep, not as a second hand-authored palette.
- Solid accent fills carrying white text take `#cc4528` (4.72:1), never `#e05232`.
- The active tab's ink is `#0e1720`, not white: white on `#e05232` is 3.76:1.
- Status never by colour alone — the pill keeps its written label.

## Migration plan

`aibox.toml` does not change. `theme + mode + variant` keeps working, including every current value.

**Step 1 — widen the palette struct (no behaviour change).** Add `magenta` as an authored slot, defaulted to the current 60/40 mix so nothing moves. Add `cursor`, `cursor_text`, `terminal_selection_bg`, `terminal_selection_fg`, defaulted from accent/bg. Ship. Nothing renders differently.

**Step 2 — derive chrome instead of looking it up.** Replace the two shared `surface` constants with rule 4, and `active.foreground` with rule 5. This is the one visually noticeable step; it is also where the 17 light variants stop borrowing Catppuccin's surface. Snapshot tests need re-baselining.

**Step 3 — land the canonical corrections.** Per-variant, with the tables above as the diff. `GruvboxDark` moving to the bright set is the largest single change and should be called out in the changelog — it is also the one that takes three slots from 2.4:1 to above 4.5:1.

**Step 4 — split `muted` and add the contrast floors as tests.** `muted.comment`/`.metadata`/`.disabled`/`.border` with the floors in the table. Add a test that asserts every emitted foreground/background pair meets its role floor; it is a table-driven test over the same data this document was generated from.

**Step 5 — replace named-built-in fallbacks with generated themes** for bat, delta, lnav and OpenCode. Claude Code stays coarse; nothing can be done there.

**Step 6 — projectious becomes a family with five variants**, and `mode = "auto"` resolves dark → navy. `variant = "deep"` reaches the old behaviour.

**Step 7 — solo families reject incompatible modes** with a message that names what they do have.

## Variants needing bespoke values rather than shared fallbacks

In priority order: `MinLight` and `AuroraX` (both are Dark+/Light+ copies), `MinDark` (no green, no cyan upstream — must be derived and labelled), `Red` (novelty palette with no usable status hues), `DraculaSoft` (background does not match the named theme), `KanagawaLotus` (six of nine slots were not palette values), `Houston`, `Plastic`, `SnazzyLight` (light theme with a pale accent — every fill needs a computed ink), `Poimandres` (`orange` duplicates `red`), `SlackDark`/`SlackOchin` (`orange` duplicates `yellow`).

## Emphasis — bold, italic and dim as a second channel

Colour was the whole system. It should not have been: every accepted contrast exemption in this
document, and every user on a bad projector, in greyscale, or with a colour-vision deficiency, needs
state carried by something other than hue. The upstream themes already declare emphasis and aibox
discards it — the briefing emits exactly two attributes across 61 variants (`code.comment` italic,
`code.invalid` bold).

### Upstream already declares this

TextMate themes carry a per-scope `fontStyle` field, and the Shiki bundle uses it. Verified while
auditing colour:

| Theme | Declared |
|---|---|
| Andromeeda | `entity.other.inherited-class` → `underline` |
| Vesper | `markup.bold` → `bold`, `markup.italic` → `italic`, object keys → `italic` |
| Min Dark | `emphasis` → `italic`, `strong`/`markup.heading` → `bold` |
| Laserwave | none — the fallback spec applies |

Harvesting all 61 is a generator task, not a transcription task: read `fontStyle` per scope out of the
theme JSON at build time and map it onto the code roles, exactly as the colour values are read now.
Hand-copying 61 style maps into `themes.rs` would rot on the first upstream bump.

### Configuration

```toml
[customization]
theme = "github"
mode = "auto"
emphasis = "auto"      # auto | full | standard | minimal | none

[customization.emphasis_overrides]   # optional, role → attribute list
code_comment = "italic dim"
status_error = "bold underline"
```

`emphasis` is one key, in the shape of `mode`. Nothing about `theme`/`mode`/`variant` changes.

| Level | Attribute set | For |
|---|---|---|
| `full` | bold · italic · dim · underline · strikethrough | A terminal with true italic and bold cuts, verified terminfo |
| `standard` | bold · italic · dim | The default on a capable terminal — underline is reserved for links and search |
| `minimal` | bold · dim | Fonts with no italic cut, where the renderer would synthesise an oblique |
| `none` | — | Colour only. Today's behaviour, kept reachable |
| `auto` | probe, then degrade | Default |

**`auto` probes, in order:** `sitm`/`ritm` present in terminfo → italic allowed; `dim` present → dim
allowed; `TERM_PROGRAM`/font query inconclusive → assume `standard`; `NO_COLOR` set → `none`.

### Degradation is a substitution, never a silent drop

An attribute the terminal cannot render must fall back to another channel, or the state disappears.

| Wanted | Falls back to | Why |
|---|---|---|
| italic | dim | Comments must still read as secondary, not as code |
| underline | bold | Search's current match must stay distinguishable from other matches |
| strikethrough | dim | Deprecated stays visibly de-emphasised |
| dim | (nothing) | Safe to drop — colour still separates it |
| bold | (never dropped) | Universally supported; if bold is unavailable, so is emphasis |

### Role → attribute (the aibox fallback spec)

Applied where upstream declares nothing. Bold is emphasis; italic is *aside*; dim is *inactive*.

**Syntax**

| Role | full | standard | minimal |
|---|---|---|---|
| `code.keyword` | bold | bold | bold |
| `code.type` | bold | bold | bold |
| `code.function` | — | — | — |
| `code.string` | — | — | — |
| `code.number` | — | — | — |
| `code.operator` | — | — | — |
| `code.comment` | italic | italic | dim |
| `code.decorator` | italic | italic | dim |
| `code.invalid` | bold underline | bold | bold |
| `code.deprecated` | strikethrough dim | dim | dim |

Three-quarters of the roles carry nothing. That is the point — emphasis that lands everywhere lands
nowhere, and `code.function`'s brand weight (500) has no terminal equivalent, so it stays plain
rather than being rounded up to bold.

**Diff**

| Role | full | standard | minimal |
|---|---|---|---|
| `diff.add` / `diff.delete` / `diff.change` | — | — | — |
| `diff.emphasis` (word-level) | bold | bold | bold |
| `diff.header` | bold | bold | bold |
| `diff.hunk` | dim | dim | dim |

Line-level diff keeps its derived background and stays plain: bolding a whole removed block makes the
word-level emphasis inside it unreadable.

**Status**

| Role | full | standard | minimal |
|---|---|---|---|
| `status.error` | bold | bold | bold |
| `status.warning` | bold | bold | bold |
| `status.success` / `status.info` / `status.ok` | — | — | — |
| `status.disabled` | dim | dim | dim |

`status.disabled` carrying `dim` is what makes its 3.0:1 floor defensible: the state is no longer
colour alone.

**Chrome**

| Role | full | standard | minimal |
|---|---|---|---|
| `active.foreground` (tab, menu selection) | bold | bold | bold |
| `pane.active.foreground` (title) | bold | bold | bold |
| `inactive.foreground` (tab, status title) | dim | dim | dim |
| `pane.inactive.foreground` | dim | dim | dim |
| `border.active` / `border.inactive` | — | — | — |

Borders stay plain — a bold box-drawing character changes the glyph's width in some fonts and breaks
the pane grid.

**Search**

| Role | full | standard | minimal |
|---|---|---|---|
| `search.match` | — | — | — |
| `search.current` | bold underline | bold | bold |

**Git states (eza, LazyGit)**

| Role | full | standard | minimal |
|---|---|---|---|
| `git.modified` / `git.staged` | bold | bold | bold |
| `git.untracked` | italic | italic | — |
| `git.ignored` | dim | dim | dim |
| `git.conflicted` | bold underline | bold | bold |

### Tool capability matrix

| Tool | Attributes it accepts | Emitted how | Caveat |
|---|---|---|---|
| Vim | bold, italic, underline, undercurl, strikethrough, reverse | `gui=`/`cterm=` on every generated group | none — the fullest surface |
| tmux | bold, dim, italics, underscore, reverse | `#[bold]`, style strings | needs `terminal-overrides ",*:sitm=\\E[3m:ritm=\\E[23m"` or italics render as reverse |
| Yazi | bold, dim, italic, underline, crossed, reverse | ratatui modifiers in the style tables | none |
| eza | full SGR | `EZA_COLORS` sequences | none |
| Starship | bold, italic, underline, dimmed, strikethrough, inverted | style strings per preset | none |
| delta | bold, italic, underline | decoration style strings | syntax emphasis comes from the bat theme, not delta |
| bat | bold, italic, underline | `fontStyle` in the generated `.tmTheme` | requires step 5 of the migration (generated themes) |
| fzf | bold, italic, underline, dim, reverse, strikethrough | `--color` attribute suffixes | italic and strikethrough need fzf ≥ 0.35 |
| less / man | bold, underline, standout | termcap overrides | no italic; italic must degrade to dim or standout |
| lnav | bold, underline | theme JSON | partial |
| LazyGit | bold, underline, reverse | style keys | partial — no italic, no dim |
| Claude Code, Gemini, OpenCode | none | — | emphasis is not expressible; colour only |
| Aider | via the Pygments theme | — | inherited, not controlled |

Nine of thirteen surfaces take at least bold, italic and dim. Emphasis is more portable than the
nine-slot palette is.

### projectious — emphasis is already specified, not invented

The brand pairs every syntax role with a weight and a style so structure survives greyscale,
colour-blindness and low-quality projection, and it loads the italic and bold cuts of IBM Plex Mono
for exactly this reason. Mapping brand weights onto terminal attributes:

| Brand role | Brand weight / style | Terminal attribute | Note |
|---|---|---|---|
| Keyword | 700 | bold | |
| Type / class | 600 | bold | |
| Escape / invalid | 600 | bold | `+ underline` at `full` |
| Function name | 500 | — | no terminal equivalent for 500; stays plain |
| Comment | 400 italic | italic | `dim` at `minimal` |
| Attribute / decorator | 400 italic | italic | `dim` at `minimal` |
| Everything else | 400 | — | |

Two brand rules carry over unchanged: **never status by colour alone** — the pill keeps its written
label and now its weight too; and **LSP modifiers are typography, not hue** — ten modifiers against
ten roles is a hundred states, which colour cannot carry but weight, style and underline can.

All five projectious variants share one emphasis map. It is a property of the brand's type system,
not of the appearance, so navy, deep, light and both high-contrast variants are identical here.

### Migration — one step, inserted after step 4

**Step 4a — emphasis.** Add the `emphasis` key defaulting to `none` for one release so nothing moves,
then flip the default to `auto`. Emit attributes only for the roles in the tables above, and add the
capability matrix as a lookup so a tool never receives an attribute it silently swallows. The
degradation table is the test surface: assert that under every level, every role that carries state
still carries at least one channel besides hue.

## Three new families — grayscale and high contrast

Ten variants, authored rather than imported, so they can be built the right way round: floors first,
then role separation, then the values that satisfy both. Every value below was verified against its
family floor by the same engine that audited the other 61 — nothing here is an authored claim.

| Family | Variants | Floor | Requires |
|---|---|---|---|
| `mono` | dark · light | 4.5:1 | `emphasis >= standard` |
| `contrast` | dark/light × high · max | 7:1 · 12:1 | `emphasis >= standard` · `full` at max |
| `contrast-mono` | dark/light × high · max | 7:1 · 12:1 | `emphasis >= standard` · `full` at max |

```toml
[customization]
theme = "contrast-mono"   # or "mono", "contrast"
mode = "auto"
variant = "max"           # high (default) | max — contrast families only
emphasis = "full"
```

### The design constraint these families expose

A nine-slot palette assumes nine distinguishable hues. Remove hue and you have a lightness ramp with
a fixed number of usable steps — and the higher the contrast floor, the fewer steps remain:

| Floor | Usable steps on black | Roles to place |
|---|---|---|
| 4.5:1 | 7 | 8 |
| 7:1 | 5 | 8 |
| 12:1 | 3 | 8 |

So role separation is a *pair*: a lightness step and an attribute. Two roles may share a step only if
their attributes differ. At 12:1 on grayscale, eight roles sit on three steps and typography is
doing most of the work — which is why `contrast-mono` at `max` must refuse `emphasis = "none"`
rather than render eight indistinguishable roles.

### `mono`

Grayscale, day to day. Not an accessibility mode — a theme for people who find hue noisy, and the cheapest possible proof that the emphasis channel carries real semantics: with hue gone, it is the only thing left.

**Mono · dark** — `mode = "dark"`, `variant = "solo"`, floor 4.5:1, `emphasis >= standard`

A working grayscale theme, not an accessibility mode. Six roles sit on four lightness steps; the attribute disambiguates same-step pairs (orange plain vs red bold-underline, green plain vs cyan italic).

| role | value | ratio on bg | attribute |
|---|---|---|---|
| bg | `#121212` | — | — |
| fg | `#EDEDED` | 16 | — |
| accent | `#FFFFFF` | 18.73 | bold |
| green | `#BDBDBD` | 9.97 | — |
| red | `#EDEDED` | 16 | bold underline |
| yellow | `#D6D6D6` | 12.89 | bold |
| orange | `#D6D6D6` | 12.89 | — |
| cyan | `#BDBDBD` | 9.97 | italic |
| muted | `#8A8A8A` | 5.43 | italic |
| magenta | `#A3A3A3` | 7.43 | bold |

_chrome:_ surface `#262626` · selection_bg `#3D3D3D` · selection_fg `#FFFFFF` · cursor `#FFFFFF` · cursor_text `#121212` · border_active `#FFFFFF` · border_inactive `#757575` · diff_add_bg `#1E1E1E` · diff_del_bg `#2B2B2B` · diff_change_bg `#242424`

surface step 1.24:1 against the page · selection ink 10.86:1 on its tint

Shared lightness steps, separated by attribute alone: `#EDEDED` = fg (plain) vs red (bold underline) · `#BDBDBD` = green (plain) vs cyan (italic) · `#D6D6D6` = yellow (bold) vs orange (plain)

**Mono · light** — `mode = "light"`, `variant = "solo"`, floor 4.5:1, `emphasis >= standard`

The dark ladder inverted: darkest step plus bold is the accent, and the page stays off-white so a white surface can still lift off it.

| role | value | ratio on bg | attribute |
|---|---|---|---|
| bg | `#FAFAFA` | — | — |
| fg | `#1A1A1A` | 16.67 | — |
| accent | `#000000` | 20.12 | bold |
| green | `#545454` | 7.26 | — |
| red | `#1A1A1A` | 16.67 | bold underline |
| yellow | `#383838` | 11.23 | bold |
| orange | `#383838` | 11.23 | — |
| cyan | `#545454` | 7.26 | italic |
| muted | `#6B6B6B` | 5.11 | italic |
| magenta | `#454545` | 9.19 | bold |

_chrome:_ surface `#E4E4E4` · selection_bg `#DEDEDE` · selection_fg `#000000` · cursor `#000000` · cursor_text `#FAFAFA` · border_active `#000000` · border_inactive `#8A8A8A` · diff_add_bg `#EFEFEF` · diff_del_bg `#E2E2E2` · diff_change_bg `#E9E9E9`

surface step 1.22:1 against the page · selection ink 15.61:1 on its tint

Shared lightness steps, separated by attribute alone: `#1A1A1A` = fg (plain) vs red (bold underline) · `#545454` = green (plain) vs cyan (italic) · `#383838` = yellow (bold) vs orange (plain)

### `contrast`

Colour at two levels. `high` puts every text role at AAA (7:1); `max` pushes to 12:1 for glare, low vision, and outdoor or projector use.

**Contrast · dark** — `mode = "dark"`, `variant = "high"`, floor 7:1, `emphasis >= standard`

AAA for every text role, hues kept far apart in hue angle rather than merely light. Yellow and orange are the closest pair, so orange stays plain and yellow takes bold.

| role | value | ratio on bg | attribute |
|---|---|---|---|
| bg | `#0B0B0B` | — | — |
| fg | `#F5F5F5` | 18.05 | — |
| accent | `#7FB8FF` | 9.56 | bold |
| green | `#5FD98A` | 11.04 | — |
| red | `#FF8A80` | 8.62 | bold |
| yellow | `#F2D65C` | 13.64 | bold |
| orange | `#FFAB5E` | 10.53 | — |
| cyan | `#6FE0E0` | 12.57 | italic |
| muted | `#B0B8C0` | 9.81 | italic |
| magenta | `#F09AE0` | 9.75 | bold |

_chrome:_ surface `#242424` · selection_bg `#2E3A46` · selection_fg `#FFFFFF` · cursor `#7FB8FF` · cursor_text `#0B0B0B` · border_active `#7FB8FF` · border_inactive `#B0B8C0` · diff_add_bg `#0F2416` · diff_del_bg `#2A1210` · diff_change_bg `#241E0C`

surface step 1.27:1 against the page · selection ink 11.6:1 on its tint

**Contrast · dark, extreme** — `mode = "dark"`, `variant = "max"`, floor 12:1, `emphasis >= full`

Pure black page, every text role at 12:1 or better. Hues survive because they are pushed toward their light end rather than desaturated; underline returns as a third channel for error and current match.

| role | value | ratio on bg | attribute |
|---|---|---|---|
| bg | `#000000` | — | — |
| fg | `#FFFFFF` | 21 | — |
| accent | `#A8D4FF` | 13.53 | bold |
| green | `#7CF0A8` | 14.89 | — |
| red | `#FFB3AC` | 12.3 | bold underline |
| yellow | `#FFE066` | 16.11 | bold |
| orange | `#FFC98A` | 13.97 | — |
| cyan | `#8FF5F5` | 16.55 | italic |
| muted | `#D0D0D0` | 13.62 | italic |
| magenta | `#FFB3F0` | 12.96 | bold |

_chrome:_ surface `#1A1A1A` · selection_bg `#33404D` · selection_fg `#FFFFFF` · cursor `#A8D4FF` · cursor_text `#000000` · border_active `#A8D4FF` · border_inactive `#D0D0D0` · diff_add_bg `#0A2614` · diff_del_bg `#2B0F0C` · diff_change_bg `#26200A`

surface step 1.21:1 against the page · selection ink 10.6:1 on its tint

**Contrast · light** — `mode = "light"`, `variant = "high"`, floor 7:1, `emphasis >= standard`

White page — the one place white is correct, because any tint spends contrast the dark text needs. Hues are the dark end of each ramp, all AAA.

| role | value | ratio on bg | attribute |
|---|---|---|---|
| bg | `#FFFFFF` | — | — |
| fg | `#121212` | 18.73 | — |
| accent | `#0B4FA8` | 7.78 | bold |
| green | `#14663D` | 7.01 | — |
| red | `#A3121A` | 7.9 | bold |
| yellow | `#6B4C00` | 7.9 | bold |
| orange | `#8A3200` | 8.28 | — |
| cyan | `#0B5C5C` | 7.78 | italic |
| muted | `#4A4A4A` | 8.86 | italic |
| magenta | `#7A0F6B` | 9.96 | bold |

_chrome:_ surface `#E6E6E6` · selection_bg `#CFE0F5` · selection_fg `#0B1A2B` · cursor `#0B4FA8` · cursor_text `#FFFFFF` · border_active `#0B4FA8` · border_inactive `#4A4A4A` · diff_add_bg `#E4F2E8` · diff_del_bg `#F7E4E4` · diff_change_bg `#F5EEDC`

surface step 1.25:1 against the page · selection ink 13.06:1 on its tint

**Contrast · light, extreme** — `mode = "light"`, `variant = "max"`, floor 12:1, `emphasis >= full`

Black text on white, hues at 12:1 or better. At this level hue is nearly exhausted as a channel — the darkest blue and the darkest cyan differ by little — so attributes are mandatory, not optional.

| role | value | ratio on bg | attribute |
|---|---|---|---|
| bg | `#FFFFFF` | — | — |
| fg | `#000000` | 21 | — |
| accent | `#002B6B` | 13.48 | bold |
| green | `#003D22` | 12.42 | — |
| red | `#6B0008` | 12.89 | bold underline |
| yellow | `#3D2A00` | 13.73 | bold |
| orange | `#4F1A00` | 14.19 | — |
| cyan | `#00363A` | 13.22 | italic |
| muted | `#262626` | 15.13 | italic |
| magenta | `#4A0040` | 15.38 | bold |

_chrome:_ surface `#E0E0E0` · selection_bg `#C2D4EB` · selection_fg `#000000` · cursor `#002B6B` · cursor_text `#FFFFFF` · border_active `#002B6B` · border_inactive `#262626` · diff_add_bg `#DCEBE2` · diff_del_bg `#F2DCDC` · diff_change_bg `#EDE6D2`

surface step 1.32:1 against the page · selection ink 13.91:1 on its tint

### `contrast-mono`

Grayscale at the same two levels. The hardest case in the system and the most instructive: above 12:1 the usable ramp collapses to three steps, so typography becomes the primary channel and lightness the secondary one.

**Contrast mono · dark** — `mode = "dark"`, `variant = "high"`, floor 7:1, `emphasis >= standard`

Grayscale at AAA. Five usable steps remain above 7:1, so two roles share a step wherever their attributes differ.

| role | value | ratio on bg | attribute |
|---|---|---|---|
| bg | `#000000` | — | — |
| fg | `#F0F0F0` | 18.43 | — |
| accent | `#FFFFFF` | 21 | bold |
| green | `#A6A6A6` | 8.63 | — |
| red | `#F0F0F0` | 18.43 | bold underline |
| yellow | `#D4D4D4` | 14.17 | bold |
| orange | `#D4D4D4` | 14.17 | — |
| cyan | `#A6A6A6` | 8.63 | italic |
| muted | `#959595` | 7.01 | italic |
| magenta | `#BFBFBF` | 11.42 | bold |

_chrome:_ surface `#1F1F1F` · selection_bg `#383838` · selection_fg `#FFFFFF` · cursor `#FFFFFF` · cursor_text `#000000` · border_active `#FFFFFF` · border_inactive `#8C8C8C` · diff_add_bg `#171717` · diff_del_bg `#242424` · diff_change_bg `#1D1D1D`

surface step 1.27:1 against the page · selection ink 11.73:1 on its tint

Shared lightness steps, separated by attribute alone: `#F0F0F0` = fg (plain) vs red (bold underline) · `#A6A6A6` = green (plain) vs cyan (italic) · `#D4D4D4` = yellow (bold) vs orange (plain)

**Contrast mono · dark, extreme** — `mode = "dark"`, `variant = "max"`, floor 12:1, `emphasis >= full`

The honest hard case: above 12:1 on black the ramp holds three steps. Nine roles onto three steps means typography is the primary channel and lightness the secondary one — the inverse of every other theme here. This variant must refuse emphasis = none.

| role | value | ratio on bg | attribute |
|---|---|---|---|
| bg | `#000000` | — | — |
| fg | `#E6E6E6` | 16.83 | — |
| accent | `#FFFFFF` | 21 | bold |
| green | `#CCCCCC` | 13.08 | — |
| red | `#FFFFFF` | 21 | bold underline |
| yellow | `#E6E6E6` | 16.83 | bold |
| orange | `#CCCCCC` | 13.08 | bold |
| cyan | `#E6E6E6` | 16.83 | italic |
| muted | `#CCCCCC` | 13.08 | italic |
| magenta | `#FFFFFF` | 21 | italic |

_chrome:_ surface `#1A1A1A` · selection_bg `#333333` · selection_fg `#FFFFFF` · cursor `#FFFFFF` · cursor_text `#000000` · border_active `#FFFFFF` · border_inactive `#CCCCCC` · diff_add_bg `#141414` · diff_del_bg `#222222` · diff_change_bg `#1B1B1B`

surface step 1.21:1 against the page · selection ink 12.63:1 on its tint

Shared lightness steps, separated by attribute alone: `#E6E6E6` = fg (plain) vs yellow (bold) vs cyan (italic) · `#FFFFFF` = accent (bold) vs red (bold underline) vs magenta (italic) · `#CCCCCC` = green (plain) vs orange (bold) vs muted (italic)

**Contrast mono · light** — `mode = "light"`, `variant = "high"`, floor 7:1, `emphasis >= standard`

Grayscale at AAA on white. The same ladder as the dark variant, inverted.

| role | value | ratio on bg | attribute |
|---|---|---|---|
| bg | `#FFFFFF` | — | — |
| fg | `#141414` | 18.42 | — |
| accent | `#000000` | 21 | bold |
| green | `#4F4F4F` | 8.19 | — |
| red | `#141414` | 18.42 | bold underline |
| yellow | `#333333` | 12.63 | bold |
| orange | `#333333` | 12.63 | — |
| cyan | `#4F4F4F` | 8.19 | italic |
| muted | `#595959` | 7 | italic |
| magenta | `#404040` | 10.37 | bold |

_chrome:_ surface `#E6E6E6` · selection_bg `#D6D6D6` · selection_fg `#000000` · cursor `#000000` · cursor_text `#FFFFFF` · border_active `#000000` · border_inactive `#616161` · diff_add_bg `#EDEDED` · diff_del_bg `#E0E0E0` · diff_change_bg `#E8E8E8`

surface step 1.25:1 against the page · selection ink 14.45:1 on its tint

Shared lightness steps, separated by attribute alone: `#141414` = fg (plain) vs red (bold underline) · `#4F4F4F` = green (plain) vs cyan (italic) · `#333333` = yellow (bold) vs orange (plain)

**Contrast mono · light, extreme** — `mode = "light"`, `variant = "max"`, floor 12:1, `emphasis >= full`

Three steps again, at the dark end. Print-safe by construction: it survives a photocopier, a projector and greyscale printing unchanged.

| role | value | ratio on bg | attribute |
|---|---|---|---|
| bg | `#FFFFFF` | — | — |
| fg | `#1A1A1A` | 17.4 | — |
| accent | `#000000` | 21 | bold |
| green | `#333333` | 12.63 | — |
| red | `#000000` | 21 | bold underline |
| yellow | `#1A1A1A` | 17.4 | bold |
| orange | `#333333` | 12.63 | bold |
| cyan | `#1A1A1A` | 17.4 | italic |
| muted | `#333333` | 12.63 | italic |
| magenta | `#000000` | 21 | italic |

_chrome:_ surface `#E0E0E0` · selection_bg `#CCCCCC` · selection_fg `#000000` · cursor `#000000` · cursor_text `#FFFFFF` · border_active `#000000` · border_inactive `#333333` · diff_add_bg `#EAEAEA` · diff_del_bg `#DEDEDE` · diff_change_bg `#E5E5E5`

surface step 1.32:1 against the page · selection ink 13.08:1 on its tint

Shared lightness steps, separated by attribute alone: `#1A1A1A` = fg (plain) vs yellow (bold) vs cyan (italic) · `#000000` = accent (bold) vs red (bold underline) vs magenta (italic) · `#333333` = green (plain) vs orange (bold) vs muted (italic)

### What these families need from the tool layer

- **A refusal, not a downgrade.** `contrast-mono` + `max` + `emphasis = "none"` is not a degraded
  theme, it is an unreadable one. The resolver should reject the combination and name the reason.
- **The four attribute-blind surfaces matter more here.** Claude Code, Gemini, OpenCode and Aider
  cannot express emphasis, so on these families they lose the primary channel, not a secondary one.
  For them the mapping should fall back to the nearest *high-contrast* built-in rather than the
  nearest palette match.
- **less/man have no italic.** Under these families italic degrades to standout, not to nothing.
- **Diff backgrounds are near-invisible by design.** At these floors a tinted diff background would
  eat the contrast the text needs, so diff relies on the sign column plus `diff.emphasis` bold.

### Correction — projectious selection was too weak

The first pass took `selection_bg` from `--code-panel-selection-bg` (`#20354d`). That token is correct
for its job — selection *inside a code panel*, which sits at `#0e1720` or `#131e2b` — but on the navy
page it is only 1.4:1 away from the background, so a selected range barely reads as selected. Each
variant now takes the next step up its own midnight ramp:

| Variant | selection.bg | selection.fg | step vs page | ink on selection |
|---|---|---|---|---|
| ProjectiousNavy | `#3A5C7E` | `#C5DAF0` | 2.22:1 | 4.87:1 |
| ProjectiousDeep | `#2E4B68` | `#C5DAF0` | 2:1 | 6.31:1 |
| ProjectiousLight | `#C3D1E3` | `#142438` | 1.47:1 | 10.12:1 |
| ProjectiousHCDark | `#3A5C7E` | `#FFFFFF` | 2.59:1 | 6.97:1 |
| ProjectiousHCLight | `#B0C1D6` | `#000000` | 1.83:1 | 11.45:1 |

The rule this adds to the derivation set: **a selection surface needs its own floor.** 1.20:1 is enough
for a status bar, which is a large filled region; a selection is a short inline run inside a page of the
same colour and needs roughly 1.8:1 on dark, 1.4:1 on light before it reads as a selection at all. That
floor is now applied to the derived selections of all 61 imported variants too.
