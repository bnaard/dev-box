/// PowerKit settings and tmux status-format rendering.
///
/// Owns: `tmux_conf`, `tmux_powerkit_settings`, `push_enabled`, and the
/// `DEFAULT_TMUX_CONF` constant.  These are intentionally split from the
/// layout and sync modules so each unit can be tested in isolation.
use anyhow::{Context as _, Result};
use std::fs;
use std::path::Path;

use crate::config::{AiboxConfig, TmuxStatusMode};

const MODEL_STATUS_PROVIDER_ORDER: &[&str] = &[
    "openai",
    "anthropic",
    "google",
    "mistral",
    "deepseek",
    "cohere",
    "xai",
    "alibaba",
    "aws",
    "meta",
    "microsoft",
    "minimax",
    "moonshot",
    "nvidia",
    "xiaomi",
    "zai",
];

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
set -g status-interval AIBOX_TMUX_STATUS_INTERVAL
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
bind-key -N "Open log pane (lnav)" L display-popup -E -w 90% -h 80% "aibox-log-viewer"

# BR-TOOLS-AS-WINDOWS (BACK-20260510_0726-GrandDaisy, v0.25.7): one-letter
# prefix shortcuts to jump directly to named tool/harness windows.
# find-window -Z focuses the target window; silently no-ops when absent.
bind-key -N "Switch to lazygit window" g find-window -Z 'lazygit'
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

AIBOX_TMUX_POWERKIT_FORMATS

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
    let (powerkit_block, powerkit_plugin, powerkit_formats) = tmux_powerkit_settings(config);

    let mut conf = DEFAULT_TMUX_CONF
        .replace("AIBOX_TMUX_PREFIX", &config.customization.tmux.prefix)
        .replace("AIBOX_TMUX_SESSION", &config.tmux_session_name())
        .replace("AIBOX_TMUX_STATUS_RIGHT", status_right)
        .replace(
            "AIBOX_TMUX_STATUS_INTERVAL",
            &config
                .customization
                .tmux
                .status
                .refresh
                .interval_seconds
                .to_string(),
        )
        .replace("AIBOX_TMUX_STATUS", status)
        .replace("AIBOX_TMUX_POWERKIT_BLOCK", &powerkit_block)
        .replace("AIBOX_TMUX_POWERKIT_PLUGIN", &powerkit_plugin)
        .replace("AIBOX_TMUX_POWERKIT_FORMATS", &powerkit_formats)
        .replace("AIBOX_TMUX_BG", bg)
        .replace("AIBOX_TMUX_FG", fg)
        .replace("AIBOX_TMUX_ACCENT", accent);

    // Keep the status block deterministic and avoid shelling out when disabled.
    if config.customization.tmux.status.mode == TmuxStatusMode::Disabled {
        conf = conf.replace(
            &format!(
                "set -g status-interval {}",
                config.customization.tmux.status.refresh.interval_seconds
            ),
            "set -g status-interval 60",
        );
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
/// used in the tmux.conf; `LINE1_RIGHT_AIBOX_METRICS_ORDER` maps the
/// internal key name to the PowerKit plugin ID after the path-a split.
const LINE1_RIGHT_ORDER: &[(&str, &str)] = &[
    ("weather", "weather"),
    ("uptime", "uptime"),
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
/// Line 2 right: PowerKit system and network metrics.
const LINE2_RIGHT_SYSTEM_ORDER: &[(&str, &str)] = &[
    ("hostname", "hostname"),
    ("external_ip", "externalip"),
    ("ssh", "ssh"),
    ("netspeed", "netspeed"),
    ("ping", "ping"),
    ("cpu", "cpu"),
    ("loadavg", "loadavg"),
    ("mem", "memory"),
    ("swap", "swap"),
    ("disk", "disk"),
    ("gpu", "gpu"),
];
/// Line 1 right: aibox-metrics block inserted before weather.
///
/// The aibox-metrics block (`aibox_log` … `aibox_mig`) uses path (a) split:
/// each metric is registered as its own individual PowerKit plugin so it
/// renders with the standard chevron/color-rotation segment styling rather
/// than as flat text inside a single `aibox` segment. One plugin → one
/// segment → visually contiguous with adjacent PowerKit segments.
const LINE1_RIGHT_AIBOX_METRICS_ORDER: &[(&str, &str)] = &[
    ("log", "aibox_log"),
    ("oom", "aibox_oom"),
    ("proc", "aibox_proc"),
    ("ai", "aibox_ai"),
    ("mcp", "aibox_mcp"),
    ("mig", "aibox_mig"),
];

fn modelstatus_plugin_name(provider: &str) -> String {
    format!("modelstatus_{provider}")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedTmuxStatusLayout {
    pub line1_left: Vec<String>,
    pub line1_right: Vec<String>,
    pub line2_left: Vec<String>,
    pub line2_right: Vec<String>,
}

pub(crate) fn resolved_tmux_status_layout(config: &AiboxConfig) -> ResolvedTmuxStatusLayout {
    let elements = &config.customization.tmux.status.elements;
    let layout = &config.customization.tmux.status.layout;

    let line2_left: Vec<String> = layout.line2_left.clone().unwrap_or_else(|| {
        [
            (elements.git, LINE2_LEFT_ORDER[0].1),
            (elements.github, LINE2_LEFT_ORDER[1].1),
            (elements.kubernetes, LINE2_LEFT_ORDER[2].1),
            (elements.terraform, LINE2_LEFT_ORDER[3].1),
            (elements.cloud, LINE2_LEFT_ORDER[4].1),
            (elements.cloudstatus, LINE2_LEFT_ORDER[5].1),
        ]
        .iter()
        .filter_map(|(en, name)| en.then_some((*name).to_string()))
        .collect()
    });

    let line2_right: Vec<String> = layout.line2_right.clone().unwrap_or_else(|| {
        [
            (elements.hostname, LINE2_RIGHT_SYSTEM_ORDER[0].1),
            (elements.external_ip, LINE2_RIGHT_SYSTEM_ORDER[1].1),
            (elements.ssh, LINE2_RIGHT_SYSTEM_ORDER[2].1),
            (elements.netspeed, LINE2_RIGHT_SYSTEM_ORDER[3].1),
            (elements.ping, LINE2_RIGHT_SYSTEM_ORDER[4].1),
            (elements.cpu, LINE2_RIGHT_SYSTEM_ORDER[5].1),
            (elements.loadavg, LINE2_RIGHT_SYSTEM_ORDER[6].1),
            (elements.mem, LINE2_RIGHT_SYSTEM_ORDER[7].1),
            (elements.swap, LINE2_RIGHT_SYSTEM_ORDER[8].1),
            (elements.disk, LINE2_RIGHT_SYSTEM_ORDER[9].1),
            (elements.gpu, LINE2_RIGHT_SYSTEM_ORDER[10].1),
        ]
        .iter()
        .filter_map(|(en, name)| en.then_some((*name).to_string()))
        .collect()
    });

    let metrics = &elements.aibox_metrics;
    let aibox_metric_plugins: Vec<String> = if elements.aibox {
        [
            (metrics.log, LINE1_RIGHT_AIBOX_METRICS_ORDER[0].1),
            (metrics.oom, LINE1_RIGHT_AIBOX_METRICS_ORDER[1].1),
            (metrics.proc, LINE1_RIGHT_AIBOX_METRICS_ORDER[2].1),
            (metrics.ai, LINE1_RIGHT_AIBOX_METRICS_ORDER[3].1),
            (metrics.mcp, LINE1_RIGHT_AIBOX_METRICS_ORDER[4].1),
            (metrics.mig, LINE1_RIGHT_AIBOX_METRICS_ORDER[5].1),
        ]
        .iter()
        .filter_map(|(en, plugin)| en.then_some((*plugin).to_string()))
        .collect()
    } else {
        Vec::new()
    };

    let line1_right: Vec<String> = layout.line1_right.clone().unwrap_or_else(|| {
        let mut plugins = Vec::new();
        plugins.extend(aibox_metric_plugins);
        if config.customization.tmux.status.model_providers.enabled {
            for known_provider in MODEL_STATUS_PROVIDER_ORDER {
                if config
                    .customization
                    .tmux
                    .status
                    .model_providers
                    .providers
                    .iter()
                    .any(|provider| provider.provider == *known_provider)
                {
                    plugins.push(modelstatus_plugin_name(known_provider));
                }
            }
        }
        plugins.extend(
            [
                (elements.weather, LINE1_RIGHT_ORDER[0].1),
                (elements.uptime, LINE1_RIGHT_ORDER[1].1),
                (elements.datetime, LINE1_RIGHT_ORDER[2].1),
            ]
            .iter()
            .filter_map(|(en, name)| en.then_some((*name).to_string())),
        );
        plugins
    });

    let line1_left = layout
        .line1_left
        .clone()
        .unwrap_or_else(|| vec!["session".to_string(), "windows".to_string()]);

    ResolvedTmuxStatusLayout {
        line1_left,
        line1_right,
        line2_left,
        line2_right,
    }
}

/// Build the `@powerkit_*` settings block and plugin run-shell line.
///
/// Returns `(powerkit_block, powerkit_plugin, powerkit_formats)`.
/// All are empty strings when
/// the status mode is not `Extended`.
pub fn tmux_powerkit_settings(config: &AiboxConfig) -> (String, String, String) {
    if config.customization.tmux.status.mode != TmuxStatusMode::Extended {
        return (String::new(), String::new(), String::new());
    }

    let resolved_layout = resolved_tmux_status_layout(config);
    let line1_left = resolved_layout.line1_left;
    let line1_right = resolved_layout.line1_right;
    let line2_left = resolved_layout.line2_left;
    let line2_right = resolved_layout.line2_right;

    // aibox-metrics block: path-a split — each metric is its own PowerKit
    // segment (plugin) so it renders with chevron/color-rotation styling.
    // Slot order fixed per DEC-20260508_2115-SilentFern.
    let refresh = &config.customization.tmux.status.refresh;
    let metrics_flags: &[(bool, &str, &str)] = &[
        (
            true,
            LINE1_RIGHT_AIBOX_METRICS_ORDER[0].0,
            LINE1_RIGHT_AIBOX_METRICS_ORDER[0].1,
        ),
        (
            true,
            LINE1_RIGHT_AIBOX_METRICS_ORDER[1].0,
            LINE1_RIGHT_AIBOX_METRICS_ORDER[1].1,
        ),
        (
            true,
            LINE1_RIGHT_AIBOX_METRICS_ORDER[2].0,
            LINE1_RIGHT_AIBOX_METRICS_ORDER[2].1,
        ),
        (
            true,
            LINE1_RIGHT_AIBOX_METRICS_ORDER[3].0,
            LINE1_RIGHT_AIBOX_METRICS_ORDER[3].1,
        ),
        (
            true,
            LINE1_RIGHT_AIBOX_METRICS_ORDER[4].0,
            LINE1_RIGHT_AIBOX_METRICS_ORDER[4].1,
        ),
        (
            true,
            LINE1_RIGHT_AIBOX_METRICS_ORDER[5].0,
            LINE1_RIGHT_AIBOX_METRICS_ORDER[5].1,
        ),
    ];

    // Individual plugin option lines for each enabled aibox metric segment.
    let metric_option_lines: String = metrics_flags
        .iter()
        .filter_map(|(_en, key, plugin)| {
            line1_right
                .iter()
                .any(|configured| configured == plugin)
                .then_some(format!(
                    "\nset -g @powerkit_plugin_{plugin}_metric \"{key}\"\nset -g @powerkit_plugin_{plugin}_cache_ttl \"{}\"",
                    refresh.aibox_metrics_cache_ttl_seconds
                ))
        })
        .collect();
    let refresh_option_lines =
        status_refresh_option_lines(config, &line1_right, &line2_left, &line2_right);
    let status_label_option_lines =
        status_label_option_lines(config, &line1_right, &line2_left, &line2_right);
    let model_provider_option_lines = model_provider_option_lines(config, &line1_right);

    let mut plugin_order = Vec::new();
    plugin_order.extend(line1_right.iter().map(String::as_str));
    plugin_order.extend(line2_left.iter().map(String::as_str));
    plugin_order.extend(line2_right.iter().map(String::as_str));

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
set -g @powerkit_status_interval "{}"
set -g @powerkit_transparent "false"
set -g @powerkit_pane_border_status "top"
set -g @powerkit_pane_border_format "#{{?client_prefix,PREFIX,NORMAL}} #{{pane_title}} #{{pane_current_command}}"
set -g @powerkit_line1_right "{}"
set -g @powerkit_line2_left "{}"
set -g @powerkit_line2_right "{}"
set -g @powerkit_plugin_netspeed_speed_width "7"{}{}{}{}"##,
        plugin_order.join(","),
        powerkit_theme,
        powerkit_variant,
        refresh.interval_seconds,
        line1_right.join(","),
        line2_left.join(","),
        line2_right.join(","),
        metric_option_lines,
        refresh_option_lines,
        status_label_option_lines,
        model_provider_option_lines
    );
    let powerkit_plugin = "if-shell '[ -f /usr/local/share/aibox/tmux/plugins/tmux-powerkit/tmux-powerkit.tmux ]' 'run-shell /usr/local/share/aibox/tmux/plugins/tmux-powerkit/tmux-powerkit.tmux'".to_string();
    let powerkit_formats =
        tmux_powerkit_status_formats(&line1_left, &line1_right, &line2_left, &line2_right);
    (powerkit_block, powerkit_plugin, powerkit_formats)
}

fn status_refresh_option_lines(
    config: &AiboxConfig,
    line1_right: &[String],
    line2_left: &[String],
    line2_right: &[String],
) -> String {
    let refresh = &config.customization.tmux.status.refresh;
    let configured = |plugin: &str| {
        line1_right.iter().any(|item| item == plugin)
            || line2_left.iter().any(|item| item == plugin)
            || line2_right.iter().any(|item| item == plugin)
    };
    let mut lines = String::new();
    for (plugin, ttl) in [
        ("netspeed", refresh.netspeed_cache_ttl_seconds),
        ("kubernetes", refresh.kubernetes_cache_ttl_seconds),
        ("cloud", refresh.cloud_cache_ttl_seconds),
    ] {
        if configured(plugin) {
            lines.push_str(&format!(
                "\nset -g @powerkit_plugin_{plugin}_cache_ttl \"{ttl}\""
            ));
        }
    }
    lines
}

fn status_label_option_lines(
    config: &AiboxConfig,
    line1_right: &[String],
    line2_left: &[String],
    line2_right: &[String],
) -> String {
    let labels = &config.customization.tmux.status.labels;
    let configured = |plugin: &str| {
        line1_right.iter().any(|item| item == plugin)
            || line2_left.iter().any(|item| item == plugin)
            || line2_right.iter().any(|item| item == plugin)
    };
    let mut lines = String::new();
    for (plugin, option, value) in [
        ("aibox_log", "label", labels.aibox_log.as_str()),
        ("aibox_oom", "label", labels.aibox_oom.as_str()),
        ("aibox_proc", "label", labels.aibox_proc.as_str()),
        ("aibox_ai", "label", labels.aibox_ai.as_str()),
        ("aibox_mcp", "label", labels.aibox_mcp.as_str()),
        ("aibox_mig", "label", labels.aibox_mig.as_str()),
        ("kubernetes", "icon", labels.kubernetes.as_str()),
        ("cloud", "icon", labels.cloud.as_str()),
        ("cloud", "icon_aws", labels.cloud_aws.as_str()),
        ("cloud", "icon_gcp", labels.cloud_gcp.as_str()),
        ("cloud", "icon_azure", labels.cloud_azure.as_str()),
        ("cloud", "icon_multi", labels.cloud_multi.as_str()),
        ("uptime", "icon", labels.uptime.as_str()),
        ("netspeed", "icon", labels.netspeed.as_str()),
        (
            "netspeed",
            "icon_download",
            labels.netspeed_download.as_str(),
        ),
        ("netspeed", "icon_upload", labels.netspeed_upload.as_str()),
    ] {
        if configured(plugin) {
            lines.push_str(&format!(
                "\nset -g @powerkit_plugin_{plugin}_{option} \"{}\"",
                tmux_option_escape(value)
            ));
        }
    }
    lines
}

fn model_provider_option_lines(config: &AiboxConfig, line1_right: &[String]) -> String {
    if !config.customization.tmux.status.model_providers.enabled {
        return String::new();
    }

    let model_providers = &config.customization.tmux.status.model_providers;
    let mut lines = String::new();
    for provider in &model_providers.providers {
        let plugin = modelstatus_plugin_name(&provider.provider);
        if !line1_right.iter().any(|configured| configured == &plugin) {
            continue;
        }
        lines.push_str(&format!(
            "\nset -g @powerkit_plugin_{plugin}_provider \"{}\"",
            tmux_option_escape(&provider.provider)
        ));
        lines.push_str(&format!(
            "\nset -g @powerkit_plugin_{plugin}_label \"{}\"",
            tmux_option_escape(&provider.label)
        ));
        lines.push_str(&format!(
            "\nset -g @powerkit_plugin_{plugin}_checks \"{}\"",
            tmux_option_escape(&provider.checks.join(","))
        ));
        lines.push_str(&format!(
            "\nset -g @powerkit_plugin_{plugin}_cache_ttl \"{}\"",
            model_providers.cache_ttl_seconds
        ));
        lines.push_str(&format!(
            "\nset -g @powerkit_plugin_{plugin}_timeout \"{}\"",
            model_providers.timeout_seconds
        ));
        lines.push_str(&format!(
            "\nset -g @powerkit_plugin_{plugin}_show_ok \"{}\"",
            model_providers.show_ok
        ));
        if let Some(status_url) = &provider.status_url {
            lines.push_str(&format!(
                "\nset -g @powerkit_plugin_{plugin}_status_url \"{}\"",
                tmux_option_escape(status_url)
            ));
        }
        if !provider.overall_components.is_empty() {
            lines.push_str(&format!(
                "\nset -g @powerkit_plugin_{plugin}_overall_components \"{}\"",
                tmux_option_escape(&provider.overall_components.join(","))
            ));
        }
        if !provider.model_components.is_empty() {
            lines.push_str(&format!(
                "\nset -g @powerkit_plugin_{plugin}_model_components \"{}\"",
                tmux_option_escape(&provider.model_components.join(","))
            ));
        }
        if !provider.harness_components.is_empty() {
            lines.push_str(&format!(
                "\nset -g @powerkit_plugin_{plugin}_harness_components \"{}\"",
                tmux_option_escape(&provider.harness_components.join(","))
            ));
        }
    }
    lines
}

fn tmux_option_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn powerkit_plugin_list_arg(plugins: &[String]) -> String {
    plugins.join(",")
}

fn tmux_powerkit_status_formats(
    line1_left: &[String],
    line1_right: &[String],
    line2_left: &[String],
    line2_right: &[String],
) -> String {
    let line1_left_format = tmux_powerkit_line1_left_format(line1_left);
    format!(
        r#"# Powerkit renders these list-specific sections; tmux only supplies alignment.
set -g status 2
set -g 'status-format[0]' '{}#[nolist align=right range=right #{{E:status-right-style}}]#[push-default]#(~/.local/bin/aibox-powerkit-render-list right {})#[pop-default]#[norange default]'
set -g 'status-format[1]' '#[align=left]#(~/.local/bin/aibox-powerkit-render-list left {})#[align=right]#(~/.local/bin/aibox-powerkit-render-list right {})'"#,
        line1_left_format,
        powerkit_plugin_list_arg(line1_right),
        powerkit_plugin_list_arg(line2_left),
        powerkit_plugin_list_arg(line2_right),
    )
}

fn tmux_powerkit_line1_left_format(line1_left: &[String]) -> String {
    let mut format = String::from("#[align=left range=left #{E:status-left-style}]");
    for entry in line1_left {
        match entry.as_str() {
            "session" => format
                .push_str("#[push-default]#(~/.local/bin/aibox-powerkit-render-session)#[pop-default]#[norange default]"),
            "windows" => format.push_str(
                "#[list=on align=#{status-justify}]#[list=left-marker]<#[list=right-marker]>#[list=on]#{W:#[range=window|#{window_index} #{E:window-status-style}#{?#{&&:#{window_last_flag},#{!=:#{E:window-status-last-style},default}}, #{E:window-status-last-style},}#{?#{&&:#{window_bell_flag},#{!=:#{E:window-status-bell-style},default}}, #{E:window-status-bell-style},#{?#{&&:#{||:#{window_activity_flag},#{window_silence_flag}},#{!=:#{E:window-status-activity-style},default}}, #{E:window-status-activity-style},}}}]#[push-default]#{T:window-status-format}#[pop-default]#[norange default]#{?loop_last_flag,,#{window-status-separator}},#[range=window|#{window_index} list=focus #{?#{!=:#{E:window-status-current-style},default},#{E:window-status-current-style},#{E:window-status-style}}}#{?#{&&:#{window_last_flag},#{!=:#{E:window-status-last-style},default}}, #{E:window-status-last-style},}#{?#{&&:#{window_bell_flag},#{!=:#{E:window-status-bell-style},default}}, #{E:window-status-bell-style},#{?#{&&:#{||:#{window_activity_flag},#{window_silence_flag}},#{!=:#{E:window-status-activity-style},default}}, #{E:window-status-activity-style},}}}]#[push-default]#{T:window-status-current-format}#[pop-default]#[norange list=on default]#{?loop_last_flag,,#{window-status-separator}}}",
            ),
            _ => {}
        }
    }
    format.push_str("#{E:@_powerkit_left_edge_sep}");
    format
}

pub const POWERKIT_RENDER_LIST_SH: &str = r#"#!/usr/bin/env bash
set -uo pipefail

side="${1:-right}"
plugins="${2:-}"
[[ -z "$plugins" ]] && exit 0

POWERKIT_ROOT="${POWERKIT_ROOT:-/usr/local/share/aibox/tmux/plugins/tmux-powerkit}"
export POWERKIT_ROOT

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
export HOME="$(cd -- "${script_dir}/../.." && pwd)"
export XDG_CACHE_HOME="${HOME}/.cache"

if [[ ! -r "${POWERKIT_ROOT}/src/core/bootstrap.sh" ]]; then
    exit 0
fi

. "${POWERKIT_ROOT}/src/core/bootstrap.sh"
load_powerkit_theme
. "${POWERKIT_ROOT}/src/renderer/segment_builder.sh"

reset_all_cycle_caches
_batch_load_tmux_options
_TMUX_OPTIONS_CACHE["@powerkit_plugins"]="$plugins"

render_plugins "$side"
"#;

pub const POWERKIT_RENDER_SESSION_SH: &str = r#"#!/usr/bin/env bash
set -uo pipefail

POWERKIT_ROOT="${POWERKIT_ROOT:-/usr/local/share/aibox/tmux/plugins/tmux-powerkit}"
export POWERKIT_ROOT

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
export HOME="$(cd -- "${script_dir}/../.." && pwd)"
export XDG_CACHE_HOME="${HOME}/.cache"

if [[ ! -r "${POWERKIT_ROOT}/src/core/bootstrap.sh" ]]; then
    exit 0
fi

. "${POWERKIT_ROOT}/src/core/bootstrap.sh"
load_powerkit_theme
. "${POWERKIT_ROOT}/src/renderer/compositor.sh"

printf '%s' "$(_render_entity session left)"
printf '%s' "$(_build_edge_separator session end left)"
"#;

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
                r#"@powerkit_plugins "aibox_log,aibox_oom,aibox_proc,aibox_ai,aibox_mcp,aibox_mig,weather,uptime,datetime,git,github,kubernetes,terraform,cloud,hostname,externalip,ssh,netspeed,ping,cpu,loadavg,memory,swap,disk,gpu""#
            )
                && conf.contains(r#"@powerkit_bar_layout "double""#)
                && conf.contains(r#"@powerkit_status_order "session,plugins""#)
                && conf.contains(r#"set -g status-interval 10"#)
                && conf.contains(r#"@powerkit_status_interval "10""#)
                && conf.contains(r#"@powerkit_transparent "false""#)
                && conf.contains(r#"@powerkit_pane_border_status "top""#)
                && conf.contains(r##"@powerkit_pane_border_format "#{?client_prefix,PREFIX,NORMAL} #{pane_title} #{pane_current_command}""##)
                && conf.contains(r#"@powerkit_line1_right "aibox_log,aibox_oom,aibox_proc,aibox_ai,aibox_mcp,aibox_mig,weather,uptime,datetime""#)
                && conf.contains(r#"@powerkit_line2_left "git,github,kubernetes,terraform,cloud""#)
                && conf.contains(
                    r#"@powerkit_line2_right "hostname,externalip,ssh,netspeed,ping,cpu,loadavg,memory,swap,disk,gpu""#
                )
                && conf.contains(r#"@powerkit_plugin_aibox_log_metric "log""#)
                && conf.contains(r#"@powerkit_plugin_aibox_oom_metric "oom""#)
                && conf.contains(r#"@powerkit_plugin_aibox_proc_metric "proc""#)
                && conf.contains(r#"@powerkit_plugin_aibox_ai_metric "ai""#)
                && conf.contains(r#"@powerkit_plugin_aibox_mcp_metric "mcp""#)
                && conf.contains(r#"@powerkit_plugin_aibox_mig_metric "mig""#)
                && conf.contains(r#"@powerkit_plugin_aibox_log_cache_ttl "30""#)
                && conf.contains(r#"@powerkit_plugin_aibox_oom_cache_ttl "30""#)
                && conf.contains(r#"@powerkit_plugin_aibox_proc_cache_ttl "30""#)
                && conf.contains(r#"@powerkit_plugin_aibox_ai_cache_ttl "30""#)
                && conf.contains(r#"@powerkit_plugin_aibox_mcp_cache_ttl "30""#)
                && conf.contains(r#"@powerkit_plugin_aibox_mig_cache_ttl "30""#)
                && conf.contains(r#"@powerkit_plugin_netspeed_cache_ttl "10""#)
                && conf.contains(r#"@powerkit_plugin_kubernetes_cache_ttl "120""#)
                && conf.contains(r#"@powerkit_plugin_cloud_cache_ttl "120""#)
                && conf.contains(r#"@powerkit_plugin_aibox_log_label "LOG""#)
                && conf.contains(r#"@powerkit_plugin_aibox_oom_label "OOM""#)
                && conf.contains(r#"@powerkit_plugin_aibox_proc_label "PROC""#)
                && conf.contains(r#"@powerkit_plugin_aibox_ai_label "AI""#)
                && conf.contains(r#"@powerkit_plugin_aibox_mcp_label "MCP""#)
                && conf.contains(r#"@powerkit_plugin_aibox_mig_label "MIG""#),
            "generated persistent tmux config should carry bounded powerkit defaults:\n{conf}"
        );
        assert!(
            conf.contains(
                r#"bind-key -N "Show aibox/tmux key bindings" ? display-popup -w 80% -h 75% -E "tmux list-keys -N | less -R""#
            )
                && conf.contains(r#"bind-key -N "Select pane left" h select-pane -L"#)
                && conf.contains(
                    r#"bind-key -N "Open log pane (lnav)" L display-popup -E -w 90% -h 80% "aibox-log-viewer""#
                ),
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
        let mut config = crate::config::test_config();
        config.aibox.project_name = "source-project".to_string();
        config.customization.tmux.session_name = "configured-session".to_string();
        let conf = tmux_conf(&config);

        assert!(conf.contains("kill tmux session configured-session?"));
        assert!(!conf.contains("kill tmux session source-project?"));
        assert!(conf.contains(
            r#"@powerkit_plugins "aibox_log,aibox_oom,aibox_proc,aibox_ai,aibox_mcp,aibox_mig,weather,uptime,datetime"#
        ));
        assert!(conf.contains("tmux-powerkit.tmux"));
        assert!(
            conf.contains(
                "aibox-powerkit-render-list right aibox_log,aibox_oom,aibox_proc,aibox_ai,aibox_mcp,aibox_mig,weather,uptime,datetime"
            ) && conf.contains("aibox-powerkit-render-session"),
            "generated status formats must use the source-owned PowerKit render helpers:\n{conf}"
        );
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

        // Line 1 right: aibox_log → aibox_oom → aibox_proc → aibox_ai → aibox_mcp
        // → aibox_mig → weather → uptime → datetime
        // (DEC-20260508_2115-SilentFern, updated by the PowerKit alignment proof)
        assert!(
            conf.contains(
                r#"@powerkit_line1_right "aibox_log,aibox_oom,aibox_proc,aibox_ai,aibox_mcp,aibox_mig,weather,uptime,datetime""#
            ),
            "line1_right slot order must be: aibox_log,aibox_oom,aibox_proc,aibox_ai,aibox_mcp,aibox_mig,weather,uptime,datetime\n{conf}"
        );

        // Line 2 left: git → github → kubernetes → terraform → cloud
        // (DEC-20260508_2115-SilentFern)
        assert!(
            conf.contains(r#"@powerkit_line2_left "git,github,kubernetes,terraform,cloud""#),
            "line2_left slot order must be: git,github,kubernetes,terraform,cloud\n{conf}"
        );

        // Line 2 right: hostname → externalip → ssh → netspeed → ping → cpu
        // → loadavg → memory → swap → disk → gpu
        // (DEC-20260508_2115-SilentFern)
        assert!(
            conf.contains(r#"@powerkit_line2_right "hostname,externalip,ssh,netspeed,ping,cpu,loadavg,memory,swap,disk,gpu""#),
            "line2_right slot order must be: hostname,externalip,ssh,netspeed,ping,cpu,loadavg,memory,swap,disk,gpu\n{conf}"
        );

        // Full plugin list snapshot (all three line components concatenated)
        assert!(
            conf.contains(
                r#"@powerkit_plugins "aibox_log,aibox_oom,aibox_proc,aibox_ai,aibox_mcp,aibox_mig,weather,uptime,datetime,git,github,kubernetes,terraform,cloud,hostname,externalip,ssh,netspeed,ping,cpu,loadavg,memory,swap,disk,gpu""#
            ),
            "full plugin list snapshot mismatch — slot order is fixed per DEC-20260508_2115-SilentFern\n{conf}"
        );

        assert!(
            conf.contains("set -g status 2")
                && conf.contains("aibox-powerkit-render-session")
                && conf.contains("aibox-powerkit-render-list right aibox_log,aibox_oom,aibox_proc,aibox_ai,aibox_mcp,aibox_mig,weather,uptime,datetime")
                && conf.contains("aibox-powerkit-render-list left git,github,kubernetes,terraform,cloud")
                && conf.contains("aibox-powerkit-render-list right hostname,externalip,ssh,netspeed,ping,cpu,loadavg,memory,swap,disk,gpu"),
            "generated status formats must keep the two-row PowerKit-aligned proof layout\n{conf}"
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

    #[test]
    fn tmux_status_model_provider_segments_are_opt_in_and_configured() {
        let mut config = crate::config::test_config();
        config.customization.tmux.status.model_providers.enabled = true;
        config
            .customization
            .tmux
            .status
            .model_providers
            .providers
            .truncate(2);
        let conf = tmux_conf(&config);

        assert!(
            conf.contains(r#"@powerkit_line1_right "aibox_log,aibox_oom,aibox_proc,aibox_ai,aibox_mcp,aibox_mig,modelstatus_openai,modelstatus_anthropic,weather,uptime,datetime""#),
            "model provider segments should render after aibox health metrics and before general segments:\n{conf}"
        );
        assert!(
            conf.contains(r#"@powerkit_plugin_modelstatus_openai_label "OAI""#)
                && conf.contains(r#"@powerkit_plugin_modelstatus_openai_checks "overall,models,harness""#)
                && conf.contains(r#"@powerkit_plugin_modelstatus_openai_status_url "https://status.openai.com/api/v2/summary.json""#)
                && conf.contains(r#"@powerkit_plugin_modelstatus_openai_model_components "Responses,Chat Completions,Embeddings,Realtime,Images""#)
                && conf.contains(r#"@powerkit_plugin_modelstatus_openai_harness_components "CLI,Codex API,Codex Web""#)
                && conf.contains(r#"@powerkit_plugin_modelstatus_anthropic_label "ANT""#)
                && conf.contains(r#"@powerkit_plugin_modelstatus_anthropic_model_components "Claude API""#)
                && conf.contains(r#"@powerkit_plugin_modelstatus_anthropic_harness_components "Claude Code""#),
            "model provider plugin options should be generated for each provider:\n{conf}"
        );
    }

    #[test]
    fn tmux_status_labels_are_configurable() {
        let mut config = crate::config::test_config();
        config.customization.tmux.status.labels.aibox_log = "L".to_string();
        config.customization.tmux.status.labels.aibox_oom = "O".to_string();
        config.customization.tmux.status.labels.kubernetes = "K8S".to_string();
        config.customization.tmux.status.labels.cloud_aws = "AWS".to_string();
        config.customization.tmux.status.labels.netspeed_download = "DN".to_string();
        config.customization.tmux.status.labels.netspeed_upload = "UP".to_string();
        let conf = tmux_conf(&config);

        assert!(
            conf.contains(r#"@powerkit_plugin_aibox_log_label "L""#)
                && conf.contains(r#"@powerkit_plugin_aibox_oom_label "O""#)
                && conf.contains(r#"@powerkit_plugin_kubernetes_icon "K8S""#)
                && conf.contains(r#"@powerkit_plugin_cloud_icon_aws "AWS""#)
                && conf.contains(r#"@powerkit_plugin_netspeed_icon_download "DN""#)
                && conf.contains(r#"@powerkit_plugin_netspeed_icon_upload "UP""#),
            "status labels should be emitted as PowerKit plugin options:\n{conf}"
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

    #[test]
    fn tmux_status_element_switches_control_powerkit_plugins() {
        fn configured_powerkit_plugins(conf: &str) -> Vec<&str> {
            let line = conf
                .lines()
                .find(|line| line.starts_with("set -g @powerkit_plugins "))
                .expect("extended status must configure @powerkit_plugins");
            line.split('"')
                .nth(1)
                .expect("@powerkit_plugins value must be quoted")
                .split(',')
                .filter(|plugin| !plugin.is_empty())
                .collect()
        }

        fn assert_switch_removes_plugin(
            plugin: &str,
            disable: impl FnOnce(&mut crate::config::TmuxStatusElementsSection),
        ) {
            let mut config = crate::config::test_config();
            disable(&mut config.customization.tmux.status.elements);
            let conf = tmux_conf(&config);
            let plugins = configured_powerkit_plugins(&conf);

            assert!(
                !plugins.contains(&plugin),
                "disabling the switch must remove {plugin} from @powerkit_plugins: {plugins:?}"
            );
        }

        assert_switch_removes_plugin("hostname", |elements| elements.hostname = false);
        assert_switch_removes_plugin("externalip", |elements| elements.external_ip = false);
        assert_switch_removes_plugin("ssh", |elements| elements.ssh = false);
        assert_switch_removes_plugin("uptime", |elements| elements.uptime = false);
        assert_switch_removes_plugin("weather", |elements| elements.weather = false);
        assert_switch_removes_plugin("datetime", |elements| elements.datetime = false);
        assert_switch_removes_plugin("git", |elements| elements.git = false);
        assert_switch_removes_plugin("github", |elements| elements.github = false);
        assert_switch_removes_plugin("kubernetes", |elements| elements.kubernetes = false);
        assert_switch_removes_plugin("terraform", |elements| elements.terraform = false);
        assert_switch_removes_plugin("cloud", |elements| elements.cloud = false);
        assert_switch_removes_plugin("cloudstatus", |elements| {
            elements.cloudstatus = true;
            elements.cloudstatus = false;
        });
        assert_switch_removes_plugin("cpu", |elements| elements.cpu = false);
        assert_switch_removes_plugin("loadavg", |elements| elements.loadavg = false);
        assert_switch_removes_plugin("memory", |elements| elements.mem = false);
        assert_switch_removes_plugin("swap", |elements| elements.swap = false);
        assert_switch_removes_plugin("disk", |elements| elements.disk = false);
        assert_switch_removes_plugin("gpu", |elements| elements.gpu = false);
        assert_switch_removes_plugin("netspeed", |elements| elements.netspeed = false);
        assert_switch_removes_plugin("ping", |elements| elements.ping = false);

        for plugin in [
            "aibox_log",
            "aibox_oom",
            "aibox_proc",
            "aibox_ai",
            "aibox_mcp",
            "aibox_mig",
        ] {
            assert_switch_removes_plugin(plugin, |elements| elements.aibox = false);
        }
        assert_switch_removes_plugin("aibox_log", |elements| elements.aibox_metrics.log = false);
        assert_switch_removes_plugin("aibox_oom", |elements| elements.aibox_metrics.oom = false);
        assert_switch_removes_plugin("aibox_proc", |elements| elements.aibox_metrics.proc = false);
        assert_switch_removes_plugin("aibox_ai", |elements| elements.aibox_metrics.ai = false);
        assert_switch_removes_plugin("aibox_mcp", |elements| elements.aibox_metrics.mcp = false);
        assert_switch_removes_plugin("aibox_mig", |elements| elements.aibox_metrics.mig = false);
    }

    #[test]
    fn tmux_status_layout_lists_control_order_and_visibility() {
        let mut config = crate::config::test_config();
        config.customization.tmux.status.layout.line1_left = Some(vec![]);
        config.customization.tmux.status.layout.line1_right =
            Some(vec!["datetime".to_string(), "weather".to_string()]);
        config.customization.tmux.status.layout.line2_left =
            Some(vec!["cloudstatus".to_string(), "git".to_string()]);
        config.customization.tmux.status.layout.line2_right =
            Some(vec!["memory".to_string(), "cpu".to_string()]);

        let conf = tmux_conf(&config);

        assert!(
            conf.contains(r#"@powerkit_line1_right "datetime,weather""#),
            "line1-right list order must be honored:\n{conf}"
        );
        assert!(
            conf.contains(r#"@powerkit_line2_left "cloudstatus,git""#),
            "line2-left list order must be honored:\n{conf}"
        );
        assert!(
            conf.contains(r#"@powerkit_line2_right "memory,cpu""#),
            "line2-right list order must be honored:\n{conf}"
        );
        let line1_format = conf
            .lines()
            .find(|line| line.starts_with("set -g 'status-format[0]'"))
            .expect("extended status must set status-format[0]");
        assert!(
            !line1_format.contains("aibox-powerkit-render-session")
                && !line1_format.contains("#{W:"),
            "empty line1-left must hide session and tmux window entries:\n{line1_format}"
        );
        assert!(
            conf.contains(r#"@powerkit_plugins "datetime,weather,cloudstatus,git,memory,cpu""#),
            "global PowerKit plugin order must follow the explicit row lists:\n{conf}"
        );
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

    /// v0.25.7: only the two stable tool-window bindings (g and s) survive the
    /// speculative-addon revert (SnappySky).  K/B/D were removed with
    /// monitoring.yaml.
    #[test]
    fn tmux_conf_has_tool_window_keybindings() {
        let config = crate::config::test_config();
        let conf = tmux_conf(&config);

        assert!(
            conf.contains(r#"bind-key -N "Switch to lazygit window" g find-window -Z 'lazygit'"#),
            "leader g must jump to lazygit window:\n{conf}"
        );
        assert!(
            conf.contains(r#"bind-key -N "Switch to shell window" s find-window -Z 'shell'"#),
            "leader s must jump to shell window:\n{conf}"
        );
    }
}
