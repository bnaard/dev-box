# Custom Prompts


# Creating Custom Prompts

aibox generates Starship prompt configurations from the selected preset and theme. You can customize the prompt by editing the generated config.

## Generated Config Location

After `aibox apply`, the Starship config is at:

```
.aibox-home/.config/starship.toml
```

## Manual Customization

Edit `.aibox-home/.config/starship.toml` directly with any valid [Starship configuration](https://starship.rs/config/). Changes take effect immediately in new shell sessions.

{{< callout type="warning" title="Apply overwrites" >}}
`aibox apply` regenerates `starship.toml` from the preset and theme. To preserve manual edits, either avoid running apply or back up your config first.
{{< /callout >}}

## Adding Custom Presets

Custom presets can be added to `cli/src/themes.rs` in the `starship_config()` function. Each preset is a Starship TOML template with color variables (`{bg}`, `{fg}`, `{accent}`, `{green}`) that are replaced with theme-specific values at generation time.

See the existing presets (default, plain, minimal, nerd-font, pastel, powerline-pastel, bracketed, arrow) as reference patterns.


---
Source: https://projectious-work.github.io/aibox/v0.x/v0.34.3/docs/customization/custom-prompts/index.md
