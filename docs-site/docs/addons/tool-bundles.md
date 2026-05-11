---
sidebar_position: 3
title: "Tool Bundles"
---

# Tool Bundle Addons

Tool bundles install infrastructure, orchestration, and cloud CLI tools.

## Git UI

```toml
[addons.git-ui.tools]
gh = {}        # GitHub CLI
lazygit = {}   # Interactive Git TUI
```

The `git-ui` addon is optional. Select it when a project needs GitHub CLI
automation or lazygit inside the container; omit it to avoid installing those
tools. This aibox repo may enable it for maintenance workflows such as release
checks and GitHub issue/release work, but downstream projects do not need it by
default.

## Preview and Archive Tools

```toml
[addons.preview-archive.tools]
chafa = {}
timg = {}
poppler-utils = {}
mupdf-tools = {}
entr = {}
p7zip = {}
resvg = {}
```

`preview-archive` contains the terminal image/PDF/SVG/archive helper binaries
used by Yazi previews and watch-mode document workflows. Keep it disabled for
lean headless projects that do not inspect media or generated documents inside
the terminal.

`preview-enhanced` layers Markdown, EPS, video, and Ghostscript support on top
of `preview-archive`.

## Audio and Voice

```toml
[audio]
enabled = true
```

Audio bridging uses the internal `audio-voice` recipe for Sox, PulseAudio
client tools, and ALSA PulseAudio plugins. aibox selects this recipe
automatically when `[audio] enabled = true` and `install = true`; projects normally do not need to add
`[addons.audio-voice.tools]` manually.

## Infrastructure

```toml
[addons.infrastructure.tools]
opentofu = {}      # Infrastructure-as-code (Terraform alternative)
ansible = {}       # Configuration management
packer = {}        # Machine image builder
```

OpenTofu and Packer are compiled in a multi-stage builder. Ansible is installed via pip.

## Kubernetes

```toml
[addons.kubernetes.tools]
kubectl = {}       # Kubernetes CLI
helm = {}          # Package manager
kustomize = {}     # Configuration customization
# k9s = {}         # Optional: terminal UI for Kubernetes
```

All tools are downloaded as static binaries in a multi-stage builder.

## Cloud Providers

### AWS

```toml
[addons.cloud-aws.tools]
aws-cli = {}
```

Installs the AWS CLI v2.

### Google Cloud

```toml
[addons.cloud-gcp.tools]
gcloud-cli = {}
```

Installs the Google Cloud CLI via the official APT repository.

### Azure

```toml
[addons.cloud-azure.tools]
azure-cli = {}
```

Installs the Azure CLI via pip.
