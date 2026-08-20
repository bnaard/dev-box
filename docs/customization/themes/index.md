# Color themes

> Configure audited color palettes and font decoration across the aibox terminal toolchain.


# Color themes

aibox themes are coordinated semantic systems, not a terminal-background setting. One selection controls terminal chrome, syntax, diffs, status states, prompts, file managers, Git tools, pagers, and supported AI TUIs.

[Open the complete 76-variant screenshot gallery]({{< relref "/themes" >}}).

## Configure a theme

Choose a family, light or dark mode, an optional family variant, and a font-decoration level:

```toml
[customization]
theme = "projectious"
mode = "dark"          # auto | light | dark
variant = "deep"       # optional; family-specific
emphasis = "auto"      # auto | full | standard | minimal | none
```

Run `aibox apply` after editing. `mode = "auto"` follows the host appearance when aibox can detect it and otherwise resolves to dark. Containers do not receive host appearance changes live, so apply again after changing the host setting.

Legacy concrete names such as `catppuccin-mocha` still parse, but the family form is canonical.

## Font decoration

Color is only one information channel. `emphasis` carries meaning through bold, italic, dim, underline, and strikethrough where the target tool supports them.

| Value | Decorations | Intended use |
|---|---|---|
| `auto` | Capability detection, then graceful degradation | Recommended default |
| `full` | Bold, italic, dim, underline, strikethrough | Terminals and fonts with complete style support |
| `standard` | Bold, italic, dim | Normal capable terminals |
| `minimal` | Bold, dim | Fonts without a true italic face |
| `none` | No font decoration | Color-only compatibility mode |

`NO_COLOR` makes `auto` resolve to `none`. When terminfo is available, aibox checks italic and dim capabilities; an inconclusive probe assumes `standard`.

Unsupported attributes degrade to another channel: italic becomes dim, underline becomes bold, and strikethrough becomes dim. Bold is retained. Mono and high-contrast variants require at least the standard typography channel; `max` requires explicit `emphasis = "full"` because several semantic roles intentionally share a color.

Override individual semantic roles when a project needs a stronger cue:

```toml
[customization.emphasis_overrides]
code_comment = "italic dim"
status_error = "bold underline"
```

Keys are semantic roles rather than tool-specific settings. Values may contain `bold`, `italic`, `dim`, `underline`, and `strikethrough`; aibox validates them, clamps them to the selected level, and degrades unsupported attributes per tool. An explicit `emphasis = "none"` still disables overrides.

![Projectious navy terminal example](/img/themes/variants/projectious-navy.png)

## Families and variants

Set the value in the Variant column with `variant = "…"`. Leave it unset for the default shown first.

Single-mode families reject an incompatible explicit mode, and every family rejects unknown or mode-incompatible variants with a message listing its available choices.

| Family | Dark | Light | Variants |
|---|---|---|---|
| `andromeeda` | default | — | — |
| `aurora-x` | default | — | — |
| `ayu` | default, mirage | default | `mirage` |
| `catppuccin` | mocha, macchiato, frappe | latte | `macchiato`, `frappe` |
| `contrast` | high, max | high, max | `max` |
| `contrast-mono` | high, max | high, max | `max` |
| `dracula` | default, soft | — | `soft` |
| `everforest` | default | default | — |
| `github` | default, dimmed, high contrast | default, high contrast | `dimmed`, `high-contrast-dark`, `high-contrast-light` |
| `gruvbox` | default | default | — |
| `houston` | default | — | — |
| `kanagawa` | wave, dragon | lotus | `dragon` |
| `laserwave` | default | — | — |
| `material` | default, ocean, palenight, darker | lighter | `ocean`, `palenight`, `darker` |
| `min` | default | default | — |
| `mono` | default | default | — |
| `monokai` | default | — | — |
| `moonlight` | default | — | — |
| `night-owl` | default | default | — |
| `nord` | default | — | — |
| `one-dark` | pro | one light | — |
| `plastic` | default | — | — |
| `poimandres` | default | — | — |
| `projectious` | navy, deep, high contrast | default, high contrast | `deep`, `high-contrast-dark`, `high-contrast-light` |
| `red` | default | — | — |
| `rose-pine` | default, moon | dawn | `moon` |
| `slack` | default | ochin | `ochin` |
| `snazzy` | — | default | — |
| `solarized` | default | default | — |
| `synthwave-84` | default | — | — |
| `tokyo-night` | default, storm | day | `storm` |
| `vesper` | default | — | — |
| `vitesse` | default, black | default | `black` |
| `vscode` | Dark+ | Light+ | — |

### Accessibility families

`mono` is a practical grayscale theme with a 4.5:1 text floor. `contrast` keeps distinct hues while raising every text role to at least 7:1 in `high` and 12:1 in `max`. `contrast-mono` combines those floors with grayscale and therefore depends most strongly on font decoration.

![Contrast dark max terminal example](/img/themes/variants/contrast-dark-max.png)

![Contrast mono light max terminal example](/img/themes/variants/contrast-mono-light-max.png)

### Projectious

Projectious is now a five-variant brand family. Navy is the default dark page, deep preserves the older code-panel depth, and both modes have high-contrast alternatives. Selection colors use a stronger midnight step so short selected ranges remain visible.

![Projectious high-contrast light terminal example](/img/themes/variants/projectious-hclight.png)

## Semantic colors

Every concrete variant authors or derives the following roles:

- Base text: background, foreground, comments/metadata, accent, green, red, yellow, orange, cyan, and magenta.
- Chrome: surface, active and inactive borders, active-tab ink, inactive-pane foreground/background, cursor and cursor text.
- Interaction: selection foreground/background and search states.
- Diffs: add, delete, change, word-level emphasis, headers, and hunks.
- Syntax: plain text, comments, operators, keywords, types, functions, strings, numbers, decorators/macros, invalid, and deprecated code.
- Status and Git: success, information, warning, error, disabled, modified, staged, untracked, ignored, and conflicted.

The shipped audit enforces a 7:1 floor for normal foreground text, 4.5:1 for colored text roles, 3:1 for non-text borders, and a visible surface/selection step.

## Governed tools

| Surface | Generated configuration |
|---|---|
| tmux and PowerKit | Pane borders, active/inactive tabs and titles, status surfaces and states |
| Vim | Complete generated syntax/UI colorscheme, selection, search, diff, and decorations |
| Yazi | Manager, tabs, modes, status, pickers, Git states, and file types |
| Starship | Prompt palette and supported style attributes |
| LazyGit | Borders, search, state colors, and supported emphasis |
| bat and delta | Shared generated TextMate syntax theme plus audited diff backgrounds |
| fzf | Selection, prompts, matches, borders, disabled state, and attributes |
| eza | File types, Git states, metadata, and ANSI attributes |
| less and man | Heading, option, search, and status capabilities |
| lnav | Generated native theme definition for text, selection, status, warnings, and errors |
| OpenCode | Exact generated JSON palette for UI, Markdown, diffs, and syntax |
| Codex | Exact generated TextMate palette at `.codex/themes/aibox.tmTheme`; `tui.theme = "aibox"` selects it |
| Claude Code | Built-in dark/light modes; mono and contrast families use the ANSI modes |
| Gemini CLI | Closest supported built-in Gemini theme |
| Aider | Light/dark mode plus the closest supported Pygments code theme |
| Tau, Hermes, Copilot, Continue, Cursor | Terminal/tmux inheritance; these harnesses currently have no aibox-generated native palette |

Generated files live below `.aibox-home/` and are refreshed by `aibox apply` or the theme-switch command. Running TUI processes may need a restart.

## Switch without rebuilding

```bash
aibox set theme.name projectious
aibox set theme.mode dark
aibox apply
```

To recreate the checked-in gallery after changing audited data:

```bash
node scripts/capture-theme-variants.mjs
```


---
Source: https://projectious-work.github.io/aibox/docs/customization/themes/index.md
