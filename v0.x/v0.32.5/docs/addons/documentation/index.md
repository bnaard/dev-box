# Documentation Frameworks

LLMS index: [llms.txt](/aibox/v0.x/v0.32.5/llms.txt)

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

Current curated pins are Docusaurus 3.10.1, Hugo 0.164.0, mdBook 0.5.4,
MkDocs 1.6.1 with Material 9.7.7, and Zensical 0.0.53. Starlight remains
scaffolded through the upstream `create-starlight` package.

## Example

```toml
[addons.docs-docusaurus.tools]
docusaurus = {}
```

After `aibox apply`, the documentation tool is available inside the container. Initialize your docs project as usual (e.g., `npx create-docusaurus@latest docs classic`).
