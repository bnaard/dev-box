# processkit-supply-chain-audit MCP server

Tools:

- `discover_manifests(project_root?, manifest_globs?, include_vendor_dir?,
  include_ci_manifests?)` -> manifest discovery payload.
- `run_supply_chain_audit(project_root?, manifest_paths?, run_security_checks?,
  run_quality_checks?, network_enabled?, write_output?, output_path?)` ->
  structured findings and summary.
- `export_supply_chain_sbom(project_root?, manifest_paths?, format?,
  write_output?, output_path?)` -> SBOM payload and persistence details.

Running:

```bash
uv run context/skills/processkit/supply-chain-audit/mcp/server.py
```

Notes:

- Network-dependent security and quality probes are opt-in and off by default.
- Keep scanner-quality checks explicit unless network checks are needed.

