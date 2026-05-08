#!/usr/bin/env bash
# PowerKit plugin: aibox oom metric segment (path-a split from aibox.sh).
#
# Renders a single PowerKit segment for the oom metric (events/kills).
# Each aibox metric is its own plugin so it gets chevron separators and
# color-rotation styling matching adjacent PowerKit segments.
# Ref: BACK-20260508_1603-QuietCedar, DEC-20260508_2115-SilentFern.

POWERKIT_ROOT="${POWERKIT_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
. "${POWERKIT_ROOT}/src/contract/plugin_contract.sh"

plugin_get_metadata() {
    metadata_set "id" "aibox_oom"
    metadata_set "name" "aibox_oom"
    metadata_set "description" "aibox OOM metric: oom_events/oom_kill counts from the aibox-status helper"
}

plugin_declare_options() {
    declare_option "cache_ttl" "number" "5" "Cache duration in seconds"
}

json_value() {
    local key="$1"
    sed -n "s/.*\"${key}\":\"\\([^\"]*\\)\".*/\\1/p"
}

plugin_collect() {
    local json
    json="$(aibox-status --plugin-json 2>/dev/null)" || return 1
    plugin_data_set "oom_events" "$(printf '%s' "${json}" | json_value oom_events)"
    plugin_data_set "oom_kill"   "$(printf '%s' "${json}" | json_value oom_kill)"
}

plugin_get_content_type() { printf 'dynamic'; }
plugin_get_presence()     { printf 'always'; }
plugin_get_state()        { printf 'active'; }

plugin_get_health() {
    local oom_kill
    oom_kill="$(plugin_data_get oom_kill)"
    if [[ "${oom_kill:-0}" != "0" ]]; then
        printf 'warning'
    else
        printf 'ok'
    fi
}

plugin_get_context() { printf 'runtime'; }
plugin_get_icon()    { printf 'OOM'; }

plugin_render() {
    printf 'OOM %s/%s' \
        "$(plugin_data_get oom_events)" \
        "$(plugin_data_get oom_kill)"
}
