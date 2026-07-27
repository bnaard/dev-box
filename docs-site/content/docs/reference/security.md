# Security Reference

This page documents aibox's security model and trust boundaries.

## Support and vulnerability reporting

aibox is actively maintained. The latest minor release line is supported;
security and correctness fixes are released on the newest version rather than
backported across older minor lines. See the repository
[`SECURITY.md`](https://github.com/projectious-work/aibox/blob/main/SECURITY.md)
for private reporting instructions and response expectations.

## Dependency and asset provenance review

The project uses several independent integrity layers:

| Surface | Provenance control | Release validation |
| --- | --- | --- |
| Rust CLI | `cli/Cargo.lock` pins the complete dependency graph | `cargo test`, Clippy with warnings denied, and `cargo audit` |
| Documentation | `docs-site/package-lock.json` pins npm dependencies | clean `npm ci`, production Docusaurus build, and npm audit |
| processkit | source, version, and release-asset SHA-256 are recorded in `aibox.lock` and live provenance | installer hash verification, three-way content comparison, and pk-doctor integrity checks |
| Base images | release-specific image tags, OCI source/profile labels, and generated version markers | cross-platform build, GHCR publication verification, and downstream runtime smoke |
| Addon downloads | pinned versions plus SHA-256, signed checksum, or upstream sidecar verification where available | release-state inventory and real container lifecycle tests |
| Release binaries | locally cross-compiled artifacts attached to a signed release tag | version smoke tests and evidence-bound artifact checksums |

The release-state report records floating inputs and available updates before
each release. A clean security audit is mandatory; routine non-security drift
may be deferred only into a processkit WorkItem. Provenance exceptions are
documented next to the relevant installer instead of being silently accepted.

This review was consolidated for
[aibox issue #80](https://github.com/projectious-work/aibox/issues/80). It is
kept current through the local release gate and the public maintenance guide.

## Data handling review

aibox does not provide a hosted service and does not send product telemetry.
The CLI operates on the local project, generates container configuration, and
contacts external services only for requested dependency, image, processkit,
GitHub, documentation, or release operations.

- `aibox.toml`, generated `.devcontainer/` files, and processkit context are
  project data and are normally committed.
- `.aibox-local.toml`, `.aibox-home/`, `.aibox/`, authentication state, SSH
  material, local caches, diagnostics, and release evidence are local state and
  must remain ignored unless a specific artifact has been reviewed for
  publication.
- Tokens enter containers through explicit local environment configuration.
  Prefer separate least-privilege tokens and select cross-account tokens per
  command instead of exposing a human account's full authorization.
- Enabled AI harnesses and MCP servers execute with the container user's access
  to the workspace and mounted credentials. Their providers may receive prompt,
  tool, and file content according to the provider's own service terms.
- `aibox doctor` and pk-doctor inspect local state. Diagnostic reports must be
  reviewed and redacted before they are attached to public issues.

No generated local credential, cache, or diagnostic directory belongs in a
release artifact. The release process builds from tracked source, verifies the
exact commit, and publishes only the declared binaries, documentation output,
and container images.

## MCP Gateway Trust Scope

### How processkit skills are registered

When `[context].mode = "processkit"` and `aibox apply` registers processkit
skills (such as `processkit-gateway`, `workitem-management`, etc.), it calls
into `cli/src/mcp_registration.rs`. For Codex, the generated project config
sets:

```toml
[project]
trust_level = "trusted"
```

This means every installed skill's `mcp/server.py` runs with **project-user
trust** inside the aibox container — the same trust level as the project owner
who launched the container. The MCP server process inherits the container
filesystem and environment, including any mounted credentials.

When `[context].mode = "harness-only"`, aibox does not install or register
processkit skills, processkit hooks, processkit preauth rules, or the
processkit MCP gateway. Team and personal MCP servers configured under
`[ai.mcp]` / `.aibox-local.toml [[mcp.servers]]` are still generated for
enabled harnesses and carry their own trust review burden.

### Implications

- A skill's `mcp/server.py` can read, write, and execute within the container
  with the same permissions as the project user.
- Skills can access mounted SSH keys (`~/.ssh`), API key env vars, and
  the full workspace at `/workspace`.
- Skills are registered at the Codex/Claude `project` scope — they
  are active for every session in the container.

### Third-party skill review checklist

Before installing a skill from a third party (outside the processkit core), verify:

1. **Source code is auditable**: the skill's `mcp/server.py` (and any
   imported modules) are readable and understandable.
2. **No unexpected outbound network calls**: the skill should not exfiltrate
   data to external endpoints.
3. **No credential access beyond stated purpose**: check for reads of
   `~/.ssh/`, env vars (`ANTHROPIC_API_KEY`, etc.), or
   `~/.claude`/`~/.codex`.
4. **Tool list is minimal**: the `allowed_tools` set registered by the skill
   should match only the capabilities the skill claims to need.
5. **Dependency supply chain**: if the skill uses `uv` or `pip` to install
   Python packages, review `pyproject.toml` / `requirements.txt` for
   unexpected dependencies.
6. **Immutable or pinned source**: prefer skills pinned to a specific git SHA
   or release tag over floating `main`/`latest` references.

### Opting out of a skill

To remove a skill and deregister its MCP server:

1. Remove the skill from `context/skills/` or update `[processkit]` config.
2. Run `aibox apply` — this rewrites the harness MCP configuration files
   and removes the skill's `allowed_tools` entries.
3. Recreate the container (`docker compose up -d --force-recreate`) so the
   new MCP configuration takes effect.

---

## seccomp=unconfined Consent Gate

The Codex CLI uses bubblewrap for Linux sandboxing. Some container runtime
seccomp profiles block unprivileged user namespace creation before bubblewrap
can set up its own sandbox. To work around this, `aibox apply` can emit
`seccomp=unconfined` in the generated `docker-compose.yml`.

**Explicit consent is required.** Without the acknowledgement flag, `aibox apply`
will error with a remediation pointer. To opt in, add to `aibox.toml`:

```toml
[security]
acknowledge_seccomp_unconfined = true
```

This setting:
- Allows `aibox apply` to emit `seccomp=unconfined` in `docker-compose.yml`.
- Suppresses the `aibox doctor` warning about unapproved seccomp relaxation.
- Documents in source control that the project owner has accepted the
  trade-off: reduced seccomp filtering in exchange for Codex bubblewrap
  user-namespace sandboxing, avoiding the broader `privileged=true` or
  `CAP_SYS_ADMIN` escalations.

`seccomp=unconfined` does not grant root or additional Linux capabilities —
it only lifts the seccomp syscall filter, allowing bubblewrap to create
user namespaces.
