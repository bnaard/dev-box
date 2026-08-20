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
set -g xterm-keys on
set -s extended-keys on
set -g allow-passthrough on
set -g set-clipboard external
set -g mode-keys vi
set -g default-terminal "tmux-256color"
set -ga terminal-features ",*:extkeys,wezterm:RGB,clipboard,xterm-256color:RGB,clipboard,tmux-256color:RGB"
set -ga terminal-overrides ",xterm-256color:Tc,tmux-256color:Tc"
set -g status-interval AIBOX_TMUX_STATUS_INTERVAL
set -g prefix AIBOX_TMUX_PREFIX
unbind C-b
bind AIBOX_TMUX_PREFIX send-prefix

# Pane navigation mirrors the old aibox leader muscle memory.
bind-key -N "Show aibox/tmux key bindings" ? display-popup -w 110 -h 90% -E "bash \"$HOME/.local/bin/aibox-tmux-cheatsheet\""
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
bind-key -N "Open log pane (lnav)" o display-popup -E -w 90% -h 80% "aibox-log-viewer"

# BR-TOOLS-AS-WINDOWS (BACK-20260510_0726-GrandDaisy, v0.25.7): one-letter
# prefix shortcuts to jump directly to named tool/harness windows.
# find-window -Z focuses the target window; silently no-ops when absent.
bind-key -N "Switch to lazygit window" g find-window -Z 'lazygit'
bind-key -N "Switch to shell window" s find-window -Z 'shell'

# Host paste sends line-feed bytes for newlines, and line-feed is C-j.
# Binding global C-j makes pasted multiline text corrupt in panes. Keep smart
# down navigation on C-Down and prefix j instead.
unbind-key -q -n C-j
unbind-key -q -T copy-mode-vi C-j
set -g @vim_navigator_mapping_down "C-Down"

AIBOX_TMUX_LAYOUT_SWITCH_BINDING
AIBOX_TMUX_THEME_SWITCH_BINDING

set -g status AIBOX_TMUX_STATUS
AIBOX_TMUX_TITLE_BLOCK
set -g status-style "bg=AIBOX_TMUX_BG,fg=AIBOX_TMUX_FG"
set -g window-status-current-style "bg=AIBOX_TMUX_ACCENT,fg=AIBOX_TMUX_BGAIBOX_TMUX_ACTIVE_ATTR"
set -g window-status-format " #I:#W "
set -g window-status-current-format " #I:#W "
# Inactive panes are dimmed a touch (bg+fg both biased ~12% toward
# muted) so the focused pane stands out without making non-focus
# content hard to read. Tweak via `themes::dim_inactive_pane_colors`.
set -g window-style "bg=AIBOX_TMUX_DIM_BG,fg=AIBOX_TMUX_DIM_FG"
set -g window-active-style "bg=AIBOX_TMUX_BG,fg=AIBOX_TMUX_FG"
set -g pane-border-style "fg=AIBOX_TMUX_MUTED,bg=AIBOX_TMUX_BG"
set -g pane-active-border-style "fg=AIBOX_TMUX_ACCENT,bg=AIBOX_TMUX_BG"
set -g message-style "bg=AIBOX_TMUX_ACCENT,fg=AIBOX_TMUX_BG"
set -g message-command-style "bg=AIBOX_TMUX_ACCENT,fg=AIBOX_TMUX_BG"
set -g mode-style "bg=AIBOX_TMUX_ACCENT,fg=AIBOX_TMUX_BG"
set -g clock-mode-colour "AIBOX_TMUX_ACCENT"
set -g popup-style "bg=AIBOX_TMUX_BG,fg=AIBOX_TMUX_FG"
set -g popup-border-style "fg=AIBOX_TMUX_ACCENT,bg=AIBOX_TMUX_BG"
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
    let (bg, fg, accent, muted, _dim_fg, _active_title_fg) =
        crate::themes::terminal_surface_colors(&theme);
    let (dim_bg, dim_fg) = crate::themes::dim_inactive_pane_colors(&theme);
    let status = match config.customization.tmux.status.mode {
        TmuxStatusMode::Extended | TmuxStatusMode::Plain => "on",
        TmuxStatusMode::Disabled => "off",
    };

    let status_right = match config.customization.tmux.status.mode {
        TmuxStatusMode::Extended => "#(aibox-status --once 2>/dev/null || true) %H:%M",
        TmuxStatusMode::Plain | TmuxStatusMode::Disabled => "%H:%M",
    };
    let (powerkit_block, powerkit_plugin, powerkit_formats) = tmux_powerkit_settings(config);
    let title_block = tmux_title_settings(config);

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
        .replace("AIBOX_TMUX_TITLE_BLOCK", &title_block)
        .replace("AIBOX_TMUX_POWERKIT_BLOCK", &powerkit_block)
        .replace("AIBOX_TMUX_POWERKIT_PLUGIN", &powerkit_plugin)
        .replace("AIBOX_TMUX_POWERKIT_FORMATS", &powerkit_formats)
        .replace(
            "AIBOX_TMUX_LAYOUT_SWITCH_BINDING",
            &layout_switch_binding(config),
        )
        .replace(
            "AIBOX_TMUX_THEME_SWITCH_BINDING",
            &theme_switch_binding(config),
        )
        .replace("AIBOX_TMUX_DIM_BG", &dim_bg)
        .replace("AIBOX_TMUX_DIM_FG", &dim_fg)
        .replace("AIBOX_TMUX_ACTIVE_ATTR", &{
            let attr = crate::themes::tmux_role_attributes(
                config.customization.emphasis,
                "active_foreground",
                Some(&config.customization.emphasis_overrides),
            );
            if attr.is_empty() {
                attr
            } else {
                format!(",{attr}")
            }
        })
        .replace("AIBOX_TMUX_BG", bg)
        .replace("AIBOX_TMUX_FG", fg)
        .replace("AIBOX_TMUX_ACCENT", accent)
        .replace("AIBOX_TMUX_MUTED", muted);

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

/// Render the tmux-owned terminal title. Runtime helpers update the
/// `@aibox_attention_*` user options; no task/message text is embedded here.
/// `elapsed` is intentionally a transition snapshot supplied by the helper,
/// rather than a continuously-updating shell command in tmux's title path.
fn tmux_title_settings(config: &AiboxConfig) -> String {
    let title = &config.customization.tmux.title;
    let mut lines = vec![
        format!(
            "set -g set-titles {}",
            if title.enabled { "on" } else { "off" }
        ),
        format!(
            "set -g @aibox_title_project \"{}\"",
            tmux_option_escape(&config.aibox.project_name)
        ),
        format!(
            "set -g @aibox_title_message_max_length \"{}\"",
            title.message_max_length
        ),
        format!(
            "set -g @aibox_title_repository_style \"{}\"",
            tmux_option_escape(&title.repository_style)
        ),
        format!(
            "set -g @aibox_title_agent_style \"{}\"",
            tmux_option_escape(&title.agent_style)
        ),
        format!(
            "set -g @aibox_done_ttl_seconds \"{}\"",
            title.done_ttl_seconds
        ),
        format!(
            "set -g @aibox_notifications_enabled \"{}\"",
            if config.customization.tmux.notifications.enabled {
                1
            } else {
                0
            }
        ),
        format!(
            "set -g @aibox_notifications_protocol \"{}\"",
            tmux_option_escape(&config.customization.tmux.notifications.protocol)
        ),
        format!(
            "set -g @aibox_notifications_include_message \"{}\"",
            if config.customization.tmux.notifications.include_message {
                1
            } else {
                0
            }
        ),
        format!(
            "set -g @aibox_notification_states \"{}\"",
            config.customization.tmux.notifications.states.join(",")
        ),
    ];
    if title.enabled {
        let mut rendered = String::new();
        let mut rest = title.format.as_str();
        while let Some(open) = rest.find('{') {
            rendered.push_str(&tmux_title_literal_escape(&rest[..open]));
            let after_open = &rest[open + 1..];
            let Some(close) = after_open.find('}') else {
                // Validation reports this to users; keep generation non-panicking
                // for programmatically assembled configs.
                rendered.push_str(&tmux_title_literal_escape(after_open));
                break;
            };
            rendered.push_str(&title_placeholder_expression(
                &after_open[..close],
                &title.directory_style,
            ));
            rest = &after_open[close + 1..];
        }
        rendered.push_str(&tmux_title_literal_escape(rest));
        let mut title_expression = format!("#{{={}:{}", title.max_length, rendered);
        title_expression.push('}');
        lines.insert(
            1,
            format!("set -g set-titles-string \"{title_expression}\""),
        );
    }
    if title.enabled || config.customization.tmux.notifications.enabled {
        // Refresh aggregate attention state when a source pane exits. Append
        // rather than replace so a user's existing hook remains authoritative.
        lines.push(
            "set-hook -g pane-died[90] 'run-shell -b \"aibox-agent-signal refresh --window \\\"#{window_id}\\\"\"'"
                .to_string(),
        );
    }
    for (state, symbol) in [
        ("working", &title.states.working),
        ("question", &title.states.question),
        ("done", &title.states.done),
        ("error", &title.states.error),
        ("idle", &title.states.idle),
    ] {
        lines.push(format!(
            "set -g @aibox_title_state_{state} \"{}\"",
            tmux_option_escape(symbol)
        ));
    }
    lines.join("\n")
}

fn tmux_title_literal_escape(value: &str) -> String {
    // `#` starts a tmux format expansion. Doubling it makes user-authored
    // punctuation literal while backslash/quote escaping protects the option.
    tmux_option_escape(value).replace('#', "##")
}

fn title_placeholder_expression(placeholder: &str, directory_style: &str) -> String {
    match placeholder {
        "state_symbol" => "#{@aibox_attention_symbol}".to_string(),
        "state" => "#{@aibox_attention_state}".to_string(),
        // Project is an option rather than a raw config string so even a
        // user-chosen project name cannot become tmux syntax.
        "project" => "#{@aibox_title_project}".to_string(),
        "session" => "#S".to_string(),
        "window" => "#W".to_string(),
        "window_index" => "#I".to_string(),
        "pane" => "#P".to_string(),
        "directory" => match directory_style {
            "full" => "#{pane_current_path}".to_string(),
            "abbreviated" => "#{s|^/home/[^/]*/|~/|:pane_current_path}".to_string(),
            // tmux's basename modifier is stable across tmux 3.x versions.
            _ => "#{b:pane_current_path}".to_string(),
        },
        "directory_path" => "#{pane_current_path}".to_string(),
        "repository" => "#{@aibox_attention_repository}".to_string(),
        "branch" => "#{@aibox_attention_branch}".to_string(),
        "harness" => "#{@aibox_attention_harness}".to_string(),
        "agent" => "#{@aibox_attention_agent}".to_string(),
        "agent_suffix" => "#{?#{@aibox_attention_harness}, — #{?#{@aibox_attention_agent},#{@aibox_attention_agent}@,}#{@aibox_attention_harness},}".to_string(),
        "task" => "#{@aibox_attention_task}".to_string(),
        "message" => "#{@aibox_attention_message}".to_string(),
        "elapsed" => "#{@aibox_attention_elapsed}".to_string(),
        // Validation rejects this; generation remains safe if called directly.
        _ => String::new(),
    }
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
    // forge is the default git-aware segment (auto-detects GitHub/GitLab/Gitea/…).
    // git and github remain in the order table so explicit layout overrides work,
    // but their elements defaults are false — forge covers them.
    ("forge", "forge"),
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

struct TmuxSurfaceColors {
    bg: String,
    active_title_fg: String,
    dim_fg: String,
    accent: String,
    muted: String,
}

fn tmux_surface_colors(theme: &crate::config::Theme) -> TmuxSurfaceColors {
    let (bg, _fg, _accent, _muted, dim_fg, active_title_fg) =
        crate::themes::terminal_surface_colors(theme);
    let (border_active, border_inactive) = crate::themes::terminal_border_colors(theme);
    TmuxSurfaceColors {
        bg: bg.to_string(),
        active_title_fg,
        dim_fg,
        accent: border_active.to_string(),
        muted: border_inactive.to_string(),
    }
}

pub(crate) fn resolved_tmux_status_layout(config: &AiboxConfig) -> ResolvedTmuxStatusLayout {
    let elements = &config.customization.tmux.status.elements;
    let layout = &config.customization.tmux.status.layout;

    let line2_left: Vec<String> = layout.line2_left.clone().unwrap_or_else(|| {
        [
            (elements.forge, LINE2_LEFT_ORDER[0].1),
            (elements.git, LINE2_LEFT_ORDER[1].1),
            (elements.github, LINE2_LEFT_ORDER[2].1),
            (elements.kubernetes, LINE2_LEFT_ORDER[3].1),
            (elements.terraform, LINE2_LEFT_ORDER[4].1),
            (elements.cloud, LINE2_LEFT_ORDER[5].1),
            (elements.cloudstatus, LINE2_LEFT_ORDER[6].1),
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
            (
                config.processkit_enabled() && metrics.mcp,
                LINE1_RIGHT_AIBOX_METRICS_ORDER[4].1,
            ),
            (
                config.processkit_enabled() && metrics.mig,
                LINE1_RIGHT_AIBOX_METRICS_ORDER[5].1,
            ),
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
    let forge_option_lines = forge_option_lines(config, &line1_right, &line2_left, &line2_right);
    let status_label_option_lines =
        status_label_option_lines(config, &line1_right, &line2_left, &line2_right);
    let model_provider_option_lines = model_provider_option_lines(config, &line1_right);

    let mut plugin_order = Vec::new();
    plugin_order.extend(line1_right.iter().map(String::as_str));
    plugin_order.extend(line2_left.iter().map(String::as_str));
    plugin_order.extend(line2_right.iter().map(String::as_str));

    let resolved_theme = config.customization.resolved_theme();
    let surface = tmux_surface_colors(&resolved_theme);
    let separators = &config.customization.tmux.status.separators;
    // Drive PowerKit from the aibox-generated palette file so chevrons land on
    // colors the aibox surface actually uses.
    let powerkit_block = format!(
        r##"# Powerkit status. Theme: {} (custom palette)
set -g @powerkit_plugins "{}"
set -g @powerkit_bar_layout "double"
set -g @powerkit_status_order "session,plugins"
set -g @powerkit_theme "custom"
set -g @powerkit_custom_theme_path "~/.config/tmux/aibox-powerkit-theme.sh"
set -g @powerkit_separator_style "{}"
set -g @powerkit_edge_separator_style "{}"
set -g @powerkit_elements_spacing "{}"
set -g @powerkit_status_interval "{}"
set -g @powerkit_transparent "false"
set -g @powerkit_pane_border_status "top"
set -g @powerkit_pane_border_unified "false"
set -g @powerkit_active_pane_border_color "{}"
set -g @powerkit_inactive_pane_border_color "{}"
set -g @powerkit_pane_border_status_bg "{}"
set -g @powerkit_pane_scrollbars_style_fg "{}"
set -g @powerkit_pane_scrollbars_style_bg "{}"
set -g @powerkit_pane_border_format "#{{?client_prefix,PREFIX,NORMAL}} #{{pane_title}} #{{pane_current_command}}"
set -g @powerkit_line1_right "{}"
set -g @powerkit_line2_left "{}"
set -g @powerkit_line2_right "{}"
set -g @powerkit_plugin_netspeed_speed_width "7"{}{}{}{}{}"##,
        resolved_theme,
        plugin_order.join(","),
        separators.style,
        separators.edge_style,
        separators.elements_spacing,
        refresh.interval_seconds,
        surface.accent,
        surface.muted,
        surface.bg,
        surface.accent,
        surface.muted,
        line1_right.join(","),
        line2_left.join(","),
        line2_right.join(","),
        metric_option_lines,
        refresh_option_lines,
        forge_option_lines,
        status_label_option_lines,
        model_provider_option_lines
    );
    let powerkit_plugin = "if-shell '[ -f /usr/local/share/aibox/tmux/plugins/tmux-powerkit/tmux-powerkit.tmux ]' 'run-shell \"/usr/local/share/aibox/tmux/plugins/tmux-powerkit/tmux-powerkit.tmux; tmux source-file ~/.config/tmux/aibox-powerkit-overrides.tmux\"'".to_string();
    let powerkit_formats = tmux_powerkit_post_render_overrides(
        &line1_left,
        &line1_right,
        &line2_left,
        &line2_right,
        &surface,
        config.customization.emphasis,
        &config.customization.emphasis_overrides,
    );
    (powerkit_block, powerkit_plugin, powerkit_formats)
}

fn forge_option_lines(
    config: &AiboxConfig,
    line1_right: &[String],
    line2_left: &[String],
    line2_right: &[String],
) -> String {
    let configured = line1_right.iter().chain(line2_left).chain(line2_right);
    if !configured.into_iter().any(|item| item == "forge") {
        return String::new();
    }
    format!(
        "\nset -g @powerkit_plugin_forge_github_hosts \"{}\"",
        config
            .customization
            .tmux
            .status
            .forge
            .github_hosts
            .join(" ")
    )
}

pub fn tmux_powerkit_overrides(config: &AiboxConfig) -> String {
    if config.customization.tmux.status.mode != TmuxStatusMode::Extended {
        return String::new();
    }

    let resolved_layout = resolved_tmux_status_layout(config);
    let resolved_theme = config.customization.resolved_theme();
    let surface = tmux_surface_colors(&resolved_theme);
    tmux_powerkit_post_render_overrides(
        &resolved_layout.line1_left,
        &resolved_layout.line1_right,
        &resolved_layout.line2_left,
        &resolved_layout.line2_right,
        &surface,
        config.customization.emphasis,
        &config.customization.emphasis_overrides,
    )
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
        ("forge", refresh.github_cache_ttl_seconds),
        ("github", refresh.github_cache_ttl_seconds),
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
    let model_providers = &config.customization.tmux.status.model_providers;
    let mut lines = String::new();
    for provider in &model_providers.providers {
        let plugin = modelstatus_plugin_name(&provider.provider);
        let plugin_configured = line1_right.iter().any(|configured| configured == &plugin);
        if !plugin_configured {
            continue;
        }
        let explicitly_configured = config
            .customization
            .tmux
            .status
            .layout
            .line1_right
            .as_ref()
            .is_some_and(|items| items.iter().any(|configured| configured == &plugin));
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
        // Glyph for the OK state: keep the legacy ✓ behind the new
        // `show_glyph_when_ok` flag, but accept the older `show_ok` field
        // for backward compatibility. The chevron color now carries the
        // ok/warning/error signal via the custom PowerKit theme.
        let show_glyph_when_ok = model_providers.show_glyph_when_ok || model_providers.show_ok;
        lines.push_str(&format!(
            "\nset -g @powerkit_plugin_{plugin}_show_ok \"{}\"",
            show_glyph_when_ok
        ));
        lines.push_str(&format!(
            "\nset -g @powerkit_plugin_{plugin}_show_glyph_when_ok \"{}\"",
            show_glyph_when_ok
        ));
        lines.push_str(&format!(
            "\nset -g @powerkit_plugin_{plugin}_force_render \"{}\"",
            explicitly_configured
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

        // ── Phase 1 — local agent count ──────────────────────────────────
        lines.push_str(&format!(
            "\nset -g @powerkit_plugin_{plugin}_show_agent_count \"{}\"",
            provider.show_agent_count
        ));
        let agent_binaries = if provider.agent_binaries.is_empty() {
            crate::config::default_agent_binaries_for(&provider.provider)
        } else {
            provider.agent_binaries.clone()
        };
        if !agent_binaries.is_empty() {
            lines.push_str(&format!(
                "\nset -g @powerkit_plugin_{plugin}_agent_binaries \"{}\"",
                tmux_option_escape(&agent_binaries.join(","))
            ));
        }

        // ── Phase 2 — rate-limit quota polling ──────────────────────────
        lines.push_str(&format!(
            "\nset -g @powerkit_plugin_{plugin}_show_quota \"{}\"",
            provider.show_quota
        ));
        lines.push_str(&format!(
            "\nset -g @powerkit_plugin_{plugin}_quota_window \"{}\"",
            tmux_option_escape(&provider.quota_window)
        ));
        if let Some(env) = &provider.quota_api_key_env {
            lines.push_str(&format!(
                "\nset -g @powerkit_plugin_{plugin}_quota_api_key_env \"{}\"",
                tmux_option_escape(env)
            ));
        }

        // ── Phase 3 — admin usage rollup (gated by section-level ack) ────
        let admin_usage_active = provider.show_admin_usage && model_providers.admin_usage_enabled;
        lines.push_str(&format!(
            "\nset -g @powerkit_plugin_{plugin}_show_admin_usage \"{}\"",
            admin_usage_active
        ));
        if let Some(env) = &provider.admin_api_key_env {
            lines.push_str(&format!(
                "\nset -g @powerkit_plugin_{plugin}_admin_api_key_env \"{}\"",
                tmux_option_escape(env)
            ));
        }
    }
    lines
}

fn tmux_option_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Render the `bind-key` line(s) for the live layout chooser. Disabled when
/// `customization.tmux.layout_switch.enabled = false`.
///
/// Always destructive — the chooser rebuilds windows in the attached
/// session. When `confirm = true` (default) the binding routes through
/// `aibox-tmux-confirm-and-switch`, which display-menus the impacted apps
/// before invoking `aibox-tmux-switch-layout`.
fn layout_switch_binding(config: &AiboxConfig) -> String {
    let ls = &config.customization.tmux.layout_switch;
    if !ls.enabled {
        return String::new();
    }
    let key = tmux_option_escape(&ls.prefix_key);
    let dispatcher = if ls.confirm {
        "aibox-tmux-confirm-and-switch"
    } else {
        "aibox-tmux-switch-layout"
    };

    // The four built-in layouts. Each gets a one-letter shortcut + a
    // visible label so the menu doubles as documentation.
    let layouts: &[(&str, &str, &str)] = &[
        ("dev", "d", "Dev (work=files|harness)"),
        ("ai", "a", "AI (work=files|harness; ai windows)"),
        ("focus", "f", "Focus (one window per harness)"),
        ("cowork", "w", "Cowork (files|shell + ai windows)"),
    ];

    match ls.style.as_str() {
        "table" => {
            let mut out =
                format!("\nbind-key -N \"Choose layout\" {key} switch-client -T aibox_layouts\n");
            for (name, k, _desc) in layouts {
                out.push_str(&format!(
                    "bind-key -T aibox_layouts {k} run-shell '{dispatcher} {name}'\n"
                ));
            }
            out.push_str(
                "bind-key -T aibox_layouts Escape display-message \"layout switch cancelled\"\n",
            );
            out
        }
        _ => {
            // "menu" (default): display-menu with one entry per layout.
            let mut out = format!(
                "\nbind-key -N \"Choose layout\" {key} display-menu -T \"#[align=centre]Layout\" -x C -y C"
            );
            for (name, k, desc) in layouts {
                out.push_str(&format!(
                    " \\\n  \"{desc}\" {k} \"run-shell '{dispatcher} {name}'\""
                ));
            }
            out.push_str(" \\\n  \"\" \\\n  \"Cancel\" q \"\"\n");
            out
        }
    }
}

/// Render the `bind-key` line for the live theme chooser. Same enable
/// gate, populated dynamically from `customization.tmux.theme_switch.themes`.
fn theme_switch_binding(config: &AiboxConfig) -> String {
    let ts = &config.customization.tmux.theme_switch;
    if !ts.enabled {
        return String::new();
    }
    let key = tmux_option_escape(&ts.prefix_key);
    let confirm_env = if ts.confirm_restart_tuis {
        "AIBOX_THEME_CONFIRM_RESTART_TUIS=true"
    } else {
        "AIBOX_THEME_CONFIRM_RESTART_TUIS=false"
    };

    let mut out = format!(
        "\nbind-key -N \"Choose theme\" {key} display-menu -T \"#[align=centre]Theme\" -x C -y C"
    );
    // First letter of each theme as the shortcut; resolve collisions by
    // suffixing a digit. The map is built deterministically.
    let mut used: std::collections::HashSet<char> = std::collections::HashSet::new();
    for theme in &ts.themes {
        let letter = pick_unique_shortcut(theme, &mut used);
        let label = theme.replace('-', " ");
        out.push_str(&format!(
            " \\\n  \"{label}\" {letter} \"run-shell 'aibox theme --theme {theme} && aibox-tmux-refresh-theme'\""
        ));
    }
    if ts.include_mode_toggle {
        let letter = pick_unique_shortcut("light_dark_toggle", &mut used);
        out.push_str(&format!(
            " \\\n  \"\" \\\n  \"Toggle light/dark\" {letter} \"run-shell 'aibox theme --mode auto && aibox-tmux-refresh-theme'\""
        ));
    }
    // Heavy tier — pre-execution confirmation respected via env var.
    out.push_str(&format!(
        " \\\n  \"\" \\\n  \"Heavy: restart TUIs (kill+respawn lazygit/lnav/AI)\" R \"run-shell '{confirm_env} aibox-tmux-refresh-theme --restart-tuis'\""
    ));
    out.push_str(" \\\n  \"\" \\\n  \"Cancel\" q \"\"\n");
    out
}

fn pick_unique_shortcut(label: &str, used: &mut std::collections::HashSet<char>) -> char {
    for ch in label.chars().filter(|c| c.is_ascii_alphabetic()) {
        let lower = ch.to_ascii_lowercase();
        if !used.contains(&lower) {
            used.insert(lower);
            return lower;
        }
    }
    // Fallback — uppercase the first letter to avoid collision; final
    // resort is '?' which tmux accepts but the user can override.
    for ch in label.chars().filter(|c| c.is_ascii_alphabetic()) {
        let upper = ch.to_ascii_uppercase();
        if !used.contains(&upper) {
            used.insert(upper);
            return upper;
        }
    }
    '?'
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

fn tmux_powerkit_post_render_overrides(
    line1_left: &[String],
    line1_right: &[String],
    line2_left: &[String],
    line2_right: &[String],
    surface: &TmuxSurfaceColors,
    emphasis: crate::config::ThemeEmphasis,
    emphasis_overrides: &std::collections::BTreeMap<String, String>,
) -> String {
    let status_formats =
        tmux_powerkit_status_formats(line1_left, line1_right, line2_left, line2_right);
    let active_attr = crate::themes::tmux_role_attributes(
        emphasis,
        "pane_active_foreground",
        Some(emphasis_overrides),
    );
    let active_attr = if active_attr.is_empty() {
        String::new()
    } else {
        format!("#[{active_attr}]")
    };
    let inactive_attr = crate::themes::tmux_role_attributes(
        emphasis,
        "pane_inactive_foreground",
        Some(emphasis_overrides),
    );
    let inactive_attr = if inactive_attr.is_empty() {
        String::new()
    } else {
        format!("#[{inactive_attr}]")
    };
    let active_style = format!("#[fg={}]{}", surface.active_title_fg, active_attr);
    let inactive_style = format!("#[fg={}]{}", surface.dim_fg, inactive_attr);
    format!(
        r##"{}

# aibox post-PowerKit overrides.
# PowerKit's renderer owns status-format and pane styles when it loads. Keep
# the aibox two-row status shape and pane surfaces authoritative after that
# render pass.
set -g pane-border-style "fg={},bg={}"
set -g pane-active-border-style "fg={},bg={}"
set -g pane-border-format "#[bg={}]#{{?pane_active,{},{} }} #{{?client_prefix,PREFIX,NORMAL}} #{{pane_title}} #{{pane_current_command}} #[bg={},fg={}] "
"##,
        status_formats,
        surface.muted,
        surface.bg,
        surface.accent,
        surface.bg,
        surface.bg,
        active_style,
        inactive_style,
        surface.bg,
        surface.bg,
    )
}

fn tmux_powerkit_line1_left_format(line1_left: &[String]) -> String {
    let mut format = String::from("#[align=left range=left #{E:status-left-style}]");
    for entry in line1_left {
        match entry.as_str() {
            "session" => format
                .push_str("#[push-default]#(~/.local/bin/aibox-powerkit-render-session)#[pop-default]#[norange default]"),
            // Keep the row-1 window list compact so it remains visible without
            // consuming the right-aligned PowerKit metrics slot.
            "windows" => format.push_str(
                "#[push-default]#{W:#[range=window|#{window_index} #{E:window-status-style}]#{T:window-status-format}#[norange default],#[range=window|#{window_index} #{E:window-status-current-style}]#{T:window-status-current-format}#[norange default]}#[pop-default]",
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
socket="${AIBOX_TMUX_SOCKET:-${HOME}/.tmux/aibox.sock}"
if [[ -S "${socket}" && -z "${TMUX:-}" ]]; then
    export TMUX="${socket},0,0"
fi
cache_root="${AIBOX_POWERKIT_CACHE_DIR:-${XDG_CACHE_HOME:-${HOME}/.cache}}"
cache_probe="${cache_root}/.aibox-powerkit-write-test"
if ! { mkdir -p "${cache_root}" && : >"${cache_probe}"; } 2>/dev/null; then
    cache_root="${AIBOX_POWERKIT_FALLBACK_CACHE_DIR:-/tmp/aibox/tmux-powerkit-cache}"
    mkdir -p "${cache_root}" 2>/dev/null || true
else
    rm -f "${cache_probe}" 2>/dev/null || true
fi
export XDG_CACHE_HOME="${cache_root}"

if [[ ! -r "${POWERKIT_ROOT}/src/core/bootstrap.sh" ]]; then
    exit 0
fi

render_powerkit_line() {
    . "${POWERKIT_ROOT}/src/core/bootstrap.sh"
    load_powerkit_theme
    . "${POWERKIT_ROOT}/src/renderer/segment_builder.sh"

    reset_all_cycle_caches
    _batch_load_tmux_options
    _TMUX_OPTIONS_CACHE["@powerkit_plugins"]="$plugins"

    render_plugins "$side"
}

cache_dir="${cache_root}/tmux-powerkit/aibox-lines"
mkdir -p "${cache_dir}" 2>/dev/null || true
cache_key="$(printf '%s' "${side}|${plugins}" | cksum | awk '{print $1}')"
cache_file="${cache_dir}/list-${cache_key}.cache"
lock_file="${cache_file}.lock"
ttl="${AIBOX_POWERKIT_LINE_CACHE_TTL:-15}"

if [[ "${AIBOX_POWERKIT_REFRESH_CACHE:-}" == "1" ]]; then
    render_powerkit_line
    exit 0
fi

now="$(date +%s)"
if [[ -s "${cache_file}" ]]; then
    mtime="$(stat -c %Y "${cache_file}" 2>/dev/null || printf '0')"
    age=$((now - mtime))
    if (( age <= ttl )); then
        cat "${cache_file}"
        exit 0
    fi

    if [[ ! -e "${lock_file}" || $((now - $(stat -c %Y "${lock_file}" 2>/dev/null || printf '0'))) -gt 60 ]]; then
        : >"${lock_file}" 2>/dev/null || true
        (
            tmp="${cache_file}.$$"
            trap 'rm -f "${lock_file}" "${tmp}"' EXIT
            if AIBOX_POWERKIT_REFRESH_CACHE=1 "$0" "$side" "$plugins" >"${tmp}" 2>/dev/null; then
                mv "${tmp}" "${cache_file}" 2>/dev/null || true
            fi
        ) >/dev/null 2>&1 &
    fi
    cat "${cache_file}"
    exit 0
fi

tmp="${cache_file}.$$"
if render_powerkit_line >"${tmp}"; then
    cat "${tmp}"
    mv "${tmp}" "${cache_file}" 2>/dev/null || true
else
    rm -f "${tmp}" 2>/dev/null || true
fi
"#;

pub const POWERKIT_RENDER_SESSION_SH: &str = r#"#!/usr/bin/env bash
set -uo pipefail

POWERKIT_ROOT="${POWERKIT_ROOT:-/usr/local/share/aibox/tmux/plugins/tmux-powerkit}"
export POWERKIT_ROOT

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
export HOME="$(cd -- "${script_dir}/../.." && pwd)"
socket="${AIBOX_TMUX_SOCKET:-${HOME}/.tmux/aibox.sock}"
if [[ -S "${socket}" && -z "${TMUX:-}" ]]; then
    export TMUX="${socket},0,0"
fi
cache_root="${AIBOX_POWERKIT_CACHE_DIR:-${XDG_CACHE_HOME:-${HOME}/.cache}}"
cache_probe="${cache_root}/.aibox-powerkit-write-test"
if ! { mkdir -p "${cache_root}" && : >"${cache_probe}"; } 2>/dev/null; then
    cache_root="${AIBOX_POWERKIT_FALLBACK_CACHE_DIR:-/tmp/aibox/tmux-powerkit-cache}"
    mkdir -p "${cache_root}" 2>/dev/null || true
else
    rm -f "${cache_probe}" 2>/dev/null || true
fi
export XDG_CACHE_HOME="${cache_root}"

if [[ ! -r "${POWERKIT_ROOT}/src/core/bootstrap.sh" ]]; then
    exit 0
fi

render_powerkit_session() {
    . "${POWERKIT_ROOT}/src/core/bootstrap.sh"
    load_powerkit_theme
    . "${POWERKIT_ROOT}/src/renderer/compositor.sh"

    printf '%s' "$(_render_entity session left)"
    printf '%s' "$(_build_edge_separator session end left)"
}

cache_dir="${cache_root}/tmux-powerkit/aibox-lines"
mkdir -p "${cache_dir}" 2>/dev/null || true
cache_file="${cache_dir}/session.cache"
lock_file="${cache_file}.lock"
ttl="${AIBOX_POWERKIT_LINE_CACHE_TTL:-15}"

if [[ "${AIBOX_POWERKIT_REFRESH_CACHE:-}" == "1" ]]; then
    render_powerkit_session
    exit 0
fi

now="$(date +%s)"
if [[ -s "${cache_file}" ]]; then
    mtime="$(stat -c %Y "${cache_file}" 2>/dev/null || printf '0')"
    age=$((now - mtime))
    if (( age <= ttl )); then
        cat "${cache_file}"
        exit 0
    fi

    if [[ ! -e "${lock_file}" || $((now - $(stat -c %Y "${lock_file}" 2>/dev/null || printf '0'))) -gt 60 ]]; then
        : >"${lock_file}" 2>/dev/null || true
        (
            tmp="${cache_file}.$$"
            trap 'rm -f "${lock_file}" "${tmp}"' EXIT
            if AIBOX_POWERKIT_REFRESH_CACHE=1 "$0" >"${tmp}" 2>/dev/null; then
                mv "${tmp}" "${cache_file}" 2>/dev/null || true
            fi
        ) >/dev/null 2>&1 &
    fi
    cat "${cache_file}"
    exit 0
fi

tmp="${cache_file}.$$"
if render_powerkit_session >"${tmp}"; then
    cat "${tmp}"
    mv "${tmp}" "${cache_file}" 2>/dev/null || true
else
    rm -f "${tmp}" 2>/dev/null || true
fi
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
                && conf.contains("set -g set-clipboard external")
                && conf.contains("set -g mode-keys vi")
                && conf.contains("set -g default-terminal \"tmux-256color\"")
                && conf.contains("set -g xterm-keys on")
                && conf.contains("set -s extended-keys on")
                && conf.contains("*:extkeys")
                && conf.contains("wezterm:RGB,clipboard"),
            "generated tmux config should enable passthrough, modified-key forwarding, clipboard, vi copy-mode, and tmux-256color defaults for terminal app compatibility:\n{conf}"
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
            conf.contains(r#"set -g @vim_navigator_mapping_down "C-Down""#)
                && conf.contains("unbind-key -q -n C-j")
                && conf.contains("unbind-key -q -T copy-mode-vi C-j")
                && !conf.contains("bind-key -n C-j"),
            "generated tmux config must not bind global C-j because pasted newlines arrive as LF/C-j:\n{conf}"
        );
        assert!(
            conf.contains(
                r#"@powerkit_plugins "aibox_log,aibox_oom,aibox_proc,aibox_ai,aibox_mcp,aibox_mig,weather,uptime,datetime,forge,kubernetes,terraform,cloud,hostname,externalip,ssh,netspeed,ping,cpu,loadavg,memory,swap,disk,gpu""#
            )
                && conf.contains(r#"@powerkit_bar_layout "double""#)
                && conf.contains(r#"@powerkit_status_order "session,plugins""#)
                && conf.contains(r#"set -g status-interval 15"#)
                && conf.contains(r#"@powerkit_status_interval "15""#)
                && conf.contains(r#"@powerkit_transparent "false""#)
                && conf.contains(r#"@powerkit_pane_border_status "top""#)
                && conf.contains(r##"@powerkit_active_pane_border_color "#FABD2F""##)
                && conf.contains(r##"@powerkit_inactive_pane_border_color "#A89984""##)
                && conf.contains(r##"@powerkit_pane_border_status_bg "#282828""##)
                && conf.contains(r##"@powerkit_pane_border_format "#{?client_prefix,PREFIX,NORMAL} #{pane_title} #{pane_current_command}""##)
                && conf.contains(r#"@powerkit_line1_right "aibox_log,aibox_oom,aibox_proc,aibox_ai,aibox_mcp,aibox_mig,weather,uptime,datetime""#)
                && conf.contains(r#"@powerkit_line2_left "forge,kubernetes,terraform,cloud""#)
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
                && conf.contains(r#"@powerkit_plugin_forge_cache_ttl "120""#)
                && conf.contains(r#"@powerkit_plugin_forge_github_hosts "github.com""#)
                && conf.contains(r#"@powerkit_plugin_aibox_log_label "󱖫""#)
                && conf.contains(r#"@powerkit_plugin_aibox_oom_label "󰍛󰚌""#)
                && conf.contains(r#"@powerkit_plugin_aibox_proc_label "󰊚""#)
                && conf.contains(r#"@powerkit_plugin_aibox_ai_label "󱙺""#)
                && conf.contains(r#"@powerkit_plugin_aibox_mcp_label "󰌹""#)
                && conf.contains(r#"@powerkit_plugin_aibox_mig_label "󰚰""#),
            "generated persistent tmux config should carry bounded powerkit defaults:\n{conf}"
        );
        assert!(
            conf.contains(
            r#"bind-key -N "Show aibox/tmux key bindings" ? display-popup -w 110 -h 90% -E "bash \"$HOME/.local/bin/aibox-tmux-cheatsheet\"""#
            )
                && conf.contains(r#"bind-key -N "Select pane left" h select-pane -L"#)
                && conf.contains(
                    r#"bind-key -N "Open log pane (lnav)" o display-popup -E -w 90% -h 80% "aibox-log-viewer""#
                ),
            "generated persistent tmux config should expose the categorized cheatsheet popup:\n{conf}"
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
    fn tmux_forge_renders_configured_github_ssh_aliases() {
        let mut config = crate::config::test_config();
        config.customization.tmux.status.forge.github_hosts = vec![
            "github.com".to_string(),
            "github-bnaard".to_string(),
            "github_work".to_string(),
        ];

        let conf = tmux_conf(&config);

        assert!(conf.contains(
            r#"@powerkit_plugin_forge_github_hosts "github.com github-bnaard github_work""#
        ));
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

        // Line 2 left: forge → kubernetes → terraform → cloud
        // (DEC-20260508_2115-SilentFern; updated to replace git+github with forge)
        assert!(
            conf.contains(r#"@powerkit_line2_left "forge,kubernetes,terraform,cloud""#),
            "line2_left slot order must be: forge,kubernetes,terraform,cloud\n{conf}"
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
                r#"@powerkit_plugins "aibox_log,aibox_oom,aibox_proc,aibox_ai,aibox_mcp,aibox_mig,weather,uptime,datetime,forge,kubernetes,terraform,cloud,hostname,externalip,ssh,netspeed,ping,cpu,loadavg,memory,swap,disk,gpu""#
            ),
            "full plugin list snapshot mismatch — slot order is fixed per DEC-20260508_2115-SilentFern\n{conf}"
        );

        assert!(
            conf.contains("set -g status 2")
                && conf.contains("aibox-powerkit-render-session")
                && conf.contains("aibox-powerkit-render-list right aibox_log,aibox_oom,aibox_proc,aibox_ai,aibox_mcp,aibox_mig,weather,uptime,datetime")
                && conf.contains("aibox-powerkit-render-list left forge,kubernetes,terraform,cloud")
                && conf.contains("aibox-powerkit-render-list right hostname,externalip,ssh,netspeed,ping,cpu,loadavg,memory,swap,disk,gpu"),
            "generated status formats must keep the two-row PowerKit-aligned proof layout\n{conf}"
        );
        let line1_format = conf
            .lines()
            .find(|line| line.starts_with("set -g 'status-format[0]'"))
            .expect("extended status must set status-format[0]");
        assert!(
            line1_format.contains("#{W:")
                && line1_format.contains("aibox-powerkit-render-list right aibox_log"),
            "line 1 must keep the compact window list on the left and PowerKit metrics on the right:\n{line1_format}"
        );
        assert!(
            conf.contains("set -g pane-border-style \"fg=#A89984,bg=#282828\"")
                && conf.contains("set -g popup-style \"bg=#282828,fg=#EBDBB2\"")
                && conf.contains("#[fg=#FABD2F]")
                && conf.contains("#[fg=#DACBA7]"),
            "tmux surface styles should be generated from the resolved theme:\n{conf}"
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
    fn powerkit_render_helpers_use_tmp_cache() {
        for helper in [POWERKIT_RENDER_LIST_SH, POWERKIT_RENDER_SESSION_SH] {
            assert!(
                helper.contains(r#"AIBOX_POWERKIT_CACHE_DIR:-${XDG_CACHE_HOME:-${HOME}/.cache}"#)
                    && helper.contains(
                        r#"AIBOX_POWERKIT_FALLBACK_CACHE_DIR:-/tmp/aibox/tmux-powerkit-cache"#
                    )
                    && helper.contains(r#"AIBOX_POWERKIT_LINE_CACHE_TTL:-15"#)
                    && helper.contains(r#"AIBOX_POWERKIT_REFRESH_CACHE"#)
                    && helper.contains(r#"export XDG_CACHE_HOME="${cache_root}""#)
                    && helper
                        .contains(r#"socket="${AIBOX_TMUX_SOCKET:-${HOME}/.tmux/aibox.sock}""#)
                    && helper.contains(r#"cat "${cache_file}""#),
                "PowerKit helpers should prefer the managed cache, fall back when it is read-only, and serve cached full-line output:\n{helper}"
            );
        }
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
    fn explicit_model_provider_segments_render_without_global_auto_add() {
        let mut config = crate::config::test_config();
        config.customization.tmux.status.model_providers.enabled = false;
        config.customization.tmux.status.layout.line1_right = Some(vec![
            "modelstatus_openai".to_string(),
            "modelstatus_anthropic".to_string(),
            "weather".to_string(),
        ]);
        let conf = tmux_conf(&config);

        assert!(
            conf.contains(r#"@powerkit_line1_right "modelstatus_openai,modelstatus_anthropic,weather""#)
                && conf.contains(r#"@powerkit_plugins "modelstatus_openai,modelstatus_anthropic,weather,forge,kubernetes,terraform,cloud,hostname,externalip,ssh,netspeed,ping,cpu,loadavg,memory,swap,disk,gpu""#),
            "explicit modelstatus_* layout entries should remain in the PowerKit plugin list:\n{conf}"
        );
        assert!(
            conf.contains(r#"@powerkit_plugin_modelstatus_openai_label "OAI""#)
                && conf.contains(r#"@powerkit_plugin_modelstatus_anthropic_label "ANT""#)
                && conf.contains(r#"@powerkit_plugin_modelstatus_openai_force_render "true""#)
                && conf.contains(r#"@powerkit_plugin_modelstatus_anthropic_force_render "true""#),
            "explicit modelstatus_* layout entries should emit options and stay visible even when healthy:\n{conf}"
        );
    }

    #[test]
    fn model_provider_segments_emit_phase1_agent_count_options() {
        let mut config = crate::config::test_config();
        config.customization.tmux.status.model_providers.enabled = true;
        config
            .customization
            .tmux
            .status
            .model_providers
            .providers
            .truncate(2); // openai + anthropic
        let conf = tmux_conf(&config);

        // Phase 1 — agent count opt-in by default; binaries auto-defaulted.
        assert!(
            conf.contains(r#"@powerkit_plugin_modelstatus_openai_show_agent_count "true""#),
            "openai segment should declare show_agent_count=true by default:\n{conf}"
        );
        assert!(
            conf.contains(r#"@powerkit_plugin_modelstatus_openai_agent_binaries "codex""#),
            "openai segment should auto-default agent_binaries to codex:\n{conf}"
        );
        assert!(
            conf.contains(r#"@powerkit_plugin_modelstatus_anthropic_agent_binaries "claude""#),
            "anthropic segment should auto-default agent_binaries to claude:\n{conf}"
        );

        // Phase 0 — ok glyph default is off; chevron color carries ok signal.
        assert!(
            conf.contains(r#"@powerkit_plugin_modelstatus_openai_show_glyph_when_ok "false""#)
                && conf.contains(r#"@powerkit_plugin_modelstatus_openai_show_ok "false""#),
            "ok-glyph should default off so the chevron color is the canonical ok signal:\n{conf}"
        );
    }

    #[test]
    fn model_provider_segments_phase2_quota_opt_in() {
        let mut config = crate::config::test_config();
        config.customization.tmux.status.model_providers.enabled = true;
        config
            .customization
            .tmux
            .status
            .model_providers
            .providers
            .truncate(2);
        // Flip quota on for openai with a custom api-key env.
        config.customization.tmux.status.model_providers.providers[0].show_quota = true;
        config.customization.tmux.status.model_providers.providers[0].quota_api_key_env =
            Some("OPENAI_PROJECT_KEY".to_string());
        config.customization.tmux.status.model_providers.providers[0].quota_window =
            "requests".to_string();

        let conf = tmux_conf(&config);
        assert!(
            conf.contains(r#"@powerkit_plugin_modelstatus_openai_show_quota "true""#),
            "show_quota must propagate to the powerkit option:\n{conf}"
        );
        assert!(
            conf.contains(r#"@powerkit_plugin_modelstatus_openai_quota_window "requests""#),
            "quota_window must propagate verbatim:\n{conf}"
        );
        assert!(
            conf.contains(
                r#"@powerkit_plugin_modelstatus_openai_quota_api_key_env "OPENAI_PROJECT_KEY""#
            ),
            "quota_api_key_env override must propagate:\n{conf}"
        );
        // Anthropic was left default — show_quota false, no quota_api_key_env line.
        assert!(
            conf.contains(r#"@powerkit_plugin_modelstatus_anthropic_show_quota "false""#),
            "default show_quota for unmodified providers should be false:\n{conf}"
        );
        assert!(
            !conf.contains(r#"@powerkit_plugin_modelstatus_anthropic_quota_api_key_env"#),
            "no quota_api_key_env line should be emitted when unset:\n{conf}"
        );
    }

    #[test]
    fn model_provider_segments_phase3_admin_usage_requires_section_ack() {
        let mut config = crate::config::test_config();
        config.customization.tmux.status.model_providers.enabled = true;
        config
            .customization
            .tmux
            .status
            .model_providers
            .providers
            .truncate(1);

        // Provider asks for admin usage, section-level ack is still false.
        config.customization.tmux.status.model_providers.providers[0].show_admin_usage = true;

        let conf = tmux_conf(&config);
        assert!(
            conf.contains(r#"@powerkit_plugin_modelstatus_openai_show_admin_usage "false""#),
            "show_admin_usage at the provider level must be gated off until the section-level admin_usage_enabled is set:\n{conf}"
        );

        // Now flip the section-level ack on; admin usage should flow through.
        config
            .customization
            .tmux
            .status
            .model_providers
            .admin_usage_enabled = true;
        config.customization.tmux.status.model_providers.providers[0].admin_api_key_env =
            Some("OPENAI_ADMIN_KEY".to_string());
        let conf = tmux_conf(&config);
        assert!(
            conf.contains(r#"@powerkit_plugin_modelstatus_openai_show_admin_usage "true""#),
            "section-level admin_usage_enabled should release per-provider show_admin_usage:\n{conf}"
        );
        assert!(
            conf.contains(
                r#"@powerkit_plugin_modelstatus_openai_admin_api_key_env "OPENAI_ADMIN_KEY""#
            ),
            "admin_api_key_env override must propagate:\n{conf}"
        );
    }

    #[test]
    fn inactive_panes_render_with_dimmed_window_style() {
        // The non-focused window-style must use the dim palette so the
        // active pane stands out. The active style must keep the raw bg/fg
        // — confusingly matching them would erase the visual cue.
        let config = crate::config::test_config();
        let conf = tmux_conf(&config);
        let theme = config.customization.resolved_theme();
        let (bg, fg, _accent, _muted, _dim_fg_old, _active_title_fg) =
            crate::themes::terminal_surface_colors(&theme);
        let (dim_bg, dim_fg) = crate::themes::dim_inactive_pane_colors(&theme);

        assert_ne!(
            dim_bg.as_str(),
            bg,
            "dim_bg must differ from active bg or the cue is invisible"
        );
        assert_ne!(
            dim_fg.as_str(),
            fg,
            "dim_fg must differ from active fg or the cue is invisible"
        );
        assert!(
            conf.contains(&format!("set -g window-style \"bg={dim_bg},fg={dim_fg}\"")),
            "inactive window-style must use dim_bg/dim_fg:\n{conf}"
        );
        assert!(
            conf.contains(&format!("set -g window-active-style \"bg={bg},fg={fg}\"")),
            "active window-style must keep the original palette bg/fg:\n{conf}"
        );
    }

    #[test]
    fn layout_switch_binding_is_emitted_by_default_with_confirm_dispatcher() {
        let config = crate::config::test_config();
        let conf = tmux_conf(&config);
        assert!(
            conf.contains(r#"bind-key -N "Choose layout" L display-menu -T"#),
            "default layout-switch binding should use display-menu + key L:\n{conf}"
        );
        assert!(
            conf.contains("aibox-tmux-confirm-and-switch dev"),
            "confirm-by-default should route dev through aibox-tmux-confirm-and-switch:\n{conf}"
        );
        assert!(
            conf.contains("aibox-tmux-confirm-and-switch focus")
                && conf.contains("aibox-tmux-confirm-and-switch ai")
                && conf.contains("aibox-tmux-confirm-and-switch cowork"),
            "every built-in layout should appear in the menu:\n{conf}"
        );
    }

    #[test]
    fn layout_switch_confirm_false_bypasses_dialog() {
        let mut config = crate::config::test_config();
        config.customization.tmux.layout_switch.confirm = false;
        let conf = tmux_conf(&config);
        assert!(
            conf.contains("aibox-tmux-switch-layout dev"),
            "with confirm=false the menu should invoke aibox-tmux-switch-layout directly:\n{conf}"
        );
        assert!(
            !conf.contains("aibox-tmux-confirm-and-switch"),
            "with confirm=false the confirm wrapper should be absent:\n{conf}"
        );
    }

    #[test]
    fn layout_switch_disabled_omits_binding() {
        let mut config = crate::config::test_config();
        config.customization.tmux.layout_switch.enabled = false;
        let conf = tmux_conf(&config);
        assert!(
            !conf.contains(r#"bind-key -N "Choose layout""#),
            "disabled layout_switch should omit the binding entirely:\n{conf}"
        );
    }

    #[test]
    fn layout_switch_table_style_emits_key_table_form() {
        let mut config = crate::config::test_config();
        config.customization.tmux.layout_switch.style = "table".to_string();
        let conf = tmux_conf(&config);
        assert!(
            conf.contains("switch-client -T aibox_layouts"),
            "table style should switch into a tmux key table:\n{conf}"
        );
        assert!(
            conf.contains("bind-key -T aibox_layouts d"),
            "table style should bind individual keys in the layouts table:\n{conf}"
        );
    }

    #[test]
    fn theme_switch_binding_lists_configured_themes() {
        let config = crate::config::test_config();
        let conf = tmux_conf(&config);
        assert!(
            conf.contains(r#"bind-key -N "Choose theme" T display-menu"#),
            "default theme-switch binding should use display-menu + key T:\n{conf}"
        );
        // Default themes from config: gruvbox-dark, catppuccin-mocha, tokyo-night, dracula, projectious.
        for theme in [
            "gruvbox-dark",
            "catppuccin-mocha",
            "tokyo-night",
            "dracula",
            "projectious",
        ] {
            assert!(
                conf.contains(&format!("aibox theme --theme {theme}")),
                "theme menu should include {theme}:\n{conf}"
            );
        }
        assert!(
            conf.contains("Toggle light/dark"),
            "include_mode_toggle default true should emit the toggle entry:\n{conf}"
        );
        assert!(
            conf.contains("Heavy: restart TUIs"),
            "the heavy tier should be exposed as a menu entry:\n{conf}"
        );
        assert!(
            conf.contains("AIBOX_THEME_CONFIRM_RESTART_TUIS=true"),
            "default confirm_restart_tuis=true must wire through to the helper:\n{conf}"
        );
    }

    #[test]
    fn theme_switch_disabled_omits_binding() {
        let mut config = crate::config::test_config();
        config.customization.tmux.theme_switch.enabled = false;
        let conf = tmux_conf(&config);
        assert!(
            !conf.contains(r#"bind-key -N "Choose theme""#),
            "disabled theme_switch should omit the binding entirely:\n{conf}"
        );
    }

    #[test]
    fn theme_switch_confirm_restart_tuis_false_propagates() {
        let mut config = crate::config::test_config();
        config.customization.tmux.theme_switch.confirm_restart_tuis = false;
        let conf = tmux_conf(&config);
        assert!(
            conf.contains("AIBOX_THEME_CONFIRM_RESTART_TUIS=false"),
            "confirm_restart_tuis=false must flow into the heavy-tier env:\n{conf}"
        );
    }

    #[test]
    fn model_provider_show_ok_legacy_alias_still_works() {
        // Older configs may still set [customization.tmux.status.model_providers].show_ok = true
        // — that should equivalently enable the ✓ glyph.
        let mut config = crate::config::test_config();
        config.customization.tmux.status.model_providers.enabled = true;
        config.customization.tmux.status.model_providers.show_ok = true;
        config
            .customization
            .tmux
            .status
            .model_providers
            .providers
            .truncate(1);

        let conf = tmux_conf(&config);
        assert!(
            conf.contains(r#"@powerkit_plugin_modelstatus_openai_show_glyph_when_ok "true""#),
            "legacy show_ok = true must flow into the new show_glyph_when_ok option:\n{conf}"
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

    #[test]
    fn tmux_status_separators_are_configurable() {
        let mut config = crate::config::test_config();
        config.customization.tmux.status.separators.style =
            crate::config::TmuxStatusSeparatorStyle::Flame;
        config.customization.tmux.status.separators.edge_style =
            crate::config::TmuxStatusSeparatorStyle::Honeycomb;
        config.customization.tmux.status.separators.elements_spacing =
            crate::config::TmuxStatusElementsSpacing::Plugins;
        let conf = tmux_conf(&config);

        assert!(
            conf.contains(r#"@powerkit_separator_style "flame""#)
                && conf.contains(r#"@powerkit_edge_separator_style "honeycomb""#)
                && conf.contains(r#"@powerkit_elements_spacing "plugins""#),
            "status separator config should be emitted as PowerKit options:\n{conf}"
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

    #[test]
    fn tmux_conf_renders_configurable_attention_title_and_runtime_options() {
        let mut config = crate::config::test_config();
        config.customization.tmux.title.format =
            "{state_symbol}{project}:{window} {directory_path} {message}{agent_suffix}".to_string();
        config.customization.tmux.title.max_length = 80;
        config.customization.tmux.notifications.enabled = true;
        let conf = tmux_conf(&config);

        assert!(conf.contains("set -g set-titles on"));
        assert!(conf.contains("#{=80:#{@aibox_attention_symbol}#{@aibox_title_project}:#W #{pane_current_path} #{@aibox_attention_message}#{?#{@aibox_attention_harness}, — #{?#{@aibox_attention_agent},#{@aibox_attention_agent}@,}#{@aibox_attention_harness},}}"));
        assert!(conf.contains("set -g @aibox_notifications_enabled \"1\""));
        assert!(conf.contains("set -g @aibox_notifications_protocol \"osc-9\""));
        assert!(conf.contains("set -g @aibox_title_state_question \"❓ \""));
        assert!(conf.contains("set-hook -g pane-died[90]"));
    }

    #[test]
    fn tmux_conf_disables_title_without_attention_hook() {
        let mut config = crate::config::test_config();
        config.customization.tmux.title.enabled = false;
        let conf = tmux_conf(&config);
        assert!(conf.contains("set -g set-titles off"));
        assert!(!conf.contains("pane-died[90]"));
    }
}
