#!/usr/bin/env bash
# PowerKit plugin: compact aibox runtime status.

POWERKIT_ROOT="${POWERKIT_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
. "${POWERKIT_ROOT}/src/contract/plugin_contract.sh"

plugin_get_metadata() {
    metadata_set "id" "aibox"
    metadata_set "name" "aibox"
    metadata_set "description" "Container runtime status from the image-owned aibox-status helper"
}

plugin_declare_options() {
    declare_option "icon" "string" "AIBOX" "Segment label"
    declare_option "cache_ttl" "number" "5" "Cache duration in seconds"
}

json_value() {
    local key="$1"
    sed -n "s/.*\"${key}\":\"\\([^\"]*\\)\".*/\\1/p"
}

plugin_collect() {
    local json memory_current memory_max oom_kill processes ai_agents processkit_mode processkit_mcp

    json="$(aibox-status --plugin-json 2>/dev/null)" || return 1
    memory_current="$(printf '%s' "${json}" | json_value memory_current)"
    memory_max="$(printf '%s' "${json}" | json_value memory_max)"
    oom_kill="$(printf '%s' "${json}" | json_value oom_kill)"
    processes="$(printf '%s' "${json}" | json_value processes)"
    ai_agents="$(printf '%s' "${json}" | json_value ai_agents)"
    processkit_mode="$(printf '%s' "${json}" | json_value processkit_mode)"
    processkit_mcp="$(printf '%s' "${json}" | json_value processkit_mcp)"

    plugin_data_set "memory_current" "${memory_current:-n/a}"
    plugin_data_set "memory_max" "${memory_max:-n/a}"
    plugin_data_set "oom_kill" "${oom_kill:-0}"
    plugin_data_set "processes" "${processes:-0}"
    plugin_data_set "ai_agents" "${ai_agents:-0}"
    plugin_data_set "processkit_mode" "${processkit_mode:-none}"
    plugin_data_set "processkit_mcp" "${processkit_mcp:-0}"
}

plugin_get_content_type() { printf 'dynamic'; }
plugin_get_presence() { printf 'always'; }
plugin_get_state() { printf 'active'; }

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
plugin_get_icon() { get_option "icon"; }

plugin_render() {
    printf 'MEM %s/%s OOM %s PROC %s AI %s MCP %s %s' \
        "$(plugin_data_get memory_current)" \
        "$(plugin_data_get memory_max)" \
        "$(plugin_data_get oom_kill)" \
        "$(plugin_data_get processes)" \
        "$(plugin_data_get ai_agents)" \
        "$(plugin_data_get processkit_mode)" \
        "$(plugin_data_get processkit_mcp)"
}
