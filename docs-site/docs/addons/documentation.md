---
sidebar_position: 4
title: "Documentation Frameworks"
---

# Documentation Framework Addons

Documentation addons install static site generators and documentation tools.

| Addon | Tool | Install Method |
|-------|------|---------------|
| `docs-mkdocs` | MkDocs + Material theme | uv |
| `docs-zensical` | Zensical | uv |
| `docs-docusaurus` | Docusaurus | npm |
| `docs-starlight` | Starlight (Astro) | npm |
| `docs-mdbook` | mdBook | Binary download |
| `docs-hugo` | Hugo Extended | Binary download |

## Example

```toml
[addons.docs-docusaurus.tools]
docusaurus = {}
```

After `aibox apply`, the documentation tool is available inside the container. Initialize your docs project as usual (e.g., `npx create-docusaurus@latest docs classic`).
