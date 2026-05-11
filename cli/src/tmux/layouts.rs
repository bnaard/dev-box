/// Tmux layout script generation.
///
/// Owns: `tmux_layout_script` and `tmux_session_script`.  Each function
/// renders a shell script that is seeded into `.config/tmux/layouts/` or
/// `.config/tmux/aibox-session.sh` at apply-time.
use crate::config::{AiboxConfig, ConfigLayout};

fn new_window(name: &str, tool: &str) -> String {
    format!(
        r#"tmux -S "$socket" new-window -t "$session:" -n "{name}" -c "$workspace" "$(tool_or_shell {tool})"
"#
    )
}

fn lazygit_window(include_lazygit: bool) -> String {
    if include_lazygit {
        new_window("lazygit", "lazygit")
    } else {
        String::new()
    }
}

fn shell_window() -> String {
    r#"tmux -S "$socket" new-window -t "$session:" -n "shell" -c "$workspace" "bash"
"#
    .to_string()
}

fn harness_ai_window(harnesses: &[&str]) -> String {
    let Some((first, rest)) = harnesses.split_first() else {
        return String::new();
    };

    let mut out = format!(
        r#"tmux -S "$socket" new-window -t "$session:" -n "ai" -c "$workspace" "$(tool_or_shell {first})"
ai_pane="$(tmux -S "$socket" display-message -p -t "$session:ai" '#{{pane_id}}')"
"#
    );
    for harness in rest {
        out.push_str(&format!(
            r#"tmux -S "$socket" split-window -t "$session:ai" -h -c "$workspace" "$(tool_or_shell {harness})"
"#
        ));
    }
    if !rest.is_empty() {
        out.push_str(
            r#"tmux -S "$socket" select-layout -t "$session:ai" even-horizontal
"#,
        );
    }
    out
}

fn focus_harness_windows(harnesses: &[&str]) -> String {
    let mut out = String::new();
    for harness in harnesses {
        out.push_str(&new_window(harness, harness));
    }
    out
}

/// Render the layout-specific `<name>.sh` script that opens a fresh tmux
/// session with the requested pane/window arrangement.
pub fn tmux_layout_script(
    layout: &ConfigLayout,
    providers: &[crate::config::AiProvider],
    include_lazygit: bool,
    _tool_windows: &[(&str, &str)],
    session_name: &str,
) -> String {
    let active_harnesses: Vec<&str> = providers
        .iter()
        .filter(|p| p.is_active())
        .map(|p| p.binary_name())
        .collect();
    let first_harness = active_harnesses.first().copied().unwrap_or("bash");
    let further_harnesses = active_harnesses.get(1..).unwrap_or(&[]);

    let layout_body = match layout {
        ConfigLayout::Ai => format!(
            r#"tmux -S "$socket" -f "$config" new-session -d -s "$session" -n "work" -c "$workspace" "$(tool_or_shell yazi)"
files_pane="$(tmux -S "$socket" display-message -p -t "$session:work" '#{{pane_id}}')"
tmux -S "$socket" split-window -t "$session:work" -h -p 50 -c "$workspace" "$(tool_or_shell {first_harness})"
tmux -S "$socket" select-pane -t "$files_pane"
{ai_window}{lazygit_window}{shell_window}tmux -S "$socket" select-window -t "$session:work"
"#,
            ai_window = harness_ai_window(further_harnesses),
            lazygit_window = lazygit_window(include_lazygit),
            shell_window = shell_window(),
        ),
        ConfigLayout::Dev => format!(
            r#"tmux -S "$socket" -f "$config" new-session -d -s "$session" -n "work" -c "$workspace" "$(tool_or_shell yazi)"
files_pane="$(tmux -S "$socket" display-message -p -t "$session:work" '#{{pane_id}}')"
tmux -S "$socket" split-window -t "$session:work" -h -p 50 -c "$workspace" "bash"
tmux -S "$socket" split-window -t "$files_pane" -v -p 50 -c "$workspace" "$(tool_or_shell {first_harness})"
tmux -S "$socket" select-pane -t "$files_pane"
{lazygit_window}{ai_window}{shell_window}tmux -S "$socket" select-window -t "$session:work"
"#,
            lazygit_window = lazygit_window(include_lazygit),
            ai_window = harness_ai_window(further_harnesses),
            shell_window = shell_window(),
        ),
        ConfigLayout::Focus => format!(
            r#"tmux -S "$socket" -f "$config" new-session -d -s "$session" -n "files" -c "$workspace" "$(tool_or_shell yazi)"
files_pane="$(tmux -S "$socket" display-message -p -t "$session:files" '#{{pane_id}}')"
{harness_windows}{lazygit_window}{shell_window}tmux -S "$socket" select-window -t "$session:files"
"#,
            harness_windows = focus_harness_windows(&active_harnesses),
            lazygit_window = lazygit_window(include_lazygit),
            shell_window = shell_window(),
        ),
        ConfigLayout::Cowork => format!(
            r#"tmux -S "$socket" -f "$config" new-session -d -s "$session" -n "work" -c "$workspace" "$(tool_or_shell yazi)"
files_pane="$(tmux -S "$socket" display-message -p -t "$session:work" '#{{pane_id}}')"
tmux -S "$socket" split-window -t "$session:work" -h -p 50 -c "$workspace" "bash"
tmux -S "$socket" select-pane -t "$files_pane"
{ai_window}{lazygit_window}tmux -S "$socket" select-window -t "$session:work"
"#,
            ai_window = harness_ai_window(&active_harnesses),
            lazygit_window = lazygit_window(include_lazygit),
        ),
    };

    format!(
        r#"#!/usr/bin/env bash
set -euo pipefail

session="${{AIBOX_TMUX_SESSION:-{session_name}}}"
workspace="${{AIBOX_WORKSPACE:-/workspace}}"
config="${{AIBOX_TMUX_CONFIG:-$HOME/.config/tmux/tmux.conf}}"
socket="${{AIBOX_TMUX_SOCKET:-$HOME/.tmux/aibox.sock}}"
mkdir -p "$(dirname "$socket")"

if tmux -S "$socket" -f "$config" has-session -t "$session" 2>/dev/null; then
  exec tmux -S "$socket" -f "$config" attach-session -t "$session"
fi

tool_or_shell() {{
  local tool="$1"
  if [[ "$tool" == "yazi" ]]; then
    printf "bash -lc 'for _ in {{1..50}}; do tmux -S %q list-clients -t %q >/dev/null 2>&1 && break; sleep 0.1; done; if command -v yazi >/dev/null 2>&1; then exec yazi; fi; exec bash'" "$socket" "$session"
    return
  fi
  printf "bash -lc 'if command -v %q >/dev/null 2>&1; then %q; fi; exec bash'" "$tool" "$tool"
}}

{layout_body}exec tmux -S "$socket" -f "$config" attach-session -t "$session"
"#
    )
}

/// Render the `aibox-session.sh` dispatcher script that reads the configured
/// layout name and execs the corresponding `layouts/<name>.sh`.
///
/// User-defined names are resolved from `~/.config/tmux/layouts/<name>.sh`.
/// The managed layouts are `ai`, `dev`, `focus`, and `cowork`.
pub fn tmux_session_script(config: &AiboxConfig) -> String {
    let session = config.tmux_session_name();
    format!(
        r#"#!/usr/bin/env bash
set -euo pipefail

layout="${{1:-${{AIBOX_TMUX_LAYOUT:-{layout}}}}}"
session="${{2:-${{AIBOX_TMUX_SESSION:-{session}}}}}"
socket="${{AIBOX_TMUX_SOCKET:-$HOME/.tmux/aibox.sock}}"

# Resolve the layout script. Search order:
#   1. User drop-in: ~/.config/tmux/layouts/<layout>.sh
#   2. System default: /usr/local/share/aibox/tmux/layouts/<layout>.sh
user_script="${{HOME}}/.config/tmux/layouts/${{layout}}.sh"
system_script="/usr/local/share/aibox/tmux/layouts/${{layout}}.sh"
if [[ -x "${{user_script}}" ]]; then
  script="${{user_script}}"
elif [[ -x "${{system_script}}" ]]; then
  script="${{system_script}}"
else
  echo "aibox-tmux-session: unknown or unavailable managed layout: ${{layout}}" >&2
  echo "  searched: ${{user_script}}" >&2
  echo "  searched: ${{system_script}}" >&2
  exit 2
fi

exec env AIBOX_TMUX_SESSION="${{session}}" AIBOX_TMUX_SOCKET="${{socket}}" "${{script}}"
"#,
        layout = config.customization.tmux_layout(),
        session = session,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AiProvider;

    fn no_tools() -> Vec<(&'static str, &'static str)> {
        vec![]
    }

    #[test]
    fn ai_layout_uses_work_ai_lazygit_shell_windows() {
        let providers = vec![AiProvider::Codex, AiProvider::Claude, AiProvider::Gemini];
        let body = tmux_layout_script(&ConfigLayout::Ai, &providers, true, &no_tools(), "aibox");
        assert!(body.contains(r#"-n "work""#));
        assert!(body.contains(r#"-n "ai""#));
        assert!(body.contains(r#"-n "lazygit""#));
        assert!(body.contains(r#"-n "shell""#));
        assert!(body.contains(r#"split-window -t "$session:work" -h -p 50"#));
        assert!(body.contains("tool_or_shell codex"));
        assert!(body.contains("tool_or_shell claude"));
        assert!(body.contains("tool_or_shell gemini"));
        assert!(body.contains(r#"select-layout -t "$session:ai" even-horizontal"#));
    }

    #[test]
    fn dev_layout_uses_work_lazygit_ai_shell_windows() {
        let providers = vec![AiProvider::Codex, AiProvider::Claude];
        let body = tmux_layout_script(&ConfigLayout::Dev, &providers, true, &no_tools(), "aibox");
        assert!(body.contains(r#"-n "work""#));
        assert!(body.contains(r#"-n "lazygit""#));
        assert!(body.contains(r#"-n "ai""#));
        assert!(body.contains(r#"-n "shell""#));
        assert!(body.contains(r#"split-window -t "$session:work" -h -p 50"#));
        assert!(body.contains(r#"split-window -t "$files_pane" -v -p 50"#));
        assert!(body.contains("tool_or_shell codex"));
        assert!(body.contains("tool_or_shell claude"));
    }

    #[test]
    fn focus_layout_creates_files_then_one_window_per_harness() {
        let providers = vec![AiProvider::Codex, AiProvider::Claude];
        let body = tmux_layout_script(&ConfigLayout::Focus, &providers, true, &no_tools(), "aibox");
        assert!(body.contains(r#"-n "files""#));
        assert!(body.contains(r#"-n "codex""#));
        assert!(body.contains(r#"-n "claude""#));
        assert!(body.contains(r#"-n "lazygit""#));
        assert!(body.contains(r#"-n "shell""#));
        assert!(!body.contains("split-window -t \"$session:files\""));
    }

    #[test]
    fn cowork_layout_uses_work_ai_lazygit_without_extra_shell_window() {
        let providers = vec![AiProvider::Codex, AiProvider::Claude];
        let body = tmux_layout_script(
            &ConfigLayout::Cowork,
            &providers,
            true,
            &no_tools(),
            "aibox",
        );
        assert!(body.contains(r#"-n "work""#));
        assert!(body.contains(r#"-n "ai""#));
        assert!(body.contains(r#"-n "lazygit""#));
        assert!(!body.contains(r#"-n "shell""#));
        assert!(body.contains(r#"split-window -t "$session:work" -h -p 50"#));
        assert!(body.contains("tool_or_shell codex"));
        assert!(body.contains("tool_or_shell claude"));
    }

    #[test]
    fn session_helper_uses_configured_session_name_and_layout() {
        let config = crate::config::test_config();
        let script = tmux_session_script(&config);
        assert!(script.contains("AIBOX_TMUX_SESSION"));
        assert!(script.contains(".config/tmux/layouts/${layout}.sh"));
        assert!(script.contains("/usr/local/share/aibox/tmux/layouts/${layout}.sh"));
    }
}
