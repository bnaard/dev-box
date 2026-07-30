# processkit-agent-card MCP server

Tools:

- `project_agent_card(artifact_id, actor_id?, output_path?, write?)`

The tool reads an Artifact with `spec.kind=agent-card`, renders a
canonical JSON card, optionally writes it to the requested project-local
path, and returns the card checksum.

