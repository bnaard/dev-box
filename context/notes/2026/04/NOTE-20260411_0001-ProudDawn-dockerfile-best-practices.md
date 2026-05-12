---
apiVersion: processkit.projectious.work/v2
kind: Note
metadata:
  id: NOTE-20260411_0001-ProudDawn-dockerfile-best-practices
  created: 2026-04-11 00:00:00+00:00
spec:
  title: Dockerfile Best Practices
  body: See Markdown body below.
  type: reference
  state: permanent
  tags:
  - docker
  - dockerfile
  - images
  - security
  - best-practices
  skill: dockerfile-review
---

# Dockerfile Best Practices

Reference for reviewing aibox image Dockerfiles. Apply during every image change.

## Layer Optimization

- Combine related `RUN` commands with `&&`
- Order by change frequency: base image → system deps → app deps → app code
- Add `# syntax=docker/dockerfile:1` at top for BuildKit features

## Cache Mounts

```dockerfile
# apt-get (use sharing=locked)
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && apt-get install -y --no-install-recommends gcc
```

With cache mounts, do NOT use `rm -rf /var/lib/apt/lists/*`.

## Version Pinning

- Pin base images by digest for production: `FROM debian:trixie-slim@sha256:...`
- Pin critical binary versions via ARG (already doing this for tmux, Yazi, etc.; aibox v0.25.x migrated the multiplexer from Zellij to tmux — see NobleCrane)

## Binary Downloads

- Always verify checksums for downloaded binaries
- Use `--fail-with-body` or `-fsSL` for curl
- Pin versions explicitly

## Security

- Use `--no-install-recommends` for apt-get
- No secrets in build layers — use BuildKit secret mounts if needed
- Consider non-root USER for derived project containers

## Size

- Multi-stage builds for compile steps (TeX Live, Yazi; pre-v0.25.x also Zellij — replaced by tmux/NobleCrane and no longer compiled from source)
- `--no-install-recommends` saves 20-50% on apt packages
- Remove unnecessary files in the same layer they're created

## Regular Review Cycle

```bash
# Scan image for CVEs
trivy image ghcr.io/projectious-work/aibox:base-latest

# Check Rust deps
cargo audit
cargo deny check

# Generate SBOM
syft ghcr.io/projectious-work/aibox:base-latest -o cyclonedx-json
```
