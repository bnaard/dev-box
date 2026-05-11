/// Tmux layout script generation.
///
/// Owns: `tmux_layout_script` and `tmux_session_script`.  Each function
/// renders a shell script that is seeded into `.config/tmux/layouts/` or
/// `.config/tmux/aibox-session.sh` at apply-time.
use crate::config::{AiboxConfig, ConfigLayout};

/// Build the shell fragment that creates additional harness panes for the `ai`
/// layout when ≥2 harnesses are active.
///
/// BR-AI-MULTIHARNESS (BACK-20260510_0336-SmartLark, v0.25.7): the 1st
/// harness gets the main agent column (created by the primary split-window).
/// Subsequent harnesses are stacked as small vertical splits beneath the main
/// pane.
///
/// With 1 secondary harness it takes 20% of the agent column height, leaving
/// the primary at 80%.  With 2+ secondary harnesses each subsequent split
/// divides the remaining tail at 50%, giving roughly equal secondary shares.
///
/// Order is determined by `[ai].harness_order` in aibox.toml; enabled
/// harnesses omitted there are appended in canonical order.
fn ai_secondary_panes(active_harnesses: &[&str]) -> String {
    // active_harnesses[0] is already created as agent_pane by the caller.
    let secondaries = active_harnesses.get(1..).unwrap_or(&[]);
    if secondaries.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    let mut prev_var = "agent_pane".to_string();
    for (idx, harness) in secondaries.iter().enumerate() {
        // First secondary pane: 20% of the agent column → primary keeps ~80%.
        // Additional secondary panes: 50% of the remaining tail each.
        let ratio = if idx == 0 { 20usize } else { 50 };
        let var_name = format!("agent_pane_{}", idx + 2);
        out.push_str(&format!(
            r#"{var_name}="$(tmux -S "$socket" split-window -t "${{{prev_var}}}" -v -p {ratio} -P -F '#{{pane_id}}' -c "$workspace" "$(tool_or_shell {harness})")"
"#
        ));
        prev_var = var_name;
    }
    out
}

/// Build the shell fragment that stacks secondary harness panes (hidden) in the
/// cowork / cowork-swap agent column.
///
/// BR-COWORK-MULTIHARNESS (BACK-20260510_0726-HappyFjord, v0.25.7, DEC-TrueClover):
/// the 1st harness is visible; secondaries are stacked as 1-line panes then
/// immediately disabled (`select-pane -d`) so they don't steal focus but remain
/// addressable via `prefix j/k`.
fn cowork_secondary_panes(active_harnesses: &[&str]) -> String {
    let secondaries = active_harnesses.get(1..).unwrap_or(&[]);
    if secondaries.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    let mut prev_var = "agent_pane".to_string();
    for (idx, harness) in secondaries.iter().enumerate() {
        let var_name = format!("agent_pane_{}", idx + 2);
        // Split at 1 line height (resize to 0% is not portable; use -l 1 for
        // a minimal footprint). The pane is then disabled so it stays hidden
        // from normal pane cycling until summoned by prefix j/k.
        out.push_str(&format!(
            r#"{var_name}="$(tmux -S "$socket" split-window -t "${{{prev_var}}}" -v -l 1 -P -F '#{{pane_id}}' -c "$workspace" "$(tool_or_shell {harness})")"
tmux -S "$socket" select-pane -t "${{{var_name}}}" -d
"#
        ));
        prev_var = var_name;
    }
    out
}

/// Build the shell fragment that creates secondary harness windows for the
/// `dev` and `focus` layouts.
///
/// BR-DEV-MULTIHARNESS (BACK-20260510_0726-HappyFjord, v0.25.7, DEC-TrueClover):
/// each secondary harness gets its own tmux window named after the harness
/// binary.  For `dev`, the window is named `dev-<harness>` to differentiate
/// it from the primary `dev` window.  For `focus`, the window is named after
/// the harness binary directly (the primary is always the 1st harness).
fn dev_secondary_windows(active_harnesses: &[&str]) -> String {
    let secondaries = active_harnesses.get(1..).unwrap_or(&[]);
    if secondaries.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    for harness in secondaries.iter() {
        out.push_str(&format!(
            r#"tmux -S "$socket" new-window -t "$session:" -n "dev-{harness}" -c "$workspace" "$(tool_or_shell {harness})"
"#
        ));
    }
    out
}

/// Build the shell fragment that creates secondary harness windows for the
/// `focus` layout.
///
/// Each secondary harness gets its own window named after its binary.
fn focus_secondary_windows(active_harnesses: &[&str]) -> String {
    let secondaries = active_harnesses.get(1..).unwrap_or(&[]);
    if secondaries.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    for harness in secondaries.iter() {
        out.push_str(&format!(
            r#"tmux -S "$socket" new-window -t "$session:" -n "{harness}" -c "$workspace" "$(tool_or_shell {harness})"
"#
        ));
    }
    out
}

/// Render the layout-specific `<name>.sh` script that opens a fresh tmux
/// session with the requested pane/window arrangement.
pub fn tmux_layout_script(
    layout: &ConfigLayout,
    providers: &[crate::config::AiProvider],
    include_lazygit: bool,
    tool_windows: &[(&str, &str)],
    session_name: &str,
) -> String {
    let provider = providers
        .iter()
        .find(|provider| provider.is_active())
        .map(|provider| provider.binary_name())
        .unwrap_or("bash");

    // Collect active harness binary names in stable list order.
    // 1st harness = first active harness after harness_order; 2nd..N follow.
    let active_harnesses: Vec<&str> = providers
        .iter()
        .filter(|p| p.is_active())
        .map(|p| p.binary_name())
        .collect();

    // Tool windows: each `(window_name, binary)` that is enabled.
    // Emitted AFTER layout windows, BEFORE existing lazygit branch.
    let mut tool_window_lines = String::new();
    for (name, binary) in tool_windows {
        tool_window_lines.push_str(&format!(
            r#"tmux -S "$socket" new-window -t "$session:" -n {name} -c "$workspace" "$(tool_or_shell {binary})"
"#
        ));
    }

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
    //
    // BR-TRUECLOVER (BACK-20260510_0726-HappyFjord, v0.25.7, DEC-TrueClover):
    // per-layout multi-harness behaviour:
    //   browse      — ≥2 harnesses: hide AI panes entirely (yazi takes 100%)
    //   cowork/-swap — secondaries stacked hidden in agent column; prefix j/k cycles
    //   dev         — secondaries as tmux windows (dev-<harness>); prefix j/k cycles
    //   focus       — secondaries as windows (named after binary); only one visible
    let layout_body = match layout {
        ConfigLayout::Dev => {
            let secondary_windows = dev_secondary_windows(&active_harnesses);
            format!(
                r#"tmux -S "$socket" -f "$config" new-session -d -s "$session" -n dev -c "$workspace" "$(tool_or_shell yazi)"
files_pane="$(tmux -S "$socket" display-message -p -t "$session:dev" '#{{pane_id}}')"
split_flag="-${{AIBOX_LAYOUT_AGENT_SPLIT:-h}}"
split_ratio="${{AIBOX_LAYOUT_AGENT_RATIO:-50}}"
agent_pane="$(tmux -S "$socket" split-window -t "$session:dev" "$split_flag" -p "$split_ratio" -P -F '#{{pane_id}}' -c "$workspace" "$(tool_or_shell {provider})")"
{secondary_windows}tmux -S "$socket" select-pane -t "$files_pane"
tmux -S "$socket" select-window -t "$session:dev"
"#
            )
        }
        ConfigLayout::Focus => {
            let secondary_windows = focus_secondary_windows(&active_harnesses);
            format!(
                r#"tmux -S "$socket" -f "$config" new-session -d -s "$session" -n focus -c "$workspace" "$(tool_or_shell {provider})"
{secondary_windows}"#
            )
        }
        ConfigLayout::Cowork => {
            let secondary_panes = cowork_secondary_panes(&active_harnesses);
            format!(
                r#"tmux -S "$socket" -f "$config" new-session -d -s "$session" -n cowork -c "$workspace" "$(tool_or_shell yazi)"
files_pane="$(tmux -S "$socket" display-message -p -t "$session:cowork" '#{{pane_id}}')"
split_flag="-${{AIBOX_LAYOUT_AGENT_SPLIT:-h}}"
split_ratio="${{AIBOX_LAYOUT_AGENT_RATIO:-50}}"
agent_pane="$(tmux -S "$socket" split-window -t "$session:cowork" "$split_flag" -p "$split_ratio" -P -F '#{{pane_id}}' -c "$workspace" "$(tool_or_shell {provider})")"
{secondary_panes}tmux -S "$socket" select-pane -t "$files_pane"
"#
            )
        }
        ConfigLayout::CoworkSwap => {
            let secondary_panes = cowork_secondary_panes(&active_harnesses);
            format!(
                r#"tmux -S "$socket" -f "$config" new-session -d -s "$session" -n cowork-swap -c "$workspace" "$(tool_or_shell yazi)"
files_pane="$(tmux -S "$socket" display-message -p -t "$session:cowork-swap" '#{{pane_id}}')"
split_flag="-${{AIBOX_LAYOUT_AGENT_SPLIT:-v}}"
split_ratio="${{AIBOX_LAYOUT_AGENT_RATIO:-45}}"
agent_pane="$(tmux -S "$socket" split-window -t "$session:cowork-swap" "$split_flag" -p "$split_ratio" -P -F '#{{pane_id}}' -c "$workspace" "$(tool_or_shell {provider})")"
{secondary_panes}tmux -S "$socket" select-pane -t "$files_pane"
"#
            )
        }
        ConfigLayout::Browse => {
            // BR-TRUECLOVER: browse is file-focused. With ≥2 harnesses, hide
            // AI panes entirely — yazi takes the full window. With a single
            // harness keep the original split so the layout is unchanged.
            if active_harnesses.len() >= 2 {
                r#"tmux -S "$socket" -f "$config" new-session -d -s "$session" -n browse -c "$workspace" "$(tool_or_shell yazi)"
files_pane="$(tmux -S "$socket" display-message -p -t "$session:browse" '#{pane_id}')"
tmux -S "$socket" select-pane -t "$files_pane"
"#.to_string()
            } else {
                format!(
                    r#"tmux -S "$socket" -f "$config" new-session -d -s "$session" -n browse -c "$workspace" "$(tool_or_shell yazi)"
files_pane="$(tmux -S "$socket" display-message -p -t "$session:browse" '#{{pane_id}}')"
split_flag="-${{AIBOX_LAYOUT_AGENT_SPLIT:-v}}"
split_ratio="${{AIBOX_LAYOUT_AGENT_RATIO:-35}}"
agent_pane="$(tmux -S "$socket" split-window -t "$session:browse" "$split_flag" -p "$split_ratio" -P -F '#{{pane_id}}' -c "$workspace" "$(tool_or_shell {provider})")"
tmux -S "$socket" select-pane -t "$files_pane"
"#
                )
            }
        }
        ConfigLayout::Ai => {
            let secondary_panes = ai_secondary_panes(&active_harnesses);
            format!(
                r#"tmux -S "$socket" -f "$config" new-session -d -s "$session" -n ai -c "$workspace" "$(tool_or_shell yazi)"
files_pane="$(tmux -S "$socket" display-message -p -t "$session:ai" '#{{pane_id}}')"
split_flag="-${{AIBOX_LAYOUT_AGENT_SPLIT:-h}}"
split_ratio="${{AIBOX_LAYOUT_AGENT_RATIO:-50}}"
agent_pane="$(tmux -S "$socket" split-window -t "$session:ai" "$split_flag" -p "$split_ratio" -P -F '#{{pane_id}}' -c "$workspace" "$(tool_or_shell {provider})")"
{secondary_panes}tmux -S "$socket" new-window -t "$session:" -n shell -c "$workspace" "bash"
tmux -S "$socket" select-window -t "$session:ai"
tmux -S "$socket" select-pane -t "$files_pane"
"#
            )
        }
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

{layout_body}{tool_window_lines}{git_window}tmux -S "$socket" select-window -t "$session:{primary_window}" 2>/dev/null || true
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
    let session = config.tmux_session_name();
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
    fn tmux_dev_layout_uses_selected_primary_provider() {
        // codex is the 1st harness → primary pane in dev window.
        // claude is the 2nd harness → secondary window named "dev-claude".
        // BR-TRUECLOVER: with 2+ harnesses, secondaries are tabbed windows.
        let providers = vec![AiProvider::Codex, AiProvider::Claude];
        let layout = tmux_layout_script(&ConfigLayout::Dev, &providers, true, &no_tools(), "aibox");

        assert!(
            layout.contains(
                "tmux -S \"$socket\" -f \"$config\" new-session -d -s \"$session\" -n dev"
            )
        );
        // codex is the 1st harness → primary agent_pane in dev window
        assert!(layout.contains("tool_or_shell codex"));
        // claude is the 2nd harness → secondary window "dev-claude"
        assert!(
            layout.contains("tool_or_shell claude"),
            "2nd harness still appears as secondary window"
        );
        // codex must appear before claude (1st before 2nd)
        let codex_pos = layout.find("tool_or_shell codex").unwrap();
        let claude_pos = layout.find("tool_or_shell claude").unwrap();
        assert!(
            codex_pos < claude_pos,
            "1st harness (codex) must appear before 2nd harness (claude)"
        );
        // claude should be in a secondary window, not as the primary agent_pane
        assert!(
            layout.contains("new-window -t \"$session:\" -n \"dev-claude\""),
            "2nd harness must become a dev-<harness> secondary window"
        );
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
            let body = tmux_layout_script(&layout, &providers, false, &no_tools(), "aibox");
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
        let layout = tmux_layout_script(&ConfigLayout::Ai, &providers, false, &no_tools(), "aibox");

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
            let body = tmux_layout_script(&layout, &providers, false, &no_tools(), "aibox");
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
        config.aibox.project_name = "project-work".to_string();
        let script = tmux_session_script(&config);

        assert!(script.contains(r#"AIBOX_TMUX_LAYOUT:-ai"#));
        assert!(script.contains(r#"AIBOX_TMUX_SESSION:-work"#));
        assert!(!script.contains(r#"AIBOX_TMUX_SESSION:-project-work"#));
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
    /// for every layout that has an agent pane (single-harness).
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
            let body = tmux_layout_script(&layout, &providers, false, &no_tools(), "aibox");
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
            let body = tmux_layout_script(&layout, &providers, false, &no_tools(), "aibox");
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

    /// BR-AI-MULTIHARNESS (BACK-20260510_0336-SmartLark, v0.25.7):
    /// with a single harness the ai layout should be identical to v0.25.6
    /// (no secondary pane code generated).
    #[test]
    fn tmux_ai_layout_single_harness_unchanged() {
        let providers = [AiProvider::Claude];
        let body = tmux_layout_script(&ConfigLayout::Ai, &providers, false, &no_tools(), "aibox");

        assert!(body.contains("tool_or_shell claude"));
        assert!(
            !body.contains("agent_pane_2"),
            "single-harness ai layout must not generate secondary pane code:\n{body}"
        );
        // Shell window for user commands
        assert!(body.contains("new-window -t \"$session:\" -n shell"));
        // Files pane selected last
        assert!(body.contains("select-pane -t \"$files_pane\""));
    }

    /// BR-AI-MULTIHARNESS (BACK-20260510_0336-SmartLark, v0.25.7):
    /// with 2 harnesses the 1st harness is the primary agent pane and
    /// the 2nd harness is stacked at 20% beneath it (leaving ~80% for the primary).
    #[test]
    fn tmux_ai_layout_two_harnesses_stacks_secondary() {
        let providers = vec![AiProvider::Claude, AiProvider::Codex];
        let body = tmux_layout_script(&ConfigLayout::Ai, &providers, false, &no_tools(), "aibox");

        // Both harness binaries must appear.
        assert!(
            body.contains("tool_or_shell claude"),
            "1st harness (claude) must be in the layout:\n{body}"
        );
        assert!(
            body.contains("tool_or_shell codex"),
            "2nd harness (codex) must be in the layout:\n{body}"
        );
        // Claude must come before codex (order preservation).
        let claude_pos = body.find("tool_or_shell claude").unwrap();
        let codex_pos = body.find("tool_or_shell codex").unwrap();
        assert!(
            claude_pos < codex_pos,
            "1st harness (claude) must be created before 2nd harness (codex):\n{body}"
        );
        // Secondary pane variable must be generated.
        assert!(
            body.contains("agent_pane_2="),
            "secondary harness pane variable agent_pane_2 must appear:\n{body}"
        );
        // Secondary pane must be a vertical split at 20%.
        assert!(
            body.contains("-v -p 20"),
            "secondary harness pane must be a 20% vertical split:\n{body}"
        );
        // Shell window still present.
        assert!(body.contains("new-window -t \"$session:\" -n shell"));
    }

    /// BR-AI-MULTIHARNESS: with 3 harnesses the third is split at 50% of
    /// the second's tail, giving roughly equal secondary shares.
    #[test]
    fn tmux_ai_layout_three_harnesses_stacks_all_secondaries() {
        let providers = vec![AiProvider::Claude, AiProvider::Codex, AiProvider::Gemini];
        let body = tmux_layout_script(&ConfigLayout::Ai, &providers, false, &no_tools(), "aibox");

        assert!(body.contains("tool_or_shell claude"));
        assert!(body.contains("tool_or_shell codex"));
        assert!(body.contains("tool_or_shell gemini"));
        assert!(
            body.contains("agent_pane_2="),
            "second pane variable missing:\n{body}"
        );
        assert!(
            body.contains("agent_pane_3="),
            "third pane variable missing:\n{body}"
        );
        // First secondary at 20%; second secondary at 50% of remainder.
        assert!(
            body.contains("-v -p 20"),
            "first secondary must use 20% split:\n{body}"
        );
        assert!(
            body.contains("-v -p 50"),
            "second secondary must use 50% split:\n{body}"
        );
    }

    /// BR-AI-MULTIHARNESS: order-resolution — the effective order from
    /// `[ai].harness_order` determines which harness is 1st. Reversing the
    /// effective list reverses which harness is primary.
    #[test]
    fn tmux_ai_layout_harness_order_follows_config_list() {
        // codex first → codex is 1st harness (primary agent_pane), claude is secondary
        let providers_codex_first = vec![AiProvider::Codex, AiProvider::Claude];
        let body = tmux_layout_script(
            &ConfigLayout::Ai,
            &providers_codex_first,
            false,
            &no_tools(),
            "aibox",
        );
        let codex_pos = body.find("tool_or_shell codex").unwrap();
        let claude_pos = body.find("tool_or_shell claude").unwrap();
        assert!(
            codex_pos < claude_pos,
            "when codex is first in effective harness order it must be the primary harness:\n{body}"
        );
    }

    // ── BR-TRUECLOVER tests (BACK-20260510_0726-HappyFjord, v0.25.7) ──────

    /// browse layout: single harness keeps the original agent split.
    #[test]
    fn tmux_browse_layout_single_harness_has_agent_pane() {
        let providers = [AiProvider::Claude];
        let body = tmux_layout_script(
            &ConfigLayout::Browse,
            &providers,
            false,
            &no_tools(),
            "aibox",
        );

        assert!(
            body.contains("tool_or_shell claude"),
            "single-harness browse must start claude:\n{body}"
        );
        assert!(
            body.contains("split_flag="),
            "single-harness browse must create agent split:\n{body}"
        );
        assert!(
            body.contains("agent_pane="),
            "single-harness browse must create agent_pane:\n{body}"
        );
    }

    /// browse layout: ≥2 harnesses — no agent pane at all (yazi takes 100%).
    #[test]
    fn tmux_browse_layout_multi_harness_hides_ai_panes() {
        let providers = vec![AiProvider::Claude, AiProvider::Codex];
        let body = tmux_layout_script(
            &ConfigLayout::Browse,
            &providers,
            false,
            &no_tools(),
            "aibox",
        );

        assert!(
            !body.contains("agent_pane="),
            "multi-harness browse must not create an agent_pane:\n{body}"
        );
        assert!(
            !body.contains("AIBOX_LAYOUT_AGENT_SPLIT"),
            "multi-harness browse must not have an agent split:\n{body}"
        );
        assert!(
            !body.contains("tool_or_shell claude"),
            "multi-harness browse must not start any harness:\n{body}"
        );
        assert!(
            !body.contains("tool_or_shell codex"),
            "multi-harness browse must not start any harness:\n{body}"
        );
        // yazi still present
        assert!(
            body.contains("tool_or_shell yazi"),
            "browse must still start yazi:\n{body}"
        );
    }

    /// cowork layout: single harness unchanged.
    #[test]
    fn tmux_cowork_layout_single_harness_unchanged() {
        let providers = [AiProvider::Claude];
        let body = tmux_layout_script(
            &ConfigLayout::Cowork,
            &providers,
            false,
            &no_tools(),
            "aibox",
        );

        assert!(body.contains("tool_or_shell claude"));
        assert!(
            !body.contains("agent_pane_2="),
            "single-harness cowork must not have secondary pane:\n{body}"
        );
        assert!(
            !body.contains("select-pane -d"),
            "single-harness cowork must not disable panes:\n{body}"
        );
    }

    /// cowork layout: ≥2 harnesses — secondaries stacked hidden in agent column.
    #[test]
    fn tmux_cowork_layout_multi_harness_stacks_secondaries_hidden() {
        let providers = vec![AiProvider::Claude, AiProvider::Codex];
        let body = tmux_layout_script(
            &ConfigLayout::Cowork,
            &providers,
            false,
            &no_tools(),
            "aibox",
        );

        assert!(
            body.contains("tool_or_shell claude"),
            "order-1 must be in layout:\n{body}"
        );
        assert!(
            body.contains("tool_or_shell codex"),
            "order-2 must be in layout:\n{body}"
        );
        assert!(
            body.contains("agent_pane_2="),
            "secondary pane variable must be generated:\n{body}"
        );
        // Secondary pane must be disabled (hidden) after creation.
        assert!(
            body.contains("select-pane -t \"${agent_pane_2}\" -d"),
            "secondary pane must be disabled with select-pane -d:\n{body}"
        );
        // Must still be a pane (not a window).
        assert!(
            !body.contains("new-window -t \"$session:\" -n codex"),
            "cowork secondaries must be panes not windows:\n{body}"
        );
        // files_pane still selected last.
        assert!(body.contains("select-pane -t \"$files_pane\""));
    }

    /// cowork layout: 3 harnesses — both secondaries stacked and disabled.
    #[test]
    fn tmux_cowork_layout_three_harnesses_all_hidden() {
        let providers = vec![AiProvider::Claude, AiProvider::Codex, AiProvider::Gemini];
        let body = tmux_layout_script(
            &ConfigLayout::Cowork,
            &providers,
            false,
            &no_tools(),
            "aibox",
        );

        assert!(
            body.contains("agent_pane_2="),
            "second pane variable missing:\n{body}"
        );
        assert!(
            body.contains("agent_pane_3="),
            "third pane variable missing:\n{body}"
        );
        assert!(
            body.contains("select-pane -t \"${agent_pane_2}\" -d"),
            "second pane must be disabled:\n{body}"
        );
        assert!(
            body.contains("select-pane -t \"${agent_pane_3}\" -d"),
            "third pane must be disabled:\n{body}"
        );
    }

    /// cowork-swap layout: mirrors cowork multi-harness behaviour.
    #[test]
    fn tmux_cowork_swap_layout_multi_harness_stacks_secondaries_hidden() {
        let providers = vec![AiProvider::Claude, AiProvider::Codex];
        let body = tmux_layout_script(
            &ConfigLayout::CoworkSwap,
            &providers,
            false,
            &no_tools(),
            "aibox",
        );

        assert!(body.contains("tool_or_shell claude"));
        assert!(body.contains("tool_or_shell codex"));
        assert!(body.contains("agent_pane_2="));
        assert!(body.contains("select-pane -t \"${agent_pane_2}\" -d"));
        // cowork-swap uses vertical split (v) as primary
        assert!(body.contains("AIBOX_LAYOUT_AGENT_SPLIT:-v}"));
    }

    /// dev layout: single harness unchanged.
    #[test]
    fn tmux_dev_layout_single_harness_unchanged() {
        let providers = [AiProvider::Claude];
        let body = tmux_layout_script(&ConfigLayout::Dev, &providers, false, &no_tools(), "aibox");

        assert!(body.contains("tool_or_shell claude"));
        assert!(
            !body.contains("new-window -t \"$session:\" -n \"dev-"),
            "single-harness dev must not have secondary windows:\n{body}"
        );
    }

    /// dev layout: ≥2 harnesses — secondaries become windows named dev-<harness>.
    #[test]
    fn tmux_dev_layout_multi_harness_creates_secondary_windows() {
        let providers = vec![AiProvider::Claude, AiProvider::Codex];
        let body = tmux_layout_script(&ConfigLayout::Dev, &providers, false, &no_tools(), "aibox");

        assert!(
            body.contains("tool_or_shell claude"),
            "order-1 in main window:\n{body}"
        );
        assert!(
            body.contains("tool_or_shell codex"),
            "order-2 harness must appear:\n{body}"
        );
        assert!(
            body.contains("new-window -t \"$session:\" -n \"dev-codex\""),
            "secondary harness must become a dev-<harness> window:\n{body}"
        );
        // Must NOT be a pane split (no agent_pane_2 variable).
        assert!(
            !body.contains("agent_pane_2="),
            "dev secondaries must be windows, not panes:\n{body}"
        );
    }

    /// dev layout: 3 harnesses — both secondaries become windows.
    #[test]
    fn tmux_dev_layout_three_harnesses_all_secondary_windows() {
        let providers = vec![AiProvider::Claude, AiProvider::Codex, AiProvider::Gemini];
        let body = tmux_layout_script(&ConfigLayout::Dev, &providers, false, &no_tools(), "aibox");

        assert!(body.contains("new-window -t \"$session:\" -n \"dev-codex\""));
        assert!(body.contains("new-window -t \"$session:\" -n \"dev-gemini\""));
    }

    /// focus layout: single harness unchanged.
    #[test]
    fn tmux_focus_layout_single_harness_unchanged() {
        let providers = [AiProvider::Claude];
        let body = tmux_layout_script(
            &ConfigLayout::Focus,
            &providers,
            false,
            &no_tools(),
            "aibox",
        );

        assert!(body.contains("tool_or_shell claude"));
        assert!(
            !body.contains("new-window -t \"$session:\" -n \"claude\""),
            "single-harness focus must not have secondary window:\n{body}"
        );
    }

    /// focus layout: ≥2 harnesses — secondaries become windows named after harness binary.
    #[test]
    fn tmux_focus_layout_multi_harness_creates_secondary_windows() {
        let providers = vec![AiProvider::Claude, AiProvider::Codex];
        let body = tmux_layout_script(
            &ConfigLayout::Focus,
            &providers,
            false,
            &no_tools(),
            "aibox",
        );

        assert!(
            body.contains("tool_or_shell claude"),
            "order-1 in focus window:\n{body}"
        );
        assert!(
            body.contains("tool_or_shell codex"),
            "order-2 must appear:\n{body}"
        );
        assert!(
            body.contains("new-window -t \"$session:\" -n \"codex\""),
            "secondary harness must become a window named after its binary:\n{body}"
        );
    }

    // ── BR-TOOLS-AS-WINDOWS tests (BACK-20260510_0726-GrandDaisy, v0.25.7) ─

    /// With no tool_windows, the layout is identical to the baseline.
    #[test]
    fn tmux_tool_windows_empty_no_change() {
        let providers = [AiProvider::Claude];
        let body_no_tools =
            tmux_layout_script(&ConfigLayout::Ai, &providers, false, &no_tools(), "aibox");
        let body_with_tools =
            tmux_layout_script(&ConfigLayout::Ai, &providers, false, &[], "aibox");
        assert_eq!(body_no_tools, body_with_tools);
    }

    /// Each enabled tool addon gets a `new-window` line with the correct name/binary.
    #[test]
    fn tmux_tool_windows_emitted_for_each_enabled_tool() {
        let providers = [AiProvider::Claude];
        let tools = vec![
            ("k9s", "k9s"),
            ("btop", "btop"),
            ("lazydocker", "lazydocker"),
        ];
        let body = tmux_layout_script(&ConfigLayout::Ai, &providers, false, &tools, "aibox");

        assert!(
            body.contains("new-window -t \"$session:\" -n k9s"),
            "k9s window must be emitted:\n{body}"
        );
        assert!(
            body.contains("new-window -t \"$session:\" -n btop"),
            "btop window must be emitted:\n{body}"
        );
        assert!(
            body.contains("new-window -t \"$session:\" -n lazydocker"),
            "lazydocker window must be emitted:\n{body}"
        );
        // Tool windows must appear after the layout body's shell window.
        let k9s_pos = body.find("new-window -t \"$session:\" -n k9s").unwrap();
        let shell_pos = body.find("new-window -t \"$session:\" -n shell").unwrap();
        assert!(
            shell_pos < k9s_pos,
            "tool windows must appear after layout shell window:\n{body}"
        );
        // Tool windows must appear before the final attach-session line.
        let final_attach = body.rfind("attach-session").unwrap();
        assert!(
            k9s_pos < final_attach,
            "tool windows must appear before final attach-session:\n{body}"
        );
    }

    /// Tool windows appear after layout body but before lazygit window.
    #[test]
    fn tmux_tool_windows_ordered_before_git_window() {
        let providers = [AiProvider::Claude];
        let tools = vec![("k9s", "k9s")];
        let body = tmux_layout_script(&ConfigLayout::Ai, &providers, true, &tools, "aibox");

        let k9s_pos = body.find("new-window -t \"$session:\" -n k9s").unwrap();
        let git_pos = body.find("new-window -t \"$session:\" -n git").unwrap();
        assert!(
            k9s_pos < git_pos,
            "tool windows must appear before lazygit window:\n{body}"
        );
    }

    /// Tool windows work with non-Ai layouts too.
    #[test]
    fn tmux_tool_windows_work_with_any_layout() {
        let providers = [AiProvider::Claude];
        let tools = vec![("btop", "btop")];
        for layout in [
            ConfigLayout::Dev,
            ConfigLayout::Focus,
            ConfigLayout::Cowork,
            ConfigLayout::CoworkSwap,
            ConfigLayout::Browse,
        ] {
            let body = tmux_layout_script(&layout, &providers, false, &tools, "aibox");
            assert!(
                body.contains("new-window -t \"$session:\" -n btop"),
                "{layout:?} must emit btop tool window:\n{body}"
            );
        }
    }

    #[test]
    fn empty_harness_slice_does_not_panic() {
        // Regression: ai_secondary_panes / cowork_secondary_panes /
        // dev_secondary_windows / focus_secondary_windows previously did
        // `&active_harnesses[1..]` which panicked on an empty slice. They
        // now use .get(1..).unwrap_or(&[]) and return an empty fragment.
        assert_eq!(super::ai_secondary_panes(&[]), "");
        assert_eq!(super::cowork_secondary_panes(&[]), "");
        assert_eq!(super::dev_secondary_windows(&[]), "");
        assert_eq!(super::focus_secondary_windows(&[]), "");
    }
}
