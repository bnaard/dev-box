#!/usr/bin/env bash
# =============================================================================
# Plugin: modelstatus_<provider>
# Description: Compact AI model-provider status indicator for aibox.
# =============================================================================

POWERKIT_ROOT="${POWERKIT_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
. "${POWERKIT_ROOT}/src/contract/plugin_contract.sh"

plugin_get_metadata() {
    metadata_set "id" "${_CURRENT_PLUGIN:-modelstatus_provider}"
    metadata_set "name" "Model Provider Status"
    metadata_set "description" "Monitor AI model-provider and harness health"
}

plugin_check_dependencies() {
    require_cmd "curl" || return 1
    require_cmd "jq" 1
    return 0
}

_default_provider() {
    local base="${_CURRENT_PLUGIN:-}"
    base="${base#modelstatus_}"
    [[ -n "$base" && "$base" != "$_CURRENT_PLUGIN" ]] && printf '%s' "$base" || printf 'unknown'
}

_default_label() {
    case "$(_default_provider)" in
        openai) printf 'OAI' ;;
        anthropic) printf 'ANT' ;;
        google) printf 'GOOG' ;;
        mistral) printf 'MST' ;;
        deepseek) printf 'DS' ;;
        cohere) printf 'COH' ;;
        xai) printf 'XAI' ;;
        alibaba) printf 'QWN' ;;
        aws) printf 'AWS' ;;
        meta) printf 'META' ;;
        microsoft) printf 'MS' ;;
        minimax) printf 'MM' ;;
        moonshot) printf 'KIMI' ;;
        nvidia) printf 'NV' ;;
        xiaomi) printf 'MI' ;;
        zai) printf 'ZAI' ;;
        *) printf 'AI' ;;
    esac
}

_default_status_url() {
    case "$(_default_provider)" in
        openai) printf 'https://status.openai.com/api/v2/summary.json' ;;
        anthropic) printf 'https://status.claude.com/api/v2/summary.json' ;;
        google) printf 'https://status.cloud.google.com/incidents.json' ;;
        mistral) printf 'https://status.mistral.ai/api/v2/summary.json' ;;
        deepseek) printf 'https://status.deepseek.com/api/v2/summary.json' ;;
        cohere) printf 'https://status.cohere.com/api/v2/summary.json' ;;
        *) printf '' ;;
    esac
}

plugin_declare_options() {
    declare_option "provider" "string" "$(_default_provider)" "Provider key"
    declare_option "label" "string" "$(_default_label)" "Short status segment header"
    declare_option "checks" "string" "overall,models,harness" "Comma-separated checks: overall,models,harness"
    declare_option "status_url" "string" "$(_default_status_url)" "Provider status JSON URL"
    declare_option "overall_components" "string" "" "Comma-separated component names for overall checks"
    declare_option "model_components" "string" "" "Comma-separated component names for model-serving checks"
    declare_option "harness_components" "string" "" "Comma-separated component names for CLI/harness checks"
    declare_option "timeout" "number" "3" "HTTP timeout in seconds"
    declare_option "cache_ttl" "number" "300" "Cache duration in seconds"
    declare_option "show_ok" "bool" "true" "Render healthy providers with a checkmark"
}

plugin_get_content_type() { printf 'dynamic'; }
plugin_get_presence() { printf 'conditional'; }

_status_rank() {
    case "$1" in
        outage) printf 3 ;;
        degraded) printf 2 ;;
        unknown) printf 1 ;;
        ok) printf 0 ;;
        *) printf 1 ;;
    esac
}

_max_status() {
    local current="${1:-ok}" candidate="${2:-unknown}"
    if (( $(_status_rank "$candidate") > $(_status_rank "$current") )); then
        printf '%s' "$candidate"
    else
        printf '%s' "$current"
    fi
}

_map_statuspage_status() {
    case "$1" in
        operational|none|ok|good|available) printf 'ok' ;;
        degraded_performance|under_maintenance|minor|maintenance|notice) printf 'degraded' ;;
        partial_outage|major_outage|critical|major|incident|outage) printf 'outage' ;;
        *) printf 'unknown' ;;
    esac
}

_component_filter_expr() {
    local csv="$1"
    [[ -z "$csv" ]] && { printf '.'; return; }
    local expr='false'
    local item
    IFS=',' read -ra items <<< "$csv"
    for item in "${items[@]}"; do
        item="${item#"${item%%[![:space:]]*}"}"
        item="${item%"${item##*[![:space:]]}"}"
        [[ -z "$item" ]] && continue
        expr="${expr} or (.name | ascii_downcase | contains(\"$(printf '%s' "$item" | tr '[:upper:]' '[:lower:]')\"))"
    done
    printf 'select(%s)' "$expr"
}

_statuspage_component_status() {
    local data="$1" filter_csv="$2"
    local worst='ok'
    local statuses status mapped
    if has_cmd jq; then
        if [[ -n "$filter_csv" ]]; then
            statuses=$(printf '%s' "$data" | jq -r ".components[]? | $(_component_filter_expr "$filter_csv") | .status // empty" 2>/dev/null)
        else
            statuses=$(printf '%s' "$data" | jq -r ".components[]? | .status // empty" 2>/dev/null)
        fi
        [[ -z "$statuses" ]] && { printf 'unknown'; return; }
        while IFS= read -r status; do
            mapped=$(_map_statuspage_status "$status")
            worst=$(_max_status "$worst" "$mapped")
        done <<< "$statuses"
        printf '%s' "$worst"
        return
    fi
    printf 'unknown'
}

_statuspage_rollup_status() {
    local data="$1" indicator mapped
    if has_cmd jq; then
        indicator=$(printf '%s' "$data" | jq -r '.status.indicator // .status.status // empty' 2>/dev/null)
        mapped=$(_map_statuspage_status "$indicator")
        [[ "$mapped" != "unknown" ]] && { printf '%s' "$mapped"; return; }
    fi
    _statuspage_component_status "$data" ""
}

_google_incident_status() {
    local data="$1" active
    if has_cmd jq; then
        active=$(printf '%s' "$data" | jq '[.[]? | select((.end // "") == "")] | length' 2>/dev/null)
        [[ "${active:-0}" -gt 0 ]] && printf 'outage' || printf 'ok'
        return
    fi
    [[ "$data" == *'"end":null'* ]] && printf 'outage' || printf 'ok'
}

plugin_collect() {
    local provider status_url timeout data checks worst overall models harness
    provider=$(get_option "provider")
    status_url=$(get_option "status_url")
    timeout=$(get_option "timeout")
    checks=$(get_option "checks")

    if [[ -z "$status_url" ]]; then
        plugin_data_set "status" "unknown"
        plugin_data_set "detail" "no public status API configured"
        return 0
    fi

    data=$(curl -fsSL --max-time "$timeout" "$status_url" 2>/dev/null) || {
        plugin_data_set "status" "unknown"
        plugin_data_set "detail" "status API unavailable"
        return 1
    }

    if [[ "$provider" == "google" ]]; then
        overall=$(_google_incident_status "$data")
        models="$overall"
        harness="unknown"
    else
        overall=$(_statuspage_rollup_status "$data")
        models=$(_statuspage_component_status "$data" "$(get_option "model_components")")
        harness=$(_statuspage_component_status "$data" "$(get_option "harness_components")")
    fi

    worst='ok'
    [[ ",$checks," == *",overall,"* ]] && worst=$(_max_status "$worst" "$overall")
    [[ ",$checks," == *",models,"* ]] && worst=$(_max_status "$worst" "$models")
    [[ ",$checks," == *",harness,"* ]] && worst=$(_max_status "$worst" "$harness")

    plugin_data_set "status" "$worst"
    plugin_data_set "detail" "overall=${overall}; models=${models}; harness=${harness}"
}

plugin_get_icon() {
    get_option "label"
}

plugin_get_state() {
    local status show_ok
    status=$(plugin_data_get "status")
    show_ok=$(get_option "show_ok")
    if [[ "$status" == "ok" && "$show_ok" != "true" ]]; then
        printf 'inactive'
    elif [[ "$status" == "degraded" || "$status" == "unknown" ]]; then
        printf 'degraded'
    elif [[ "$status" == "outage" ]]; then
        printf 'failed'
    else
        printf 'active'
    fi
}

plugin_get_health() {
    case "$(plugin_data_get "status")" in
        ok) printf 'ok' ;;
        degraded) printf 'warning' ;;
        outage) printf 'error' ;;
        *) printf 'info' ;;
    esac
}

plugin_get_context() {
    plugin_data_get "detail"
}

plugin_render() {
    case "$(plugin_data_get "status")" in
        ok) printf '✓' ;;
        degraded) printf '!' ;;
        outage) printf '!!' ;;
        *) printf '?' ;;
    esac
}
