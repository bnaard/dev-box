---
argument-hint: >
  --root=<path> [--discover-only] [--run-security]
  [--run-quality] [--manifest <path>...]
  [--export-sbom <path>]
allowed-tools: []
---

Run the supply-chain-audit skill.

Default behavior:

- Discover manifests under `--root` (or the project root).
- Run local manifest checks only.
- Keep security and quality probes off unless `--run-security`
  and/or `--run-quality` are passed.

