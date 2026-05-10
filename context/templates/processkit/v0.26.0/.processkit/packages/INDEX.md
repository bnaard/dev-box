# src/packages/

Package definitions — opinionated bundles of skills that a consumer selects
in their `aibox.toml`:

```toml
[context]
packages = ["managed"]
```

Packages compose via `spec.extends` — a package implicitly includes everything
from its parents. Consumers can also add specific skills with
`[skills] include = [...]` or remove them with `[skills] exclude = [...]`.

## The five tiers

| Package    | Extends      | Intended for                                               |
|------------|--------------|------------------------------------------------------------|
| `minimal`  | —            | Solo developers and small side projects.                   |
| `managed`  | `minimal`    | Small teams with a shared backlog and process cadences.    |
| `software` | `managed`    | Software engineering teams building production systems.   |
| `research` | `managed`    | Research, data science, and ML engineering projects.       |
| `product`  | `software`   | Full product development (engineering + design + ops).   |

`managed` is the recommended default. Start there; graduate to `software`,
`research`, or `product` when the project takes shape.

## Composition rule

`spec.extends` is a list of package names. The effective skill set is:

```
effective_skills(pkg) = union(effective_skills(p) for p in pkg.spec.extends)
                       ∪ pkg.spec.includes.skills
```

Cycles are not allowed. `aibox lint` (when it validates packages) will reject
circular extensions.

## Package file format

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: Package
metadata:
  id: PKG-<name>
  name: <name>
  version: "<semver>"
  created: <iso8601>
spec:
  description: "..."
  intended_for: "..."
  extends: [<parent-package>]        # optional
  includes:
    skills:
      - <skill-name>
      - ...
---
```

## Overrides in aibox.toml

Consumers can fine-tune a selected package without creating their own:

```toml
[context]
packages = ["software"]

[skills]
include = ["logo-design"]      # add to software
exclude = ["terraform-basics"] # remove from software
```

For deeper customization, projects should create their own package file
under `context/packages/` and select it instead.
