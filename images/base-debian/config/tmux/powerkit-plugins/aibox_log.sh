#!/usr/bin/env bash
# PowerKit plugin: aibox log metric segment (path-a split from aibox.sh).
#
# Renders a single PowerKit segment for the log metric (warn/error counts).
# Each aibox metric is its own plugin so it gets chevron separators and
# color-rotation styling matching adjacent PowerKit segments — fixes the flat
# text rendering of the old single-segment aibox plugin.
# Ref: BACK-20260508_1603-QuietCedar, DEC-20260508_2115-SilentFern.

POWERKIT_ROOT="${POWERKIT_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
. "${POWERKIT_ROOT}/src/contract/plugin_contract.sh"

plugin_get_metadata() {
    metadata_set "id" "aibox_log"
    metadata_set "name" "aibox_log"
    metadata_set "description" "aibox log metric: warn/error counts from the aibox-status helper"
}

plugin_declare_options() {
    declare_option "label" "string" "󱖫" "Segment label"
    declare_option "cache_ttl" "number" "30" "Cache duration in seconds"
}

json_value() {
    local key="$1"
    sed -n "s/.*\"${key}\":\"\\([^\"]*\\)\".*/\\1/p"
}

plugin_collect() {
    local json
    json="$(aibox-status --plugin-json 2>/dev/null)" || return 1
    local warn error
    warn="$(printf '%s' "${json}" | json_value log_warn)"
    error="$(printf '%s' "${json}" | json_value log_error)"
    plugin_data_set "log_warn" "${warn:-0}"
    plugin_data_set "log_error" "${error:-0}"
}

plugin_get_content_type() { printf 'dynamic'; }
plugin_get_presence()     { printf 'always'; }
plugin_get_state()        { printf 'active'; }
plugin_get_health() {
    local err warn
    err="$(plugin_data_get log_error)"
    warn="$(plugin_data_get log_warn)"
    if [[ -n "${err}" && "${err}" =~ ^[0-9]+$ && "${err}" -gt 0 ]]; then
        printf 'error'
    elif [[ -n "${warn}" && "${warn}" =~ ^[0-9]+$ && "${warn}" -gt 0 ]]; then
        printf 'warning'
    else
        printf 'ok'
    fi
}
plugin_get_context()      { printf 'runtime'; }
plugin_get_icon()         { get_option "label"; }

plugin_render() {
    printf '%s/%s' \
        "$(plugin_data_get log_warn)" \
        "$(plugin_data_get log_error)"
}
