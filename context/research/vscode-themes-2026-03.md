# VS Code Themes Audit and Expansion Research

**Date:** 2026-03-26
**Purpose:** Audit current aibox theme lineup and identify candidate additions

---

## 1. Current aibox Theme Inventory

aibox ships 7 themes, each applied across 5 tools (Zellij, Vim, Yazi, lazygit, Starship):

| Theme            | Palette Type   | Background | Notes                        |
|------------------|---------------|------------|------------------------------|
| gruvbox-dark     | Warm retro     | Dark       | Default theme. Earthy tones  |
| catppuccin-mocha | Pastel         | Dark       | Soothing pastels on dark bg  |
| catppuccin-latte | Pastel         | Light      | Only light theme in lineup   |
| dracula          | High contrast  | Dark       | Purple-heavy, vibrant        |
| tokyo-night      | Cool neon      | Dark       | Blues and purples, modern     |
| nord             | Cool muted     | Dark       | Nordic frost palette          |
| projectious      | Cool branded   | Dark       | Custom brand theme            |

### Palette Gap Analysis

- **Warm dark:** gruvbox-dark (strong)
- **Cool dark:** tokyo-night, nord (well covered)
- **Pastel dark:** catppuccin-mocha (good)
- **High contrast dark:** dracula (good)
- **Light:** catppuccin-latte (only one)
- **Green/nature dark:** MISSING
- **Warm muted dark:** MISSING (gruvbox is warm but high-saturation)
- **Japanese/ink aesthetic:** MISSING
- **Classic/legacy schemes:** MISSING (no Solarized, no Monokai, no Atom/OneDark)

---

## 2. Top VS Code Themes by Downloads (2025-2026)

Ranked by approximate VS Code marketplace install counts:

| Rank | Theme              | Approx. Installs | Notes                                    |
|------|--------------------|-------------------|------------------------------------------|
| 1    | One Dark Pro       | ~12.5M+           | Most installed dark theme overall         |
| 2    | GitHub Theme       | ~8.4M+            | Multiple variants (dark, light, dimmed)   |
| 3    | Dracula Official   | ~5.1M+            | Already in aibox                          |
| 4    | Material Theme     | ~2.3M+            | Google Material Design inspired           |
| 5    | Catppuccin         | ~2M+ (estimated)  | Already in aibox (mocha + latte)          |
| 6    | Ayu                | ~2.5M+            | Three variants: dark, light, mirage       |
| 7    | Shades of Purple   | ~2M+              | Bold purple, single variant               |
| 8    | Night Owl          | ~1.9M+            | Accessibility-focused, by Sarah Drasner   |
| 9    | Monokai Pro        | ~2M+              | Paid (6 filter variants)                  |
| 10   | One Monokai        | ~1.7M+            | Free Monokai + OneDark hybrid             |
| 11   | Tokyo Night        | ~1.5M+ (est.)     | Already in aibox                          |
| 12   | Cobalt2            | ~1.6M+            | Bold blue, by Wes Bos                     |
| 13   | Nord               | ~1M+ (est.)       | Already in aibox                          |
| 14   | Gruvbox Theme      | ~800K+ (est.)     | Already in aibox                          |
| 15   | Rose Pine          | ~500K+ (est.)     | Growing fast, 213 ecosystem ports         |

### Rising Themes (Neovim-first, growing in VS Code)

| Theme       | Neovim GitHub Stars | VS Code Status           | Trend         |
|-------------|--------------------:|--------------------------|---------------|
| Kanagawa    | ~5,500              | Community ports available | Rising fast   |
| Everforest  | ~3,700 (vim) / ~3,800 (nvim) | Official extension | Steady growth |
| Rose Pine   | ~2,900              | Official extension       | Strong growth |
| Nightfox    | ~3,800              | Limited VS Code presence | Niche         |

---

## 3. Candidate Theme Evaluation

### Tier 1: Strong Recommendations

#### Rose Pine -- RECOMMENDED

- **VS Code:** Official extension (mvllow.rose-pine), estimated 500K+ installs, 3 variants (main, moon, dawn)
- **Vim:** Official port (rose-pine/vim) + Neovim (rose-pine/neovim, 2,900 stars)
- **Zellij:** Official port (rose-pine/zellij) with all 3 variants
- **Yazi:** Community port available
- **lazygit:** Custom theme via gui.theme config (colors well-documented)
- **Starship:** Color values documented, easy to create palette
- **Ecosystem:** 213 ports across tools -- one of the largest theme ecosystems, comparable to Catppuccin
- **Maintenance:** Actively maintained org (github.com/rose-pine), regular updates through 2025
- **Visual gap filled:** Warm muted rose/pink palette -- completely distinct from everything in aibox
- **DHH's Omakub** ships Rose Pine as one of 4 alt themes alongside Gruvbox, Everforest, Catppuccin
- **Verdict: ADD.** Excellent multi-tool coverage, fills a unique warm-muted niche, strong community.

#### Everforest -- RECOMMENDED

- **VS Code:** Official extension (sainnhe.everforest), dark and light variants with soft/medium/hard contrast
- **Vim:** Original home (sainnhe/everforest, ~3,700 stars) -- mature, well-maintained
- **Zellij:** Community port (everforest-dark-zellij), color values available
- **Yazi:** Part of everforest_collection (neuromaancer/everforest_collection)
- **lazygit:** Custom theme feasible (green-based palette well-documented)
- **Starship:** Color values documented
- **Ecosystem:** Growing collection of ports, featured in Omakub
- **Maintenance:** Active (last Vim update Dec 2025), same author as sonokai and edge themes
- **Visual gap filled:** Nature-inspired green/olive palette -- warm but desaturated, unlike gruvbox
- **Verdict: ADD.** Fills the "green/nature" gap, excellent Vim pedigree, easy to implement across all 5 tools.

#### Kanagawa -- RECOMMENDED

- **VS Code:** Multiple community ports (qufiwefefwoyn.kanagawa being most popular)
- **Vim:** kanagawa.nvim (rebelot, ~5,500 stars) -- very popular in Neovim community
- **Zellij:** JSON terminal colors available, Kanagawa Paper variant includes Zellij theme
- **Yazi:** Community theme available
- **lazygit:** Custom theme feasible from documented palette
- **Starship:** Color values well-documented
- **Ecosystem:** Moderate (not as large as Rose Pine/Catppuccin, but growing)
- **Maintenance:** Active, multiple community variants (kanagawa-paper, kanagawa-dragon)
- **Visual gap filled:** Japanese ink-painting aesthetic -- deep indigo/navy with warm accents, nothing like it in current lineup
- **Verdict: ADD.** Unique aesthetic, very popular in terminal/Neovim crowd (our target users), fills the Japanese ink niche.

### Tier 2: Worth Considering

#### One Dark (OneDark Pro) -- CONDITIONAL RECOMMEND

- **VS Code:** One Dark Pro has ~12.5M installs -- single most popular dark theme
- **Vim:** joshdick/onedark.vim (~3,980 stars), olimorris/onedarkpro.nvim, navarasu/onedark.nvim
- **Zellij:** Colors available in onedarkpro.nvim extras folder
- **Yazi:** Would need to be created (straightforward from palette)
- **lazygit:** Custom theme feasible
- **Starship:** Feasible from palette
- **Ecosystem:** Massive (Atom legacy, VS Code giant)
- **Visual gap filled:** Moderate -- similar cool-dark territory as Tokyo Night but warmer/more muted
- **Concern:** Visually quite close to Tokyo Night. Both are cool-toned dark themes with blue/purple accents. Adding both may feel redundant.
- **Verdict: CONSIDER as 4th addition.** Huge name recognition, but overlaps with Tokyo Night aesthetically. If adding 5 themes, include it; if adding 3-4, skip it.

#### Solarized Dark -- CONDITIONAL RECOMMEND

- **VS Code:** Built-in to VS Code (no extension needed), also available as extensions
- **Vim:** Built-in classic, solarized8 (lifepillar), solarized-osaka.nvim (craftzdog, trending)
- **Zellij:** Built-in (ships with Zellij as default theme option)
- **Yazi:** Community port (Solarized Osaka)
- **lazygit:** Custom theme feasible
- **Starship:** Community preset exists
- **Ecosystem:** One of the oldest and most ported schemes (created 2011, Ethan Schoonover)
- **Visual gap filled:** Unique blue-grey base with precisely calibrated warm/cool accents
- **Concern:** Shows its age visually. Many developers have moved on. Dark variant can feel low-contrast.
- **Verdict: SKIP for now.** Iconic but declining in popularity. The Solarized Osaka variant is more modern but fragmentary. Can revisit if users request it.

### Tier 3: Evaluated but Not Recommended

#### Monokai Pro

- **VS Code:** ~2M installs, 6 filter variants, 3.5-star rating
- **Vim:** Multiple community ports (loctvl842/monokai-pro.nvim, phanviet/vim-monokai-pro)
- **Concern:** The official Monokai Pro is a **paid theme** ($9.95). Free community ports exist but vary in quality and fidelity. This creates licensing ambiguity for bundling.
- **Visual overlap:** High-contrast with warm accents -- similar territory to Dracula
- **Verdict: SKIP.** Paid licensing model is a poor fit for bundling. Dracula already covers the high-contrast vibrant niche.

#### Ayu

- **VS Code:** ~2.5M installs, 3 variants (dark, light, mirage)
- **Vim:** ayu-vim exists but less actively maintained
- **Zellij/lazygit/Yazi:** Limited ecosystem ports
- **Concern:** Thin multi-tool ecosystem compared to Rose Pine or Catppuccin. The "mirage" variant is the most distinctive, but dark/light overlap with existing themes.
- **Verdict: SKIP.** Weaker multi-tool ecosystem. Rose Pine fills a similar "warm muted" niche better.

#### Night Owl

- **VS Code:** ~1.9M installs, accessibility-focused design
- **Vim:** Community port exists (haishanh/night-owl.vim)
- **Concern:** Deep blue base overlaps significantly with Tokyo Night and Kanagawa territory
- **Verdict: SKIP.** Too close to Tokyo Night + Kanagawa. Not enough visual distinction to justify a slot.

#### GitHub Theme

- **VS Code:** ~8.4M installs (2nd most popular)
- **Vim:** projekt0n/github-nvim-theme exists
- **Concern:** Designed for GitHub's UI context. Terminal versions feel less cohesive. Light variant overlaps with catppuccin-latte territory.
- **Verdict: SKIP.** Strong in VS Code but not a natural fit for terminal-first tooling.

#### Material Theme

- **VS Code:** ~2.3M installs
- **Vim:** Community ports exist
- **Concern:** Google Material Design aesthetic doesn't translate well to terminal tools. Many variants create choice overload.
- **Verdict: SKIP.** Poor terminal fit.

#### Shades of Purple

- **VS Code:** ~2M installs
- **Vim:** Limited ports
- **Concern:** Very niche aesthetic, thin ecosystem outside VS Code
- **Verdict: SKIP.** Too niche, poor multi-tool support.

---

## 4. Comparison Table: All Candidates

| Theme          | VS Code Installs | Vim Scheme | Zellij Port | Yazi Port | lazygit | Starship | Ecosystem Size | Distinct from Current? | Recommendation |
|----------------|:----------------:|:----------:|:-----------:|:---------:|:-------:|:--------:|:--------------:|:---------------------:|:--------------:|
| Rose Pine      | ~500K+           | Yes (official) | Yes (official) | Yes  | Feasible | Feasible | 213 ports      | Yes (warm rose)       | **ADD**        |
| Everforest     | ~300K+ (est.)    | Yes (original home) | Yes (community) | Yes | Feasible | Feasible | Growing        | Yes (green/nature)    | **ADD**        |
| Kanagawa       | ~200K+ (est.)    | Yes (5.5K stars) | Yes (community) | Yes | Feasible | Feasible | Moderate       | Yes (ink/indigo)      | **ADD**        |
| One Dark       | ~12.5M           | Yes (4K stars) | Yes (extras) | Needs work | Feasible | Feasible | Massive        | Partial (vs Tokyo Night) | CONSIDER    |
| Solarized      | Built-in         | Yes (classic) | Yes (built-in) | Yes  | Feasible | Feasible | Massive        | Yes (blue-grey)       | SKIP (aging)   |
| Monokai Pro    | ~2M              | Yes (community) | Limited   | Limited   | Feasible | Feasible | Moderate       | Partial (vs Dracula)  | SKIP (paid)    |
| Ayu            | ~2.5M            | Yes (aging) | Limited     | Limited   | Feasible | Feasible | Small          | Partial               | SKIP           |
| Night Owl      | ~1.9M            | Yes (community) | Limited  | Limited   | Feasible | Feasible | Small          | No (vs Tokyo Night)   | SKIP           |
| GitHub Theme   | ~8.4M            | Yes        | Limited     | Limited   | Feasible | Feasible | Small          | Partial               | SKIP           |

---

## 5. Current Theme Marketplace/Open-VSIX Availability

All 7 current aibox themes are available in the VS Code marketplace and/or Open VSX:

| aibox Theme      | VS Code Marketplace Extension          | Open VSX Available |
|------------------|----------------------------------------|--------------------|
| gruvbox-dark     | jdinhlife.gruvbox, others              | Yes                |
| catppuccin-mocha | Catppuccin.catppuccin-vsc              | Yes                |
| catppuccin-latte | Catppuccin.catppuccin-vsc              | Yes                |
| dracula          | dracula-theme.theme-dracula            | Yes                |
| tokyo-night      | enkia.tokyo-night                      | Yes                |
| nord             | arcticicestudio.nord-visual-studio-code| Yes                |
| projectious      | N/A (custom/brand theme)               | N/A                |

All recommended additions (Rose Pine, Everforest, Kanagawa) also have VS Code marketplace extensions and are available on Open VSX.

---

## 6. Final Recommendations

### Add These 3 Themes (Primary)

1. **Rose Pine** (main variant, dark) -- Warm muted rose/pink/gold on dark base. Largest ecosystem of ports (213). Fills the "warm muted" gap.
2. **Everforest** (dark medium) -- Green/olive nature palette. Originated in Vim, excellent terminal pedigree. Fills the "nature/green" gap.
3. **Kanagawa** (wave variant, dark) -- Japanese ink aesthetic with deep indigo base and warm foreground accents. Hugely popular in Neovim community (our target audience). Fills the "artistic/ink" gap.

### Optional 4th Addition

4. **One Dark** -- Only if demand warrants it. Massive VS Code install base provides name recognition, but visually close to Tokyo Night.

### Updated Lineup (10 themes)

After additions, the lineup covers a much broader spectrum:

| Theme            | Palette Character         | Temperature | Contrast  |
|------------------|--------------------------|-------------|-----------|
| gruvbox-dark     | Warm retro earth         | Warm        | Medium    |
| catppuccin-mocha | Soothing pastel          | Neutral     | Low-Med   |
| catppuccin-latte | Light pastel             | Neutral     | Medium    |
| dracula          | Vibrant purple           | Cool        | High      |
| tokyo-night      | Neon cityscape           | Cool        | Medium    |
| nord             | Arctic frost             | Cool        | Low       |
| projectious      | Brand blue               | Cool        | Medium    |
| **rose-pine**    | **Warm rose/pink muted** | **Warm**    | **Low**   |
| **everforest**   | **Green/olive nature**   | **Warm**    | **Low-Med** |
| **kanagawa**     | **Japanese ink indigo**  | **Mixed**   | **Medium** |

### Implementation Notes

- All 3 recommended themes have existing Vim colorschemes that can be bundled as single `.vim` files
- Zellij themes can be defined inline in KDL format (same as current themes)
- Yazi themes exist for all 3 or can be created from documented palettes
- lazygit themes are custom YAML configs using hex colors (same pattern as current themes)
- Starship palettes just need bg/fg/accent/green/red values from the theme palette
- Rose Pine dawn variant could serve as a second light theme option in the future

---

## Sources

- [GitKraken: 10 Popular VS Code Color Themes 2025](https://www.gitkraken.com/blog/10-best-vs-code-color-themes-2025)
- [GeeksforGeeks: 20 Best VSCode Themes 2025](https://www.geeksforgeeks.org/blogs/best-vscode-themes/)
- [Snappify: 19 Best VSCode Themes 2025](https://snappify.com/blog/best-vscode-themes)
- [DevDreaming: Top 10 Popular VS Code Themes 2025](https://devdreaming.com/blogs/top-10-popular-vs-code-themes)
- [Jit: 20 Best VS Code Themes 2026](https://www.jit.io/blog/best-vs-code-themes-2023)
- [Hackr.io: 20 Best VSCode Themes 2026](https://hackr.io/blog/best-vscode-themes)
- [Rose Pine Theme Directory](https://rosepinetheme.com/themes/)
- [Rose Pine Zellij](https://github.com/rose-pine/zellij)
- [Rose Pine Neovim](https://github.com/rose-pine/neovim)
- [Rose Pine Vim](https://github.com/rose-pine/vim)
- [Everforest Vim (sainnhe)](https://github.com/sainnhe/everforest)
- [Everforest Collection](https://github.com/neuromaancer/everforest_collection)
- [Everforest Zellij](https://github.com/n1yn/everforest-dark-zellij)
- [Kanagawa.nvim](https://github.com/rebelot/kanagawa.nvim)
- [Kanagawa VS Code](https://marketplace.visualstudio.com/items?itemName=qufiwefefwoyn.kanagawa)
- [OneDark.vim](https://github.com/joshdick/onedark.vim)
- [OneDarkPro.nvim](https://github.com/olimorris/onedarkpro.nvim)
- [Solarized (Schoonover)](https://github.com/altercation/solarized)
- [Monokai Pro](https://monokai.pro/)
- [Dotfyle: Top Neovim Colorschemes 2026](https://dotfyle.com/neovim/colorscheme/top)
- [Slant: 28 Best Vim Color Schemes 2025](https://www.slant.co/topics/480/~best-vim-color-schemes)
- [DHH on Omakub theme choices](https://x.com/dhh/status/1798501261605834789)
- [Zellij Theme List](https://zellij.dev/documentation/theme-list.html)
