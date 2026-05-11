#!/usr/bin/env bash
# PowerKit plugin: aibox Kubernetes context override.
#
# Status rendering must never perform live cluster or auth checks. The upstream
# plugin probes cluster connectivity with kubectl, which can briefly surface
# auth/network failures in the second status row. This override renders only
# local kubeconfig state and hides itself when no context is configured.

POWERKIT_ROOT="${POWERKIT_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
. "${POWERKIT_ROOT}/src/contract/plugin_contract.sh"

plugin_get_metadata() {
    metadata_set "id" "kubernetes"
    metadata_set "name" "Kubernetes"
    metadata_set "description" "Display Kubernetes context from local kubeconfig without live cluster probes"
}

plugin_declare_options() {
    declare_option "show_context" "bool" "true" "Show context name"
    declare_option "show_namespace" "bool" "true" "Show namespace"
    declare_option "separator" "string" "/" "Separator between context and namespace"
    declare_option "warn_on_prod" "bool" "true" "Show warning health when in production context"
    declare_option "prod_keywords" "string" "prod,production,prd" "Comma-separated production keywords"
    declare_option "icon" "icon" $'\U000F10FE' "Plugin icon"
    declare_option "cache_ttl" "number" "30" "Cache duration in seconds"
}

plugin_get_content_type() { printf 'dynamic'; }
plugin_get_presence() { printf 'conditional'; }

_kubeconfig_paths() {
    local raw="${KUBECONFIG:-$HOME/.kube/config}"
    local path
    IFS=':' read -ra paths <<< "$raw"
    for path in "${paths[@]}"; do
        [[ -n "$path" && -r "$path" ]] && printf '%s\n' "$path"
    done
}

_current_context_from_file() {
    local kubeconfig
    while IFS= read -r kubeconfig; do
        awk '/^current-context:[[:space:]]*/ {
            sub(/^current-context:[[:space:]]*/, "")
            print
            exit
        }' "$kubeconfig" 2>/dev/null
    done < <(_kubeconfig_paths) | awk 'NF { print; exit }'
}

_namespace_from_file() {
    local context="$1"
    local kubeconfig
    while IFS= read -r kubeconfig; do
        awk -v ctx="$context" '
            /^contexts:/ { in_contexts=1; next }
            in_contexts && /^[^[:space:]-]/ { in_contexts=0 }
            in_contexts && /^[[:space:]]*-[[:space:]]*context:/ { in_block=1; ns=""; name=""; next }
            in_block && /^[[:space:]]*namespace:/ {
                sub(/^[[:space:]]*namespace:[[:space:]]*/, "")
                ns=$0
                next
            }
            in_block && /^[[:space:]]*name:/ {
                sub(/^[[:space:]]*name:[[:space:]]*/, "")
                name=$0
                if (name == ctx) {
                    print ns
                    exit
                }
            }
            in_block && /^[[:space:]]*-[[:space:]]/ { in_block=0; ns=""; name="" }
        ' "$kubeconfig" 2>/dev/null
    done < <(_kubeconfig_paths) | awk 'NF { print; exit }'
}

_current_context() {
    local context
    if has_cmd kubectl; then
        context="$(kubectl config current-context 2>/dev/null | awk 'NF { print; exit }')" || context=""
        [[ -n "$context" ]] && { printf '%s' "$context"; return 0; }
    fi
    _current_context_from_file
}

_current_namespace() {
    local context="$1"
    local namespace
    if has_cmd kubectl; then
        namespace="$(kubectl config view --minify --output 'jsonpath={..namespace}' 2>/dev/null)" || namespace=""
        [[ -n "$namespace" ]] && { printf '%s' "$namespace"; return 0; }
    fi
    _namespace_from_file "$context"
}

plugin_collect() {
    local context namespace
    context="$(_current_context)"
    [[ -z "$context" ]] && return 0

    namespace="$(_current_namespace "$context")"
    [[ -z "$namespace" ]] && namespace="default"

    plugin_data_set "context" "$context"
    plugin_data_set "namespace" "$namespace"
}

plugin_get_state() {
    [[ -n "$(plugin_data_get context)" ]] && printf 'active' || printf 'inactive'
}

plugin_get_health() {
    local context warn_on_prod prod_keywords keyword
    context="$(plugin_data_get context)"
    warn_on_prod="$(get_option warn_on_prod)"
    prod_keywords="$(get_option prod_keywords)"

    if [[ "$warn_on_prod" == "true" && -n "$context" ]]; then
        local IFS=','
        for keyword in $prod_keywords; do
            [[ "${context,,}" == *"${keyword,,}"* ]] && { printf 'warning'; return; }
        done
    fi

    printf 'ok'
}

plugin_get_context() {
    local context prod_keywords keyword
    context="$(plugin_data_get context)"
    [[ -z "$context" ]] && { printf 'no_context'; return; }

    prod_keywords="$(get_option prod_keywords)"
    local IFS=','
    for keyword in $prod_keywords; do
        [[ "${context,,}" == *"${keyword,,}"* ]] && { printf 'production'; return; }
    done

    if [[ "${context,,}" == *stag* || "${context,,}" == *staging* ]]; then
        printf 'staging'
    elif [[ "${context,,}" == *dev* || "${context,,}" == *development* ]]; then
        printf 'development'
    elif [[ "${context,,}" == *local* || "${context,,}" == *minikube* || "${context,,}" == *docker-desktop* || "${context,,}" == *kind* || "${context,,}" == *k3* ]]; then
        printf 'local'
    else
        printf 'configured'
    fi
}

plugin_get_icon() { get_option "icon"; }

plugin_render() {
    local context namespace show_context show_namespace separator display result
    context="$(plugin_data_get context)"
    namespace="$(plugin_data_get namespace)"
    [[ -z "$context" ]] && return 0

    show_context="$(get_option show_context)"
    show_namespace="$(get_option show_namespace)"
    separator="$(get_option separator)"

    display="${context##*@}"
    display="${display##*:}"
    result=""

    [[ "$show_context" == "true" ]] && result="$display"
    if [[ "$show_namespace" == "true" ]]; then
        [[ -n "$result" ]] && result+="$separator"
        result+="${namespace:-default}"
    fi

    printf '%s' "$result"
}
