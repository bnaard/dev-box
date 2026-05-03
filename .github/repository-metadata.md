# GitHub Repository Metadata

GitHub repository metadata is configured through GitHub, not through a file in
the repository. Keep this file as the source text to apply when repository
metadata needs to be refreshed.

## Short Description

Reproducible AI-ready devcontainers from one aibox.toml, with addons,
provider-neutral agent context, and processkit integration.

## Homepage

https://projectious-work.github.io/aibox/

## Topics

- ai-development
- devcontainers
- developer-tools
- rust-cli
- processkit
- zellij
- docker
- podman
- reproducible-environments

## Suggested Command

```bash
gh repo edit projectious-work/aibox \
  --description "Reproducible AI-ready devcontainers from one aibox.toml, with addons, provider-neutral agent context, and processkit integration." \
  --homepage "https://projectious-work.github.io/aibox/" \
  --add-topic ai-development \
  --add-topic devcontainers \
  --add-topic developer-tools \
  --add-topic rust-cli \
  --add-topic processkit \
  --add-topic zellij \
  --add-topic docker \
  --add-topic podman \
  --add-topic reproducible-environments
```
