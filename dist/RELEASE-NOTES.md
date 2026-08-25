# aibox v0.35.0 — 2026-08-25

**Summary:** This minor release adds six period-terminal theme variants and
three opt-in graphics-rendering addons for reproducible Hugo and documentation
builds. No processkit migration is required.

## Added

- Add Borland, Norton, and Phosphor theme families, each with a
  period-authentic Classic variant and a higher-contrast Optimized variant.
- Add the `diagramming` addon with checksum-verified D2 0.7.1 binaries for
  Linux AMD64/ARM64 and Graphviz.
- Add the Node-backed `data-visualization` addon with Vega CLI 6.4.0 and
  Vega-Lite 6.4.3.
- Add the Node-backed `mermaid` addon with Mermaid CLI 11.16.0, Puppeteer
  25.9.0, and a compatible Chrome headless shell.

## Changed

- Expand the generated terminal theme catalog from 76 to 82 concrete themes
  across tmux, Yazi, Vim, Bat, Lazygit, Starship, and documentation previews.
- Publish the new addon definitions through both the embedded CLI catalog and
  the standalone installer.
- Update the README, compatibility matrix, public changelog, addon guides, and
  documentation version menu for v0.35.0.

## Compatibility

- Minimum processkit version remains v0.28.8.
- Graphics addons remain opt-in and are not implicit dependencies of Hugo.
- D2 supports Linux AMD64 and ARM64 release artifacts in this version.
- Mermaid's bundled browser runtime is intentionally heavyweight; projects
  with an external compatible Chrome installation can disable Puppeteer and
  provide their own configuration.

## Upgrade notes

Run `aibox apply` to refresh generated configuration comments and runtime theme
files. Enable only the graphics addons required by the project's build:

```toml
[addons.diagramming.tools]

[addons.data-visualization.tools]

[addons.mermaid.tools]
```

[v0.35.0]: https://github.com/projectious-work/aibox/compare/v0.34.7...v0.35.0
