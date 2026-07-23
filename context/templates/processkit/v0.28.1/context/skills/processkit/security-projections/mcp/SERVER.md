# processkit-security-projections MCP server

Tools:

- `project_agent_ids_rule(artifact_id, output_path?, write?)`
- `project_tetragon_tracing_policy(artifact_id, output_path?, write?)`

Both tools read policy Artifacts, render canonical runtime files, and
return checksums. Output paths must remain under the project root.

