# aibox theme-system design review briefing

Audience: an external design AI agent reviewing the visual system implemented by aibox across its terminal tools and AI harnesses.

Status: source-derived snapshot of the v0.x implementation on 2026-08-20. This document describes current behavior; it is not a proposed replacement palette.

## Review objective

Review the 31 user-facing theme families and 61 concrete variants as one cross-tool design system. Assess semantic consistency, contrast, state differentiation, light/dark behavior, selection legibility, inactive-pane legibility, and the quality loss introduced when a tool only accepts a named built-in theme.

The review should return:

1. Per-variant accessibility or contrast problems, with the exact token pair.
2. Cross-tool semantic inconsistencies.
3. Missing semantic tokens that should become first-class configuration.
4. Recommended token names and derivation rules.
5. A migration plan that preserves existing `aibox.toml` family/mode/variant behavior.
6. A shortlist of variants that need bespoke values instead of shared fallback surfaces.
7. Explicit WCAG-style contrast calculations for all foreground/background pairs that carry text.

## Configuration model

```toml
[customization]
theme = "github"       # one of 31 families
mode = "auto"          # auto | light | dark
# variant = "dimmed"   # optional family-specific alternate
```

`auto` follows the host appearance when detectable and otherwise resolves dark. Solo families ignore `mode` and `variant`. The active concrete variant produces one nine-slot palette:

```text
bg, fg, accent, green, red, yellow, orange, cyan, muted
```

No arbitrary color overrides currently exist in `aibox.toml`.

## Proposed semantic vocabulary for the review

This vocabulary amends the initial Terminal Chrome and Code lists to cover every role actually emitted across all tools.

### Terminal and application chrome

| Semantic token | Current source | Current uses |
|---|---|---|
| `background` | `bg` | terminal/app base, active pane, popup base, gutters |
| `foreground` | `fg` | ordinary text, status text, popup/menu text |
| `surface` | per-variant lookup | status bars, inactive tabs, popups, hover/selection surfaces |
| `accent` | `accent` | focus, active tabs, borders, titles, directories, primary action |
| `muted` | `muted` | comments, inactive labels/borders, separators, metadata |
| `selection.background` | `surface` | Vim Visual/QuickFixLine, Yazi preview hover, FZF selected row |
| `selection.foreground` | inherited `fg`; Yazi hovered uses `bg` | selection text; inconsistent by control |
| `active.background` | `accent` | tmux/Yazi active tabs, menus, current state |
| `active.foreground` | `bg` | text drawn on accent |
| `inactive.background` | `surface` | inactive tabs/status components |
| `inactive.foreground` | `muted` or derived `dim label` | inactive labels and status titles |
| `pane.active.background` | `bg` | active pane |
| `pane.active.foreground` | `fg` | active pane text |
| `pane.inactive.background` | 88% `bg` + 12% `muted` | tmux inactive pane |
| `pane.inactive.foreground` | 75% `fg` + 25% `muted` | tmux inactive pane text |
| `border.active` | `accent` | pane/popup/menu borders |
| `border.inactive` | `muted` | inactive panes and controls |
| `status.ok` | 35% `accent` + 65% `muted` | neutral healthy PowerKit segment |
| `status.success` | `green` | positive state/executable/success prompt |
| `status.info` | `cyan` | informational state |
| `status.warning` | `yellow` | warning state |
| `status.error` | `red` | error state |
| `status.disabled` | `muted` | disabled state |
| `magenta` | 60% `red` + 40% `accent` | Yazi unset/copy/image/untracked and PowerKit command state |
| `search.match` | `yellow` on `bg` | Vim search |
| `search.current` | `accent` on `bg` | Vim current search |
| `cursor` | **not configured** | inherited from terminal emulator |
| `cursor.text` | **not configured** | inherited from terminal emulator |
| `terminal.selection.background` | **not configured** | inherited from terminal emulator |
| `terminal.selection.foreground` | **not configured** | inherited from terminal emulator |

### Code and document syntax

| Semantic token | Exact aibox Vim value | Related uses |
|---|---|---|
| `code.plain` | `fg` | identifiers, delimiters, ordinary code |
| `code.comment` | `muted`, italic | comments, ignored text |
| `code.operator` | `cyan` | operators and special syntax |
| `code.keyword` | `red` | statements, conditionals, repeats, exceptions |
| `code.type` | `yellow` | types, classes/storage/structures/typedefs |
| `code.function` | `accent` | functions, tags, links |
| `code.string` | `green` | strings and characters |
| `code.number` | `orange` | numbers, booleans, floats, constants |
| `code.decorator` | `accent` | macros; preprocessors use `yellow` |
| `code.invalid` | `red`, bold | errors and invalid syntax |
| `code.label` | `yellow` | labels and TODO foreground |
| `code.special` | `cyan` | special tokens |
| `code.special-character` | `orange` | escapes/special characters |
| `code.background` | `bg` | editor background |
| `code.current-line.background` | `surface` | cursor line/column/color column |
| `code.selection.background` | `surface` | Visual and VisualNOS |
| `code.selection.foreground` | inherited `fg` | not explicitly set |
| `code.search.background` | `yellow` | search hit |
| `code.search.foreground` | `bg` | search-hit text |
| `code.invalid.background` | inherited `bg` | no explicit invalid background |
| `diff.add` | `green` on `bg` | added lines/signs |
| `diff.change` | `yellow` on `bg` | changed lines/signs |
| `diff.delete` | `red` on `bg` | removed lines/signs |
| `diff.emphasis` | `accent` on `bg` | emphasized diff text |

Important: these exact syntax assignments apply to generated Vim. Bat, delta, Aider, Gemini, OpenCode, Claude Code, and lnav use named third-party themes, so their token-level syntax colors are not controlled by the nine-slot palette.

## Tool coverage and configuration fidelity

| Tool/surface | Configuration emitted by aibox | Fidelity |
|---|---|---|
| tmux core | exact bg/fg/accent/muted plus derived inactive pane colors | exact |
| tmux PowerKit | exact status, session, window, pane, popup, menu, state, orange and magenta roles | exact |
| Vim | generated `aibox.vim` UI, syntax, diff, spelling, Git, Markdown, help, popup groups | exact |
| Yazi | generated manager, tabs, modes, status, input, pick, completion, tasks, help, Git and filetype styles | exact |
| LazyGit | active/inactive borders, option text, selected line, cherry-pick, unstaged, default fg, search border | exact but partial |
| Starship | exact palette roles; usage varies by eight prompt presets | exact |
| fzf | exact normal/selected bg+fg, highlights, pointer, marker, spinner, info, header, border, prompt, query, disabled, gutter, preview, separator and label | exact |
| eza | exact directory, executable, symlink, orphan/missing, timestamps/owners, Git states and size-unit colors | exact foregrounds; regular file inherits terminal |
| less/man | exact heading, underline, reverse/search, and exported error SGR | exact partial |
| Git delta | aibox accent/muted decorations and fixed diff backgrounds; syntax uses Bat named theme | mixed |
| Bat | nearest installed built-in syntax theme | approximation |
| lnav | nearest built-in theme: solarized-light/dark, dracula, night-owl, tokyo-night, or monocai | approximation |
| Claude Code | only `light` or `dark` | coarse |
| Aider | nearest Pygments `code-theme` plus dark/light background flag | approximation |
| Gemini CLI | nearest built-in theme | approximation |
| OpenCode | nearest built-in theme | approximation |
| terminal emulator chrome | no cursor or terminal-selection palette emitted | missing |
| shell/readline | no base palette; inherits terminal, with themed tool integrations | inherited |

### Exact per-tool role mapping

- tmux core: status `surface/fg`; active tab `accent/bg`; inactive tab inherits status styling; active pane `bg/fg`; inactive pane uses derived dim bg/fg; active border `accent`; inactive border `muted`; message and copy mode `accent/bg`; popup `bg/fg`.
- PowerKit: session `accent/bg`; prefix `red`; copy `cyan`; search `yellow`; command `magenta`; active window `accent`; inactive window `muted`; zoom `cyan`; popup/menu `surface/fg`; selected menu `accent/bg`.
- Yazi: selected markers `green`; copied markers currently `accent`; cut markers `red`; active tabs `accent/bg`; inactive tabs `surface/muted`; normal mode `green/bg`; select mode `accent/bg`; unset mode `magenta/bg`; progress normal `accent/surface`; progress error `red/surface`.
- LazyGit: selected-line background is currently `bg`, not `surface`; this deserves review because it can make selection indistinguishable from the base.
- Delta: minus backgrounds are fixed `#3B1F22`/`#6B1E25`; plus backgrounds are fixed `#1F3B25`/`#1F5B33` for every theme, including light variants. This is a major review target.
- Vim: selection sets only background; selected text inherits foreground. CursorLine sets only background. No Cursor highlight is generated.
- FZF: selected and unselected foreground are both `fg`; selection is conveyed only by `surface`.
- Yazi input `selected` uses terminal reverse-video rather than explicit fg/bg.

## Third-party named-theme mappings

- Bat/delta: exact family variants exist for Gruvbox, Catppuccin, Dracula, Nord, Solarized, Monokai, One Half, and selected GitHub/VS Code cases; most others fall back to Coldark Dark/Cold.
- lnav: light variants generally map to `solarized-light`; Solarized Dark, Dracula, Night Owl, and Tokyo Night families have closer mappings; all remaining variants use `monocai`.
- Claude Code: every concrete theme collapses to `light` or `dark`.
- Aider: Gruvbox always maps to `gruvbox-dark` even for Gruvbox Light; Solarized, Dracula, GitHub canonical, Nord, and light/default cases are closer; most dark variants map to `monokai`.
- Gemini: closest built-ins are Dracula, GitHub, GitHub Light, Ayu, Ayu Light, Monokai, Atom One Dark, Default Light, or Default.
- OpenCode: closest families are Gruvbox, Catppuccin, Dracula, Tokyo Night, Nord, Rose Pine, One Dark, Ayu, Kanagawa, Everforest, and Monokai; unsupported families fall back to `opencode`.

## Complete variant palette registry

All colors are exact uppercase hex values from `cli/src/themes.rs`. Derived values use integer channel mixing with truncation, exactly matching the Rust implementation.

- `surface` is a lookup, not a formula.
- `magenta` = 60% red + 40% accent.
- `neutral OK` = 35% accent + 65% muted.
- `inactive pane bg` = 88% bg + 12% muted.
- `inactive pane fg` = 75% fg + 25% muted.
- `dim label` = 50% fg + 50% muted.
- `active title fg` = 85% accent + 15% fg.

### andromeeda

| Concrete variant | Mode | `variant` | bg | fg | accent | green | red | yellow | orange | cyan | muted |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Andromeeda | dark | solo | #23262E | #D5CED9 | #00E8C6 | #89E044 | #EE5D43 | #FFCC00 | #F39C12 | #00E8C6 | #6B6B6B |

| Concrete variant | surface / selection bg | magenta | neutral OK | inactive pane bg | inactive pane fg | dim label | active title fg |
|---|---|---|---|---|---|---|---|
| Andromeeda | #313244 | #8E9477 | #45968A | #2B2E35 | #BAB5BD | #A09CA2 | #1FE4C8 |

### aurora-x

| Concrete variant | Mode | `variant` | bg | fg | accent | green | red | yellow | orange | cyan | muted |
|---|---|---|---|---|---|---|---|---|---|---|---|
| AuroraX | dark | solo | #07090F | #D4D4D4 | #569CD6 | #B5CEA8 | #F44747 | #CE9178 | #CE9178 | #4EC9B0 | #5C6370 |

| Concrete variant | surface / selection bg | magenta | neutral OK | inactive pane bg | inactive pane fg | dim label | active title fg |
|---|---|---|---|---|---|---|---|
| AuroraX | #313244 | #B46980 | #597693 | #11131A | #B6B7BB | #989BA2 | #68A4D5 |

### ayu

| Concrete variant | Mode | `variant` | bg | fg | accent | green | red | yellow | orange | cyan | muted |
|---|---|---|---|---|---|---|---|---|---|---|---|
| AyuDark | dark | default | #0A0E14 | #B3B1AD | #39BAE6 | #AAD94C | #F07178 | #FFB454 | #FF8F40 | #95E6CB | #626A73 |
| AyuMirage | dark | mirage | #1F2430 | #CCCAC2 | #5CCFE6 | #BAE67E | #F28779 | #FFD173 | #FFAD66 | #95E6CB | #707A8C |
| AyuLight | light | default | #FAFAFA | #5C6773 | #55B4D4 | #86B300 | #E7676A | #FA8D3E | #F07171 | #4CBF99 | #ABB0B6 |

| Concrete variant | surface / selection bg | magenta | neutral OK | inactive pane bg | inactive pane fg | dim label | active title fg |
|---|---|---|---|---|---|---|---|
| AyuDark | #313244 | #A68EA4 | #53869B | #14191F | #9E9F9E | #8A8D90 | #4BB8DD |
| AyuMirage | #313244 | #B6A3A4 | #6997AB | #282E3B | #B5B6B4 | #9EA2A7 | #6CCEE0 |
| AyuLight | #CCD0DA | #AC8594 | #8CB1C0 | #F0F1F1 | #6F7983 | #838B94 | #56A8C5 |

### catppuccin

| Concrete variant | Mode | `variant` | bg | fg | accent | green | red | yellow | orange | cyan | muted |
|---|---|---|---|---|---|---|---|---|---|---|---|
| CatppuccinMocha | dark | default | #1E1E2E | #CDD6F4 | #89B4FA | #A6E3A1 | #F38BA8 | #F9E2AF | #FAB387 | #94E2D5 | #6C7086 |
| CatppuccinMacchiato | dark | macchiato | #24273A | #CAD3F5 | #8AADF4 | #A6DA95 | #ED8796 | #EED49F | #F5A97F | #8BD5CA | #6E738D |
| CatppuccinFrappe | dark | frappe | #303446 | #C6D0F5 | #8CAAEE | #A6D189 | #E78284 | #E5C890 | #EF9F76 | #81C8BE | #737994 |
| CatppuccinLatte | light | default | #EFF1F5 | #4C4F69 | #1E66F5 | #40A02B | #D20F39 | #DF8E1D | #FE640B | #179299 | #9CA0B0 |

| Concrete variant | surface / selection bg | magenta | neutral OK | inactive pane bg | inactive pane fg | dim label | active title fg |
|---|---|---|---|---|---|---|---|
| CatppuccinMocha | #313244 | #C89BC8 | #7687AE | #272738 | #B4BCD8 | #9CA3BD | #93B9F9 |
| CatppuccinMacchiato | #313244 | #C596BB | #7787B1 | #2C3043 | #B3BBDB | #9CA3C1 | #93B2F4 |
| CatppuccinFrappe | #313244 | #C292AE | #7B8AB3 | #383C4F | #B1BADC | #9CA4C4 | #94AFEF |
| CatppuccinLatte | #CCD0DA | #8A3184 | #6F8BC8 | #E5E7EC | #60637A | #74778C | #2462E0 |

### dracula

| Concrete variant | Mode | `variant` | bg | fg | accent | green | red | yellow | orange | cyan | muted |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Dracula | dark | default | #282A36 | #F8F8F2 | #BD93F9 | #50FA7B | #FF5555 | #F1FA8C | #FFB86C | #8BE9FD | #6272A4 |
| DraculaSoft | dark | soft | #22212C | #F8F8F2 | #C8A8F9 | #62E884 | #E76D6D | #E9E987 | #FFCA80 | #A1F0FE | #7970A9 |

| Concrete variant | surface / selection bg | magenta | neutral OK | inactive pane bg | inactive pane fg | dim label | active title fg |
|---|---|---|---|---|---|---|---|
| Dracula | #44475A | #E46D96 | #817DC1 | #2E3243 | #D2D6DE | #ADB5CB | #C5A2F7 |
| DraculaSoft | #44475A | #DA84A5 | #9483C5 | #2C2A3B | #D8D6DF | #B8B4CD | #CFB4F7 |

### everforest

| Concrete variant | Mode | `variant` | bg | fg | accent | green | red | yellow | orange | cyan | muted |
|---|---|---|---|---|---|---|---|---|---|---|---|
| EverforestDark | dark | default | #2D353B | #D3C6AA | #7FBBB3 | #A7C080 | #E67E80 | #DBBC7F | #D699B6 | #83C092 | #7A8478 |
| EverforestLight | light | default | #FDF6E3 | #5C6A72 | #3A94C5 | #8DA101 | #F85552 | #DFA000 | #DF69BA | #35A77C | #939F91 |

| Concrete variant | surface / selection bg | magenta | neutral OK | inactive pane bg | inactive pane fg | dim label | active title fg |
|---|---|---|---|---|---|---|---|
| EverforestDark | #313244 | #BC9694 | #7B978C | #363E42 | #BCB59D | #A6A591 | #8BBCB1 |
| EverforestLight | #CCD0DA | #AC6E80 | #739BA3 | #F0EBD9 | #697779 | #778481 | #3F8DB8 |

### github

| Concrete variant | Mode | `variant` | bg | fg | accent | green | red | yellow | orange | cyan | muted |
|---|---|---|---|---|---|---|---|---|---|---|---|
| GithubDark | dark | default | #0D1117 | #C9D1D9 | #58A6FF | #3FB950 | #F85149 | #D29922 | #DB6D28 | #79C0FF | #8B949E |
| GithubLight | light | default | #FFFFFF | #24292F | #0969DA | #1A7F37 | #CF222E | #9A6700 | #BC4C00 | #218BFF | #6E7781 |
| GithubDarkDimmed | dark | dimmed | #22272E | #ADBAC7 | #539BF5 | #57AB5A | #F47067 | #C69026 | #F47067 | #6CB6FF | #768390 |
| GithubDarkHighContrast | dark | high-contrast-dark | #0A0C10 | #F0F3F6 | #71B7FF | #26CD4D | #FF6A69 | #F0B72F | #FF6A69 | #91CBFF | #9198A1 |
| GithubLightHighContrast | light | high-contrast-light | #FFFFFF | #0E1116 | #1A69DB | #104F24 | #A0111F | #7D4E00 | #A0111F | #034188 | #69717B |

| Concrete variant | surface / selection bg | magenta | neutral OK | inactive pane bg | inactive pane fg | dim label | active title fg |
|---|---|---|---|---|---|---|---|
| GithubDark | #313244 | #B87391 | #799ABF | #1C2027 | #B9C1CA | #AAB2BB | #68ACF9 |
| GithubLight | #CCD0DA | #7F3E72 | #4A72A0 | #EDEEEF | #363C43 | #495058 | #0D5FC0 |
| GithubDarkDimmed | #313244 | #B3819F | #698BB3 | #2C3239 | #9FACB9 | #919EAB | #609FEE |
| GithubDarkHighContrast | #313244 | #C688A5 | #85A2C1 | #1A1C21 | #D8DCE0 | #C0C5CB | #84C0FD |
| GithubLightHighContrast | #CCD0DA | #6A346A | #4D6E9C | #EDEDEF | #24292F | #3B4148 | #185BBD |

### gruvbox

| Concrete variant | Mode | `variant` | bg | fg | accent | green | red | yellow | orange | cyan | muted |
|---|---|---|---|---|---|---|---|---|---|---|---|
| GruvboxDark | dark | default | #282828 | #D5C4A1 | #D79921 | #98971A | #CC241D | #D79921 | #D65D0E | #689D6A | #928374 |
| GruvboxLight | light | default | #FBF1C7 | #3C3836 | #D65D0E | #79740E | #CC241D | #B57614 | #D65D0E | #076678 | #928374 |

| Concrete variant | surface / selection bg | magenta | neutral OK | inactive pane bg | inactive pane fg | dim label | active title fg |
|---|---|---|---|---|---|---|---|
| GruvboxDark | #3C3836 | #D0521E | #AA8A56 | #343231 | #C4B395 | #B3A38A | #D69F34 |
| GruvboxLight | #EBDBB2 | #D03A17 | #A97550 | #EEE3BD | #514A45 | #675D55 | #BE5714 |

### houston

| Concrete variant | Mode | `variant` | bg | fg | accent | green | red | yellow | orange | cyan | muted |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Houston | dark | solo | #17191E | #CDD6F4 | #F9C86A | #4AF2C8 | #FF5370 | #FFA726 | #81D4FA | #4AF2C8 | #545878 |

| Concrete variant | surface / selection bg | magenta | neutral OK | inactive pane bg | inactive pane fg | dim label | active title fg |
|---|---|---|---|---|---|---|---|
| Houston | #313244 | #FC816D | #8D7F73 | #1E2028 | #AEB6D5 | #9097B6 | #F2CA7E |

### kanagawa

| Concrete variant | Mode | `variant` | bg | fg | accent | green | red | yellow | orange | cyan | muted |
|---|---|---|---|---|---|---|---|---|---|---|---|
| KanagawaWave | dark | default | #1F1F28 | #DCD7BA | #7E9CD8 | #98BB6C | #C34043 | #FF9E3B | #D27E99 | #7AA89F | #727169 |
| KanagawaDragon | dark | dragon | #181616 | #C5C9C5 | #7EB3C9 | #87A987 | #C4746E | #B6927B | #C4746E | #8EA4A2 | #8A8980 |
| KanagawaLotus | light | default | #F2ECBC | #545464 | #1F5F8A | #4E7C3F | #C84053 | #835C00 | #B5485D | #536A5B | #A09F8F |

| Concrete variant | surface / selection bg | magenta | neutral OK | inactive pane bg | inactive pane fg | dim label | active title fg |
|---|---|---|---|---|---|---|---|
| KanagawaWave | #313244 | #A7647E | #76808F | #28282F | #C1BDA5 | #A7A491 | #8CA4D3 |
| KanagawaDragon | #313244 | #A88D92 | #859799 | #252322 | #B6B9B3 | #A7A9A2 | #88B6C8 |
| KanagawaLotus | #CCD0DA | #844C69 | #72888D | #E8E2B6 | #67666E | #7A7979 | #265D84 |

### laserwave

| Concrete variant | Mode | `variant` | bg | fg | accent | green | red | yellow | orange | cyan | muted |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Laserwave | dark | solo | #27212E | #FFFFFF | #EB64B9 | #74DFC4 | #FE4450 | #FFEE79 | #FFEE79 | #74DFC4 | #6B5F7D |

| Concrete variant | surface / selection bg | magenta | neutral OK | inactive pane bg | inactive pane fg | dim label | active title fg |
|---|---|---|---|---|---|---|---|
| Laserwave | #313244 | #F6507A | #976092 | #2F2837 | #DAD7DE | #B5AFBE | #EE7BC3 |

### material

| Concrete variant | Mode | `variant` | bg | fg | accent | green | red | yellow | orange | cyan | muted |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Material | dark | default | #263238 | #EEFFFF | #82AAFF | #C3E88D | #F07178 | #FFCB6B | #F78C6C | #89DDFF | #546E7A |
| MaterialOcean | dark | ocean | #0F111A | #A6ACCD | #82AAFF | #C3E88D | #F07178 | #FFCB6B | #F78C6C | #89DDFF | #464B5D |
| MaterialPalenight | dark | palenight | #292D3E | #A6ACCD | #82AAFF | #C3E88D | #F07178 | #FFCB6B | #F78C6C | #89DDFF | #676E95 |
| MaterialLighter | light | default | #FAFAFA | #546E7A | #6182B8 | #91B859 | #E53935 | #F6A434 | #F76D47 | #39ADB5 | #90A4AE |
| MaterialDarker | dark | darker | #212121 | #EEFFFF | #89DDFF | #C3E88D | #FF5370 | #FFCB6B | #F78C6C | #82AAFF | #546E7A |

| Concrete variant | surface / selection bg | magenta | neutral OK | inactive pane bg | inactive pane fg | dim label | active title fg |
|---|---|---|---|---|---|---|---|
| Material | #313244 | #C487AE | #6483A8 | #2B393F | #C7DADD | #A1B6BC | #92B6FF |
| MaterialOcean | #313244 | #C487AE | #5B6C95 | #151722 | #8E93B1 | #767B95 | #87AAF7 |
| MaterialPalenight | #313244 | #C487AE | #7083BA | #303448 | #969CBF | #868DB1 | #87AAF7 |
| MaterialLighter | #CCD0DA | #B05669 | #7F98B1 | #EDEFF0 | #637B87 | #728994 | #5F7FAE |
| MaterialDarker | #313244 | #CF8AA9 | #6694A8 | #272A2B | #C7DADD | #A1B6BC | #98E2FF |

### min

| Concrete variant | Mode | `variant` | bg | fg | accent | green | red | yellow | orange | cyan | muted |
|---|---|---|---|---|---|---|---|---|---|---|---|
| MinDark | dark | default | #1F1F1F | #B2B2B2 | #569CD6 | #B5CEA8 | #F44747 | #CCA700 | #CE9178 | #4EC9B0 | #525252 |
| MinLight | light | default | #F8F8F8 | #333333 | #0000FF | #098658 | #E50000 | #865F00 | #C1440E | #267F99 | #9A9A9A |

| Concrete variant | surface / selection bg | magenta | neutral OK | inactive pane bg | inactive pane fg | dim label | active title fg |
|---|---|---|---|---|---|---|---|
| MinDark | #313244 | #B46980 | #536B80 | #252525 | #9A9A9A | #828282 | #639FD0 |
| MinLight | #CCD0DA | #890066 | #6464BD | #ECECEC | #4C4C4C | #666666 | #0707E0 |

### monokai

| Concrete variant | Mode | `variant` | bg | fg | accent | green | red | yellow | orange | cyan | muted |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Monokai | dark | solo | #272822 | #F8F8F2 | #F92672 | #A6E22E | #F92672 | #E6DB74 | #AE81FF | #66D9EF | #75715E |

| Concrete variant | surface / selection bg | magenta | neutral OK | inactive pane bg | inactive pane fg | dim label | active title fg |
|---|---|---|---|---|---|---|---|
| Monokai | #313244 | #F92672 | #A35665 | #303029 | #D7D6CD | #B6B4A8 | #F84585 |

### moonlight

| Concrete variant | Mode | `variant` | bg | fg | accent | green | red | yellow | orange | cyan | muted |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Moonlight | dark | solo | #212337 | #C8D3F5 | #82AAFF | #C3E88D | #FF757F | #FFC777 | #F78C6C | #86E1FC | #7A88CF |

| Concrete variant | surface / selection bg | magenta | neutral OK | inactive pane bg | inactive pane fg | dim label | active title fg |
|---|---|---|---|---|---|---|---|
| Moonlight | #283457 | #CD8AB2 | #7C93DF | #2B2F49 | #B4C0EB | #A1ADE2 | #8CB0FD |

### night-owl

| Concrete variant | Mode | `variant` | bg | fg | accent | green | red | yellow | orange | cyan | muted |
|---|---|---|---|---|---|---|---|---|---|---|---|
| NightOwl | dark | default | #011627 | #D6DEEB | #82AAFF | #22DA6E | #EF5350 | #C5E478 | #F78C6C | #21C7A8 | #637777 |
| NightOwlLight | light | default | #FBFBFB | #403F53 | #4876D6 | #2AA298 | #D3423E | #DAA520 | #DD6A58 | #08916A | #989FB1 |

| Concrete variant | surface / selection bg | magenta | neutral OK | inactive pane bg | inactive pane fg | dim label | active title fg |
|---|---|---|---|---|---|---|---|
| NightOwl | #313244 | #C37596 | #6D88A6 | #0C2130 | #B9C4CE | #9CAAB1 | #8EB1FC |
| NightOwlLight | #CCD0DA | #9B567A | #7C90BD | #EFEFF2 | #56576A | #6C6F82 | #466DC2 |

### nord

| Concrete variant | Mode | `variant` | bg | fg | accent | green | red | yellow | orange | cyan | muted |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Nord | dark | solo | #2E3440 | #D8DEE9 | #88C0D0 | #A3BE8C | #BF616A | #EBCB8B | #D08770 | #81A1C1 | #4C566A |

| Concrete variant | surface / selection bg | magenta | neutral OK | inactive pane bg | inactive pane fg | dim label | active title fg |
|---|---|---|---|---|---|---|---|
| Nord | #3B4252 | #A98792 | #617B8D | #313845 | #B5BCC9 | #929AA9 | #94C4D3 |

### one-dark

| Concrete variant | Mode | `variant` | bg | fg | accent | green | red | yellow | orange | cyan | muted |
|---|---|---|---|---|---|---|---|---|---|---|---|
| OneDarkPro | dark | default | #282C34 | #ABB2BF | #61AFEF | #98C379 | #E06C75 | #E5C07B | #D19A66 | #56B6C2 | #5C6370 |
| OneLight | light | default | #FAFAFA | #383A42 | #4078F2 | #50A14F | #CA1243 | #C18401 | #986801 | #0184BC | #A0A1A7 |

| Concrete variant | surface / selection bg | magenta | neutral OK | inactive pane bg | inactive pane fg | dim label | active title fg |
|---|---|---|---|---|---|---|---|
| OneDarkPro | #313244 | #AD86A5 | #5D7D9C | #2E323B | #979EAB | #838A97 | #6CAFE7 |
| OneLight | #CCD0DA | #923A89 | #7E92C1 | #EFEFF0 | #52535B | #6C6D74 | #3E6ED7 |

### plastic

| Concrete variant | Mode | `variant` | bg | fg | accent | green | red | yellow | orange | cyan | muted |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Plastic | dark | solo | #1B1D23 | #ABB2BF | #61AFEF | #98C379 | #E06C75 | #E5C07B | #D19A66 | #56B6C2 | #7A7E8A |

| Concrete variant | surface / selection bg | magenta | neutral OK | inactive pane bg | inactive pane fg | dim label | active title fg |
|---|---|---|---|---|---|---|---|
| Plastic | #313244 | #AD86A5 | #718FAD | #26282F | #9EA5B1 | #9298A4 | #6CAFE7 |

### poimandres

| Concrete variant | Mode | `variant` | bg | fg | accent | green | red | yellow | orange | cyan | muted |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Poimandres | dark | solo | #1B1E28 | #A6ACCD | #A6DA95 | #5DE4C7 | #D0679D | #FFFAC2 | #D0679D | #ADD7FF | #767C9D |

| Concrete variant | surface / selection bg | magenta | neutral OK | inactive pane bg | inactive pane fg | dim label | active title fg |
|---|---|---|---|---|---|---|---|
| Poimandres | #313244 | #BF9599 | #869C9A | #252936 | #9AA0C1 | #8E94B5 | #A6D39D |

### projectious

| Concrete variant | Mode | `variant` | bg | fg | accent | green | red | yellow | orange | cyan | muted |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Projectious | dark | solo | #0E1720 | #C5DAF0 | #E05232 | #4FB07A | #E55B5B | #E0B85B | #F2A65A | #8AACC8 | #7B8DA3 |

| Concrete variant | surface / selection bg | magenta | neutral OK | inactive pane bg | inactive pane fg | dim label | active title fg |
|---|---|---|---|---|---|---|---|
| Projectious | #131E2B | #E3574A | #9E787B | #1B252F | #B2C6DC | #A0B3C9 | #DB664E |

### red

| Concrete variant | Mode | `variant` | bg | fg | accent | green | red | yellow | orange | cyan | muted |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Red | dark | solo | #390000 | #F8F8F8 | #FF6666 | #F4C2C2 | #FF0000 | #FF8800 | #FFD0D0 | #FF9999 | #A06060 |

| Concrete variant | surface / selection bg | magenta | neutral OK | inactive pane bg | inactive pane fg | dim label | active title fg |
|---|---|---|---|---|---|---|---|
| Red | #313244 | #FF2828 | #C16262 | #450B0B | #E2D2D2 | #CCACAC | #FD7B7B |

### rose-pine

| Concrete variant | Mode | `variant` | bg | fg | accent | green | red | yellow | orange | cyan | muted |
|---|---|---|---|---|---|---|---|---|---|---|---|
| RosePine | dark | default | #191724 | #E0DEF4 | #C4A7E7 | #31748F | #EB6F92 | #F6C177 | #EA9A97 | #9CCFD8 | #6E6A86 |
| RosePineMoon | dark | moon | #232136 | #E0DEF4 | #C4A7E7 | #3E8FB0 | #EB6F92 | #F6C177 | #EA9A97 | #9CCFD8 | #6E6A86 |
| RosePineDawn | light | default | #FAF4ED | #575279 | #907AA9 | #56949F | #B4637A | #EA9D34 | #D7827E | #286983 | #9893A5 |

| Concrete variant | surface / selection bg | magenta | neutral OK | inactive pane bg | inactive pane fg | dim label | active title fg |
|---|---|---|---|---|---|---|---|
| RosePine | #313244 | #DB85B4 | #8C7FA7 | #23202F | #C3C1D8 | #A7A4BD | #C8AFE8 |
| RosePineMoon | #313244 | #DB85B4 | #8C7FA7 | #2C293F | #C3C1D8 | #A7A4BD | #C8AFE8 |
| RosePineDawn | #CCD0DA | #A56C8C | #958AA6 | #EEE8E4 | #676284 | #77728F | #8774A1 |

### slack

| Concrete variant | Mode | `variant` | bg | fg | accent | green | red | yellow | orange | cyan | muted |
|---|---|---|---|---|---|---|---|---|---|---|---|
| SlackDark | dark | default | #222529 | #D1D2D3 | #8CC4FF | #AFE3A4 | #E07070 | #DFC55A | #DFC55A | #98D1E0 | #60656A |
| SlackOchin | light | ochin | #F9F9F9 | #383A3C | #0070D1 | #268829 | #D0104C | #C64B10 | #C64B10 | #007A7A | #A0A4A8 |

| Concrete variant | surface / selection bg | magenta | neutral OK | inactive pane bg | inactive pane fg | dim label | active title fg |
|---|---|---|---|---|---|---|---|
| SlackDark | #313244 | #BE91A9 | #6F869E | #292C30 | #B4B6B8 | #989B9E | #96C6F8 |
| SlackOchin | #CCD0DA | #7C3681 | #6891B6 | #EEEEEF | #525457 | #6C6F72 | #0867BA |

### snazzy

| Concrete variant | Mode | `variant` | bg | fg | accent | green | red | yellow | orange | cyan | muted |
|---|---|---|---|---|---|---|---|---|---|---|---|
| SnazzyLight | light | solo | #FAFBFC | #2D2D2D | #57C7FF | #5AF78E | #FF5C57 | #FF9F43 | #FF6AC1 | #57C7FF | #9E9E9E |

| Concrete variant | surface / selection bg | magenta | neutral OK | inactive pane bg | inactive pane fg | dim label | active title fg |
|---|---|---|---|---|---|---|---|
| SnazzyLight | #CCD0DA | #BB869A | #85ACBF | #EEEFF0 | #494949 | #656565 | #50AFDF |

### solarized

| Concrete variant | Mode | `variant` | bg | fg | accent | green | red | yellow | orange | cyan | muted |
|---|---|---|---|---|---|---|---|---|---|---|---|
| SolarizedDark | dark | default | #002B36 | #93A1A1 | #268BD2 | #859900 | #DC322F | #B58900 | #CB4B16 | #2AA198 | #657B83 |
| SolarizedLight | light | default | #FDF6E3 | #586E75 | #268BD2 | #859900 | #DC322F | #B58900 | #CB4B16 | #2AA198 | #93A1A1 |

| Concrete variant | surface / selection bg | magenta | neutral OK | inactive pane bg | inactive pane fg | dim label | active title fg |
|---|---|---|---|---|---|---|---|
| SolarizedDark | #313244 | #935570 | #4E809E | #0C343F | #879799 | #7C8E92 | #368ECA |
| SolarizedLight | #CCD0DA | #935570 | #6C99B2 | #F0EBDB | #667A80 | #75878B | #2D86C4 |

### synthwave-84

| Concrete variant | Mode | `variant` | bg | fg | accent | green | red | yellow | orange | cyan | muted |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Synthwave84 | dark | solo | #2A2139 | #FFFFFF | #36F9F6 | #FF7EDB | #FE4450 | #FEDE5D | #F97E72 | #36F9F6 | #848082 |

| Concrete variant | surface / selection bg | magenta | neutral OK | inactive pane bg | inactive pane fg | dim label | active title fg |
|---|---|---|---|---|---|---|---|
| Synthwave84 | #313244 | #AE8C92 | #68AAAA | #342C41 | #E0DFDF | #C1BFC0 | #54F9F7 |

### tokyo-night

| Concrete variant | Mode | `variant` | bg | fg | accent | green | red | yellow | orange | cyan | muted |
|---|---|---|---|---|---|---|---|---|---|---|---|
| TokyoNight | dark | default | #1A1B26 | #C0CAF5 | #7AA2F7 | #9ECE6A | #F7768E | #E0AF68 | #FF9E64 | #7DCFFF | #565F89 |
| TokyoNightStorm | dark | storm | #24283B | #C0CAF5 | #7AA2F7 | #9ECE6A | #F7768E | #E0AF68 | #FF9E64 | #7DCFFF | #565F89 |
| TokyoNightDay | light | default | #E1E2E7 | #3760BF | #2E7DE9 | #587539 | #F52A65 | #8C6C3E | #B15C00 | #007197 | #7B8496 |

| Concrete variant | surface / selection bg | magenta | neutral OK | inactive pane bg | inactive pane fg | dim label | active title fg |
|---|---|---|---|---|---|---|---|
| TokyoNight | #283457 | #C587B8 | #6276AF | #212331 | #A5AFDA | #8B94BF | #84A8F6 |
| TokyoNightStorm | #283457 | #C587B8 | #6276AF | #2A2E44 | #A5AFDA | #8B94BF | #84A8F6 |
| TokyoNightDay | #CCD0DA | #A54B99 | #6081B3 | #D4D6DD | #4869B4 | #5972AA | #2F78E2 |

### vesper

| Concrete variant | Mode | `variant` | bg | fg | accent | green | red | yellow | orange | cyan | muted |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Vesper | dark | solo | #101010 | #FFFFFF | #FF7B00 | #99FFE4 | #F44747 | #FF7B00 | #FFC799 | #FFC799 | #5C5C5C |

| Concrete variant | surface / selection bg | magenta | neutral OK | inactive pane bg | inactive pane fg | dim label | active title fg |
|---|---|---|---|---|---|---|---|
| Vesper | #313244 | #F85B2A | #95663B | #191919 | #D6D6D6 | #ADADAD | #FF8E26 |

### vitesse

| Concrete variant | Mode | `variant` | bg | fg | accent | green | red | yellow | orange | cyan | muted |
|---|---|---|---|---|---|---|---|---|---|---|---|
| VitesseDark | dark | default | #121212 | #DBD7CA | #4D9375 | #C98A7D | #E06C75 | #D4976C | #6496C8 | #80A0C0 | #758575 |
| VitesseLight | light | default | #FFFFFF | #393A34 | #1E754F | #B56959 | #AB5959 | #B07D48 | #296AA3 | #2E808F | #A0A077 |
| VitesseBlack | dark | black | #000000 | #DBD7CA | #4D9375 | #C98A7D | #E06C75 | #D4976C | #6496C8 | #80A0C0 | #606060 |

| Concrete variant | surface / selection bg | magenta | neutral OK | inactive pane bg | inactive pane fg | dim label | active title fg |
|---|---|---|---|---|---|---|---|
| VitesseDark | #313244 | #A57B75 | #678975 | #1D1F1D | #C1C2B4 | #A8AE9F | #629D81 |
| VitesseLight | #CCD0DA | #726455 | #729069 | #F3F3EE | #525344 | #6C6D55 | #226C4A |
| VitesseBlack | #313244 | #A57B75 | #597167 | #0B0B0B | #BCB9AF | #9D9B95 | #629D81 |

### vscode

| Concrete variant | Mode | `variant` | bg | fg | accent | green | red | yellow | orange | cyan | muted |
|---|---|---|---|---|---|---|---|---|---|---|---|
| VsCodeDarkPlus | dark | default | #1E1E1E | #D4D4D4 | #569CD6 | #B5CEA8 | #F44747 | #CCA700 | #CE9178 | #4EC9B0 | #6A9955 |
| VsCodeLightPlus | light | default | #FFFFFF | #000000 | #0000FF | #098658 | #CD3131 | #A65E00 | #A31515 | #267F99 | #008000 |

| Concrete variant | surface / selection bg | magenta | neutral OK | inactive pane bg | inactive pane fg | dim label | active title fg |
|---|---|---|---|---|---|---|---|
| VsCodeDarkPlus | #313244 | #B46980 | #639A82 | #272C24 | #B9C5B4 | #9FB694 | #68A4D5 |
| VsCodeLightPlus | #CCD0DA | #7B1D83 | #005359 | #E0EFE0 | #002000 | #004000 | #0000D8 |

## Design questions and known gaps

1. Should cursor, cursor text, terminal selection bg, and terminal selection fg become first-class semantic tokens and be emitted through terminal-specific integrations?
2. Should `selection.foreground` always be explicit instead of inherited or reverse-video?
3. Is one shared `#CCD0DA` surface defensible across 17 light variants? It is not derived from each palette and may clash or fail contrast.
4. Is one shared `#313244` surface defensible across most dark and solo variants?
5. Should active/inactive/hover/selected/focused be distinct roles instead of reusing `accent`, `surface`, and `muted`?
6. Should code semantics have independent tokens? The current mapping makes red both errors and keywords, yellow both warnings and types, and accent both focus and functions/macros.
7. Should syntax palettes be exported to tools that support custom themes, reducing nearest-built-in drift?
8. Should fixed Delta diff backgrounds be replaced with per-theme derived add/delete surfaces, especially for light themes?
9. Should status semantics use dedicated success/info/warning/error colors rather than positional hue names?
10. Should a contrast-aware foreground be computed for every accent/green/yellow/orange/red/cyan background rather than always using `bg`?
11. Should solo theme families honor `mode` by gaining an alternate variant or explicitly reject incompatible mode values?
12. Should the palette add purple/magenta as a first-class slot instead of mixing it from error red and accent?
13. Should `muted` split into comment text, disabled text, inactive chrome, borders, and metadata?
14. Should `surface` split into raised surface, selected surface, hover surface, current-line surface, and status-bar surface?
15. Should LazyGit selected lines use a distinct derived selection surface?
16. Should semantic assignments be validated with automated contrast tests in addition to rendered-color presence tests?

## Source-of-truth files

- `aibox.toml`: current family/mode/variant selection and documented catalog.
- `cli/src/config.rs`: family, concrete variant, mode, variant resolution, and auto-mode behavior.
- `cli/src/themes.rs`: palettes, derivations, exact generators, and named-theme mappings.
- `cli/src/tmux/status.rs`: tmux chrome usage and inactive-pane application.
- `cli/src/seed.rs`: generated runtime files and AI-harness theme synchronization.
- `cli/src/templates/aibox-home/`: non-palette runtime templates populated by the generators.

## Constraint for recommendations

Keep provider neutrality and a single source of truth. Prefer semantic tokens in aibox that can be rendered into each tool’s native format. Do not require a provider-specific directory or make a terminal emulator mandatory. Preserve the current `theme + mode + optional variant` interface unless there is a compelling migration case.
