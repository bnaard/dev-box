---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260511_1019-SmartButter-override-powerkit-uptime-with-container-uptime
  created: '2026-05-11T10:19:41+00:00'
spec:
  title: Override PowerKit uptime with container uptime
  state: accepted
  decision: aibox will ship an image-owned PowerKit uptime plugin override that reports
    container uptime from PID 1 rather than kernel uptime from /proc/uptime.
  context: The upstream tmux-powerkit uptime plugin reads /proc/uptime, which reports
    host or VM kernel uptime inside containers. In aibox tmux status this is misleading
    because users expect container uptime.
  rationale: A maintained override in images/base-debian/config/tmux/powerkit-plugins
    follows the existing aibox PowerKit plugin pattern, keeps the status configuration
    unchanged, and makes the uptime segment semantically correct for container runtimes.
  consequences: The runtime image will copy aibox's uptime.sh over the upstream tmux-powerkit
    plugin. Future upstream changes to uptime.sh must be reviewed manually if needed,
    but the override is explicit and source-owned.
  decided_at: '2026-05-11T10:19:41+00:00'
---
