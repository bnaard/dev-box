# Tool Bundles

LLMS index: [llms.txt](/aibox/v0.x/v0.31.2/llms.txt)

---

# Tool Bundle Addons

Tool bundles install infrastructure, orchestration, and cloud CLI tools.

## Supply Chain

```toml
[addons.supply-chain.tools]
gitleaks = {}
osv-scanner = {}
syft = {}
grype = {}
cosign = {}
```

This language-neutral bundle covers secret scanning, dependency and artifact
vulnerability checks, SBOM generation, and Sigstore signing. Release archives
are pinned and checked against upstream-published checksums.

## Release Validation

```toml
[addons.release.tools]
shellcheck = {}
hadolint = {}
```

The language-neutral `release` bundle validates shell scripts and Dockerfiles.
Go projects should normally select `[addons.go.release]`, which composes this
bundle with the `go-release` recipe and GoReleaser.

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
poppler = {}
mupdf = {}
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

## Data Preview

```toml
[addons.data-preview.tools]
sqlite3 = {}
csvkit = {}
```

`data-preview` adds read-only SQLite inspection and CSV, TSV, XLS, and XLSX
formatting for the generated Yazi preview plugins.

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
podman = {}        # Optional rootless container engine + Compose
```

OpenTofu defaults to 1.12.5, Packer to 1.16.0, and Ansible to 14.2.0.
OpenTofu and Packer are installed in a multi-stage builder. Ansible is installed via pip.
Podman is optional and installs the Debian-packaged rootless engine, Compose
provider, user-namespace helpers, and overlay/networking prerequisites. Nested
containers still depend on the outer runtime allowing user namespaces; FUSE
overlay is used when `/dev/fuse` is available, with Podman's normal fallback
otherwise.

## Kubernetes

```toml
[addons.kubernetes.tools]
kubectl = {}       # Kubernetes CLI
helm = {}          # Package manager
kustomize = {}     # Configuration customization
# k9s = {}         # Optional: terminal UI for Kubernetes
```

kubectl defaults to 1.36.3, Helm to 4.2.3, Kustomize to 5.8.1, and k9s to 0.51.0.
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
