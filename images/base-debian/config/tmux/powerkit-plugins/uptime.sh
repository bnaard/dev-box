#!/usr/bin/env bash
# PowerKit plugin: aibox container uptime override.
#
# Upstream tmux-powerkit reads /proc/uptime, which reports host/VM kernel
# uptime inside containers. aibox users expect this segment to show the
# lifetime of the current dev container, so this override derives uptime from
# PID 1 start time.

POWERKIT_ROOT="${POWERKIT_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
. "${POWERKIT_ROOT}/src/contract/plugin_contract.sh"

plugin_get_metadata() {
    metadata_set "id" "uptime"
    metadata_set "name" "Uptime"
    metadata_set "description" "Display container uptime"
}

plugin_declare_options() {
    declare_option "icon" "icon" $'\uf254' "Plugin icon"
    declare_option "cache_ttl" "number" "300" "Cache duration in seconds"
}

plugin_get_content_type() { printf 'dynamic'; }
plugin_get_presence() { printf 'always'; }
plugin_get_state() { printf 'active'; }
plugin_get_health() { printf 'ok'; }

_container_uptime_from_proc() {
    [[ -r /proc/1/stat && -r /proc/stat ]] || return 1

    local stat_line stat_rest start_ticks boot_time hz now started_at uptime_seconds
    stat_line="$(cat /proc/1/stat 2>/dev/null)" || return 1
    stat_rest="${stat_line#*) }"
    set -- ${stat_rest}
    start_ticks="${20:-}"
    [[ "${start_ticks}" =~ ^[0-9]+$ ]] || return 1

    boot_time="$(awk '$1 == "btime" { print $2; exit }' /proc/stat 2>/dev/null)" || return 1
    [[ "${boot_time}" =~ ^[0-9]+$ ]] || return 1

    hz="$(getconf CLK_TCK 2>/dev/null || printf '100')"
    [[ "${hz}" =~ ^[0-9]+$ && "${hz}" -gt 0 ]] || hz=100

    now="${EPOCHSECONDS:-$(date +%s)}"
    started_at=$((boot_time + (start_ticks / hz)))
    uptime_seconds=$((now - started_at))
    (( uptime_seconds >= 0 )) || return 1

    printf '%d' "${uptime_seconds}"
}

_container_uptime_from_ps() {
    local uptime_seconds
    uptime_seconds="$(ps -p 1 -o etimes= 2>/dev/null | awk '{ print $1; exit }')" || return 1
    [[ "${uptime_seconds}" =~ ^[0-9]+$ ]] || return 1
    printf '%d' "${uptime_seconds}"
}

_kernel_uptime_fallback() {
    [[ -r /proc/uptime ]] || return 1
    awk '{ printf "%d", $1 }' /proc/uptime 2>/dev/null
}

plugin_get_context() {
    local uptime_str
    uptime_str="$(plugin_data_get "uptime")"

    if [[ "${uptime_str}" == *d* ]]; then
        printf 'days'
    elif [[ "${uptime_str}" == *h* ]]; then
        printf 'hours'
    else
        printf 'minutes'
    fi
}

plugin_collect() {
    local uptime_seconds=0
    uptime_seconds="$(_container_uptime_from_proc)" \
        || uptime_seconds="$(_container_uptime_from_ps)" \
        || uptime_seconds="$(_kernel_uptime_fallback)" \
        || uptime_seconds=0

    plugin_data_set "uptime" "$(format_uptime_seconds "${uptime_seconds}")"
}

plugin_render() {
    plugin_data_get "uptime"
}

plugin_get_icon() {
    get_option "icon"
}
