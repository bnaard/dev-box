/// Tmux layout script generation.
///
/// Owns: `tmux_layout_script` and `tmux_session_script`.  Each function
/// renders a shell script that is seeded into `.config/tmux/layouts/` or
/// `.config/tmux/aibox-session.sh` at apply-time.
use crate::config::{AiboxConfig, ConfigLayout};

/// Render the layout-specific `<name>.sh` script that opens a fresh tmux
/// session with the requested pane/window arrangement.
pub fn tmux_layout_script(
    layout: &ConfigLayout,
    providers: &[crate::config::AiProvider],
    include_lazygit: bool,
    session_name: &str,
) -> String {
    let provider = providers
        .iter()
        .find(|provider| provider.is_active())
        .map(|provider| provider.binary_name())
        .unwrap_or("bash");
    let git_window = if include_lazygit {
        r#"tmux -S "$socket" new-window -t "$session:" -n git -c "$workspace" "$(tool_or_shell lazygit)"
"#
    } else {
        ""
    };

    let primary_window = match layout {
        ConfigLayout::Dev => "dev",
        ConfigLayout::Focus => "focus",
        ConfigLayout::Cowork => "cowork",
        ConfigLayout::CoworkSwap => "cowork-swap",
        ConfigLayout::Browse => "browse",
        ConfigLayout::Ai => "ai",
    };

    // BR-VIM-HARDCUT (DEC-20260508_1604-LuckySeal, v0.25.6): the persistent
    // vim/editor pane has been removed from every layout. Yazi `e` opens
    // vim in a full-screen tmux popup that closes on `:q`; `Enter` runs
    // vim via the yazi `[opener.edit]` (suspends yazi until `:q`). Layout
    // bodies below intentionally do NOT create an `editor_pane` or an
    // `editor` window.
    //
    // BR-LAYOUT-KNOBS (BACK-20260509_1316-SnappyWolf, v0.25.7, "4c"): the
    // agent-pane split direction and split percentage are runtime-tunable
    // via env vars. Each layout ships sensible defaults and reads:
    //   AIBOX_LAYOUT_AGENT_SPLIT   — "h" or "v" (default per layout)
    //   AIBOX_LAYOUT_AGENT_RATIO   — 1..99 percent (default per layout)
    // Unset env vars fall back to the layout-specific defaults; invalid
    // values are silently ignored by tmux.
    let layout_body = match layout {
        ConfigLayout::Dev => format!(
            r#"tmux -S "$socket" -f "$config" new-session -d -s "$session" -n dev -c "$workspace" "$(tool_or_shell yazi)"
files_pane="$(tmux -S "$socket" display-message -p -t "$session:dev" '#{{pane_id}}')"
split_flag="-${{AIBOX_LAYOUT_AGENT_SPLIT:-h}}"
split_ratio="${{AIBOX_LAYOUT_AGENT_RATIO:-50}}"
agent_pane="$(tmux -S "$socket" split-window -t "$session:dev" "$split_flag" -p "$split_ratio" -P -F '#{{pane_id}}' -c "$workspace" "$(tool_or_shell {provider})")"
tmux -S "$socket" select-pane -t "$files_pane"
"#
        ),
        ConfigLayout::Focus => format!(
            r#"tmux -S "$socket" -f "$config" new-session -d -s "$session" -n focus -c "$workspace" "$(tool_or_shell {provider})"
"#
        ),
        ConfigLayout::Cowork => format!(
            r#"tmux -S "$socket" -f "$config" new-session -d -s "$session" -n cowork -c "$workspace" "$(tool_or_shell yazi)"
files_pane="$(tmux -S "$socket" display-message -p -t "$session:cowork" '#{{pane_id}}')"
split_flag="-${{AIBOX_LAYOUT_AGENT_SPLIT:-h}}"
split_ratio="${{AIBOX_LAYOUT_AGENT_RATIO:-50}}"
agent_pane="$(tmux -S "$socket" split-window -t "$session:cowork" "$split_flag" -p "$split_ratio" -P -F '#{{pane_id}}' -c "$workspace" "$(tool_or_shell {provider})")"
tmux -S "$socket" select-pane -t "$files_pane"
"#
        ),
        ConfigLayout::CoworkSwap => format!(
            r#"tmux -S "$socket" -f "$config" new-session -d -s "$session" -n cowork-swap -c "$workspace" "$(tool_or_shell yazi)"
files_pane="$(tmux -S "$socket" display-message -p -t "$session:cowork-swap" '#{{pane_id}}')"
split_flag="-${{AIBOX_LAYOUT_AGENT_SPLIT:-v}}"
split_ratio="${{AIBOX_LAYOUT_AGENT_RATIO:-45}}"
agent_pane="$(tmux -S "$socket" split-window -t "$session:cowork-swap" "$split_flag" -p "$split_ratio" -P -F '#{{pane_id}}' -c "$workspace" "$(tool_or_shell {provider})")"
tmux -S "$socket" select-pane -t "$files_pane"
"#
        ),
        ConfigLayout::Browse => format!(
            r#"tmux -S "$socket" -f "$config" new-session -d -s "$session" -n browse -c "$workspace" "$(tool_or_shell yazi)"
files_pane="$(tmux -S "$socket" display-message -p -t "$session:browse" '#{{pane_id}}')"
split_flag="-${{AIBOX_LAYOUT_AGENT_SPLIT:-v}}"
split_ratio="${{AIBOX_LAYOUT_AGENT_RATIO:-35}}"
agent_pane="$(tmux -S "$socket" split-window -t "$session:browse" "$split_flag" -p "$split_ratio" -P -F '#{{pane_id}}' -c "$workspace" "$(tool_or_shell {provider})")"
tmux -S "$socket" select-pane -t "$files_pane"
"#
        ),
        ConfigLayout::Ai => format!(
            r#"tmux -S "$socket" -f "$config" new-session -d -s "$session" -n ai -c "$workspace" "$(tool_or_shell yazi)"
files_pane="$(tmux -S "$socket" display-message -p -t "$session:ai" '#{{pane_id}}')"
split_flag="-${{AIBOX_LAYOUT_AGENT_SPLIT:-h}}"
split_ratio="${{AIBOX_LAYOUT_AGENT_RATIO:-50}}"
agent_pane="$(tmux -S "$socket" split-window -t "$session:ai" "$split_flag" -p "$split_ratio" -P -F '#{{pane_id}}' -c "$workspace" "$(tool_or_shell {provider})")"
tmux -S "$socket" new-window -t "$session:" -n shell -c "$workspace" "bash"
tmux -S "$socket" select-window -t "$session:ai"
tmux -S "$socket" select-pane -t "$files_pane"
"#
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

{layout_body}{git_window}tmux -S "$socket" select-window -t "$session:{primary_window}" 2>/dev/null || true
exec tmux -S "$socket" -f "$config" attach-session -t "$session"
"#,
        primary_window = primary_window,
    )
}

/// Render the `aibox-session.sh` dispatcher script that reads the configured
/// layout name and execs the corresponding `layouts/<name>.sh`.
///
/// BR-LAYOUT-DROPIN (BACK-20260509_1316-SnappyWolf, v0.25.7, "4b"): the
/// dispatcher resolves layouts from the user's home directory first, then
/// falls back to the system-wide install path. This means a user can drop
/// a custom `~/.config/tmux/layouts/<name>.sh` and run it via
/// `aibox-tmux-session <name>` without modifying anything aibox-managed.
/// Reserved layout names (dev, focus, cowork, cowork-swap, browse, ai) are
/// re-seeded by `aibox apply`; user-defined names are not touched.
pub fn tmux_session_script(config: &AiboxConfig) -> String {
    format!(
        r#"#!/usr/bin/env bash
set -euo pipefail

layout="${{1:-${{AIBOX_TMUX_LAYOUT:-{layout}}}}}"
session="${{2:-${{AIBOX_TMUX_SESSION:-{session}}}}}"
socket="${{AIBOX_TMUX_SOCKET:-$HOME/.tmux/aibox.sock}}"

# Resolve the layout script. Search order (BR-LAYOUT-DROPIN):
#   1. User drop-in: ~/.config/tmux/layouts/<layout>.sh
#   2. System default: /usr/local/share/aibox/tmux/layouts/<layout>.sh
# This lets users add custom layouts without conflicting with aibox-managed
# layout names. aibox apply re-seeds the six managed layouts only.
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
        session = &config.customization.tmux.session_name,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AiProvider;

    #[test]
    fn tmux_dev_layout_uses_selected_primary_provider() {
        let providers = vec![AiProvider::Codex, AiProvider::Claude];
        let layout = tmux_layout_script(&ConfigLayout::Dev, &providers, true, "aibox");

        assert!(
            layout.contains(
                "tmux -S \"$socket\" -f \"$config\" new-session -d -s \"$session\" -n dev"
            )
        );
        assert!(layout.contains("tool_or_shell codex"));
        assert!(!layout.contains("tool_or_shell claude"));
        assert!(layout.contains("tmux -S \"$socket\" new-window -t \"$session:\" -n git"));
        assert!(layout.contains("socket=\"${AIBOX_TMUX_SOCKET:-$HOME/.tmux/aibox.sock}\""));
        assert!(!layout.contains("zellij"));
    }

    #[test]
    fn tmux_layouts_start_expected_windows_and_panes() {
        let providers = [AiProvider::Claude];
        let layouts = [
            (ConfigLayout::Dev, "dev"),
            (ConfigLayout::Focus, "focus"),
            (ConfigLayout::Cowork, "cowork"),
            (ConfigLayout::CoworkSwap, "cowork-swap"),
            (ConfigLayout::Browse, "browse"),
            (ConfigLayout::Ai, "ai"),
        ];

        for (layout, name) in layouts {
            let body = tmux_layout_script(&layout, &providers, false, "aibox");
            assert!(
                body.contains(&format!("-n {name}")),
                "{name} layout should name its first tmux window:\n{body}"
            );
            assert!(
                body.contains(&format!(
                    "tmux -S \"$socket\" select-window -t \"$session:{name}\""
                )),
                "{name} layout should reselect the named primary window, not a numeric index:\n{body}"
            );
            assert!(
                !body.contains("tmux select-window -t \"$session:1\""),
                "layout script should not assume window index 1:\n{body}"
            );
            assert!(!body.contains("start_suspended"));
            assert!(!body.contains("zellij"));
        }
    }

    #[test]
    fn tmux_ai_layout_omits_unselected_claude() {
        let providers = vec![AiProvider::Codex];
        let layout = tmux_layout_script(&ConfigLayout::Ai, &providers, false, "aibox");

        assert!(layout.contains("tool_or_shell codex"));
        assert!(!layout.contains("tool_or_shell claude"));
        // BR-VIM-HARDCUT (v0.25.6): no editor window in any layout.
        assert!(!layout.contains("-n editor"));
        assert!(!layout.contains("editor_pane"));
        assert!(layout.contains("tmux -S \"$socket\" new-window -t \"$session:\" -n shell"));
    }

    #[test]
    fn tmux_layouts_have_no_persistent_editor_pane() {
        // BR-VIM-HARDCUT (DEC-20260508_1604-LuckySeal): the persistent
        // vim/editor pane was removed from every layout in v0.25.6.
        let providers = [AiProvider::Claude];
        for layout in [
            ConfigLayout::Dev,
            ConfigLayout::Focus,
            ConfigLayout::Cowork,
            ConfigLayout::CoworkSwap,
            ConfigLayout::Browse,
            ConfigLayout::Ai,
        ] {
            let body = tmux_layout_script(&layout, &providers, false, "aibox");
            assert!(
                !body.contains("editor_pane"),
                "{layout:?} must not create an editor_pane:\n{body}"
            );
            assert!(
                !body.contains("-n editor"),
                "{layout:?} must not create an editor window:\n{body}"
            );
            assert!(
                !body.contains("tool_or_shell vim"),
                "{layout:?} must not start a persistent vim:\n{body}"
            );
        }
    }

    #[test]
    fn tmux_session_helper_dispatches_to_provider_aware_layout_script() {
        let mut config = crate::config::test_config();
        config.customization.layout = ConfigLayout::Browse;
        config.customization.tmux.layout = Some(ConfigLayout::Ai);
        config.customization.tmux.session_name = "work".to_string();
        let script = tmux_session_script(&config);

        assert!(script.contains(r#"AIBOX_TMUX_LAYOUT:-ai"#));
        assert!(script.contains(r#"AIBOX_TMUX_SESSION:-work"#));
        assert!(script.contains(r#"AIBOX_TMUX_SOCKET:-$HOME/.tmux/aibox.sock"#));
        assert!(script.contains(r#".config/tmux/layouts/${layout}.sh"#));
        assert!(script.contains(
            r#"exec env AIBOX_TMUX_SESSION="${session}" AIBOX_TMUX_SOCKET="${socket}" "${script}""#
        ));
        assert!(!script.contains("tmux new-session"));
    }

    /// BR-LAYOUT-DROPIN (BACK-20260509_1316-SnappyWolf, v0.25.7): the
    /// dispatcher must search the user drop-in path before falling back to
    /// the system default, so user-defined layouts work without colliding
    /// with aibox-managed names.
    #[test]
    fn tmux_session_helper_searches_user_dropin_before_system_default() {
        let config = crate::config::test_config();
        let script = tmux_session_script(&config);

        assert!(
            script.contains(r#"user_script="${HOME}/.config/tmux/layouts/${layout}.sh""#),
            "dispatcher should resolve user drop-in layouts first:\n{script}"
        );
        assert!(
            script.contains(r#"system_script="/usr/local/share/aibox/tmux/layouts/${layout}.sh""#),
            "dispatcher should fall back to system layouts:\n{script}"
        );
        // Order matters: user must be checked before system.
        let user_pos = script
            .find(r#"if [[ -x "${user_script}" ]]"#)
            .expect("user-script existence check missing");
        let system_pos = script
            .find(r#"elif [[ -x "${system_script}" ]]"#)
            .expect("system-script existence check missing");
        assert!(
            user_pos < system_pos,
            "user drop-in must be checked before system default:\n{script}"
        );
    }

    /// BR-LAYOUT-KNOBS (BACK-20260509_1316-SnappyWolf, v0.25.7): the
    /// agent-pane split direction and percentage must be runtime-tunable
    /// for every layout that has an agent pane.
    #[test]
    fn tmux_layouts_expose_agent_split_knobs() {
        let providers = [AiProvider::Claude];
        for (layout, has_agent) in [
            (ConfigLayout::Dev, true),
            (ConfigLayout::Focus, false),
            (ConfigLayout::Cowork, true),
            (ConfigLayout::CoworkSwap, true),
            (ConfigLayout::Browse, true),
            (ConfigLayout::Ai, true),
        ] {
            let body = tmux_layout_script(&layout, &providers, false, "aibox");
            if has_agent {
                assert!(
                    body.contains(r#"split_flag="-${AIBOX_LAYOUT_AGENT_SPLIT:-"#),
                    "{layout:?} agent-pane split direction must be env-tunable:\n{body}"
                );
                assert!(
                    body.contains(r#"split_ratio="${AIBOX_LAYOUT_AGENT_RATIO:-"#),
                    "{layout:?} agent-pane split ratio must be env-tunable:\n{body}"
                );
                assert!(
                    body.contains(r#""$split_flag" -p "$split_ratio""#),
                    "{layout:?} split-window must use the parameterized flag/ratio:\n{body}"
                );
            } else {
                assert!(
                    !body.contains("AIBOX_LAYOUT_AGENT_"),
                    "{layout:?} (no agent pane) should not reference layout-knob env vars:\n{body}"
                );
            }
        }
    }

    /// Defaults must match the v0.25.6 hard-coded values so the unconfigured
    /// experience is identical.
    #[test]
    fn tmux_layout_defaults_preserve_v0_25_6_geometry() {
        let providers = [AiProvider::Claude];
        // (layout, default split flag, default ratio)
        let cases = [
            (ConfigLayout::Dev, "h", "50"),
            (ConfigLayout::Cowork, "h", "50"),
            (ConfigLayout::CoworkSwap, "v", "45"),
            (ConfigLayout::Browse, "v", "35"),
            (ConfigLayout::Ai, "h", "50"),
        ];
        for (layout, split, ratio) in cases {
            let body = tmux_layout_script(&layout, &providers, false, "aibox");
            assert!(
                body.contains(&format!("AIBOX_LAYOUT_AGENT_SPLIT:-{split}}}")),
                "{layout:?} default split flag must be {split}:\n{body}"
            );
            assert!(
                body.contains(&format!("AIBOX_LAYOUT_AGENT_RATIO:-{ratio}}}")),
                "{layout:?} default ratio must be {ratio}:\n{body}"
            );
        }
    }
}
