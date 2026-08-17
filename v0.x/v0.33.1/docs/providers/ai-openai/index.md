# OpenAI (Codex CLI)

LLMS index: [llms.txt](/aibox/v0.x/v0.33.1/llms.txt)

---

# OpenAI Codex CLI

[Codex CLI](https://github.com/openai/codex) is OpenAI's open-source terminal coding agent. Built in Rust, GA since April 2025.

## Setup

```toml
[ai]
harnesses = [
  { harness = "codex", enable = true, install = true },
]
```

Run `aibox apply`, then inside the container:

```bash
codex    # Launches OpenAI Codex CLI
```

## API Key

```toml
[container.environment]
OPENAI_API_KEY = "sk-..."
```

Alternatively, use a ChatGPT Plus/Pro/Team/Enterprise account — Codex prompts for authentication on first launch.

## Configuration

Codex's home-directory state is persisted in `.aibox-home/.codex/`, mounted at `/home/aibox/.codex/`. This survives devcontainer rebuilds, so device sign-in only needs to be completed once per host cache unless you clear it.

Key files:
- `.aibox-home/.codex/auth.json` — cached ChatGPT/device authentication reused across rebuilds
- `.aibox-home/.codex/rules/` — home-directory Codex rules and local state
- `.aibox-home/.codex/sessions/` — Codex session history
- `.aibox-home/.codex/prompts/pk-*.md` — generated processkit custom-prompt aliases

Separately, aibox also generates project-local `.codex/config.toml` MCP server registration. In processkit mode it also generates `.codex/hooks.json` processkit hook configuration.

## processkit commands

After `aibox apply`, processkit workflows are available through both Codex
invocation surfaces:

- Type `$pk-resume`, `$pk-doctor`, and similar names to invoke the generated
  project skills under `.agents/skills/`. You can also select them through
  `/skills`.
- Type `/prompts:pk-resume`, `/prompts:pk-doctor`, and similar names to use the
  generated custom-prompt aliases persisted under
  `.aibox-home/.codex/prompts/`.

Codex reserves top-level slash commands and does not support registering a
custom `/pk-resume` command. Custom prompts always use the `/prompts:`
namespace. Restart the Codex session after the first `aibox apply` if newly
generated prompt aliases do not appear immediately.

## MCP Integration

Codex has a native MCP client. aibox generates `.codex/config.toml` automatically on `aibox apply`, merging processkit MCP entries in processkit mode, team servers from `aibox.toml [ai.mcp]`, and personal servers from `.aibox-local.toml [mcp]`. With current processkit releases and `[ai.mcp.gateway].mode = "auto"`, Codex uses the `processkit-gateway` stdio proxy instead of one Python process per skill.

`.codex/config.toml` and `.codex/hooks.json` are **gitignored** — they are regenerated on every `aibox apply` and must not be committed.

To add MCP servers:

```toml
# aibox.toml — team-shared servers
[[ai.mcp.servers]]
name    = "github"
command = "npx"
args    = ["-y", "@modelcontextprotocol/server-github"]

# .aibox-local.toml — personal servers
[[mcp.servers]]
name    = "my-internal-tool"
command = "npx"
args    = ["-y", "@acme/internal-mcp-server"]
```

## Installation

Codex CLI is installed via npm (`npm install -g @openai/codex`). To pin a specific version, set it in `aibox.toml`:

```toml
[ai]
harnesses = [
  { harness = "codex", enable = true, install = true, version = "0.1.0" },
]
```

## Sandbox prerequisites

aibox images include Debian's `bubblewrap` package so Codex can use the OS-provided Linux sandbox helper instead of falling back to its vendored copy. Codex still needs the container runtime and host kernel to allow unprivileged user namespaces; if namespace creation is blocked, Codex can start but sandboxed shell commands fail before the project command runs.

The preferred fix is to enable unprivileged user namespaces on the host or container runtime. Avoid `privileged: true` and avoid adding `SYS_ADMIN` to the main development container for Codex; those grants are broader than Codex's bubblewrap sandbox requires. When Codex is selected, generated `docker-compose.yml` includes a narrow `security_opt: seccomp=unconfined` fallback because Docker/Podman seccomp profiles can block bubblewrap before the project command runs:

```yaml
services:
  <container-name>:
    security_opt:
      - seccomp=unconfined
```

This does not grant `privileged` or `SYS_ADMIN`; it only relaxes the runtime syscall filter enough for user-namespace creation. Keep Codex in `workspace-write` with approvals.

`aibox doctor` checks the Codex sandbox posture when Codex is selected. It
verifies that `bwrap`/`bubblewrap` is available, runs a user-namespace smoke
probe that matches Codex's sandbox requirement, warns if the generated service is missing Compose `init: true`, and
warns if the main aibox service uses broad grants such as `privileged: true` or
`SYS_ADMIN`.

See OpenAI's [Codex sandbox prerequisites](https://developers.openai.com/codex/concepts/sandboxing#prerequisites) for the upstream requirements.
