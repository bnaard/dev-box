# Powerkit renders these list-specific sections; tmux only supplies alignment.
set -g status 2
set -g 'status-format[0]' '#[align=left range=left #{E:status-left-style}]#[push-default]#(~/.local/bin/aibox-powerkit-render-session)#[pop-default]#[norange default]#[push-default]#{W:#[range=window|#{window_index} #{E:window-status-style}]#{T:window-status-format}#[norange default],#[range=window|#{window_index} #{E:window-status-current-style}]#{T:window-status-current-format}#[norange default]}#[pop-default]#{E:@_powerkit_left_edge_sep}#[nolist align=right range=right #{E:status-right-style}]#[push-default]#(~/.local/bin/aibox-powerkit-render-list right aibox_ai,modelstatus_openai,modelstatus_anthropic,aibox_log,aibox_oom,aibox_proc,aibox_mcp,aibox_mig,weather,uptime,datetime)#[pop-default]#[norange default]'
set -g 'status-format[1]' '#[align=left]#(~/.local/bin/aibox-powerkit-render-list left forge,kubernetes,terraform)#[align=right]#(~/.local/bin/aibox-powerkit-render-list right hostname,externalip,ssh,netspeed,ping,cpu,loadavg,memory,swap,disk,gpu)'

# aibox post-PowerKit overrides.
# PowerKit's renderer owns status-format and pane styles when it loads. Keep
# the aibox two-row status shape and pane surfaces authoritative after that
# render pass.
set -g pane-border-style "fg=#8B949E,bg=#0D1117"
set -g pane-active-border-style "fg=#58A6FF,bg=#0D1117"
set -g pane-border-format "#[bg=#0D1117]#{?pane_active,#[fg=#58A6FF],#[fg=#CFD7DE] } #{?client_prefix,PREFIX,NORMAL} #{pane_title} #{pane_current_command} #[bg=#0D1117,fg=#0D1117] "
