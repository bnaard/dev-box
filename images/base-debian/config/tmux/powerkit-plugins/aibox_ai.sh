#!/usr/bin/env bash
# PowerKit plugin: aibox ai metric segment (path-a split from aibox.sh).
#
# Renders a single PowerKit segment for the ai metric (active AI agent count).
# Each aibox metric is its own plugin so it gets chevron separators and
# color-rotation styling matching adjacent PowerKit segments.
# Ref: BACK-20260508_1603-QuietCedar, DEC-20260508_2115-SilentFern.

POWERKIT_ROOT="${POWERKIT_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
. "${POWERKIT_ROOT}/src/contract/plugin_contract.sh"

plugin_get_metadata() {
    metadata_set "id" "aibox_ai"
    metadata_set "name" "aibox_ai"
    metadata_set "description" "aibox AI metric: active AI agent count from the aibox-status helper"
}

plugin_declare_options() {
    declare_option "label" "string" "󱙺" "Segment label"
    declare_option "cache_ttl" "number" "30" "Cache duration in seconds"
}

json_value() {
    local key="$1"
    sed -n "s/.*\"${key}\":\"\\([^\"]*\\)\".*/\\1/p"
}

plugin_collect() {
    local json
    json="$(aibox-status --plugin-json 2>/dev/null)" || return 1
    plugin_data_set "ai_agents" "$(printf '%s' "${json}" | json_value ai_agents)"
}

plugin_get_content_type() { printf 'dynamic'; }
plugin_get_presence()     { printf 'always'; }
plugin_get_state()        { printf 'active'; }
plugin_get_health()       { printf 'ok'; }
plugin_get_context()      { printf 'runtime'; }
plugin_get_icon()         { get_option "label"; }

plugin_render() {
    printf '%s' "$(plugin_data_get ai_agents)"
}
