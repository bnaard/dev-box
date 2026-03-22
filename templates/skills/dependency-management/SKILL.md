---
name: dependency-management
description: Cross-language dependency management including lockfiles, version pinning, update strategies, and license compliance. Use when managing project dependencies, reviewing dependency changes, or setting up automated updates.
allowed-tools: Bash Read Write
---

# Dependency Management

## When to Use

When the user asks to:
- Set up or review lockfile strategy
- Choose a version pinning approach
- Configure automated dependency updates (Dependabot, Renovate)
- Handle monorepo dependency management
- Audit dependencies for security or license issues
- Decide whether to vendor dependencies

## Instructions

### 1. Lockfiles

Lockfiles pin the exact resolved versions of all direct and transitive dependencies.

- **Always commit lockfiles** for applications and services
- For libraries, commit the lockfile for CI reproducibility but do not publish it
- Never manually edit lockfiles -- regenerate with the package manager

| Ecosystem | Manifest | Lockfile |
|-----------|----------|----------|
| Rust | `Cargo.toml` | `Cargo.lock` |
| Python (uv/pip) | `pyproject.toml` | `uv.lock` / `requirements.txt` |
| Node.js (npm) | `package.json` | `package-lock.json` |
| Node.js (pnpm) | `package.json` | `pnpm-lock.yaml` |
| Go | `go.mod` | `go.sum` |

### 2. Version Pinning Strategies

- **Exact pins** (`==1.2.3`): Maximum reproducibility. Use for applications and Docker images.
- **Compatible range** (`^1.2.3` or `~=1.2`): Allow patch/minor updates. Use for libraries to avoid conflicts with consumers.
- **Minimum bound** (`>=1.2.0`): Widest compatibility. Use sparingly -- can lead to untested combinations.

Rule of thumb: pin tightly in applications, pin loosely in libraries.

### 3. Automated Update Workflows

**Dependabot** (GitHub native):
- Configure `.github/dependabot.yml` with ecosystems, schedule, and reviewers
- Set `open-pull-requests-limit` to avoid PR floods (5-10 is reasonable)
- Group minor/patch updates to reduce noise

**Renovate** (more flexible):
- Supports monorepos, grouping by package pattern, auto-merge for patch updates
- Configure `renovate.json` with `extends: ["config:recommended"]` as a baseline
- Use `matchPackagePatterns` to group related packages (e.g., all `@aws-sdk/*`)

For both tools:
- Require CI to pass before merging update PRs
- Auto-merge patch updates for well-tested projects
- Review major updates manually -- read the changelog

### 4. Monorepo Dependencies

- Use workspace features: Cargo workspaces, npm/pnpm workspaces, uv workspaces
- Share common dependency versions at the workspace root
- Pin shared tooling (linters, formatters) at the root level
- Let individual packages specify their own domain-specific dependencies
- Run `cargo update --workspace` or equivalent to update all packages together

### 5. Security Scanning

Integrate vulnerability scanning into CI:

- `cargo audit` (Rust) -- checks against RustSec advisory database
- `npm audit` / `pnpm audit` (Node.js) -- checks against npm advisory database
- `pip-audit` or `uv pip audit` (Python) -- checks against OSV/PyPI advisories
- `govulncheck` (Go) -- checks against Go vulnerability database

Block merges on critical/high vulnerabilities. Track accepted risks with ignore rules and expiry dates.

### 6. License Compliance

- Use SPDX identifiers for consistency (`MIT`, `Apache-2.0`, `GPL-3.0-only`)
- Maintain an allow-list of acceptable licenses for your project
- Tools: `cargo-deny` (Rust), `license-checker` (Node.js), `pip-licenses` (Python)
- Flag copyleft licenses (GPL, AGPL) in proprietary projects for legal review
- Document exceptions in a `deny.toml` or `.licensrc` with justification

### 7. Vendoring

Copy dependencies into the repository for offline builds or supply-chain control:

- Rust: `cargo vendor` + `.cargo/config.toml` with `[source.vendored-sources]`
- Go: `go mod vendor` + build with `-mod=vendor`
- Node.js: rarely vendored; use `npm pack` + local tarball if needed

Vendor when: building in air-gapped environments, pinning against registry outages, or needing full audit of dependency source.

### 8. Dependency Review Process

For every PR that changes dependencies:

1. Check **what changed**: new dependency, version bump, or removal
2. Verify **why**: is there a clear reason (feature, security fix, deprecation)?
3. Review **license**: does the new dependency's license fit the project?
4. Check **size/scope**: is the dependency proportional to the problem it solves?
5. Look for **alternatives**: is there a lighter or stdlib-based option?
6. Run **security scan**: does the new version have known vulnerabilities?

## Examples

### Example 1: Dependabot Configuration

```yaml
# .github/dependabot.yml
version: 2
updates:
  - package-ecosystem: cargo
    directory: /
    schedule:
      interval: weekly
      day: monday
    open-pull-requests-limit: 10
    reviewers:
      - "team-backend"
    groups:
      minor-and-patch:
        update-types: ["minor", "patch"]

  - package-ecosystem: npm
    directory: /frontend
    schedule:
      interval: weekly
    groups:
      react:
        patterns: ["react*", "@types/react*"]
```

### Example 2: cargo-deny for License and Advisory Checks

```toml
# deny.toml
[advisories]
vulnerability = "deny"
unmaintained = "warn"
yanked = "deny"

[licenses]
allow = ["MIT", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause", "ISC", "Unicode-3.0"]
copyleft = "deny"

[[licenses.exceptions]]
allow = ["MPL-2.0"]
crates = ["webpki-roots"]

[bans]
multiple-versions = "warn"
deny = [
    { crate = "openssl", use-instead = "rustls" },
]
```

### Example 3: Renovate with Auto-Merge (renovate.json)

```json
{
  "$schema": "https://docs.renovatebot.com/renovate-schema.json",
  "extends": ["config:recommended"],
  "packageRules": [
    {
      "matchUpdateTypes": ["patch"],
      "automerge": true,
      "automergeType": "pr",
      "requiredStatusChecks": ["ci"]
    },
    {
      "matchPackagePatterns": ["^@aws-sdk/"],
      "groupName": "AWS SDK",
      "schedule": ["before 8am on monday"]
    },
    {
      "matchUpdateTypes": ["major"],
      "labels": ["breaking-change"],
      "automerge": false
    }
  ]
}
```
