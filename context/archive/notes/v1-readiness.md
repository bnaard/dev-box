# v1.0.0 Readiness Plan

Based on research of how uv, Ruff, Zellij, and other projects approach v1.0.

## What SemVer 1.0.0 Means

"Version 1.0.0 defines the public API. The moment you declare a public API and commit to backward compatibility, you are at 1.0.0."

Note: uv (v0.10.x), Ruff (v0.15.x), and Zellij (v0.41.x) are all successful, widely-used projects that remain pre-1.0. There is no rush.

## Checklist Before v1.0.0

### API Stability
- [ ] `aibox.toml` schema is stable and documented
- [ ] CLI commands, flags, and exit codes are stable
- [ ] Published a versioning policy (what's breaking vs non-breaking)
- [ ] Preview mode for experimental features (e.g., `--preview` flag)

### Quality
- [ ] Comprehensive test coverage (all public API paths)
- [ ] cargo clippy clean, cargo fmt
- [ ] Zero panics in normal usage
- [ ] Clear, actionable error messages for all failure modes

### Security
- [ ] cargo audit clean (no known CVEs in dependencies)
- [ ] cargo deny check (license compliance, banned crates)
- [ ] All 8 images scanned with Trivy, no HIGH/CRITICAL
- [ ] Binary downloads verified with checksums
- [ ] SBOM generated for each published image

### Documentation
- [ ] Complete user-facing docs for all features
- [ ] CLI help text covers all commands and flags
- [ ] Configuration reference with all fields documented
- [ ] Migration guide from 0.x to 1.0
- [ ] Contributing guide
- [ ] Architecture overview

### Real-World Usage
- [ ] Multiple derived projects using aibox in production
- [ ] Feedback incorporated from external users
- [ ] Known issues documented and prioritized

### Release Process
- [ ] Documented, repeatable release process
- [ ] Changelog maintained for every release
- [ ] Container image signing (sigstore/cosign)
- [ ] Install script verification documented

## Recommended Security Tooling

| Tool | Purpose | Integration point |
|------|---------|-------------------|
| cargo audit | Rust CVE checking | CI + aibox doctor |
| cargo deny | License + security policy | CI |
| Trivy | Container image scanning | Image build script |
| Syft | SBOM generation | Image release |
| pip-audit | Python dep scanning | CI for python images |

## Timeline Estimate

v1.0.0 should happen when the addon system, AI flexibility, and aibox sync are stable and battle-tested. Realistic target: after v0.6 or v0.7, once the core UX has been validated by multiple derived projects.
