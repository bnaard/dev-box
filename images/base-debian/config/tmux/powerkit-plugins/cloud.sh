#!/usr/bin/env bash
# PowerKit plugin: aibox cloud context override.
#
# The upstream plugin falls back to live provider checks such as aws sts,
# gcloud auth, and az account show. Those calls are too expensive and noisy for
# status rendering, and can briefly expose auth errors in the second row. This
# override reads local config/env only and hides itself when no context exists.

POWERKIT_ROOT="${POWERKIT_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
. "${POWERKIT_ROOT}/src/contract/plugin_contract.sh"

plugin_get_metadata() {
    metadata_set "id" "cloud"
    metadata_set "name" "Cloud"
    metadata_set "description" "Display local cloud context without live provider auth probes"
}

plugin_declare_options() {
    declare_option "providers" "string" "all" "Cloud providers to monitor (all|aws,gcp,azure)"
    declare_option "show_region" "bool" "false" "Show AWS region in display"
    declare_option "icon" "icon" $'\U000F0163' "Default cloud icon"
    declare_option "icon_aws" "icon" $'\U000F0E0F' "AWS icon"
    declare_option "icon_gcp" "icon" $'\U000F0B20' "GCP icon"
    declare_option "icon_azure" "icon" $'\U000F0805' "Azure icon"
    declare_option "icon_multi" "icon" $'\U000F0164' "Multi-provider icon"
    declare_option "cache_ttl" "number" "120" "Cache duration in seconds"
}

plugin_get_content_type() { printf 'dynamic'; }
plugin_get_presence() { printf 'conditional'; }

_first_aws_profile_from_config() {
    local cfg="${AWS_CONFIG_FILE:-$HOME/.aws/config}"
    [[ -r "$cfg" ]] || return 1

    awk '
        /^\[default\]/ { print "default"; exit }
        /^\[profile [^]]+\]/ {
            sub(/^\[profile /, "")
            sub(/\]$/, "")
            print
            exit
        }
    ' "$cfg" 2>/dev/null
}

_aws_region_from_config() {
    local profile="$1"
    local cfg="${AWS_CONFIG_FILE:-$HOME/.aws/config}"
    [[ -r "$cfg" ]] || return 1

    awk -v p="$profile" '
        /^\[[^]]+\]/ { in_profile=0 }
        $0 == "[profile " p "]" || (p == "default" && $0 == "[default]") { in_profile=1; next }
        in_profile && /^[[:space:]]*region[[:space:]]*=/ {
            sub(/^[[:space:]]*region[[:space:]]*=[[:space:]]*/, "")
            print
            exit
        }
    ' "$cfg" 2>/dev/null
}

_aws_context() {
    local profile region show_region
    profile="${AWS_PROFILE:-${AWS_DEFAULT_PROFILE:-}}"
    [[ -z "$profile" ]] && profile="$(_first_aws_profile_from_config)"
    [[ -z "$profile" ]] && return 1

    show_region="$(get_option show_region)"
    region="${AWS_REGION:-${AWS_DEFAULT_REGION:-}}"
    [[ -z "$region" ]] && region="$(_aws_region_from_config "$profile")"

    if [[ "$show_region" == "true" && -n "$region" ]]; then
        printf '%s@%s' "$profile" "$region"
    else
        printf '%s' "$profile"
    fi
}

_gcp_config_file() {
    local active_config cfg_dir
    cfg_dir="${CLOUDSDK_CONFIG:-$HOME/.config/gcloud}"
    active_config="default"

    if [[ -r "$cfg_dir/active_config" ]]; then
        active_config="$(awk 'NF { print; exit }' "$cfg_dir/active_config" 2>/dev/null)"
        [[ -z "$active_config" ]] && active_config="default"
    fi

    printf '%s/configurations/config_%s' "$cfg_dir" "$active_config"
}

_gcp_value_from_config() {
    local key="$1"
    local cfg
    cfg="$(_gcp_config_file)"
    [[ -r "$cfg" ]] || return 1
    awk -F '=' -v key="$key" '
        $1 ~ "^[[:space:]]*" key "[[:space:]]*$" {
            value=$2
            sub(/^[[:space:]]*/, "", value)
            sub(/[[:space:]]*$/, "", value)
            print value
            exit
        }
    ' "$cfg" 2>/dev/null
}

_gcp_context() {
    local project account
    project="${CLOUDSDK_CORE_PROJECT:-${GOOGLE_CLOUD_PROJECT:-}}"
    [[ -z "$project" ]] && project="$(_gcp_value_from_config project)"
    account="$(_gcp_value_from_config account)"

    if [[ -n "$project" ]]; then
        printf '%s' "$project"
    elif [[ -n "$account" ]]; then
        printf '%s' "$account"
    else
        return 1
    fi
}

_azure_context() {
    local profile sub
    [[ -n "${AZURE_SUBSCRIPTION_ID:-}" ]] && { printf '%s' "$AZURE_SUBSCRIPTION_ID"; return 0; }

    profile="${AZURE_CONFIG_DIR:-$HOME/.azure}/azureProfile.json"
    [[ -r "$profile" ]] || return 1

    if has_cmd jq; then
        sub="$(jq -r '.subscriptions[]? | select(.isDefault==true) | (.name // .id // empty)' "$profile" 2>/dev/null | awk 'NF { print; exit }')"
        [[ -z "$sub" ]] && sub="$(jq -r '.subscriptions[0]? | (.name // .id // empty)' "$profile" 2>/dev/null | awk 'NF { print; exit }')"
    else
        sub="$(sed -n 's/.*"name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$profile" 2>/dev/null | awk 'NF { print; exit }')"
    fi

    [[ -n "$sub" ]] && printf '%s' "$sub"
}

_provider_enabled() {
    local provider="$1"
    local providers
    providers="$(get_option providers)"
    [[ "$providers" == "all" ]] && return 0
    [[ ",${providers,,}," == *",${provider},"* ]]
}

_join_contexts() {
    local sep="$1"
    shift

    local result="" item
    for item in "$@"; do
        if [[ -z "$result" ]]; then
            result="$item"
        else
            result+="${sep}${item}"
        fi
    done
    printf '%s' "$result"
}

plugin_collect() {
    local providers=()
    local contexts=()
    local ctx

    if _provider_enabled aws && ctx="$(_aws_context)"; then
        providers+=("aws")
        contexts+=("$ctx")
    fi

    if _provider_enabled gcp && ctx="$(_gcp_context)"; then
        providers+=("gcp")
        contexts+=("$ctx")
    fi

    if _provider_enabled azure && ctx="$(_azure_context)"; then
        providers+=("azure")
        contexts+=("$ctx")
    fi

    [[ ${#providers[@]} -eq 0 ]] && return 0

    if [[ ${#providers[@]} -eq 1 ]]; then
        plugin_data_set "provider" "${providers[0]}"
        plugin_data_set "context" "${contexts[0]}"
    else
        plugin_data_set "provider" "multi"
        plugin_data_set "context" "$(_join_contexts " | " "${contexts[@]}")"
    fi
}

plugin_get_state() {
    [[ -n "$(plugin_data_get provider)" ]] && printf 'active' || printf 'inactive'
}

plugin_get_health() { printf 'ok'; }

plugin_get_context() {
    local provider
    provider="$(plugin_data_get provider)"
    printf '%s' "${provider:-none}"
}

plugin_get_icon() {
    local provider
    provider="$(plugin_data_get provider)"

    case "$provider" in
        aws) get_option "icon_aws" ;;
        gcp) get_option "icon_gcp" ;;
        azure) get_option "icon_azure" ;;
        multi) get_option "icon_multi" ;;
        *) get_option "icon" ;;
    esac
}

plugin_render() {
    plugin_data_get context
}
