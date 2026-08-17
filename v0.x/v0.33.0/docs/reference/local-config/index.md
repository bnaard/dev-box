# Local Config (.aibox-local.toml)

LLMS index: [llms.txt](/aibox/v0.x/v0.33.0/llms.txt)

---

# Local Config (.aibox-local.toml)

`.aibox-local.toml` is a personal, gitignored overlay that sits next to `aibox.toml` in the project root. It exists for secrets and per-developer settings that must never be committed to version control — API tokens, personal credential paths, and similar values that differ between contributors.

## Why it exists

`aibox.toml` is committed and shared across the team. That's the right place
for project-wide settings: container name, context mode, processkit version
when processkit is enabled, addons, shared environment variables, and so on.
But tokens and personal bind mounts don't belong there. `.aibox-local.toml`
gives every developer a private escape valve without requiring `.gitignore`
discipline on every secret.

## Location and gitignore

`.aibox-local.toml` lives in the **project root**, next to `aibox.toml`:

```
my-project/
├── aibox.toml               ← committed, shared
├── .aibox-local.toml        ← gitignored, personal
├── .devcontainer/
└── context/
```

`aibox init` and `aibox apply` automatically add `.aibox-local.toml` to `.gitignore`. You do not need to do this manually.

## Supported sections

Three sections are supported. Everything else must remain in `aibox.toml`.

### [container.environment]

Inject environment variables into the container. These are merged **on top of** any `[container.environment]` values in `aibox.toml`. If the same key appears in both files, the local value wins.

```toml
[container.environment]
GH_TOKEN            = "github_pat_xxxxxxxxxxxx"
ANTHROPIC_API_KEY   = "sk-ant-api03-..."
OPENAI_API_KEY      = "sk-proj-..."
AWS_PROFILE         = "my-dev-profile"
```

`aibox apply` writes these values to the gitignored `.aibox-local.env`, which
Docker Compose loads into the container. The values therefore survive
container replacement and image rebuilds. They are still normal container
environment variables: processes running as the container user, including an
AI agent, can read them.

### [[container.extra_volumes]]

Personal bind mounts appended **after** any volumes declared in `aibox.toml`. Each entry requires `source` (host path) and `target` (container path). `read_only` defaults to `false`.

```toml
[[container.extra_volumes]]
source = "~/.aws"
target = "/home/aibox/.aws"
read_only = true

[[container.extra_volumes]]
source = "~/.ssh/id_ed25519"
target = "/home/aibox/.ssh/id_ed25519"
read_only = true
```

### [mcp]

Personal MCP servers appended to the generated MCP client configs on `aibox apply`. Use this section for servers you want only on your machine — internal tools, local scripts, or servers that require credentials you don't want to share.

Each server entry is an `[[mcp.servers]]` table with the same fields as committed `[[ai.mcp.servers]]` in `aibox.toml`:

```toml
[[mcp.servers]]
name    = "my-internal-tool"
command = "npx"
args    = ["-y", "@acme/internal-mcp-server"]

[[mcp.servers]]
name    = "local-notes"
command = "/home/user/bin/notes-mcp"
args    = ["--db", "~/notes.db"]

[[mcp.servers]]
name    = "stripe"
command = "npx"
args    = ["-y", "@stripe/mcp"]
[mcp.servers.env]
STRIPE_SECRET_KEY = "sk_test_..."
```

`aibox apply` merges personal servers with team servers (from
`aibox.toml [ai.mcp]`) and, in processkit mode, built-in processkit servers,
then regenerates all MCP client config files. The generated files are
**gitignored** — they are never committed to version control, so personal keys
and server definitions stay private.

## Merge behavior

| Section | Merge rule |
|---------|-----------|
| `[container.environment]` | Merged with `aibox.toml`; local values win on key conflicts |
| `[[container.extra_volumes]]` | Appended after `aibox.toml` volumes; no deduplication |
| `[[mcp.servers]]` | Appended after `aibox.toml` MCP servers; all sources merged into each generated config file |

## GitHub authentication

Choose the authentication model according to how much GitHub access the
container and its AI agents should receive. A narrowly scoped personal access
token (PAT) is the recommended default. An interactive GitHub CLI login is more
convenient, but may grant the container substantially broader access.

### Recommended: least-privilege PATs

Put the token used for normal GitHub CLI commands in `GH_TOKEN`. GitHub CLI
reads it automatically:

```toml
[container.environment]
GH_TOKEN = "github_pat_default_project_token"
```

Grant this token only the repositories and permissions the project normally
needs. When one workflow needs access to another repository or organization,
add a second, purpose-specific variable instead of broadening the default
token. For example, a derived project can receive permission to report issues
to an upstream project without receiving wider upstream access:

```toml
[container.environment]
GH_TOKEN = "github_pat_default_project_token"
PROJECTXXX_ISSUES_TOKEN = "github_pat_upstream_issues_token"
```

Select the second credential only for the command that needs it:

```bash
GH_TOKEN="$PROJECTXXX_ISSUES_TOKEN" \
  gh issue create --repo projectious-work/aibox
```

The temporary assignment overrides `GH_TOKEN` for that invocation only. The
default token remains active for subsequent commands. Give the additional PAT
only the target repository's `Issues: read and write` permission plus the
metadata access GitHub requires.

This arrangement makes the authorization boundary visible in both the local
configuration and the command. It also lets a human decide exactly which
rights are available to an AI agent in the container.

For a fine-grained PAT that targets an organization repository, select that
organization as the token's resource owner and include the target repository.
Organization policy may require an administrator to approve the token. The
fact that the user can create an issue in a public repository through the
GitHub website does not automatically authorize a repository-scoped PAT to do
the same through the API.

### Alternative: persistent GitHub CLI login

For a trusted personal workspace where broad account access is acceptable, log
in from inside the running container:

```bash
gh auth login --hostname github.com --web --git-protocol https --insecure-storage
```

`--insecure-storage` tells GitHub CLI to store its OAuth token in its config
file instead of a system keyring. In an aibox container that file is under
`/home/aibox/.config/gh/`, backed by the project's gitignored
`.aibox-home/.config/gh/` directory. It survives container restarts,
replacements, and image rebuilds. The token is a GitHub bearer credential; it
is not tied to a particular container ID or image.

<div class="alert alert-warning" role="alert"><div class="h4 alert-heading" role="heading">Remove token environment variables first</div>



`GH_TOKEN` and `GITHUB_TOKEN` take precedence over credentials saved by
`gh auth login`. To use the stored login, remove both variables from
`.aibox-local.toml` and from any other container environment configuration,
then run `aibox apply` to regenerate `.aibox-local.env` and recreate or restart
the container as needed.

Check the effective authentication inside the container with:

```bash
env | grep -E '^(GH_TOKEN|GITHUB_TOKEN)='
gh auth status
gh api user --jq .login
```

The first command should produce no output.

</div>


The stored OAuth token is plaintext in `.aibox-home/.config/gh/hosts.yml`.
Gitignore prevents accidental normal commits, but it does not encrypt the
credential or protect it from the host user, container processes, AI agents,
backups, malware, or an explicit `git add --force`. Treat `.aibox-home/` as
secret-bearing local state. Prefer scoped PATs when the container should not
inherit the human user's broader GitHub authority.

## Full example

A typical `.aibox-local.toml` for a developer working with Claude, GitHub, and AWS, plus a personal MCP server:

```toml
[container.environment]
ANTHROPIC_API_KEY = "sk-ant-api03-..."
GH_TOKEN          = "github_pat_xxxxxxxxxxxx"
AWS_PROFILE       = "my-dev-profile"
AWS_REGION        = "eu-west-1"

[[container.extra_volumes]]
source = "~/.aws"
target = "/home/aibox/.aws"
read_only = true

[[container.extra_volumes]]
source = "~/.ssh/id_ed25519"
target = "/home/aibox/.ssh/id_ed25519"
read_only = true

[[mcp.servers]]
name    = "my-internal-tool"
command = "npx"
args    = ["-y", "@acme/internal-mcp-server"]
```

## What is NOT supported

Everything outside of `[container.environment]`, `[[container.extra_volumes]]`, and `[[mcp.servers]]` is ignored. The following must remain in `aibox.toml`:

- Container name, hostname, user, lifecycle, image, and generated paths
- `[context]` — context mode and processkit package selection
- `[addons]` — addon configuration
- `[processkit]` — content source and version pin when processkit mode is enabled
- `[skills]` — enabled/disabled lists when processkit mode is enabled
- `[ai]` — harnesses, agents, and team MCP servers
- `[customization]` — theme, mode, prompt, layout
- `[audio]` — audio bridging

<div class="alert alert-success" role="alert"><div class="h4 alert-heading" role="heading">Applying changes</div>


After editing `.aibox-local.toml`, run `aibox apply` (or `aibox apply --no-build` for a config-only refresh) to regenerate `.devcontainer/` files with the updated environment and volumes, and MCP client config files with the updated server list.
</div>
