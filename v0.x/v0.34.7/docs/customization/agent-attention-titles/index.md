# Agent attention titles


When an AI harness needs a human response, aibox can mark the tmux window in
the terminal tab title. This is useful when the agent is running in a
background pane or another tmux window: the title remains visible even when
that pane is not selected.

## Configuration

Titles are configured in `aibox.toml`:

```toml
[customization.tmux.title]
enabled = true
format = "{state_symbol}{project}:{window} — {directory}"
max-length = 60
directory-style = "basename"
done-ttl-seconds = 10
message-max-length = 32

[customization.tmux.title.states]
working = "● "
question = "❓ "
done = "✓ "
error = "! "
idle = ""

[customization.tmux.notifications]
enabled = false
protocol = "osc-9" # or "bell"
states = ["question", "error"]
include-message = true
```

`enabled = false` disables generated title settings. The default title is
short and project-oriented; use a custom `format` when session or harness
identity is more useful than the directory:

```toml
format = "{state_symbol}{harness} — {project}:{window}"
```

Supported placeholders are:

| Placeholder | Meaning |
|---|---|
| `{state_symbol}` | Configured symbol for the aggregate window state |
| `{state}` | State name: `idle`, `working`, `question`, `done`, or `error` |
| `{project}` | aibox project name |
| `{session}` | tmux session name |
| `{window}` / `{window_index}` | tmux window name or index |
| `{pane}` | active pane index |
| `{directory}` | Current directory in the selected directory style |
| `{directory_path}` | Full current-directory path |
| `{repository}` / `{branch}` | Git repository and branch, when available |
| `{harness}` / `{agent}` | Harness and agent identity, when supplied |
| `{agent_suffix}` | Conditional ` — agent@harness`, ` — harness`, or empty suffix |
| `{task}` / `{message}` | Short task or question/error text, sanitized and bounded |
| `{elapsed}` | Elapsed time captured at the most recent state aggregation |

Use `{agent_suffix}` when the same format applies to both agent and non-agent
windows. It includes its own separator and omits the entire suffix when no
harness is active, avoiding dangling punctuation:

```toml
format = "{state_symbol}{repository}{agent_suffix}"
```

`repository-style` controls the value of `{repository}`:

```toml
[customization.tmux.title]
repository-style = "basename" # aibox
# repository-style = "full"   # projectious-work/aibox
```

The full form is derived from the configured Git remote path, not from a
forge API. It therefore works with HTTPS and SSH remotes on GitHub, GitLab,
Gitea, Forgejo, and compatible self-hosted instances. Nested namespaces are
preserved (for example, `group/platform/repository`). If no usable remote is
configured, both styles fall back to the repository root directory name.

`agent-style` controls the value of `{agent}`:

```toml
[customization.tmux.title]
agent-style = "basename" # gpt-5.6-sol
# agent-style = "full"   # gpt-5.6-sol low
```

The full form appends the active reasoning-effort level when the harness
exposes one reliably. Codex resolves both values from current local thread
metadata, Claude reads its hook payload and transcript, Gemini reads the
documented `BeforeModel` request, and OpenCode reads model-bearing plugin
events. Copilot and Cursor consume model metadata when their hook payload or
transcript provides it. Aider, Continue, Hermes, and Tau use their launch-time
configuration as a guarded fallback; an explicit runtime signal supersedes it
after an in-session model switch.

Every harness can also supply an exact runtime identity with `--agent MODEL
--effort LEVEL` or the `AIBOX_AGENT_NAME` and
`AIBOX_AGENT_REASONING_EFFORT` environment variables.
`{harness}` remains the CLI harness reporting the lifecycle event. Explicit
arguments and environment variables take precedence over automatic detection.

State is aggregated across all panes in the window. The precedence is
`error > question > working > done > idle`, so a question in a background
pane cannot be hidden by an idle active pane. Completion markers are temporary
and use `done-ttl-seconds`.

## Harness support and fallback

Harness integration is capability-based:

| Tier | Behavior |
|---|---|
| Native | Lifecycle hooks report working, question, completion, and error states. |
| Partial | Reliable lifecycle hooks report only the states the harness exposes. |
| Wrapper | aibox reports process start/exit; question detection is unavailable. |
| Manual | Use the explicit `aibox-agent-signal` helper from a key binding or wrapper. |

Current generated integrations:

| Harness | Tier | Native signals |
|---|---|---|
| Claude Code | Native | prompt, permission/elicitation, stop, failure |
| Codex CLI | Native | prompt, permission, tool resumption/completion, stop, session end |
| OpenCode | Native | session status, permission, explicit question, error, idle |
| Gemini CLI | Native | before/after agent, tool permission, session end |
| GitHub Copilot CLI | Native | prompt, permission/elicitation, stop, error, session end |
| Cursor | Partial | prompt and stop |

Gemini and Codex classify an after-turn response ending in a question mark as
`question`. OpenCode uses its explicit `question.asked`, `question.replied`,
and `question.rejected` events. Copilot reports permission and elicitation
dialogs as questions; its stop payload does not expose final response text.
Codex permission replies resume the current turn without submitting a new
prompt, so aibox also maps Codex pre- and post-tool lifecycle events to
`working`; this clears the question marker when approved work continues.

aibox does not infer a question from process idleness. For a harness without a
question hook, signal it explicitly, for example:

```sh
aibox-agent-signal question --harness my-harness --message 'Choose an option'
aibox-agent-signal working --harness my-harness
aibox-agent-signal done --harness my-harness
```

Signals are scoped to the current tmux pane and are safe to repeat. Outside
tmux they are a no-op, so the same wrapper can be used in and out of aibox
workspaces without affecting the terminal.

## Terminal and shell title ownership

Terminal emulators normally display the title emitted by tmux through the
standard title control sequence. Do not configure a competing fixed tab title
for an aibox tab. Inside tmux, tmux should be the sole title writer; otherwise
a shell `precmd` hook can overwrite the attention marker. This title path does
not depend on Ghostty or any other specific terminal emulator.

If you currently set the title from zsh, keep that behavior outside tmux only:

```zsh
if [[ -z "$TMUX" ]]; then
  precmd() {
    print -Pn '\e]2;%~\a'
  }
fi
```

Terminal notifications are optional and transition-based. Configure them with
`[customization.tmux.notifications]`:

| Field | Meaning |
|---|---|
| `enabled` | Enable terminal attention notifications (default `false`) |
| `protocol` | `osc-9` for a message-bearing desktop notification on supporting terminals, or `bell` for the portable terminal attention signal |
| `states` | State transitions that trigger a notification; supported values are `working`, `question`, `done`, `error`, and `idle` |
| `include-message` | Include the sanitized question/error text when available (default `true`) |

Notifications are emitted only from inside tmux and only on aggregate state
transitions. OSC 9 is supported by terminals including Ghostty and iTerm2 but
is not universal; choose `bell` when portability matters and configure the
terminal's bell attention behavior as desired. Title rendering remains
independent and non-fatal if notifications are unavailable.

## Security and limits

Agent-provided messages are stripped of terminal control characters and
truncated to `message-max-length`, then the complete title is bounded by
`max-length`. This prevents task text or a question from injecting terminal
escape sequences.


---
Source: https://projectious-work.github.io/aibox/v0.x/v0.34.7/docs/customization/agent-attention-titles/index.md
