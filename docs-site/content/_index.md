+++
title = "aibox"
description = "Reproducible AI workspaces from one aibox.toml"
eyebrow = "reproducible AI workspaces"
tagline = "Generate a predictable development environment, selected tools, provider-neutral agent context, and a terminal workspace from one project contract."
[[cta]]
  label = "Get started"
  href = "/docs/getting-started/installation/"
[[cta]]
  label = "Read the overview"
  href = "/docs/overview/"
  variant = "secondary"
+++

## One contract, a complete workspace

`aibox.toml` is the single source of truth for container identity, addons, AI
harnesses, theme, layout, runtime settings, and processkit integration.

{{< cards >}}
  {{< card title="Declarative workspaces" subtitle="One inspectable contract produces a reproducible development environment." link="/docs/" icon="settings" >}}
  {{< card title="Standard output" subtitle="Dockerfile, Compose, and Dev Container files remain readable and interoperable." link="/docs/overview/" icon="file-code" >}}
  {{< card title="Provider neutral" subtitle="Agent context lives in the project, with thin provider-specific entry points." link="/docs/context/" icon="users" >}}
{{< /cards >}}

## Quick start

{{< terminal title="create a workspace" >}}
curl -fsSL https://raw.githubusercontent.com/projectious-work/aibox/main/scripts/install.sh | bash
mkdir my-project && cd my-project
aibox init my-project --harness claude --addon python
aibox apply
aibox up
{{< /terminal >}}
