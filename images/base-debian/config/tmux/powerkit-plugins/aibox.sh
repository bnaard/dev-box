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
    declare_option "metrics" "string" "log,oom,proc,ai,mcp,mig" "Enabled aibox metrics"
}

json_value() {
    local key="$1"
    sed -n "s/.*\"${key}\":\"\\([^\"]*\\)\".*/\\1/p"
}

plugin_collect() {
    local json memory_current memory_max oom_events oom_kill cpu_throttling load_average net
    local processes threads ai_agents processkit_mode processkit_mcp processkit_display disk_used disk_total
    local log_info log_warn log_error migrations degraded container_uptime host

    json="$(aibox-status --plugin-json 2>/dev/null)" || return 1
    host="$(printf '%s' "${json}" | json_value host)"
    memory_current="$(printf '%s' "${json}" | json_value memory_current)"
    memory_max="$(printf '%s' "${json}" | json_value memory_max)"
    oom_events="$(printf '%s' "${json}" | json_value oom_events)"
    oom_kill="$(printf '%s' "${json}" | json_value oom_kill)"
    cpu_throttling="$(printf '%s' "${json}" | json_value cpu_throttling)"
    load_average="$(printf '%s' "${json}" | json_value load_average)"
    net="$(printf '%s' "${json}" | json_value net)"
    processes="$(printf '%s' "${json}" | json_value processes)"
    threads="$(printf '%s' "${json}" | json_value threads)"
    ai_agents="$(printf '%s' "${json}" | json_value ai_agents)"
    processkit_mode="$(printf '%s' "${json}" | json_value processkit_mode)"
    processkit_mcp="$(printf '%s' "${json}" | json_value processkit_mcp)"
    processkit_display="$(printf '%s' "${json}" | json_value processkit_display)"
    disk_used="$(printf '%s' "${json}" | json_value disk_used)"
    disk_total="$(printf '%s' "${json}" | json_value disk_total)"
    log_info="$(printf '%s' "${json}" | json_value log_info)"
    log_warn="$(printf '%s' "${json}" | json_value log_warn)"
    log_error="$(printf '%s' "${json}" | json_value log_error)"
    migrations="$(printf '%s' "${json}" | json_value migrations)"
    degraded="$(printf '%s' "${json}" | sed -n 's/.*"degraded":\([^,}]*\).*/\1/p')"
    container_uptime="$(printf '%s' "${json}" | json_value container_uptime)"

    plugin_data_set "host" "${host:-n/a}"
    plugin_data_set "memory_current" "${memory_current:-n/a}"
    plugin_data_set "memory_max" "${memory_max:-n/a}"
    plugin_data_set "oom_events" "${oom_events:-0}"
    plugin_data_set "oom_kill" "${oom_kill:-0}"
    plugin_data_set "cpu_throttling" "${cpu_throttling:-n/a}"
    plugin_data_set "load_average" "${load_average:-n/a}"
    plugin_data_set "net" "${net:-n/a}"
    plugin_data_set "processes" "${processes:-0}"
    plugin_data_set "threads" "${threads:-${processes:-0}}"
    plugin_data_set "ai_agents" "${ai_agents:-0}"
    plugin_data_set "processkit_mode" "${processkit_mode:-none}"
    plugin_data_set "processkit_mcp" "${processkit_mcp:-0}"
    plugin_data_set "processkit_display" "${processkit_display:-${processkit_mode:-none}/${processkit_mcp:-0}}"
    plugin_data_set "disk_used" "${disk_used:-n/a}"
    plugin_data_set "disk_total" "${disk_total:-n/a}"
    plugin_data_set "log_info" "${log_info:-0}"
    plugin_data_set "log_warn" "${log_warn:-0}"
    plugin_data_set "log_error" "${log_error:-0}"
    plugin_data_set "migrations" "${migrations:-0}"
    plugin_data_set "degraded" "${degraded:-false}"
    plugin_data_set "container_uptime" "${container_uptime:-n/a}"
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
    local enabled metrics_csv
    metrics_csv="$(get_option "metrics")"
    if [[ -z "${metrics_csv}" ]]; then
        metrics_csv="log,oom,proc,ai,mcp,mig"
    fi

    contains_metric() {
        [[ ",${metrics_csv}," == *",$1,"* ]]
    }

    append_metric() {
        local key="$1"
        local value="$2"
        if contains_metric "${key}"; then
            enabled+=" ${value}"
        fi
    }

    local deg=""
    if [[ "$(plugin_data_get degraded)" == "true" ]]; then
        deg=" DEG yes"
    fi

    append_metric "log" "LOG $(plugin_data_get log_info)/$(plugin_data_get log_warn)/$(plugin_data_get log_error)"
    append_metric "oom" "OOM $(plugin_data_get oom_events)/$(plugin_data_get oom_kill)"
    append_metric "proc" "PROC $(plugin_data_get processes)/$(plugin_data_get threads)"
    append_metric "ai" "AI $(plugin_data_get ai_agents)"
    append_metric "mcp" "MCP $(plugin_data_get processkit_display)"
    append_metric "mig" "MIG $(plugin_data_get migrations)"

    enabled="${enabled# }"
    printf '%s%s' "${enabled:-OK}" "${deg}"
}
