---
title: aibox
description: "Reproducible AI workspaces from one aibox.toml"
---

{{< blocks/cover title="Reproducible AI workspaces" image_anchor="center" height="min" color="light" >}}

<div class="mx-auto">
  <p class="lead mb-4">Generate standard devcontainer files, selected tool addons, provider-neutral agent context, and a terminal workspace from one project contract.</p>
  <a class="btn btn-lg btn-primary me-3 mb-4" href="{{< relref "/docs/getting-started/installation" >}}">
    Get started <i class="fa-solid fa-arrow-right ms-2"></i>
  </a>
  <a class="btn btn-lg btn-outline-light mb-4" href="{{< relref "/docs/overview" >}}">
    Read the overview
  </a>
</div>

{{< /blocks/cover >}}

{{% blocks/lead color="primary" %}}
`aibox.toml` is the single source of truth for container identity, addons, AI
harnesses, theme, layout, runtime settings, and processkit integration.
{{% /blocks/lead %}}

{{< blocks/section color="white" >}}
<div class="row g-4 aibox-feature-grid">
{{% blocks/feature icon="fa-solid fa-box" title="Declarative workspaces" %}}
One inspectable contract produces a reproducible development environment.
{{% /blocks/feature %}}

{{% blocks/feature icon="fa-solid fa-code-branch" title="Standard output" %}}
Dockerfile, Compose, and Dev Container files remain readable and interoperable.
{{% /blocks/feature %}}

{{% blocks/feature icon="fa-solid fa-people-group" title="Provider neutral" %}}
Agent context lives in the project, with thin provider-specific entry points.
{{% /blocks/feature %}}
</div>
{{< /blocks/section >}}

{{< blocks/section color="dark" >}}
<div class="col-12">

<h2>Quick start</h2>
<pre><code class="language-bash">
curl -fsSL https://raw.githubusercontent.com/projectious-work/aibox/main/scripts/install.sh | bash
mkdir my-project && cd my-project
aibox init my-project --harness claude --addon python
aibox apply
aibox up
</code></pre>

</div>
{{< /blocks/section >}}
