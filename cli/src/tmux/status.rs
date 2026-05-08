/// PowerKit settings and tmux status-format rendering.
///
/// Owns: `tmux_conf`, `tmux_powerkit_settings`, `push_enabled`, and the
/// `DEFAULT_TMUX_CONF` constant.  These are intentionally split from the
/// layout and sync modules so each unit can be tested in isolation.
use anyhow::{Context as _, Result};
use std::fs;
use std::path::Path;

use crate::config::{AiboxConfig, TmuxStatusMode};

/// Default tmux.conf template.  Placeholder tokens (`AIBOX_TMUX_*`) are
/// replaced by `tmux_conf` at apply-time.
pub(super) const DEFAULT_TMUX_CONF: &str = r##"# aibox tmux configuration
set -g default-shell /bin/bash
set -g default-command /bin/bash
set -g base-index 1
setw -g pane-base-index 1
set -g renumber-windows on
set -g mouse on
set -g history-limit 50000
set -g escape-time 10
set -g focus-events on
set -g allow-passthrough on
set -g default-terminal "tmux-256color"
set -ga terminal-features ",xterm-256color:RGB,tmux-256color:RGB"
set -ga terminal-overrides ",xterm-256color:Tc,tmux-256color:Tc"
set -g status-interval 5
set -g prefix AIBOX_TMUX_PREFIX
unbind C-b
bind AIBOX_TMUX_PREFIX send-prefix

# Pane navigation mirrors the old aibox leader muscle memory.
bind-key -N "Show aibox/tmux key bindings" ? display-popup -w 80% -h 75% -E "tmux list-keys -N | less -R"
bind-key -N "Select pane left" h select-pane -L
bind-key -N "Select pane down" j select-pane -D
bind-key -N "Select pane up" k select-pane -U
bind-key -N "Select pane right" l select-pane -R
bind-key -N "Split pane right" r split-window -h -c "#{pane_current_path}"
bind-key -N "Split pane down" d split-window -v -c "#{pane_current_path}"
bind-key -N "Kill pane" x kill-pane
bind-key -N "Toggle pane zoom" f resize-pane -Z
bind-key -N "Kill tmux session" q confirm-before -p "kill tmux session AIBOX_TMUX_SESSION? (y/n)" kill-session
bind-key -N "Reload tmux config" R source-file ~/.config/tmux/tmux.conf \; display-message "aibox tmux config reloaded"
bind-key -N "Open log pane (lnav)" L display-popup -E -w 90% -h 80% "lnav -q /workspace/.aibox/aibox.log /workspace/.aibox/aibox.log.1 2>/dev/null || less /workspace/.aibox/aibox.log"

set -g status AIBOX_TMUX_STATUS
set -g status-style "bg=AIBOX_TMUX_BG,fg=AIBOX_TMUX_FG"
set -g window-status-current-style "bg=AIBOX_TMUX_ACCENT,fg=AIBOX_TMUX_BG,bold"
set -g window-status-format " #I:#W "
set -g window-status-current-format " #I:#W "
set -g status-left " #S "
set -g status-right " AIBOX_TMUX_STATUS_RIGHT "

AIBOX_TMUX_POWERKIT_BLOCK

# aibox-managed plugins are installed and pinned by the runtime image. TPM is
# only a user convenience layer for additional personal plugins.
if-shell '[ -f /usr/local/share/aibox/tmux/plugins/tmux-sensible/sensible.tmux ]' 'run-shell /usr/local/share/aibox/tmux/plugins/tmux-sensible/sensible.tmux'
AIBOX_TMUX_POWERKIT_PLUGIN
if-shell '[ -f /usr/local/share/aibox/tmux/plugins/vim-tmux-navigator/vim-tmux-navigator.tmux ]' 'run-shell /usr/local/share/aibox/tmux/plugins/vim-tmux-navigator/vim-tmux-navigator.tmux'
if-shell '[ -f /usr/local/share/aibox/tmux/plugins/tmux-yank/yank.tmux ]' 'run-shell /usr/local/share/aibox/tmux/plugins/tmux-yank/yank.tmux'

# Persistence plugins are installed for later policy work but disabled by
# default in v0.25.0. Do not enable continuum/resurrect implicitly.
set -g @continuum-restore 'off'
set -g @continuum-save-interval '0'
set -g @resurrect-capture-pane-contents 'off'

if-shell '[ -x ~/.tmux/plugins/tpm/tpm ]' 'run-shell ~/.tmux/plugins/tpm/tpm'
"##;

/// Render a complete `tmux.conf` from the active `AiboxConfig`.
pub fn tmux_conf(config: &AiboxConfig) -> String {
    let theme = config.customization.resolved_theme();
    let (bg, fg, accent) = crate::themes::tmux_status_colors(&theme);
    let status = match config.customization.tmux.status.mode {
        TmuxStatusMode::Extended | TmuxStatusMode::Plain => "on",
        TmuxStatusMode::Disabled => "off",
    };

    let status_right = match config.customization.tmux.status.mode {
        TmuxStatusMode::Extended => "#(aibox-status --once 2>/dev/null || true) %H:%M",
        TmuxStatusMode::Plain | TmuxStatusMode::Disabled => "%H:%M",
    };
    let (powerkit_block, powerkit_plugin) = tmux_powerkit_settings(config);

    let mut conf = DEFAULT_TMUX_CONF
        .replace("AIBOX_TMUX_PREFIX", &config.customization.tmux.prefix)
        .replace(
            "AIBOX_TMUX_SESSION",
            &config.customization.tmux.session_name,
        )
        .replace("AIBOX_TMUX_STATUS_RIGHT", status_right)
        .replace("AIBOX_TMUX_STATUS", status)
        .replace("AIBOX_TMUX_POWERKIT_BLOCK", &powerkit_block)
        .replace("AIBOX_TMUX_POWERKIT_PLUGIN", &powerkit_plugin)
        .replace("AIBOX_TMUX_BG", bg)
        .replace("AIBOX_TMUX_FG", fg)
        .replace("AIBOX_TMUX_ACCENT", accent);

    // Keep the status block deterministic and avoid shelling out when disabled.
    if config.customization.tmux.status.mode == TmuxStatusMode::Disabled {
        conf = conf.replace("set -g status-interval 5", "set -g status-interval 60");
    }
    conf
}

pub(super) fn push_enabled(items: &mut Vec<&'static str>, enabled: bool, id: &'static str) {
    if enabled {
        items.push(id);
    }
}

/// Build the `@powerkit_*` settings block and plugin run-shell line.
///
/// Returns `(powerkit_block, powerkit_plugin)`.  Both are empty strings when
/// the status mode is not `Extended`.
pub fn tmux_powerkit_settings(config: &AiboxConfig) -> (String, String) {
    if config.customization.tmux.status.mode != TmuxStatusMode::Extended {
        return (String::new(), String::new());
    }

    let elements = &config.customization.tmux.status.elements;

    let mut line1_right = Vec::new();
    push_enabled(&mut line1_right, elements.hostname, "hostname");
    push_enabled(&mut line1_right, elements.external_ip, "external_ip");
    push_enabled(&mut line1_right, elements.ssh, "ssh");
    push_enabled(&mut line1_right, elements.uptime, "uptime");
    push_enabled(&mut line1_right, elements.weather, "weather");
    push_enabled(&mut line1_right, elements.datetime, "datetime");

    let mut line2_left = Vec::new();
    push_enabled(&mut line2_left, elements.git, "git");
    push_enabled(&mut line2_left, elements.github, "github");
    push_enabled(&mut line2_left, elements.kubernetes, "kubernetes");
    push_enabled(&mut line2_left, elements.terraform, "terraform");
    push_enabled(&mut line2_left, elements.cloud, "cloud");
    push_enabled(&mut line2_left, elements.cloudstatus, "cloudstatus");

    let mut line2_right = Vec::new();
    push_enabled(&mut line2_right, elements.cpu, "cpu");
    push_enabled(&mut line2_right, elements.loadavg, "loadavg");
    push_enabled(&mut line2_right, elements.mem, "memory");
    push_enabled(&mut line2_right, elements.swap, "swap");
    push_enabled(&mut line2_right, elements.disk, "disk");
    push_enabled(&mut line2_right, elements.gpu, "gpu");
    push_enabled(&mut line2_right, elements.netspeed, "netspeed");
    push_enabled(&mut line2_right, elements.ping, "ping");
    push_enabled(&mut line2_right, elements.aibox, "aibox");

    let mut plugin_order = Vec::new();
    plugin_order.extend(line1_right.iter().copied());
    plugin_order.extend(line2_left.iter().copied());
    plugin_order.extend(line2_right.iter().copied());

    let metrics = &elements.aibox_metrics;
    let aibox_metrics = [
        (metrics.log, "log"),
        (metrics.oom, "oom"),
        (metrics.proc, "proc"),
        (metrics.ai, "ai"),
        (metrics.mcp, "mcp"),
        (metrics.mig, "mig"),
    ]
    .iter()
    .filter_map(|(enabled, key)| enabled.then_some(*key))
    .collect::<Vec<_>>()
    .join(",");

    let (powerkit_theme, powerkit_variant) =
        crate::themes::tmux_powerkit_theme(&config.customization.resolved_theme());
    let powerkit_block = format!(
        r##"# Powerkit status.
set -g @powerkit_plugins "{}"
set -g @powerkit_bar_layout "double"
set -g @powerkit_status_order "session,plugins"
set -g @powerkit_theme "{}"
set -g @powerkit_theme_variant "{}"
set -g @powerkit_separator_style "rounded"
set -g @powerkit_elements_spacing "both"
set -g @powerkit_status_interval "5"
set -g @powerkit_transparent "false"
set -g @powerkit_pane_border_status "top"
set -g @powerkit_pane_border_format "#{{?client_prefix,PREFIX,NORMAL}} #{{pane_title}} #{{pane_current_command}}"
set -g @powerkit_line1_right "{}"
set -g @powerkit_line2_left "{}"
set -g @powerkit_line2_right "{}"
set -g @powerkit_plugin_aibox_metrics "{}""##,
        plugin_order.join(","),
        powerkit_theme,
        powerkit_variant,
        line1_right.join(","),
        line2_left.join(","),
        line2_right.join(","),
        aibox_metrics
    );
    let powerkit_plugin = "if-shell '[ -f /usr/local/share/aibox/tmux/plugins/tmux-powerkit/tmux-powerkit.tmux ]' 'run-shell /usr/local/share/aibox/tmux/plugins/tmux-powerkit/tmux-powerkit.tmux'".to_string();
    (powerkit_block, powerkit_plugin)
}

/// Hard-delete the tmux-powerkit plugin cache. Variant 1: every apply
/// regenerates the powerkit configuration from scratch, so any stashed cache
/// is by definition stale and must not survive across applies.
pub fn cleanup_tmux_powerkit_cache(root: &Path) -> Result<Vec<String>> {
    let mut updated = Vec::new();
    let cache_dir = root.join(".cache").join("tmux-powerkit");
    if cache_dir.exists() {
        fs::remove_dir_all(&cache_dir)
            .with_context(|| format!("Failed to remove {}", cache_dir.display()))?;
        updated.push(".cache/tmux-powerkit (removed stale tmux-powerkit cache)".to_string());
    }
    Ok(updated)
}

/// Walk `.aibox-home/.tmux/plugins/<plugin>` and remove any plugin that the
/// generated tmux.conf no longer references. The set of "referenced" plugins
/// is computed from the rendered tmux.conf so this stays in lockstep with
/// whatever plugins the active config asks for.
///
/// `tpm` is preserved unconditionally — it is the user-facing plugin manager
/// and is not aibox-managed in the strict sense. Zellij plugins are skipped
/// here too, because BR-ZELLIJ-EXCISE owns those.
pub fn cleanup_stale_tmux_plugins(config: &AiboxConfig, root: &Path) -> Result<Vec<String>> {
    let plugins_dir = root.join(".tmux").join("plugins");
    let mut updated = Vec::new();
    let Ok(entries) = fs::read_dir(&plugins_dir) else {
        return Ok(updated);
    };

    let conf = tmux_conf(config);
    for entry in entries {
        let entry =
            entry.with_context(|| format!("Failed to read {}", plugins_dir.display()))?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy().to_string();
        if name == "tpm" {
            continue;
        }
        if name.to_ascii_lowercase().contains("zellij") {
            continue;
        }
        // A plugin is "referenced" if the rendered tmux.conf names its
        // directory anywhere (e.g. `tmux-powerkit/tmux-powerkit.tmux`).
        let referenced = conf.contains(&format!("/{}/", name))
            || conf.contains(&format!("/{}.", name));
        if referenced {
            continue;
        }
        fs::remove_dir_all(&path)
            .with_context(|| format!("Failed to remove {}", path.display()))?;
        updated.push(format!(
            ".tmux/plugins/{} (removed stale tmux plugin)",
            name
        ));
    }
    Ok(updated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tmux_config_uses_pinned_managed_plugins_without_default_persistence() {
        let config = crate::config::test_config();
        let conf = tmux_conf(&config);

        assert!(
            conf.contains("set -g allow-passthrough on")
                && conf.contains("set -g default-terminal \"tmux-256color\""),
            "generated tmux config should enable passthrough and tmux-256color defaults for terminal app compatibility:\n{conf}"
        );
        assert!(
            conf.contains("/usr/local/share/aibox/tmux/plugins/tmux-sensible/sensible.tmux")
                && conf.contains(
                    "/usr/local/share/aibox/tmux/plugins/tmux-powerkit/tmux-powerkit.tmux"
                )
                && conf.contains(
                    "/usr/local/share/aibox/tmux/plugins/vim-tmux-navigator/vim-tmux-navigator.tmux"
                )
                && conf.contains("/usr/local/share/aibox/tmux/plugins/tmux-yank/yank.tmux"),
            "aibox-managed tmux plugins should load from preinstalled pinned runtime paths:\n{conf}"
        );
        assert!(
            conf.contains(
                r#"@powerkit_plugins "hostname,external_ip,ssh,uptime,weather,datetime,git,github,kubernetes,terraform,cloud,cloudstatus,cpu,loadavg,memory,swap,disk,gpu,netspeed,ping,aibox""#
            )
                && conf.contains(r#"@powerkit_bar_layout "double""#)
                && conf.contains(r#"@powerkit_status_order "session,plugins""#)
                && conf.contains(r#"@powerkit_status_interval "5""#)
                && conf.contains(r#"@powerkit_transparent "false""#)
                && conf.contains(r#"@powerkit_pane_border_status "top""#)
                && conf.contains(r##"@powerkit_pane_border_format "#{?client_prefix,PREFIX,NORMAL} #{pane_title} #{pane_current_command}""##)
                && conf.contains(r#"@powerkit_line1_right "hostname,external_ip,ssh,uptime,weather,datetime""#)
                && conf.contains(r#"@powerkit_line2_left "git,github,kubernetes,terraform,cloud,cloudstatus""#)
                && conf.contains(
                    r#"@powerkit_line2_right "cpu,loadavg,memory,swap,disk,gpu,netspeed,ping,aibox""#
                )
                && conf.contains(r#"@powerkit_plugin_aibox_metrics "log,oom,proc,ai,mcp,mig""#),
            "generated persistent tmux config should carry bounded powerkit defaults:\n{conf}"
        );
        assert!(
            conf.contains(
                r#"bind-key -N "Show aibox/tmux key bindings" ? display-popup -w 80% -h 75% -E "tmux list-keys -N | less -R""#
            ) && conf.contains(r#"bind-key -N "Select pane left" h select-pane -L"#),
            "generated persistent tmux config should expose native tmux keybinding help:\n{conf}"
        );
        assert!(
            !conf.contains("set -g @plugin 'tmux-plugins/tmux-continuum'")
                && !conf.contains("set -g @plugin 'tmux-plugins/tmux-resurrect'")
                && conf.contains("@continuum-restore 'off'")
                && conf.contains("@continuum-save-interval '0'")
                && conf.contains("@resurrect-capture-pane-contents 'off'"),
            "resurrect/continuum should stay disabled by default until persistence policy is decided:\n{conf}"
        );
        assert!(
            conf.contains("TPM is\n# only a user convenience layer"),
            "TPM should be documented as user convenience, not the managed plugin source:\n{conf}"
        );
    }

    #[test]
    fn tmux_status_layout_uses_image_status_binary() {
        let config = crate::config::test_config();
        let conf = tmux_conf(&config);

        assert!(conf.contains(r#"@powerkit_plugins "hostname,external_ip"#));
        assert!(conf.contains("tmux-powerkit.tmux"));
        assert!(
            !conf.contains("$HOME/.local/bin/aibox-status"),
            "tmux status must use the image-owned Rust status binary"
        );
    }

    #[test]
    fn tmux_plain_status_omits_runtime_segment() {
        let mut config = crate::config::test_config();
        config.customization.tmux.status.mode = TmuxStatusMode::Plain;
        let conf = tmux_conf(&config);

        assert!(conf.contains("set -g status on"));
        assert!(!conf.contains("aibox-status --once"));
        assert!(!conf.contains("@powerkit_plugins"));
        assert!(!conf.contains("tmux-powerkit.tmux"));
    }

    #[test]
    fn tmux_status_right_placeholder_is_replaced_before_status() {
        let config = crate::config::test_config();
        let conf = tmux_conf(&config);

        assert!(
            conf.contains(
                r#"set -g status-right " #(aibox-status --once 2>/dev/null || true) %H:%M ""#
            ),
            "status-right should retain runtime segment, not be truncated by status replacement:\n{conf}"
        );
        assert!(
            !conf.contains("on_RIGHT"),
            "status placeholder replacement order must not produce on_RIGHT artifacts:\n{conf}"
        );
    }

    #[test]
    fn tmux_disabled_status_turns_status_line_off() {
        let mut config = crate::config::test_config();
        config.customization.tmux.status.mode = TmuxStatusMode::Disabled;
        let conf = tmux_conf(&config);

        assert!(conf.contains("set -g status off"));
        assert!(!conf.contains("@powerkit_plugins"));
        assert!(!conf.contains("tmux-powerkit.tmux"));
    }
}
