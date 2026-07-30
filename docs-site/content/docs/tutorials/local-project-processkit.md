---
title: "Create a local project with processkit"
description: "Build and deploy a new project locally with aibox v1, then install the processkit process base."
weight: 1
---

# Create a local project with processkit

This tutorial starts with an empty directory, deploys a workspace through the
v1 Compose backend, and installs the processkit v1 process base into the
project. Both products are prereleases, so the example uses exact versions.

## Before you begin

Install a Docker- or Podman-compatible Compose runtime, `curl`, `tar`, and
`sha256sum`. Then install the exact aibox v1 release shown on the
[releases page](https://github.com/projectious-work/aibox/releases):

```sh
curl -fsSL https://raw.githubusercontent.com/projectious-work/aibox/main/scripts/install.sh |
  VERSION=1.0.0-alpha.1 bash
aibox --version
```

If that release is not published yet, use the latest v1 prerelease shown on the
releases page and substitute its version below.

## Scaffold the project

```sh
mkdir hello-aibox
cd hello-aibox
aibox init hello-aibox --context-mode harness-only --harness codex --no-container
```

`harness-only` keeps aibox's legacy processkit installer out of the project.
The process base is installed later by the processkit v1 CLI, through its own
native boundary.

Add an application and image source:

```dockerfile
# image-source/Containerfile
FROM docker.io/library/alpine:3.22
CMD ["sh", "-c", "while true; do date; sleep 30; done"]
```

Create `image-source/` and save the file there. Build and push the image to a
registry you can access:

```sh
aibox image build --push --output json
```

Before running that command, add a complete `[orchestration]` contract to
`aibox.toml`. Replace the image reference, digest, and owner with your values:

```toml
[orchestration]
enabled = true

[orchestration.image]
reference = "ghcr.io/YOUR-ACCOUNT/hello-aibox:dev"
digest = "sha256:0000000000000000000000000000000000000000000000000000000000000000"
platform = "linux-amd64"

[orchestration.image.build]
context = "image-source"
dockerfile = "Containerfile"

[orchestration.fleet]
name = "hello-aibox"
services = [{ name = "workspace" }]

[orchestration.target]
backend = "compose"
reference = "docker-context:default"
scope = "hello-aibox"

[orchestration.deployment]
name = "hello-aibox-local"
owner_id = "YOUR-STABLE-OWNER-ID"

[[orchestration.connections]]
name = "shell"
service = "workspace"
transport = "compose-exec"
command = ["sh"]
```

The first build uses that syntactically valid, all-zero placeholder digest.
After the push,
copy the returned registry manifest digest into `orchestration.image.digest`.
The deployment never consumes a mutable tag by itself.

## Plan and deploy locally

```sh
aibox config compile --output json
aibox deploy plan --output json
aibox deploy apply --output json
aibox deploy status
aibox connect shell
```

The plan is read-only. Apply creates only the Compose resources carrying this
deployment's ownership labels and writes its local deployment record.

## Install the processkit process base

Download one exact processkit v1 release, verify both release assets, and run
the native installer. The filenames and checksums are published with every
[processkit release](https://github.com/projectious-work/processkit/releases).

```sh
PROCESSKIT_VERSION=v1.0.0-alpha.4
PROCESSKIT_TARGET=aarch64-unknown-linux-gnu
PROCESSKIT_DIR=.aibox/processkit-release
mkdir -p "${PROCESSKIT_DIR}"

curl -fL -o "${PROCESSKIT_DIR}/processkit.tar.gz" \
  "https://github.com/projectious-work/processkit/releases/download/${PROCESSKIT_VERSION}/processkit-${PROCESSKIT_VERSION}.tar.gz"
curl -fL -o "${PROCESSKIT_DIR}/processkit.tar.gz.sha256" \
  "https://github.com/projectious-work/processkit/releases/download/${PROCESSKIT_VERSION}/processkit-${PROCESSKIT_VERSION}.tar.gz.sha256"
curl -fL -o "${PROCESSKIT_DIR}/processkit" \
  "https://github.com/projectious-work/processkit/releases/download/${PROCESSKIT_VERSION}/processkit-${PROCESSKIT_VERSION}-${PROCESSKIT_TARGET}"
curl -fL -o "${PROCESSKIT_DIR}/processkit.sha256" \
  "https://github.com/projectious-work/processkit/releases/download/${PROCESSKIT_VERSION}/processkit-${PROCESSKIT_VERSION}-${PROCESSKIT_TARGET}.sha256"

(cd "${PROCESSKIT_DIR}" && sed 's/  .*/  processkit.tar.gz/' processkit.tar.gz.sha256 | sha256sum --check)
(cd "${PROCESSKIT_DIR}" && sed 's/  .*/  processkit/' processkit.sha256 | sha256sum --check)
chmod +x "${PROCESSKIT_DIR}/processkit"
tar -xzf "${PROCESSKIT_DIR}/processkit.tar.gz" -C "${PROCESSKIT_DIR}"

"${PROCESSKIT_DIR}/processkit" install \
  --root . \
  --distribution "${PROCESSKIT_DIR}/processkit-${PROCESSKIT_VERSION}" \
  --profile managed \
  --harness codex \
  --yes
"${PROCESSKIT_DIR}/processkit" verify --root . --json
```

Choose the target matching your host (`aarch64-unknown-linux-gnu` is the
current v1 alpha target). The `managed` profile installs the shared process
base, canonical `AGENTS.md`, and runtime policy. `verify` checks installed
provenance and managed-path drift without changing the project.

Commit `aibox.toml`, `aibox.lock`, `AGENTS.md`, and the installed `context/`
tree. Do not commit `.aibox/processkit-release/`; it is only a download staging
directory.

## Clean up

```sh
aibox deploy destroy --output json
```

Destroy refuses resources that do not match the recorded identity and ownership
labels.
