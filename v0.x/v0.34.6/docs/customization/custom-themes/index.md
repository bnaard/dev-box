# Extending the theme catalog


# Extending the theme catalog

The built-in catalog is generated from one audited semantic dataset at `cli/assets/aibox-theme-corrections.toml`. Do not add a palette by copying unrelated built-in themes into individual tools: every concrete variant must define the same semantic contract and pass the same contrast checks.

## Required palette roles

Each variant defines:

- `bg`, `fg`, `accent`, `green`, `red`, `yellow`, `orange`, `cyan`, `muted`, and `magenta`;
- a chrome surface, cursor and cursor text;
- selection foreground and background;
- active and inactive borders;
- inactive-pane foreground and background;
- add, delete, and change diff backgrounds;
- readable ink for accent-filled active controls.

A family also declares its mode and variant name. Add the corresponding concrete `Theme` and user-facing `ThemeFamily` resolution entries in `cli/src/config.rs`.

## Decoration roles

The shared emphasis model maps semantic roles—not individual color literals—to attributes. Syntax keywords and types are bold; comments and decorators are italic or dim; invalid and conflicted states gain bold and, at the full level, underline. Inactive and disabled roles are dim.

When adding a renderer, clamp those roles to the target tool's supported attributes and use the documented degradation rules. Never silently remove the only channel distinguishing two roles.

## Generated consumers

A palette is complete only when these generated outputs use it:

| Consumer | Output |
|---|---|
| tmux and PowerKit | Chrome, panes, tabs, menus, status states |
| Vim | UI, syntax, search, selection, diffs |
| Yazi | Manager, tabs, modes, status, Git and file types |
| Starship | Prompt palette and decorations |
| LazyGit | Borders, selection, search and Git states |
| bat and delta | Generated TextMate theme and diff surfaces |
| fzf and eza | Color and attribute environment specifications |
| less/man | Terminal capability sequences |
| lnav | Native generated theme definition |
| OpenCode | Native generated JSON theme |
| Codex | Native generated TextMate theme in `$CODEX_HOME/themes/aibox.tmTheme` |
| Other AI TUIs | Their exposed native theme controls and terminal inheritance |

Run the theme matrix tests after any dataset or mapping change. They assert that every exposed concrete `Theme` has audited palette, chrome, cursor, and selection values and that every generated renderer resolves without placeholders.

## Gallery assets

The gallery is reproducible from the design review and audited TOML:

```bash
cd docs-site
npm install
cd ..
node scripts/generate-theme-catalog.mjs
```

The script exports all approved, Projectious, and accessibility variants from
the same TOML into `docs-site/data/theme_catalog.json`. Hugo renders the
catalogue and terminal specimens as responsive HTML.

## Local overrides

Files below `.aibox-home/` are managed output and are overwritten by `aibox apply`. For durable project-specific changes, extend the audited dataset and generator in source instead of patching generated files.


---
Source: https://projectious-work.github.io/aibox/v0.x/v0.34.6/docs/customization/custom-themes/index.md
