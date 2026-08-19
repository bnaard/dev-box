# Installation


# Installation

## Prerequisites

aibox requires a container runtime and a Compose-compatible provider on your
host machine.

### Podman (recommended)

```bash
# macOS
brew install podman
podman machine init
podman machine start

# Fedora / RHEL
sudo dnf install podman podman-compose

# Ubuntu / Debian
sudo apt install podman podman-compose
```

### Docker

```bash
# macOS
brew install --cask docker
# Then launch Docker Desktop

# Linux — follow the official install guide
# https://docs.docker.com/engine/install/
```

aibox auto-detects which runtime is available. If both are installed, Podman takes priority.
OrbStack works through its Docker-compatible runtime and Compose integration.

## Install script (recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/projectious-work/aibox/main/scripts/install.sh | bash
```

Downloads the correct pre-built binary for your platform (Linux or macOS, x86_64 or ARM64) and installs it to `~/.local/bin/`.

Options:

```bash
# Install a specific version
curl -fsSL https://raw.githubusercontent.com/projectious-work/aibox/main/scripts/install.sh | VERSION=X.Y.Z bash

# Install to a custom directory
curl -fsSL https://raw.githubusercontent.com/projectious-work/aibox/main/scripts/install.sh | INSTALL_DIR=/usr/local/bin sudo -E bash
```

## Manual download

Download the binary for your platform from the [releases page](https://github.com/projectious-work/aibox/releases):

```bash
# Example for macOS ARM64
tar xzf aibox-vX.Y.Z-aarch64-apple-darwin.tar.gz
mv aibox-vX.Y.Z-aarch64-apple-darwin ~/.local/bin/aibox
chmod +x ~/.local/bin/aibox
```

Replace `X.Y.Z` with the release version you downloaded.

Available binaries:

| Platform | File |
|----------|------|
| macOS ARM64 (Apple Silicon) | `aibox-vX.Y.Z-aarch64-apple-darwin.tar.gz` |
| macOS x86_64 (Intel) | `aibox-vX.Y.Z-x86_64-apple-darwin.tar.gz` |
| Linux ARM64 | `aibox-vX.Y.Z-aarch64-unknown-linux-gnu.tar.gz` |
| Linux x86_64 | `aibox-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz` |

## Build from source

Requires a [Rust toolchain](https://rustup.rs/):

```bash
git clone https://github.com/projectious-work/aibox.git
cd aibox
cargo install --path cli
```

Installs the binary to `~/.cargo/bin/`.

## Verify

```bash
aibox --version
# aibox X.Y.Z
```

## Shell completion scripts

```bash
# Add to your shell profile for persistent completion scripts:

# Bash (~/.bashrc)
eval "$(aibox self completion bash)"

# Zsh (~/.zshrc)
eval "$(aibox self completion zsh)"

# Fish (~/.config/fish/config.fish)
aibox self completion fish | source
```

## Next steps

- [Create a new project](new-project.md)
- [Add aibox to an existing project](existing-project.md)
- [Read the overview](../overview.md)


---
Source: https://projectious-work.github.io/aibox/v0.x/v0.34.0/docs/getting-started/installation/index.md
