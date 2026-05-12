---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260511_1109-TidyPine-render-compact-mcp-topology-states-in
  created: '2026-05-11T11:09:45+00:00'
spec:
  title: Render compact MCP topology states in tmux status
  state: accepted
  decision: 'The aibox tmux MCP status segment will render active ProcessKit topologies as compact mode/server/process triples: gateway as gw/1/N and separate granular MCP as sep/1/N. In inactive or unsafe scan states it will render only the state label: none, unkwn, or degraded, without counts.'
  context: The current status line shows values like MCP gateway/5, where the mode string is long and the count is a supporting process count rather than a topology count. The user requested explicit compact renderings for gateway, separate, none, unknown, and degraded states.
  rationale: Compact labels preserve limited tmux status width while making the first count represent the detected topology instance and the second count represent supporting MCP processes. Omitting counts for none, unknown, and degraded avoids misleading zeroes.
  consequences: The status JSON can keep exposing processkit_mode and processkit_mcp for compatibility, but status rendering should use a compact display value that suppresses counts for none, unknown, and degraded states.
  decided_at: '2026-05-11T11:09:45+00:00'
---
