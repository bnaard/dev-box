# src/.processkit/

Catalog tooling content — **not installed** in a consumer project.

Files here support processkit development, diffing, and packaging but are
not copied into a consumer's project root at install time.

## Contents

```
.processkit/
├── packages/            ← package tier definitions (minimal, managed, software, research, product)
│   ├── INDEX.md
│   ├── minimal.yaml
│   ├── managed.yaml
│   ├── software.yaml
│   ├── research.yaml
│   └── product.yaml
├── FORMAT.md            ← entity format spec (primitives documentation)
├── primitives-INDEX.md  ← primitives catalog index
└── INDEX.md             ← this file
```

## packages/

Each `.yaml` file defines one package tier — the set of skills that aibox
installs for a project using that tier. Referenced by `aibox.toml` via
`context.packages = [...]`.

## FORMAT.md

The entity format specification: how primitives are structured, the
YAML-frontmatter schema, the `privacy` field semantics, the `kind`
registry.

## primitives-INDEX.md

Index of all 19 processkit primitives with descriptions. Useful as a
quick reference when deciding which entity kind to use.
