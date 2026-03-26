---
sidebar_position: 2
title: "Container Configuration"
---

# Container Configuration

The `[container]` section in `aibox.toml` controls per-project container settings.

## Container Identity

```toml
[container]
name = "my-project"        # Container name (used by compose)
hostname = "my-project"    # Container hostname
user = "aibox"             # Container user (default: aibox)
```

The `user` field determines the non-root user inside the container. The default `aibox` user (UID 1000) is recommended. Set `user = "root"` only if needed for specific tools.

## Port Forwarding

```toml
[container]
ports = ["8080:80", "5432:5432"]
```

Maps host ports to container ports in `host:container` format. These are passed directly to docker-compose.

## Extra Packages

Install additional apt packages not covered by addons:

```toml
[container]
extra_packages = ["universal-ctags", "graphviz", "postgresql-client"]
```

These are installed via `apt-get install` in the generated Dockerfile.

## VS Code Extensions

Auto-install VS Code extensions when using the devcontainer:

```toml
[container]
vscode_extensions = ["eamodio.gitlens", "esbenp.prettier-vscode"]
```

Addon-specific extensions (Python, Rust, LaTeX, etc.) are added automatically when the corresponding addon is enabled.

## Post-Create Command

Run a command after the container is first created:

```toml
[container]
post_create_command = "npm install"
```

This maps to devcontainer.json's `postCreateCommand`.

## Extra Volumes

Mount additional host directories into the container:

```toml
[[container.extra_volumes]]
source = "/host/data"
target = "/container/data"
read_only = true
```

Each volume entry has:
- `source` — host path
- `target` — container path
- `read_only` — optional, defaults to `false`

## Environment Variables

Set environment variables inside the container:

```toml
[container.environment]
DATABASE_URL = "postgres://localhost/mydb"
NODE_ENV = "development"
```

## Network Keepalive

Prevent OrbStack/VM NAT from dropping idle connections:

```toml
[container]
keepalive = true
```

This sends a lightweight DNS lookup every 2 minutes via the devcontainer `postStartCommand`.

## Compose Override

For project-specific services (databases, sidecars, test companions), use Docker Compose's standard override mechanism. During `aibox init`, an empty `.devcontainer/docker-compose.override.yml` is scaffolded with example usage.

Docker Compose automatically merges the override file with the generated `docker-compose.yml` using a strategic merge — maps (services, environment) are deep-merged by key, lists (ports, volumes) are appended, and scalars (image, command) are replaced.

When `aibox sync` detects the override file, it wires both files into `devcontainer.json` so VS Code picks them up.

**Example — add a PostgreSQL sidecar:**

```yaml
services:
  postgres:
    image: postgres:16
    environment:
      POSTGRES_PASSWORD: dev
    ports:
      - "5432:5432"
```

**Example — add `depends_on` to the main service:**

```yaml
services:
  my-project:            # must match [container] name in aibox.toml
    depends_on:
      - postgres
```

:::tip
The override file is never overwritten by `aibox sync` — you own it, just like `Dockerfile.local`.
:::
