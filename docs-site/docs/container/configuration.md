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

## Post-Create Command

Run a command after the container is first created:

```toml
[container]
post_create_command = "npm install"
```

This maps to devcontainer.json's `postCreateCommand`.

## Network Keepalive

Prevent OrbStack/VM NAT from dropping idle connections:

```toml
[container]
keepalive = true
```

This sends a lightweight DNS lookup every 2 minutes via the devcontainer `postStartCommand`.

## Custom Packages, Ports, Volumes, and Environment Variables

Container customizations such as extra packages, port forwarding, volume mounts, and environment variables are handled through standard Docker mechanisms rather than `aibox.toml`.

### Extra Packages — `Dockerfile.local`

Install additional apt packages by adding them to `.devcontainer/Dockerfile.local`, which is appended to the generated Dockerfile at build time:

```dockerfile
RUN apt-get update && apt-get install -y --no-install-recommends \
    universal-ctags graphviz postgresql-client \
    && rm -rf /var/lib/apt/lists/*
```

### Ports, Volumes, and Environment Variables — `docker-compose.override.yml`

Use `.devcontainer/docker-compose.override.yml` to add port mappings, volume mounts, and environment variables:

```yaml
services:
  my-project:                # must match [container] name in aibox.toml
    ports:
      - "8080:80"
      - "5432:5432"
    volumes:
      - /host/data:/container/data:ro
    environment:
      DATABASE_URL: "postgres://localhost/mydb"
      NODE_ENV: "development"
```

Both `Dockerfile.local` and `docker-compose.override.yml` are scaffolded by `aibox init` and are never overwritten by `aibox sync`.

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
