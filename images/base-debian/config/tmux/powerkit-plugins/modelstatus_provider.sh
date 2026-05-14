#!/usr/bin/env bash
# =============================================================================
# Plugin: modelstatus_<provider>
# Description: Compact AI model-provider status indicator for aibox.
#
# Phase 1: local agent count (show_agent_count / agent_binaries)
# Phase 2: rate-limit quota polling (show_quota / quota_window / quota_api_key_env)
# Phase 3: admin usage rollup (show_admin_usage / admin_api_key_env)
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
    # ── Existing options ────────────────────────────────────────────────────
    declare_option "provider" "string" "$(_default_provider)" "Provider key"
    declare_option "label" "string" "$(_default_label)" "Short status segment header"
    declare_option "checks" "string" "overall,models,harness" "Comma-separated checks: overall,models,harness"
    declare_option "status_url" "string" "$(_default_status_url)" "Provider status JSON URL"
    declare_option "overall_components" "string" "" "Comma-separated component names for overall checks"
    declare_option "model_components" "string" "" "Comma-separated component names for model-serving checks"
    declare_option "harness_components" "string" "" "Comma-separated component names for CLI/harness checks"
    declare_option "timeout" "number" "3" "HTTP timeout in seconds"
    declare_option "cache_ttl" "number" "300" "Cache duration in seconds"

    # ── Phase 0 — glyph control ─────────────────────────────────────────────
    # show_glyph_when_ok: new canonical name; defaults to false so the
    # chevron color carries the ok signal without a redundant ✓.
    declare_option "show_glyph_when_ok" "bool" "false" "Render ✓ glyph for ok state (chevron color already signals it)"
    # show_ok: legacy alias kept for backward compatibility
    declare_option "show_ok" "bool" "false" "Legacy alias for show_glyph_when_ok"

    # ── Phase 1 — local agent count ─────────────────────────────────────────
    declare_option "show_agent_count" "bool" "true" "Append ×N (local agent count) to segment text"
    declare_option "agent_binaries" "string" "" "Comma-separated CLI binary names that count as agents for this provider"

    # ── Phase 2 — rate-limit quota polling ──────────────────────────────────
    declare_option "show_quota" "bool" "false" "Poll provider API for rate-limit % remaining (~\$0.03/day)"
    declare_option "quota_window" "string" "tokens" "tokens or requests"
    declare_option "quota_api_key_env" "string" "" "Env-var name holding the API key (falls back to provider default)"

    # ── Phase 3 — admin usage rollup ────────────────────────────────────────
    declare_option "show_admin_usage" "bool" "false" "Poll provider admin API for monthly usage (requires admin key)"
    declare_option "admin_api_key_env" "string" "" "Env-var name holding the admin API key (falls back to <PROVIDER>_ADMIN_KEY)"
}

plugin_get_content_type() { printf 'dynamic'; }
plugin_get_presence() { printf 'conditional'; }

# ---------------------------------------------------------------------------
# Status helpers (unchanged)
# ---------------------------------------------------------------------------

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

# ---------------------------------------------------------------------------
# Phase 1 helpers — local agent count
# ---------------------------------------------------------------------------

# _resolve_agent_count: call aibox-status --plugin-json, extract
# ai_agents_breakdown, sum entries for each binary in agent_binaries csv.
# Prints an integer (0 if nothing found or on error).
_resolve_agent_count() {
    local binaries_csv="$1"
    [[ -z "$binaries_csv" ]] && { printf '0'; return; }

    local json total bin val
    json=$(aibox-status --plugin-json 2>/dev/null) || { printf '0'; return; }
    total=0

    IFS=',' read -ra bins <<< "$binaries_csv"
    for bin in "${bins[@]}"; do
        bin="${bin#"${bin%%[![:space:]]*}"}"
        bin="${bin%"${bin##*[![:space:]]}"}"
        [[ -z "$bin" ]] && continue

        if has_cmd jq; then
            val=$(printf '%s' "$json" | jq -r ".ai_agents_breakdown[\"${bin}\"] // 0" 2>/dev/null)
        else
            # Fallback: grep for "bin":N pattern
            val=$(printf '%s' "$json" | grep -o "\"${bin}\":[0-9]*" | grep -o '[0-9]*$')
        fi
        # ensure numeric
        [[ "$val" =~ ^[0-9]+$ ]] || val=0
        total=$(( total + val ))
    done
    printf '%d' "$total"
}

# ---------------------------------------------------------------------------
# Phase 2 helpers — quota polling
# ---------------------------------------------------------------------------

# _default_key_env: given a provider name, return the conventional env var
_default_key_env() {
    case "$1" in
        anthropic) printf 'ANTHROPIC_API_KEY' ;;
        openai)    printf 'OPENAI_API_KEY' ;;
        mistral)   printf 'MISTRAL_API_KEY' ;;
        deepseek)  printf 'DEEPSEEK_API_KEY' ;;
        cohere)    printf 'COHERE_API_KEY' ;;
        xai)       printf 'XAI_API_KEY' ;;
        *)         printf '' ;;
    esac
}

# _resolve_api_key: given an override env name and provider, print the key
_resolve_api_key() {
    local override_env="$1" provider="$2"
    local key=""
    if [[ -n "$override_env" ]]; then
        key="${!override_env:-}"
    fi
    if [[ -z "$key" ]]; then
        local default_env
        default_env=$(_default_key_env "$provider")
        [[ -n "$default_env" ]] && key="${!default_env:-}"
    fi
    printf '%s' "$key"
}

# _parse_ratelimit_percent: given response headers text, window ("tokens"|"requests"),
# provider ("anthropic"|"openai"), print integer percent or empty on failure.
_parse_ratelimit_percent() {
    local headers="$1" window="${2:-tokens}" provider="${3:-openai}"
    local remaining limit pct

    if [[ "$provider" == "anthropic" ]]; then
        if [[ "$window" == "requests" ]]; then
            remaining=$(printf '%s' "$headers" | grep -i '^anthropic-ratelimit-requests-remaining:' | grep -o '[0-9]*' | head -1)
            limit=$(printf '%s' "$headers" | grep -i '^anthropic-ratelimit-requests-limit:' | grep -o '[0-9]*' | head -1)
        else
            remaining=$(printf '%s' "$headers" | grep -i '^anthropic-ratelimit-tokens-remaining:' | grep -o '[0-9]*' | head -1)
            limit=$(printf '%s' "$headers" | grep -i '^anthropic-ratelimit-tokens-limit:' | grep -o '[0-9]*' | head -1)
        fi
    else
        # OpenAI-style
        if [[ "$window" == "requests" ]]; then
            remaining=$(printf '%s' "$headers" | grep -i '^x-ratelimit-remaining-requests:' | grep -o '[0-9]*' | head -1)
            limit=$(printf '%s' "$headers" | grep -i '^x-ratelimit-limit-requests:' | grep -o '[0-9]*' | head -1)
        else
            remaining=$(printf '%s' "$headers" | grep -i '^x-ratelimit-remaining-tokens:' | grep -o '[0-9]*' | head -1)
            limit=$(printf '%s' "$headers" | grep -i '^x-ratelimit-limit-tokens:' | grep -o '[0-9]*' | head -1)
        fi
    fi

    [[ -z "$remaining" || -z "$limit" || "$limit" -eq 0 ]] 2>/dev/null && { printf ''; return; }
    pct=$(( remaining * 100 / limit ))
    printf '%d' "$pct"
}

# _poll_quota_anthropic: fire a minimal Anthropic request, parse rate-limit headers.
# Prints integer percent or empty.
_poll_quota_anthropic() {
    local key="$1" window="${2:-tokens}" timeout="${3:-3}"
    local headers
    headers=$(curl -sSf -D - -o /dev/null \
        --max-time "$timeout" \
        -X POST https://api.anthropic.com/v1/messages \
        -H "x-api-key: ${key}" \
        -H "anthropic-version: 2023-06-01" \
        -H "content-type: application/json" \
        -d '{"model":"claude-haiku-4-5-20251001","max_tokens":1,"messages":[{"role":"user","content":"."}]}' \
        2>/dev/null) || { printf ''; return; }
    _parse_ratelimit_percent "$headers" "$window" "anthropic"
}

# _poll_quota_openai: fire a minimal OpenAI request, parse rate-limit headers.
# Prints integer percent or empty.
_poll_quota_openai() {
    local key="$1" window="${2:-tokens}" timeout="${3:-3}"
    local headers
    headers=$(curl -sSf -D - -o /dev/null \
        --max-time "$timeout" \
        -X POST https://api.openai.com/v1/chat/completions \
        -H "Authorization: Bearer ${key}" \
        -H "content-type: application/json" \
        -d '{"model":"gpt-4.1-mini","max_tokens":1,"messages":[{"role":"user","content":"."}]}' \
        2>/dev/null) || { printf ''; return; }
    _parse_ratelimit_percent "$headers" "$window" "openai"
}

# ---------------------------------------------------------------------------
# Phase 3 helpers — admin usage rollup
# ---------------------------------------------------------------------------

# _format_si: format a large integer with SI suffix (1k, 1.5k, 1M, etc.)
_format_si() {
    local n="$1"
    [[ "$n" =~ ^[0-9]+$ ]] || { printf '%s' "$n"; return; }
    if (( n >= 1000000 )); then
        local m=$(( n / 1000000 ))
        local r=$(( (n % 1000000) / 100000 ))
        if (( r > 0 )); then
            printf '%s.%sM' "$m" "$r"
        else
            printf '%sM' "$m"
        fi
    elif (( n >= 1000 )); then
        local k=$(( n / 1000 ))
        local r=$(( (n % 1000) / 100 ))
        if (( r > 0 )); then
            printf '%s.%sk' "$k" "$r"
        else
            printf '%sk' "$k"
        fi
    else
        printf '%d' "$n"
    fi
}

# _resolve_admin_key: given an override env name and provider, print the admin key
_resolve_admin_key() {
    local override_env="$1" provider="$2"
    local key=""
    if [[ -n "$override_env" ]]; then
        key="${!override_env:-}"
    fi
    if [[ -z "$key" ]]; then
        # Default: <PROVIDER>_ADMIN_KEY (uppercased)
        local default_env
        default_env="${provider^^}_ADMIN_KEY"
        key="${!default_env:-}"
    fi
    printf '%s' "$key"
}

# _month_start_ts: print the Unix timestamp of the first second of this month (UTC)
_month_start_ts() {
    date -u +%s -d "$(date -u +%Y-%m-01) 00:00:00 UTC" 2>/dev/null \
        || date -u -j -f "%Y-%m-%d %H:%M:%S" "$(date -u +%Y-%m-01) 00:00:00" +%s 2>/dev/null \
        || printf '0'
}

# _poll_admin_usage_anthropic: fetch monthly token usage from Anthropic admin API.
# Prints "<used>/<limit>" or just "<used>" when limit is unavailable.
_poll_admin_usage_anthropic() {
    local key="$1" timeout="${2:-5}"
    local ts body total_tokens
    ts=$(_month_start_ts)
    local starting_at
    starting_at=$(date -u -d "@${ts}" +%Y-%m-%dT%H:%M:%SZ 2>/dev/null \
        || date -u -r "${ts}" +%Y-%m-%dT%H:%M:%SZ 2>/dev/null \
        || printf '')
    [[ -z "$starting_at" ]] && { printf ''; return; }

    body=$(curl -sSf --max-time "$timeout" \
        "https://api.anthropic.com/v1/organizations/usage_report/messages?starting_at=${starting_at}" \
        -H "x-api-key: ${key}" \
        2>/dev/null) || { printf ''; return; }

    if has_cmd jq; then
        total_tokens=$(printf '%s' "$body" | jq '
            [.data[]? | (.input_tokens // 0) + (.output_tokens // 0)] | add // 0
        ' 2>/dev/null)
    else
        # crude fallback: sum all input_tokens and output_tokens values
        total_tokens=$(printf '%s' "$body" | grep -o '"input_tokens":[0-9]*\|"output_tokens":[0-9]*' \
            | grep -o '[0-9]*' | awk '{s+=$1} END{print s+0}')
    fi
    [[ -z "$total_tokens" || "$total_tokens" == "null" ]] && { printf ''; return; }
    [[ "$total_tokens" =~ ^[0-9]+$ ]] || { printf ''; return; }
    _format_si "$total_tokens"
}

# _poll_admin_usage_openai: fetch monthly token usage from OpenAI admin API.
# Prints "<used>" (limit not easily available from this endpoint).
_poll_admin_usage_openai() {
    local key="$1" timeout="${2:-5}"
    local ts body total_tokens
    ts=$(_month_start_ts)

    body=$(curl -sSf --max-time "$timeout" \
        "https://api.openai.com/v1/organization/usage/completions?start_time=${ts}" \
        -H "Authorization: Bearer ${key}" \
        2>/dev/null) || { printf ''; return; }

    if has_cmd jq; then
        total_tokens=$(printf '%s' "$body" | jq '
            [.data[]? | (.input_tokens // 0) + (.output_tokens // 0) + (.input_cached_tokens // 0)] | add // 0
        ' 2>/dev/null)
    else
        total_tokens=$(printf '%s' "$body" | grep -o '"input_tokens":[0-9]*\|"output_tokens":[0-9]*\|"input_cached_tokens":[0-9]*' \
            | grep -o '[0-9]*' | awk '{s+=$1} END{print s+0}')
    fi
    [[ -z "$total_tokens" || "$total_tokens" == "null" ]] && { printf ''; return; }
    [[ "$total_tokens" =~ ^[0-9]+$ ]] || { printf ''; return; }
    _format_si "$total_tokens"
}

# ---------------------------------------------------------------------------
# plugin_collect — status page + agent count + quota + admin usage
# ---------------------------------------------------------------------------

plugin_collect() {
    local provider status_url timeout data checks worst overall models harness
    provider=$(get_option "provider")
    status_url=$(get_option "status_url")
    timeout=$(get_option "timeout")
    checks=$(get_option "checks")

    # ── Status page poll ────────────────────────────────────────────────────
    if [[ -z "$status_url" ]]; then
        plugin_data_set "status" "unknown"
        plugin_data_set "detail" "no public status API configured"
    else
        data=$(curl -fsSL --max-time "$timeout" "$status_url" 2>/dev/null) || {
            plugin_data_set "status" "unknown"
            plugin_data_set "detail" "status API unavailable"
            data=""
        }

        if [[ -n "$data" ]]; then
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
        fi
    fi

    # ── Phase 1 — local agent count ────────────────────────────────────────
    local show_agent_count agent_binaries_csv
    show_agent_count=$(get_option "show_agent_count")
    if [[ "$show_agent_count" == "true" ]]; then
        agent_binaries_csv=$(get_option "agent_binaries")
        if [[ -n "$agent_binaries_csv" ]]; then
            local agent_count
            agent_count=$(_resolve_agent_count "$agent_binaries_csv")
            plugin_data_set "agent_count" "$agent_count"
        fi
    fi

    # ── Phase 2 — quota polling (opt-in, billable) ──────────────────────────
    local show_quota
    show_quota=$(get_option "show_quota")
    if [[ "$show_quota" == "true" ]]; then
        local quota_key_env quota_window api_key quota_pct
        quota_key_env=$(get_option "quota_api_key_env")
        quota_window=$(get_option "quota_window")
        api_key=$(_resolve_api_key "$quota_key_env" "$provider")

        if [[ -n "$api_key" ]]; then
            quota_pct=""
            case "$provider" in
                anthropic)
                    quota_pct=$(_poll_quota_anthropic "$api_key" "$quota_window" "$timeout")
                    ;;
                openai)
                    quota_pct=$(_poll_quota_openai "$api_key" "$quota_window" "$timeout")
                    ;;
                mistral|deepseek|cohere|xai)
                    # Stub: OpenAI-compatible headers but endpoints/models vary per provider.
                    # These providers will be implemented in a follow-up once endpoints are
                    # confirmed; for now silently skip to avoid bad requests.
                    quota_pct=""
                    ;;
                *)
                    quota_pct=""
                    ;;
            esac
            [[ -n "$quota_pct" ]] && plugin_data_set "quota_percent" "$quota_pct"
        fi
    fi

    # ── Phase 3 — admin usage rollup (gated upstream, opt-in) ───────────────
    local show_admin_usage
    show_admin_usage=$(get_option "show_admin_usage")
    if [[ "$show_admin_usage" == "true" ]]; then
        local admin_key_env admin_key usage_text
        admin_key_env=$(get_option "admin_api_key_env")
        admin_key=$(_resolve_admin_key "$admin_key_env" "$provider")

        if [[ -n "$admin_key" ]]; then
            usage_text=""
            case "$provider" in
                anthropic)
                    usage_text=$(_poll_admin_usage_anthropic "$admin_key" "$timeout")
                    ;;
                openai)
                    usage_text=$(_poll_admin_usage_openai "$admin_key" "$timeout")
                    ;;
                *)
                    usage_text=""
                    ;;
            esac
            [[ -n "$usage_text" ]] && plugin_data_set "admin_usage_text" "$usage_text"
        fi
    fi

    return 0
}

# ---------------------------------------------------------------------------
# Display callbacks
# ---------------------------------------------------------------------------

plugin_get_icon() {
    get_option "label"
}

plugin_get_state() {
    local status show_ok show_glyph_when_ok show_glyph
    status=$(plugin_data_get "status")
    show_ok=$(get_option "show_ok")
    show_glyph_when_ok=$(get_option "show_glyph_when_ok")
    show_glyph="false"
    [[ "$show_ok" == "true" || "$show_glyph_when_ok" == "true" ]] && show_glyph="true"

    if [[ "$status" == "ok" && "$show_glyph" != "true" ]]; then
        # Check whether there is anything to show (agent count / quota / admin)
        local agent_count quota_pct admin_text
        agent_count=$(plugin_data_get "agent_count")
        quota_pct=$(plugin_data_get "quota_percent")
        admin_text=$(plugin_data_get "admin_usage_text")
        if [[ -z "$agent_count" || "$agent_count" == "0" ]] && \
           [[ -z "$quota_pct" ]] && \
           [[ -z "$admin_text" ]]; then
            printf 'inactive'
            return
        fi
    fi

    if [[ "$status" == "degraded" || "$status" == "unknown" ]]; then
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
    local status show_ok show_glyph_when_ok show_glyph glyph tail parts

    status=$(plugin_data_get "status")
    show_ok=$(get_option "show_ok")
    show_glyph_when_ok=$(get_option "show_glyph_when_ok")
    show_glyph="false"
    [[ "$show_ok" == "true" || "$show_glyph_when_ok" == "true" ]] && show_glyph="true"

    # ── Glyph ───────────────────────────────────────────────────────────────
    case "$status" in
        ok)
            if [[ "$show_glyph" == "true" ]]; then
                glyph='✓'
            else
                glyph=''
            fi
            ;;
        degraded) glyph='󰀦' ;;
        outage)   glyph='󰚌' ;;
        *)        glyph='?' ;;
    esac

    # ── Tail components ─────────────────────────────────────────────────────
    parts=()

    # Phase 1: agent count
    local show_agent_count agent_count
    show_agent_count=$(get_option "show_agent_count")
    if [[ "$show_agent_count" == "true" ]]; then
        agent_count=$(plugin_data_get "agent_count")
        if [[ "$agent_count" =~ ^[0-9]+$ && "$agent_count" -gt 0 ]]; then
            parts+=("×${agent_count}")
        fi
    fi

    # Phase 2: quota percent
    local show_quota quota_pct
    show_quota=$(get_option "show_quota")
    if [[ "$show_quota" == "true" ]]; then
        quota_pct=$(plugin_data_get "quota_percent")
        if [[ "$quota_pct" =~ ^[0-9]+$ ]]; then
            parts+=("${quota_pct}%")
        fi
    fi

    # Phase 3: admin usage
    local show_admin_usage admin_text
    show_admin_usage=$(get_option "show_admin_usage")
    if [[ "$show_admin_usage" == "true" ]]; then
        admin_text=$(plugin_data_get "admin_usage_text")
        if [[ -n "$admin_text" ]]; then
            parts+=("${admin_text}")
        fi
    fi

    # ── Combine glyph + tail ────────────────────────────────────────────────
    if [[ "${#parts[@]}" -eq 0 ]]; then
        printf '%s' "$glyph"
    else
        tail="${parts[*]}"   # space-separated by default IFS
        if [[ -n "$glyph" ]]; then
            printf '%s %s' "$glyph" "$tail"
        else
            printf '%s' "$tail"
        fi
    fi
}
