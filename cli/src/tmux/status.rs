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
bind-key -N "Select pane down / next harness pane" j select-pane -D
bind-key -N "Select pane up / prev harness pane" k select-pane -U
bind-key -N "Select pane right" l select-pane -R
bind-key -N "Split pane right" r split-window -h -c "#{pane_current_path}"
bind-key -N "Split pane down" d split-window -v -c "#{pane_current_path}"
bind-key -N "Kill pane" x kill-pane
bind-key -N "Toggle pane zoom" f resize-pane -Z
bind-key -N "Toggle pane zoom (alias)" z resize-pane -Z
bind-key -N "Kill tmux session" q confirm-before -p "kill tmux session AIBOX_TMUX_SESSION? (y/n)" kill-session
bind-key -N "Reload tmux config" R source-file ~/.config/tmux/tmux.conf \; display-message "aibox tmux config reloaded"
bind-key -N "Open log pane (lnav)" L display-popup -E -w 90% -h 80% "lnav -q /workspace/.aibox/aibox.log /workspace/.aibox/aibox.log.1 2>/dev/null || less /workspace/.aibox/aibox.log"

# BR-TOOLS-AS-WINDOWS (BACK-20260510_0726-GrandDaisy, v0.25.7): one-letter
# prefix shortcuts to jump directly to named tool/harness windows.
# find-window -Z focuses the target window; silently no-ops when absent.
bind-key -N "Switch to git/lazygit window" g find-window -Z 'git'
bind-key -N "Switch to k9s/kubernetes window" K find-window -Z 'k9s'
bind-key -N "Switch to btop/system monitor window" B find-window -Z 'btop'
bind-key -N "Switch to lazydocker/containers window" D find-window -Z 'lazydocker'
bind-key -N "Switch to shell window" s find-window -Z 'shell'

set -g status AIBOX_TMUX_STATUS
set -g status-style "bg=AIBOX_TMUX_BG,fg=AIBOX_TMUX_FG"
set -g window-status-current-style "bg=AIBOX_TMUX_ACCENT,fg=AIBOX_TMUX_BG,bold"
set -g window-status-format " #I:#W "
set -g window-status-current-format " #I:#W "
set -g status-left " #S #{W:#I:#W ,#[reverse]#I:#W#[noreverse] }"
set -g status-left-length 80
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

/// Slot order constants — intentionally fixed per DEC-20260508_2115-SilentFern.
///
/// Any reordering requires: (a) a schema bump on the relevant aibox.toml
/// customization key, (b) a paired Migration entity in context/migrations/pending/,
/// (c) an update to the snapshot test below, and (d) docs-site screenshot updates.
/// See DEC-20260508_2115-SilentFern-powerkit-status-format-slot-order-is for rationale.
///
/// `LINE1_RIGHT_ORDER` and `LINE2_LEFT_ORDER` map 1:1 to the plugin names
/// used in the tmux.conf; `LINE2_RIGHT_AIBOX_METRICS_ORDER` maps the
/// internal key name to the PowerKit plugin ID after the path-a split.
const LINE1_RIGHT_ORDER: &[(&str, &str)] = &[
    ("hostname", "hostname"),
    ("external_ip", "externalip"),
    ("ssh", "ssh"),
    ("uptime", "uptime"),
    ("weather", "weather"),
    ("datetime", "datetime"),
];
const LINE2_LEFT_ORDER: &[(&str, &str)] = &[
    ("git", "git"),
    ("github", "github"),
    ("kubernetes", "kubernetes"),
    ("terraform", "terraform"),
    ("cloud", "cloud"),
    ("cloudstatus", "cloudstatus"),
];
/// Line 2 right: PowerKit system metrics followed by aibox-metrics block.
///
/// The aibox-metrics block (`aibox_log` … `aibox_mig`) uses path (a) split:
/// each metric is registered as its own individual PowerKit plugin so it
/// renders with the standard chevron/color-rotation segment styling rather
/// than as flat text inside a single `aibox` segment.  One plugin → one
/// segment → visually contiguous with adjacent PowerKit segments.
const LINE2_RIGHT_SYSTEM_ORDER: &[(&str, &str)] = &[
    ("cpu", "cpu"),
    ("loadavg", "loadavg"),
    ("mem", "memory"),
    ("swap", "swap"),
    ("disk", "disk"),
    ("gpu", "gpu"),
    ("netspeed", "netspeed"),
    ("ping", "ping"),
];
const LINE2_RIGHT_AIBOX_METRICS_ORDER: &[(&str, &str)] = &[
    ("log", "aibox_log"),
    ("oom", "aibox_oom"),
    ("proc", "aibox_proc"),
    ("ai", "aibox_ai"),
    ("mcp", "aibox_mcp"),
    ("mig", "aibox_mig"),
];

/// Build the `@powerkit_*` settings block and plugin run-shell line.
///
/// Returns `(powerkit_block, powerkit_plugin)`.  Both are empty strings when
/// the status mode is not `Extended`.
pub fn tmux_powerkit_settings(config: &AiboxConfig) -> (String, String) {
    if config.customization.tmux.status.mode != TmuxStatusMode::Extended {
        return (String::new(), String::new());
    }

    let elements = &config.customization.tmux.status.elements;

    // Line 1 right — slot order fixed per DEC-20260508_2115-SilentFern.
    // Correlates LINE1_RIGHT_ORDER keys against element enable-flags.
    let l1r_flags: &[(bool, &str)] = &[
        (elements.hostname, LINE1_RIGHT_ORDER[0].1),
        (elements.external_ip, LINE1_RIGHT_ORDER[1].1),
        (elements.ssh, LINE1_RIGHT_ORDER[2].1),
        (elements.uptime, LINE1_RIGHT_ORDER[3].1),
        (elements.weather, LINE1_RIGHT_ORDER[4].1),
        (elements.datetime, LINE1_RIGHT_ORDER[5].1),
    ];
    let line1_right: Vec<&str> = l1r_flags
        .iter()
        .filter_map(|(en, name)| en.then_some(*name))
        .collect();

    // Line 2 left — slot order fixed per DEC-20260508_2115-SilentFern.
    let l2l_flags: &[(bool, &str)] = &[
        (elements.git, LINE2_LEFT_ORDER[0].1),
        (elements.github, LINE2_LEFT_ORDER[1].1),
        (elements.kubernetes, LINE2_LEFT_ORDER[2].1),
        (elements.terraform, LINE2_LEFT_ORDER[3].1),
        (elements.cloud, LINE2_LEFT_ORDER[4].1),
        (elements.cloudstatus, LINE2_LEFT_ORDER[5].1),
    ];
    let line2_left: Vec<&str> = l2l_flags
        .iter()
        .filter_map(|(en, name)| en.then_some(*name))
        .collect();

    // Line 2 right — system metrics, slot order fixed per DEC-20260508_2115-SilentFern.
    let l2r_system_flags: &[(bool, &str)] = &[
        (elements.cpu, LINE2_RIGHT_SYSTEM_ORDER[0].1),
        (elements.loadavg, LINE2_RIGHT_SYSTEM_ORDER[1].1),
        (elements.mem, LINE2_RIGHT_SYSTEM_ORDER[2].1),
        (elements.swap, LINE2_RIGHT_SYSTEM_ORDER[3].1),
        (elements.disk, LINE2_RIGHT_SYSTEM_ORDER[4].1),
        (elements.gpu, LINE2_RIGHT_SYSTEM_ORDER[5].1),
        (elements.netspeed, LINE2_RIGHT_SYSTEM_ORDER[6].1),
        (elements.ping, LINE2_RIGHT_SYSTEM_ORDER[7].1),
    ];
    let mut line2_right: Vec<&str> = l2r_system_flags
        .iter()
        .filter_map(|(en, name)| en.then_some(*name))
        .collect();

    // aibox-metrics block: path-a split — each metric is its own PowerKit
    // segment (plugin) so it renders with chevron/color-rotation styling.
    // Slot order fixed per DEC-20260508_2115-SilentFern.
    let metrics = &elements.aibox_metrics;
    let metrics_flags: &[(bool, &str, &str)] = &[
        (
            metrics.log,
            LINE2_RIGHT_AIBOX_METRICS_ORDER[0].0,
            LINE2_RIGHT_AIBOX_METRICS_ORDER[0].1,
        ),
        (
            metrics.oom,
            LINE2_RIGHT_AIBOX_METRICS_ORDER[1].0,
            LINE2_RIGHT_AIBOX_METRICS_ORDER[1].1,
        ),
        (
            metrics.proc,
            LINE2_RIGHT_AIBOX_METRICS_ORDER[2].0,
            LINE2_RIGHT_AIBOX_METRICS_ORDER[2].1,
        ),
        (
            metrics.ai,
            LINE2_RIGHT_AIBOX_METRICS_ORDER[3].0,
            LINE2_RIGHT_AIBOX_METRICS_ORDER[3].1,
        ),
        (
            metrics.mcp,
            LINE2_RIGHT_AIBOX_METRICS_ORDER[4].0,
            LINE2_RIGHT_AIBOX_METRICS_ORDER[4].1,
        ),
        (
            metrics.mig,
            LINE2_RIGHT_AIBOX_METRICS_ORDER[5].0,
            LINE2_RIGHT_AIBOX_METRICS_ORDER[5].1,
        ),
    ];
    let aibox_metric_plugins: Vec<&str> = metrics_flags
        .iter()
        .filter_map(|(en, _key, plugin)| en.then_some(*plugin))
        .collect();
    line2_right.extend(aibox_metric_plugins.iter().copied());

    // Individual plugin option lines for each enabled aibox metric segment.
    let metric_option_lines: String = metrics_flags
        .iter()
        .filter_map(|(en, key, plugin)| {
            en.then_some(format!(
                "\nset -g @powerkit_plugin_{plugin}_metric \"{key}\""
            ))
        })
        .collect();

    let mut plugin_order = Vec::new();
    plugin_order.extend(line1_right.iter().copied());
    plugin_order.extend(line2_left.iter().copied());
    plugin_order.extend(line2_right.iter().copied());

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
set -g @powerkit_line2_right "{}"{}"##,
        plugin_order.join(","),
        powerkit_theme,
        powerkit_variant,
        line1_right.join(","),
        line2_left.join(","),
        line2_right.join(","),
        metric_option_lines
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
        let entry = entry.with_context(|| format!("Failed to read {}", plugins_dir.display()))?;
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
        let referenced =
            conf.contains(&format!("/{}/", name)) || conf.contains(&format!("/{}.", name));
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
                r#"@powerkit_plugins "hostname,externalip,ssh,uptime,weather,datetime,git,github,kubernetes,terraform,cloud,cloudstatus,cpu,loadavg,memory,swap,disk,gpu,netspeed,ping,aibox_log,aibox_oom,aibox_proc,aibox_ai,aibox_mcp,aibox_mig""#
            )
                && conf.contains(r#"@powerkit_bar_layout "double""#)
                && conf.contains(r#"@powerkit_status_order "session,plugins""#)
                && conf.contains(r#"@powerkit_status_interval "5""#)
                && conf.contains(r#"@powerkit_transparent "false""#)
                && conf.contains(r#"@powerkit_pane_border_status "top""#)
                && conf.contains(r##"@powerkit_pane_border_format "#{?client_prefix,PREFIX,NORMAL} #{pane_title} #{pane_current_command}""##)
                && conf.contains(r#"@powerkit_line1_right "hostname,externalip,ssh,uptime,weather,datetime""#)
                && conf.contains(r#"@powerkit_line2_left "git,github,kubernetes,terraform,cloud,cloudstatus""#)
                && conf.contains(
                    r#"@powerkit_line2_right "cpu,loadavg,memory,swap,disk,gpu,netspeed,ping,aibox_log,aibox_oom,aibox_proc,aibox_ai,aibox_mcp,aibox_mig""#
                )
                && conf.contains(r#"@powerkit_plugin_aibox_log_metric "log""#)
                && conf.contains(r#"@powerkit_plugin_aibox_oom_metric "oom""#)
                && conf.contains(r#"@powerkit_plugin_aibox_proc_metric "proc""#)
                && conf.contains(r#"@powerkit_plugin_aibox_ai_metric "ai""#)
                && conf.contains(r#"@powerkit_plugin_aibox_mcp_metric "mcp""#)
                && conf.contains(r#"@powerkit_plugin_aibox_mig_metric "mig""#),
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

        assert!(conf.contains(r#"@powerkit_plugins "hostname,externalip"#));
        assert!(conf.contains("tmux-powerkit.tmux"));
        // aibox metrics use path-a split: per-metric plugin segments, not a
        // single flat-text segment. Confirm the old single-segment is gone.
        assert!(
            !conf.contains(r#"@powerkit_plugins "hostname,externalip,ssh,uptime,weather,datetime,git,github,kubernetes,terraform,cloud,cloudstatus,cpu,loadavg,memory,swap,disk,gpu,netspeed,ping,aibox""#),
            "aibox single-segment plugin must be replaced by per-metric split plugins:\n{conf}"
        );
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

    /// Snapshot / literal-string test asserting the exact two-line status bar
    /// element ordering (DEC-20260508_2115-SilentFern — slot order is fixed).
    ///
    /// This test locks in the byte-equivalent plugin strings for both lines so
    /// any future reorder surfaces here rather than silently passing via
    /// substring matching.
    #[test]
    fn tmux_status_powerline_slot_order_snapshot() {
        let config = crate::config::test_config();
        let conf = tmux_conf(&config);

        // Line 1 right: hostname → externalip → ssh → uptime → weather → datetime
        // (DEC-20260508_2115-SilentFern)
        assert!(
            conf.contains(
                r#"@powerkit_line1_right "hostname,externalip,ssh,uptime,weather,datetime""#
            ),
            "line1_right slot order must be: hostname,externalip,ssh,uptime,weather,datetime\n{conf}"
        );

        // Line 2 left: git → github → kubernetes → terraform → cloud → cloudstatus
        // (DEC-20260508_2115-SilentFern)
        assert!(
            conf.contains(
                r#"@powerkit_line2_left "git,github,kubernetes,terraform,cloud,cloudstatus""#
            ),
            "line2_left slot order must be: git,github,kubernetes,terraform,cloud,cloudstatus\n{conf}"
        );

        // Line 2 right: system metrics + aibox-metrics block (path-a split, each metric its own segment)
        // (DEC-20260508_2115-SilentFern)
        assert!(
            conf.contains(r#"@powerkit_line2_right "cpu,loadavg,memory,swap,disk,gpu,netspeed,ping,aibox_log,aibox_oom,aibox_proc,aibox_ai,aibox_mcp,aibox_mig""#),
            "line2_right slot order must be: cpu,loadavg,memory,swap,disk,gpu,netspeed,ping,aibox_log,aibox_oom,aibox_proc,aibox_ai,aibox_mcp,aibox_mig\n{conf}"
        );

        // Full plugin list snapshot (all three line components concatenated)
        assert!(
            conf.contains(
                r#"@powerkit_plugins "hostname,externalip,ssh,uptime,weather,datetime,git,github,kubernetes,terraform,cloud,cloudstatus,cpu,loadavg,memory,swap,disk,gpu,netspeed,ping,aibox_log,aibox_oom,aibox_proc,aibox_ai,aibox_mcp,aibox_mig""#
            ),
            "full plugin list snapshot mismatch — slot order is fixed per DEC-20260508_2115-SilentFern\n{conf}"
        );

        // Per-metric plugin option registrations confirm path-a split is in effect
        // (chevron styling comes from each being an independent PowerKit segment)
        assert!(
            conf.contains(r#"@powerkit_plugin_aibox_log_metric "log""#)
                && conf.contains(r#"@powerkit_plugin_aibox_oom_metric "oom""#)
                && conf.contains(r#"@powerkit_plugin_aibox_proc_metric "proc""#)
                && conf.contains(r#"@powerkit_plugin_aibox_ai_metric "ai""#)
                && conf.contains(r#"@powerkit_plugin_aibox_mcp_metric "mcp""#)
                && conf.contains(r#"@powerkit_plugin_aibox_mig_metric "mig""#),
            "each aibox metric must register as an individual PowerKit plugin segment\n{conf}"
        );

        // Old single-segment flat-text rendering must be absent
        assert!(
            !conf.contains(r#"@powerkit_plugin_aibox_metrics"#),
            "old single-segment @powerkit_plugin_aibox_metrics must be absent after path-a split\n{conf}"
        );
    }

    /// Verify that partially-disabled aibox metrics omit the correct plugin names.
    #[test]
    fn tmux_status_powerline_partial_aibox_metrics() {
        let mut config = crate::config::test_config();
        // Disable oom and mig
        config.customization.tmux.status.elements.aibox_metrics.oom = false;
        config.customization.tmux.status.elements.aibox_metrics.mig = false;
        let conf = tmux_conf(&config);

        assert!(conf.contains("aibox_log"), "aibox_log should be present");
        assert!(!conf.contains("aibox_oom"), "aibox_oom should be absent");
        assert!(conf.contains("aibox_proc"), "aibox_proc should be present");
        assert!(conf.contains("aibox_ai"), "aibox_ai should be present");
        assert!(conf.contains("aibox_mcp"), "aibox_mcp should be present");
        assert!(!conf.contains("aibox_mig"), "aibox_mig should be absent");
    }

    /// BR-AI-MULTIHARNESS (BACK-20260510_0336-SmartLark, v0.25.7):
    /// leader j/k are already bound to down/up pane selection, which cycles
    /// between stacked harness panes in the ai layout.  leader z must be
    /// explicitly bound as a zoom toggle alias alongside the existing leader f.
    #[test]
    fn tmux_conf_has_zoom_toggle_on_z_and_j_k_harness_nav() {
        let config = crate::config::test_config();
        let conf = tmux_conf(&config);

        // z must be an explicit zoom toggle alias.
        assert!(
            conf.contains(r#"bind-key -N "Toggle pane zoom (alias)" z resize-pane -Z"#),
            "leader z must be bound as zoom toggle alias:\n{conf}"
        );
        // j / k pane navigation (down/up) serves as next/prev harness pane.
        assert!(
            conf.contains(r#"bind-key -N "Select pane down / next harness pane" j select-pane -D"#),
            "leader j must be bound to select-pane -D (next harness pane):\n{conf}"
        );
        assert!(
            conf.contains(r#"bind-key -N "Select pane up / prev harness pane" k select-pane -U"#),
            "leader k must be bound to select-pane -U (prev harness pane):\n{conf}"
        );
        // f zoom toggle must still exist (original binding preserved).
        assert!(
            conf.contains(r#"bind-key -N "Toggle pane zoom" f resize-pane -Z"#),
            "original leader f zoom toggle must remain:\n{conf}"
        );
    }

    /// BR-TOOLS-AS-WINDOWS (BACK-20260510_0726-GrandDaisy, v0.25.7):
    /// one-letter prefix bindings for tool windows must be present in the
    /// generated tmux.conf.  All bindings use `find-window -Z` so they
    /// silently no-op when the target window does not exist.
    #[test]
    fn tmux_conf_has_tool_window_keybindings() {
        let config = crate::config::test_config();
        let conf = tmux_conf(&config);

        assert!(
            conf.contains(r#"bind-key -N "Switch to git/lazygit window" g find-window -Z 'git'"#),
            "leader g must jump to git/lazygit window:\n{conf}"
        );
        assert!(
            conf.contains(r#"bind-key -N "Switch to k9s/kubernetes window" K find-window -Z 'k9s'"#),
            "leader K must jump to k9s window:\n{conf}"
        );
        assert!(
            conf.contains(r#"bind-key -N "Switch to btop/system monitor window" B find-window -Z 'btop'"#),
            "leader B must jump to btop window:\n{conf}"
        );
        assert!(
            conf.contains(r#"bind-key -N "Switch to lazydocker/containers window" D find-window -Z 'lazydocker'"#),
            "leader D must jump to lazydocker window:\n{conf}"
        );
        assert!(
            conf.contains(r#"bind-key -N "Switch to shell window" s find-window -Z 'shell'"#),
            "leader s must jump to shell window:\n{conf}"
        );
    }
}
