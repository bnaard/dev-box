---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260511_1032-WiseLark-filter-aibox-log-status-by-runtime
  created: '2026-05-11T10:32:15+00:00'
spec:
  title: Filter aibox log status by runtime session id
  state: accepted
  decision: aibox will create a unique runtime session id for each container start,
    stamp runtime aibox log entries with that id when available, and make the tmux/aibox-status
    log counter prefer filtering by the current runtime_session_id with a container-start-time
    fallback for legacy entries.
  context: The tmux aibox_log segment currently counts entries from .aibox/aibox.log
    that may come from earlier container builds or runtime sessions. A fixed time
    window is not precise enough for a per-runtime health indicator.
  rationale: A runtime_session_id is explicit, self-describing, and avoids clock-window
    ambiguity. It lets the status counter and future log viewers filter the same way
    while keeping a fallback for old logs and older images.
  consequences: Container startup must write a small runtime-session metadata file.
    The aibox log schema gains optional runtime_session_id and runtime_started_at
    fields. Status counting must handle both new session-stamped entries and legacy
    unstamped entries safely.
  decided_at: '2026-05-11T10:32:15+00:00'
---
