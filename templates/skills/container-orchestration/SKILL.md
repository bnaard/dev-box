---
name: container-orchestration
description: Docker Compose patterns for multi-service architectures. Health checks, networking, volumes, and service dependencies. Use when designing docker-compose files, debugging container networking, or managing multi-container applications.
allowed-tools: Bash(docker:*) Bash(docker-compose:*) Read Write
---

# Container Orchestration

## When to Use

- Writing or editing Docker Compose files
- Designing multi-service application architectures
- Debugging container networking or connectivity issues
- Setting up health checks and service dependencies
- Managing volumes, bind mounts, and persistent data
- Configuring environment variables and secrets for services
- Using Compose profiles for optional services

## Instructions

### Compose File Structure

Always specify the services, networks, and volumes at the top level. Use named volumes for persistent data and named networks for inter-service communication:

```yaml
services:
  app:
    build: .
    networks:
      - backend
    volumes:
      - app-data:/data

networks:
  backend:

volumes:
  app-data:
```

### Build vs Image

Use `build` for services you develop and `image` for third-party services. Combine both to tag locally built images:

```yaml
services:
  api:
    build:
      context: ./api
      dockerfile: Dockerfile
      args:
        PYTHON_VERSION: "3.12"
    image: myproject/api:dev
  redis:
    image: redis:7-alpine
```

### Health Checks

Always define health checks for services that other services depend on. This ensures `depends_on` with `condition: service_healthy` works correctly:

```yaml
services:
  db:
    image: postgres:16-alpine
    environment:
      POSTGRES_PASSWORD: ${DB_PASSWORD}
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 5s
      timeout: 3s
      retries: 5
      start_period: 10s
  app:
    build: .
    depends_on:
      db:
        condition: service_healthy
```

Without health checks, `depends_on` only waits for the container to start, not for the service inside to be ready.

### Networking

Compose creates a default network for each project. Use custom networks to isolate traffic:

```yaml
services:
  web:
    networks: [frontend, backend]
  api:
    networks: [backend]
  db:
    networks: [backend]

networks:
  frontend:
  backend:
```

Services resolve each other by service name as hostname. Only expose ports to the host when necessary -- inter-service communication uses the container port directly. Use `expose` for documentation, `ports` for host binding.

### Volume Strategies

- **Named volumes** -- persistent data managed by Docker, survives `docker compose down`
- **Bind mounts** -- map host paths for development (live reload)
- **tmpfs** -- in-memory, for sensitive data or caches that should not persist

```yaml
volumes:
  - pg-data:/var/lib/postgresql/data    # named volume
  - ./src:/app/src                       # bind mount
  - type: tmpfs
    target: /tmp
```

Use `docker compose down -v` to remove named volumes (destructive). Back up volume data with `docker run --rm -v pg-data:/data -v $(pwd):/backup alpine tar czf /backup/pg-data.tar.gz /data`.

### Environment Management

Use `.env` files for defaults and override per environment:

```yaml
services:
  app:
    env_file:
      - .env
      - .env.local    # overrides, git-ignored
    environment:
      - NODE_ENV=production   # explicit override
```

Never put secrets directly in `docker-compose.yml`. Use `env_file` with git-ignored files or Docker secrets for swarm mode.

### Profiles

Use profiles to make services optional:

```yaml
services:
  app:
    build: .
  mailhog:
    image: mailhog/mailhog
    profiles: [debug]
  adminer:
    image: adminer
    profiles: [debug]
```

Start with `docker compose --profile debug up` to include debug services. Services without a profile always start.

### Common Commands

```bash
docker compose up -d              # start all services detached
docker compose up -d --build      # rebuild images before starting
docker compose ps                 # list running services
docker compose logs -f app        # follow logs for a service
docker compose exec app sh        # shell into running container
docker compose down               # stop and remove containers/networks
docker compose down -v            # also remove volumes (destructive)
docker compose config             # validate and display resolved config
```

See `references/compose-patterns.md` for complete architecture examples.

## Examples

### Debug a service that cannot connect to the database

```bash
# 1. Check both services are on the same network
docker compose ps
docker network inspect $(docker compose config --format json | jq -r '.networks | keys[0]')
# 2. Verify the db is healthy
docker compose exec db pg_isready -U postgres
# 3. Test connectivity from the app container
docker compose exec app ping db
docker compose exec app nc -zv db 5432
# 4. Check environment variables
docker compose exec app env | grep DB
```

### Add a new service with zero-downtime restart

```bash
# 1. Add the service to docker-compose.yml
# 2. Validate the config
docker compose config --quiet
# 3. Start only the new service (existing services unaffected)
docker compose up -d new-service
# 4. Verify health
docker compose ps new-service
docker compose logs new-service
```

### Run one-off commands in a service context

```bash
# Database migration
docker compose run --rm app python manage.py migrate
# Interactive database shell
docker compose exec db psql -U postgres
# Run tests with the full service stack
docker compose run --rm app pytest
```
