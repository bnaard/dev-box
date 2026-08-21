# projectious.work — Design System

A complete design system for **projectious.work**, a virtual IT consulting company built around three practice areas — **Cloud · Agile · Agentic AI** — and one radical premise: in the era of agentic AI, the way companies work is about to be fundamentally redesigned.

This system contains the brand foundations (logo, color, type), low-level tokens (spacing, radius, elevation, motion, breakpoints, terminal and syntax palettes), high-level UI components, slide templates, and SKILL.md so the system works equally well from this project or as a portable Agent Skill in Claude Code.

> **Synced from upstream v2.1.1** (`projectious-work/brand`, 2026-08-10). `colors_and_type.css` mirrors `brand/tokens/variables.css`, which is generated from `src/data/brand.yaml`. Change values upstream and re-sync — do not hand-edit token values here.

---

## Brand snapshot

- **Name:** projectious.work
- **Tagline:** *Redesigning work*
- **Positioning:** Cloud · Agile · Agentic AI — agent-first consulting
- **Mission:** Augment people's strengths to improve life for everyone.
- **Vision:** Humanistic ideals determine actions in an AI-completed world.
- **Convictions:**
  - *Do more with more* — augment people's strengths so the same or more people produce more, higher quality, for broader markets.
  - *Specialized beats generic now* — agents collapse the cost of bespoke software. Lot-of-one systems — built for one team's exact workflow — are now economically rational. Underserved companies stop bending themselves to fit generic SaaS or Excel.
  - *Provider independence* — agnostic, self-hosted where needed. No single AI vendor becomes a single point of failure.
  - *We run what we recommend* — the company eats its own dogfood.
- **Voice keywords:** Modern · Composable · Authentic · No-nonsense · Agent-first · Humanistic
- **Audience:** Technical leaders (CTOs, VPs Eng, platform leads) at European/international organizations adopting AI-native workflows.

---

## Sources used to build this system

This design system mirrors the **`projectious-work/brand`** GitHub repository (default branch: `main`), published as documentation at <https://projectious-work.github.io/brand/>.

| Source | Path | Used for |
|---|---|---|
| Token exports | `brand/tokens/variables.css`, `tokens.json`, `tailwind.config.js` | All 273 custom properties — scales in both modes, semantics, surfaces, type ramp, breakpoints, terminal |
| Brand README | `brand/README.md` | Brand description, font and color summary, license info |
| Changelog | `CHANGELOG.md` | What changed between releases and why — the record of breaking token changes |
| Master brand system | `brand/html/projectious-brand-system-final.html` | Theme narrative, font roles, color scales, spacing/grid, component tokens |
| Component kitchen sink | `brand/html/projectious-kitchen-sink.html` | Full component library in light + dark modes |
| Logo system | `brand/html/projectious-logo-system-v3.html` | The four lockups and their construction |
| Slide deck templates | `brand/html/projectious-slides.html` | Slide types and rules |
| Digital experience | `brand/html/projectious-digital-experience.html` | Syntax theme, dark-mode strategy, Lucide icon rules, form patterns |
| Motion & media | `brand/html/projectious-motion-media.html` | Motion, audio, video, photography, presentations |
| Worked examples | `brand/examples/` | Dashboard, mobile flow, slide deck, CV, one-pager, diagram gallery, newsletter |
| Logo assets | `brand/logo/svg/`, `png-1x/`, `favicon/` | Imported into `assets/logo/` |
| Governance | `brand/PROVENANCE.md`, `LICENSE.md`, `TRADEMARK.md` | Licensing split, trademark rules, per-asset provenance |

The reader is **not assumed** to have access to these sources — everything needed lives inside this project. Paths above are recorded for traceability.

---

## Index — what's in this folder

| File / folder | What it is |
|---|---|
| `README.md` | This file — start here |
| `SKILL.md` | Agent Skill manifest (works in Claude Code too) |
| `upstream-sync-note.md` | Local changes not in upstream `brand.yaml` — port these before the next sync |
| `styles.css` | Entry point — link this one file |
| `colors_and_type.css` | Token sheet mirroring upstream `variables.css`, plus base element styles |
| `assets/logo/` | All logo assets — SVG (scalable), PNG (1×), monochrome variants, favicons |
| `preview/` | Small HTML cards used by the Design System tab in this project |
| `ui_kits/marketing-site/` | UI kit recreating the projectious.work marketing/landing experience |
| `ui_kits/agent-console/` | UI kit recreating an agent-first ops console (dashboards, pipelines, terminals) |
| `slides/` | Slide templates rendered as 1280×720 HTML files |

---

## Components

Two UI kits ship as exported React components. Both are composed on their kit's `index.html`.

**`ui_kits/marketing-site/`** — the marketing and landing surface.

| Component | What it is |
|---|---|
| `Header` | Sticky marketing header — mark, lockup, nav, one accent CTA. Takes `dark`. |
| `Hero` | Landing hero — display headline, positioning line, paired actions. |
| `Pillars` | Three-up practice-area cards: Cloud, Agile, Agentic AI. |
| `CodeShowcase` | Two-column section pairing prose with a dark terminal panel. |
| `Convictions` | Numbered conviction list, one rule per row. |
| `CTA` | Closing call to action on a midnight panel. |
| `Footer` | Site footer — brand mark left, page metadata right. |

**`ui_kits/agent-console/`** — an agent-first ops console. Every component takes a `theme` object from `THEMES` in `theme.jsx`, which carries the light and dark variants.

| Component | What it is |
|---|---|
| `Sidebar` | Brand lockup, grouped navigation, user chip. |
| `Topbar` | View title, search, and the colour-mode control. |
| `PipelineList` | Selectable table of pipelines with status, owner, run count. |
| `StatusPill` | Status chip — solid tint plus its on-tint foreground. |
| `RunDetail` | Detail panel for one run — stages, metadata, log output. |
| `StatusBar` | Bottom bar — connection, region, build metadata. |

---

## Content fundamentals — how copy is written

projectious.work writes like an honest principal engineer who has stopped trying to impress. The voice is **direct, technical, humanistic, restrained**, and never marketing-speak.

**Tone**
- *No-nonsense.* Plain words first, jargon only when it's the most precise word.
- *Confident, not loud.* Conviction comes from the claim, not from punctuation or capitalization.
- *Humanistic.* The point of better systems is better life — say it.
- *Agent-first but neutral.* Don't gush about AI; treat agents as a normal part of how good work happens now.

**Casing**
- Sentence case for everything except proper nouns: *"Redesigning work"*, *"Why agent-first?"*, never *"Why Agent-First?"*
- The brand name is lowercase: **projectious.work** — never *Projectious* or *PROJECTIOUS*. The dot is part of the name.
- Practice areas are capitalized as proper nouns: **Cloud**, **Agile**, **Agentic AI**. Joined with a `·` middle dot, never an `&`: *"Cloud · Agile · Agentic AI"*.

**Pronouns and stance**
- *We* (the company) and *you* (the reader). Never *I*. Never *the team*.
- Address the reader's situation, not the reader's feelings: *"You ship the same software with continuous audit"*, not *"You'll feel confident"*.
- State opinions plainly. The brand has a thesis — write like it.

**Punctuation and rhythm**
- Em dashes for asides — used sparingly, paired with spaces in body, tight in headlines.
- The middle dot `·` separates practice areas and inline metadata: *"v2.1.0 · Released today"*.
- Sentences are short to medium. Avoid commas-stacked sentences.

**Emoji**
- Not used in product UI, marketing copy, slides, or technical writing.
- Acceptable in informal Slack/standup contexts only — not part of the brand surface.

**Numerals**
- Numbers as digits in technical contexts (*"3 stages"*, *"40 px height"*).
- Spell out one through nine in prose unless paired with a unit.

**Examples — voice in action**

> ✅ **Redesigning work**
> *Cloud · Agile · Agentic AI. Agent-first consulting. We run what we recommend.*

> ✅ **Why agent-first?**
> *Do more with more — same or more people, broader output. Quality is non-negotiable — agents audit continuously. Provider independent — no vendor lock-in.*

> ✅ **Specialized beats generic now.**

> ❌ Avoid: *"Unleash the power of AI to revolutionize your team's velocity 🚀"*

**Stock taglines (drop-in, in priority order)**
- Cloud · Agile · Agentic AI *(primary)*
- Redesigning work *(alternative)*
- Agent-first consulting, built on what we teach *(long-form)*

---

## Visual foundations

The visual system is **disciplined, dual-mode, and quietly opinionated**. It looks like infrastructure — readable, repairable, no decorative noise.

### Three appearances

The system supports **three** appearances, not two:

| Appearance | Selector | Page | Raised | Subtle | Border |
|---|---|---|---|---|---|
| Light | `[data-theme="light"]` | `#f8f9fb` | `#ffffff` | `#f0f3f8` | `#cdd0d5` |
| Navy dark *(default dark)* | `[data-theme="dark"]` | `#132440` | `#1a2b3e` | `#20354d` | `#2e4b68` |
| Deep dark | `[data-theme="dark"][data-surface="deep"]` | `#0e1720` | `#131e2b` | `#1a2b3e` | `#263f5a` |

With no explicit `data-theme`, the system follows `prefers-color-scheme`; dark resolves to **navy dark**, and `data-surface="deep"` opts back into deep dark.

**Navy is the default dark, not deep dark.** Deep dark sits at the very bottom of the ramp, which leaves panels almost no room above the page and reads as heavy across a whole interface. Navy starts higher, so raised surfaces and code panels separate properly while the appearance still reads as dark. Deep dark remains a full, supported appearance — it is simply not what anyone sees first. *(This inverts the upstream briefing's default; see `upstream-sync-note.md`.)*

**Navy dark is a documented derivative of deep dark, not a separate theme.** It overrides only midnight steps 1–5 and the surfaces, borders and neutral tag background that read off them. Orange and slate, every text and status role, the terminal palette and the syntax palette are inherited from deep dark unchanged. Code and terminal panels deliberately stay on `#0e1720` — deeper than the navy page — so a code block still reads as an inset panel instead of merging into the surface behind it.

Because navy is now the default, the *deep* block is the one that carries the overrides. `colors_and_type.css` prints the complete list as an **override manifest** above that block — fourteen names — so a consumer can check "did I miss one?" mechanically instead of by eye.

### Color
- **Three-color core.** Midnight (`#1d3352`) for authority and depth. Tempered orange (`#e05232`) for moments of decision. Slate (`#546a82`) for connective tissue.
- **12-step scales** (Radix convention) for midnight, orange, and slate, in both light and dark modes — see `colors_and_type.css`. Step 9 is the saturated brand color and stays constant across modes.
- **The light app background is `midnight-1` (`#f8f9fb`)**, not white. Step 1 is the app background in the step roles. White (`#fff`) returns as the **raised surface**, so a card lifts off the page instead of separating by border alone. *(Changed in upstream v2.0.0 — the previous warm paper `#f5f4f2` and white page background are both retired.)*
- **Step roles are binding.** 1 app bg · 2 subtle bg · 3 elem bg · 4 hover bg · 5 active bg · 6 subtle border · 7 border · 8 strong border · 9 solid bg · 10 solid hover · 11 low text · 12 high text. Only 11 and 12 are text roles — a border step never gets used as text.
- **The accent is never body text.** `#E05232` measures 3.87:1 on white. Accent text takes `orange-11` (`#c04424`) on light, `accent-light` (`#ea7558`) on dark. `#E05232` remains the identity colour for marks, borders, active states and syntax.
- **Solid accent fills carrying white text use `--color-accent-solid` (`#cc4528`)**, 4.72:1 — not `orange-9`.
- **Tag pairs are text-role over element-background.** `--tag-bg` / `--tag-text` and `--tag-bg-accent` / `--tag-text-accent`: step 3 as the tint, step 12 as the ink on light (8.54:1 for the accent pair — step 11 lands at 4.10 and was corrected). Solid pale tint, no outline.
- **Semantic colours ship as a triple:** solid, tint (`-bg`), and on-tint foreground (`-fg`). Use `-fg` on the tint; the solid does not always clear AA against its own tint. Dark mode re-picks all three rather than reusing the light values.
- **Links take `midnight-11`**, not the accent.
- **No bluish-purple gradients.** No emoji-decorated cards. No rainbow categorization. Color is meaning, not garnish.

### Typography
- **Three families, three roles.** Plus Jakarta Sans = conviction (display, headings, buttons). Source Sans 3 = clarity (body, UI, descriptions). IBM Plex Mono = precision (code, data, terminal output, version strings).
- **Never mix roles.** The rule of thumb: *Is this guiding attention (Jakarta), being read (Source Sans), or showing system output (Plex Mono)?*
- **Tight letter-spacing on display** (`-0.5px`), normal on body, mild positive tracking only on overline labels.
- **Maximum two type sizes per slide.**

### Spacing & layout
- **4 px base** scale: 4, 8, 12, 16, 24, 32, 48, 64, 96.
- **12-column grid**, 16 px gutter, max content width **1100 px**.
- Slides are designed at **1280×720** (16:9).
- Padding and gap are explicit — never lean on margin collapsing or whitespace text nodes for layout.

### Backgrounds & imagery
- **No full-bleed photography** as default. No hand-drawn illustrations. No textures. No grain.
- **Warm paper** for the page, **white** for surfaces, **midnight-dark** for hero/dark panels.
- **Diagrams over photos.** When imagery is needed, prefer abstract system diagrams in the palette. Photography (when used) is candid, desaturated, warm-graded — real workspaces, never stock.
- **Accent decoration** is rare: a top-right circular orange "glow" at 6% opacity on dark hero panels is the one approved decorative motif.

### Borders & corners
- **Borders** are `slate-5` (`#cdd0d5`) light, `midnight-dark-5` (`#263f5a`) dark. 1.5 px on inputs, 1 px on cards. *(Changed in upstream v2.0.0 — more contrast at the same 1 px weight, and the dark divider now carries the midnight hue.)*
- **Radii ladder:** sm 3 px (tags/chips), md 6 px (buttons/inputs/code blocks), lg 9 px (cards/panels), xl 13 px (large panels/modals), full (pills).
- Cards: 1 px border + radius-lg. Shadow only on elevation level 2+.

### Elevation (shadow system)
- **Level 0** — flat (default). Most surfaces.
- **Level 1** — `0 1px 3px rgba(0,0,0,0.06)` — cards under hover.
- **Level 2** — `0 4px 12px rgba(0,0,0,0.08)` — popovers, dropdowns.
- **Level 3** — `0 8px 24px rgba(0,0,0,0.12)` — modals.
- No inner shadows. No glow shadows. No colored shadows.

### Hover and press
- **Hover:** opacity drops to `0.88` for primary surfaces; border color shifts to `--midnight-7` for cards. **Never** lighten or darken brand colors on hover — the structure provides the hover affordance.
- **Press (:active):** no scale-shrink; the button color simply locks at its hover state.
- **Focus:** 2 px ring, `box-shadow: 0 0 0 2px var(--focus-ring-default)`. The conforming ring is `--focus-ring-strong`, applied by `data-focus="strong"` — see *Known default* below.

### Animation & motion
- **Four durations:** 100 ms (micro/hover), 200 ms (standard/toggle), 300 ms (expand/accordion), 400 ms (page/modal).
- **Two eases:** `ease-out` for things appearing, `ease-in` for things leaving.
- **No bounces, no springs, no overshoots.** No parallax. No autoplay video.
- **Slide transitions:** cut, or 200 ms fade. Staggered fade-up (40–80 ms increments) for sequential elements.

### Overlays and fixed UI

Overlay alphas and non-theming values are **named tokens**, never literals repeated through component CSS: `--scrim-modal`, `--scrim-palette`, `--scrim-lightbox`, `--tint-accent-active`, `--shadow-focus-light`, `--tint-code-header`, `--tint-highlight-line`, and the three dark code tints (`--tint-code-header-dark`, `--tint-code-divider-dark`, `--tint-code-control-dark`). Values that must *not* follow the theme — white on a solid control, print output, the reduced-transparency fallback — are `--fixed-*`.

`data-transparency="reduced"` solidifies all three scrims, not just `--scrim`, so they cannot diverge under that setting.

**Print is a `--fixed-*` context.** Paper does not follow `data-theme`, so the print set is the light value, always: `--fixed-print-page`, `--fixed-print-text`, `--fixed-print-muted`, `--fixed-print-border`, `--fixed-print-panel`, `--fixed-print-panel-bar`, plus `--fixed-print-code-bg` / `-border` / `-text` and `--fixed-print-external-url`. A print stylesheet carries no literals.

**Selection.** The page pair is `--orange-3` under `--orange-12`, both theme-following. Code panels take their own pair (`--code-panel-selection-bg` / `-fg`, and `--code-panel-light-selection-*` for the opt-in light panel, reached with `data-code-surface="light"`) — the page tint is unreadable over a dark panel.

Hex output is lowercase throughout.

### Transparency & blur
- Used sparingly. Backdrop-blur only on dark-on-dark overlays (modal scrim) and on the dark-panel inner cards (`background: rgba(255,255,255,0.04)`). No frosted glass on light surfaces.
- Tag/badge backgrounds use solid pale tints, not alpha overlays.

### Responsive
- **Four breakpoints:** 640 (large/landscape phone), 768 (small tablet — the first two-column layout), 1024 (tablet landscape, small laptop — sidebars appear), 1280 (desktop — the 1100 px measure is fully margined). Below 640 is the base, not a breakpoint.
- The **12-column grid collapses to 8 at md and 4 at base**.
- **44 px touch-target floor.** The 32/40/48 px control heights sit inside a 44 px hit area; they do not replace it.
- **Mobile navigation:** tab bar for application surfaces, drawer for documentation.

### Terminal & syntax
- **One dark terminal surface** (`#0e1720`) with sixteen ANSI slots plus chrome roles, all measured against that surface at a 4.95:1 floor. Configurations for tmux, WezTerm, Kitty, iTerm2, Zellij and Ghostty are documented upstream. *(This bullet is about the terminal; a **code panel** is a different, lighter surface — see the dark-surface table below.)*
- **The active tab's ink is a token, not white.** `--terminal-active-tab-fill` is `orange-9`, and white on it measures 3.76:1, so the label takes `--terminal-active-tab-text` (`#0e1720`, 4.81:1) — the same inversion the dark-mode status solids use.
- **Ten syntax roles**, with hues assigned by measured perceptual distance rather than taste. Only the three neutrals (plain, operators, comments) share the blue-grey band, separated by lightness; every chromatic role holds its own hue.
- **LSP modifiers are typography, not hue.** Ten modifiers against ten roles is a hundred states, which colour cannot carry.

### States
- **Empty states:** only the first-run state gets an accent button. A filtered-empty and an error-empty do not.
- **Skeleton vs spinner** by expected duration, not by preference.
- Failures are **scoped to the region that failed** — an inline message in the panel, not a page-level takeover.

### Accessibility
- Reading and focus order match; skip links and landmarks on every page shell.
- Focus **moves deliberately** on open, close and delete.
- Two WCAG 2.1 SC 1.4.3 exemptions are applied deliberately and are encoded in the upstream audits: **logotypes** (the wordmark sets "work" in the identity accent) and **inactive user-interface components** (the disabled control spec).

### Layout rules
- **Header on light:** sentence-case nav, lowercase brand mark, single accent CTA on the right.
- **Footer:** brand mark left, page metadata right (small grey caption-12).
- **No fixed sticky decoration.** Headers can stick; nothing else.

### Card recipe
```
background: var(--elevated-1);     /* resolves to the surface; pairs with --shadow-1 */
border: 1px solid var(--color-border);
border-radius: var(--radius-lg);   /* 9px */
padding: var(--space-4) var(--space-5);   /* 16px 24px */
/* shadow optional, level 1 only on raised cards */
```

### Image vibe (when used)
- **Warm but desaturated**, never cool/blue cast. Slight grain ok at 4–6%. No high-contrast B&W. No tropical saturation.

---

## Iconography

- **Library:** [Tabler Icons](https://tabler.io/icons) (MIT). Stroke-only outline set, never filled — Tabler also ships a filled variant; it is not used.
- **Stroke width:** 1.5 px. **Caps & joins:** round. Tabler ships at `stroke-width="2"`; override it to 1.5 — the set is drawn so the stroke can be varied.
- **Sizes:** 16 px (inline with text), 20 px (button glyphs), 24 px (nav, large affordances). Tabler's native grid is 24 px, so all three are clean multiples.
- **Color:** default `slate-11` (`#5c6f82`), active/selected `midnight-9` (`#1d3352`), danger `--danger`. Never colored unless meaningful (e.g. status).
- **Custom icons** (when Tabler doesn't have it) must match: 24 px grid, 1.5 px stroke, round caps, single color, no fills.
- **CDN usage:** per-icon `<img src="https://cdn.jsdelivr.net/npm/@tabler/icons@latest/icons/outline/check.svg">`, or the webfont `<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@tabler/icons-webfont@latest/tabler-icons.min.css">` then `<i class="ti ti-check"></i>`. For React surfaces, inline the SVG rather than loading a script — a DOM-mutating icon script is overwritten on re-render. The icon set is **not** bundled into this project — pull from CDN at use time.
- **Emoji:** not used in any brand surface (UI, slides, marketing, docs). See content fundamentals.
- **Unicode glyphs as iconography:** the middle dot `·` (U+00B7) is approved for inline metadata separators and brand wordmarks. The check `✓`, dot `●`, and circle `○` are used inside terminal/log output where copy/paste compatibility matters. Otherwise prefer Lucide.
- **The brand mark** (the stencil "peony bud" icon — see `assets/logo/icon-light.svg`) is **not** an icon — never use it as a generic glyph. It only appears in lockups.

---

## Accessibility

**Every setting is opt-in.** Nothing in the accessibility layer applies until an attribute is set on `<html>`, so adopting the system never changes how an existing page renders. Compose them freely:

```html
<html data-a11y="auto" data-focus="strong" data-link-underline="on">
```

| Attribute | Values | What it does |
|---|---|---|
| `data-a11y` | `auto` | Follow the OS: `prefers-reduced-motion`, `prefers-reduced-transparency`, `prefers-contrast`. |
| `data-font-size` | `lg` `xl` `xxl` `xxxl` | 112.5% · 125% · 150% · 200%. Scales every `--type-*-size` through a shared `--font-scale`, so no per-component work. 200% is the WCAG 1.4.4 target. |
| `data-contrast` | `high` | Flattens tints to solid borders, pushes text roles to the ends of their scale. Light and dark variants. |
| `data-focus` | `strong` | 2px solid `midnight-9` ring with 2px offset — 8.59:1, meets WCAG 1.4.11. |
| `data-link-underline` | `on` | Underlines links in running text (`p`, `li`, `dd`, `figcaption`, `blockquote`, `.prose`) so link identity is not colour alone (WCAG 1.4.1). `.link-plain` opts an individual link out. |
| `data-motion` | `reduced` | Near-zero transitions and animations (WCAG 2.3.3). |
| `data-transparency` | `reduced` | Drops `backdrop-filter`, solidifies the scrim. |
| `data-text-spacing` | `loose` | Applies the WCAG 1.4.12 minimums (1.5 line-height, 0.12em letter, 0.16em word, 2em paragraph). A **diagnostic** for proving nothing clips — not a house style. |
| `data-theme` | `light` `dark` | Pins the mode, ignoring `prefers-color-scheme`. |

### Utilities

| Class | Use |
|---|---|
| `.sr-only` | Visually hidden, still announced — the accessible name for icon-only controls. |
| `.sr-only-focusable` | Same, but reveals itself on focus. |
| `.skip-link` | Skip-to-content. First focusable element in the document; targets your `<main id="main">`. |
| `.target` | Enforces the `--touch-target` 44px floor (WCAG 2.5.8 asks 24px). |
| `.measure` | Caps line length at `--measure` (65ch). |

### Known default

The base focus ring (`--focus-ring-default`, `rgba(29,51,82,0.15)`) measures ~1.2:1 and does **not** meet WCAG 1.4.11. It is left as the default so that adopting this stylesheet changes nothing visually; `data-focus="strong"` is the conforming ring. New work should set it.

### Non-negotiable: never status by colour alone

Every status, validation state, or category must carry a second channel — a text label, a glyph, or a shape. The console's `StatusPill` pairs its dot with a written label; terminal output pairs `✓` and `●` with the line. A red border with no message is not an error state.

See `preview/accessibility.html` for a live toggle demo of every setting above.

### Syntax and terminal — dark by default, light is optional

The brand rule is **code blocks always dark by default**. Two dark surfaces, not one:

| | Token | Dark value | Why |
|---|---|---|---|
| Terminal | `--terminal-surface` | `#0e1720` | Black-ish — a terminal should read as a device surface |
| Code panel | `--code-panel-surface` | `#131e2b` | One step up, so a code block reads as a panel rather than a hole in the page |

The code surface is the same in both dark appearances: lighter than the deep-dark page, darker than the navy page, and every syntax role clears **4.79:1** on it. Lifting it further to `#1a2b3e` would drop `invalid` to 4.10 and break AA, which is why it stops here.

An optional companion set exists for the rare light-surface specimen: `--syntax-<role>-light` (same ten roles, each measured at **4.5–5.8:1 (AA)** on the light panel and pitched toward chroma rather than sitting at the contrast ceiling). These were briefly held at AAA; at that level every role collapses toward black and the hues stop being tellable apart, which defeats the purpose of a syntax palette. AA with real chroma is the correct trade here — a code panel is not running prose and `--terminal-light-*` (full 16-slot ANSI set plus chrome, same slot names). Nothing switches to these automatically — a component opts in by using the token, the same way any other colour choice is made. See `preview/colors-syntax.html` and `preview/colors-terminal.html`, which show both sets together, and `preview/components-code-blocks-light.html` for the light-panel code specimen.

### Card convention — every card carries the appearance switch

**One rule, no exceptions: every preview card has the same three-way appearance switch** — Light, Deep dark, Navy dark — injected by `preview/card-mode.js`. There is no mixed set of light-only cards, dark-only cards and side-by-side cards; each card is read in whichever appearance you need.

The switch sets `data-theme` (and `data-surface` for navy) on **`body`**, not on `<html>`: a card may be embedded as markup rather than loaded as a document, and an `<html>` attribute is lost when it is. `colors_and_type.css` therefore declares light, deep dark and navy dark as plain attribute blocks that work on any element.

Because of that, **cards must be token-driven**. A literal like `color:#142438` reads correctly in one appearance and fails in the other two, which is precisely the defect class this convention removes. Two consequences worth stating:

- **Solid controls take `--fixed-control-text` (white), not `--surface`.** A solid status fill takes `--on-solid-<role>`, which is white on light and `#0e1720` on the dark appearances — the dark status solids are light tints, where white measures 2.41:1.
- **Elevation on dark is a surface step, not a shadow.** Pair `--shadow-N` with `--elevated-N`; a black shadow has nothing to darken on a dark page.
- **Never use step 9 as a text colour.** Midnight/orange/slate step 9 is constant across all three appearances by design, so text set in it fails on dark. Text takes `--fg-1` / `--fg-2` / `--fg-3` (or steps 11–12).
- **Status text takes the matching `-fg` on the matching `-bg`.** Never a literal tint under a token foreground.

Three cards deliberately hold fixed values, because those values are the card's *content* rather than its chrome:

- **`Colors · Core`** — the identity colours, which are mode-independent by definition.
- **`Brand · Logo marks`** — each mark variant sits on the background that variant requires. Its *contextual* uses (size ladder, clear space, transparent surface) do follow the appearance.
- **Code specimens are one card per language** (`Code · Python`, `Code · Rust`, `Code · YAML`, `Code · TOML`, `Code · LaTeX`), each at a readable 17px and each carrying the switch. They sit directly after `Colors · Syntax roles` and share `preview/code-card.css`, which maps the nine highlight classes onto the canonical ten roles — so a code block cannot drift from the roles card.

`preview/_audit.html` renders all 28 cards in all three appearances and reports any text under 3:1 against its own background. It should always report `ALL CLEAR`.

**Colour is not the only channel.** Both palettes pair hue with weight and style, so structure survives greyscale, colour-blindness, and low-quality projection:

| Role | Weight | Style |
|---|---|---|
| Keyword | 700 | — |
| Type / class | 600 | — |
| Escape / invalid | 600 | — |
| Function name | 500 | — |
| Comment | 400 | *italic* |
| Attribute / decorator | 400 | *italic* |
| Everything else | 400 | — |

This requires the italic and bold cuts of IBM Plex Mono, which the font `@import` now loads (`ital,wght@0,400;0,500;0,600;0,700;1,400;1,500`). Without them the browser synthesises a faux oblique and no bold at all.

**Using them on a `<pre>` needs one extra line.** The base sheet pins every `pre` to the dark terminal surface, so a light code panel must neutralise it explicitly — styling only the wrapper leaves the `<pre>` painting dark on top of your light background:

```css
.code pre { background: transparent; color: var(--syntax-plain-light); padding: 0; }
```

## Logo system

- **Primary lockup (two-line):** mark + "projectious" stacked over "·work" — for headers, slides, hero cards.
- **Compact (one-line):** mark + "projectious·work" inline — for navbars, footers, signatures.
- **Dot-replace (inline):** "projectious[mark]work" — for documents, CLI prompts. The mark replaces the dot.
- **Stacked (portrait):** mark above wordmark — for avatars, app icons.
- **Mark variants** (`assets/logo/`): `icon-light.svg` (color, on light bg), `icon-dark.svg` (color, on dark bg), `icon-mono-black.svg`, `icon-mono-white.svg`, `icon-mono-gray.svg`.
- **Minimum size:** 16 px for the mark; 24 px for any lockup with wordmark.
- **Clear space:** half the mark's height on all sides.
- **Product-line extension:** swap `.work` for `.guard`, `.flow`, etc — same construction, accent stays orange-9.

---

## Files & fonts notes

- **Fonts** (Plus Jakarta Sans, Source Sans 3, IBM Plex Mono) are **all SIL OFL 1.1** and loaded from Google Fonts via `@import` in `colors_and_type.css`. No `.ttf`/`.woff2` files are bundled. **Substitution flag:** none — these are the brand-specified fonts and Google Fonts is the canonical delivery channel per the source brand.
- **Tabler Icons** are pulled from CDN at use time, not bundled. **Substitution flag:** yes — upstream `brand/` recommends Lucide; this project uses Tabler at the same 24 px grid and 1.5 px stroke. See `upstream-sync-note.md`.

### Distributable-theme profile

Google Fonts and icon-CDN-at-use-time are the defaults, not a requirement. A theme, plugin or documentation site that must build offline or pin its supply chain **may self-host**, under these terms.

**Fonts.** Self-hosting is permitted; all three families are SIL OFL 1.1, so ship the licence text alongside the files and do not rename the families. The required cuts are not negotiable, because two of them carry meaning:

| Family | Cuts |
|---|---|
| Plus Jakarta Sans | 400 500 600 700 800, upright |
| Source Sans 3 | 400 500 600, upright |
| IBM Plex Mono | 400 500 600 700 in **both** upright and italic |

The syntax contract carries role identity in weight and italic as well as hue (keyword 700, type 600, function 500, comment italic). Ship IBM Plex Mono without the bold or the italic cuts and the browser synthesises a faux oblique and no bold, so the roles stop being tellable apart in greyscale — which is exactly what the second channel exists to prevent.

**Icons.** Vendoring a Tabler subset is permitted (MIT; keep the licence file). Vendor the outline set only, at the native 24 px grid, and override `stroke-width` to 1.5 rather than re-drawing. Do not mix in another library to fill gaps — a custom icon drawn to the brand rules is the approved fallback.

**Opt-out.** A theme that self-hosts should say so in its own README and state the version it pinned, so a token re-sync does not silently leave the fonts behind.

---

## Quick start (in this project)

```html
<link rel="stylesheet" href="styles.css">
<link rel="icon" href="assets/logo/favicon-32.png">
```

```html
<h1>Redesigning work</h1>
<p class="caption">Cloud · Agile · Agentic AI</p>
<button style="background:var(--color-accent-solid);color:#fff;border:0;padding:10px 22px;border-radius:var(--radius-md);font-family:var(--font-heading);font-weight:500;">Get started</button>
```

---

## Licensing

The split matters, and the documentation states it on every page:

- **Code and tokens are MIT.** Take them.
- **Brand marks are not.** The logo, the wordmark and the name are reserved — see `TRADEMARK.md` upstream.
- **Fonts** (Plus Jakarta Sans, Source Sans 3, IBM Plex Mono) are SIL OFL 1.1.
- **Tabler Icons** are MIT.
- A per-asset provenance inventory records the licence of every third-party dependency (`brand/PROVENANCE.md` upstream).
