#!/usr/bin/env bash
# PowerKit plugin: aibox network speed override.
#
# Upstream tmux-powerkit renders network rates with natural width, which makes
# the right-aligned status line move as rates change. This override keeps the
# upstream behavior but pads each rate to a fixed configurable width.

POWERKIT_ROOT="${POWERKIT_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
. "${POWERKIT_ROOT}/src/contract/plugin_contract.sh"

plugin_get_metadata() {
    metadata_set "id" "netspeed"
    metadata_set "name" "Net Speed"
    metadata_set "description" "Display network upload/download speed"
}

plugin_check_dependencies() {
    if is_macos; then
        require_cmd "netstat" || return 1
    fi
    return 0
}

plugin_declare_options() {
    declare_option "interface" "string" "auto" "Network interface to monitor"
    declare_option "display" "enum" "both" "What to display: both, upload, download"
    declare_option "separator" "string" " | " "Separator between up/down"
    declare_option "speed_width" "number" "7" "Fixed width for each rendered speed value"

    declare_option "icon" "icon" $'\U0000F0B5' "Plugin icon"
    declare_option "icon_download" "icon" $'\U000F01DA' "Icon for download"
    declare_option "icon_upload" "icon" $'\U000F0552' "Icon for upload"

    declare_option "cache_ttl" "number" "2" "Cache duration in seconds"
}

plugin_get_content_type() { printf 'dynamic'; }
plugin_get_presence() { printf 'always'; }
plugin_get_state() { printf 'active'; }
plugin_get_health() { printf 'ok'; }

plugin_get_context() {
    local rx_rate tx_rate
    rx_rate=$(plugin_data_get "rx_rate")
    tx_rate=$(plugin_data_get "tx_rate")

    rx_rate="${rx_rate:-0}"
    tx_rate="${tx_rate:-0}"

    local total=$((rx_rate + tx_rate))
    if (( total == 0 )); then
        printf 'idle'
    elif (( rx_rate > tx_rate )); then
        printf 'downloading'
    elif (( tx_rate > rx_rate )); then
        printf 'uploading'
    else
        printf 'active'
    fi
}

plugin_get_icon() { get_option "icon"; }

_get_active_interface() {
    if is_macos; then
        route -n get default 2>/dev/null | awk '/interface:/ {print $2}'
    else
        ip route | awk '/default/ {print $5; exit}'
    fi
}

_get_network_stats() {
    local interface
    interface=$(get_option "interface")
    [[ "${interface}" == "auto" ]] && interface=$(_get_active_interface)
    [[ -z "${interface}" ]] && return 1

    local rx_bytes tx_bytes
    if is_macos; then
        local netstat_line
        netstat_line=$(netstat -ib -I "${interface}" 2>/dev/null | awk 'NR==2 {print $7, $10}')
        read -r rx_bytes tx_bytes <<< "${netstat_line}"
    elif [[ -f "/sys/class/net/${interface}/statistics/rx_bytes" ]]; then
        rx_bytes=$(cat "/sys/class/net/${interface}/statistics/rx_bytes" 2>/dev/null)
        tx_bytes=$(cat "/sys/class/net/${interface}/statistics/tx_bytes" 2>/dev/null)
    else
        return 1
    fi

    [[ -z "${rx_bytes}" || -z "${tx_bytes}" ]] && return 1

    local cache_ttl=60
    local prev_rx prev_tx prev_time curr_time
    prev_rx=$(cache_get "netspeed_prev_rx" "${cache_ttl}")
    prev_tx=$(cache_get "netspeed_prev_tx" "${cache_ttl}")
    prev_time=$(cache_get "netspeed_prev_time" "${cache_ttl}")
    curr_time=$EPOCHSECONDS

    if [[ -n "${prev_rx}" && -n "${prev_time}" ]]; then
        local time_diff rx_rate tx_rate
        time_diff=$((curr_time - prev_time))
        [[ "${time_diff}" -eq 0 ]] && time_diff=1

        rx_rate=$(((rx_bytes - prev_rx) / time_diff / 1024))
        tx_rate=$(((tx_bytes - prev_tx) / time_diff / 1024))

        (( rx_rate < 0 )) && rx_rate=0
        (( tx_rate < 0 )) && tx_rate=0

        cache_set "netspeed_prev_rx" "${rx_bytes}"
        cache_set "netspeed_prev_tx" "${tx_bytes}"
        cache_set "netspeed_prev_time" "${curr_time}"

        printf '%s|%s' "${rx_rate}" "${tx_rate}"
    else
        cache_set "netspeed_prev_rx" "${rx_bytes}"
        cache_set "netspeed_prev_tx" "${tx_bytes}"
        cache_set "netspeed_prev_time" "${curr_time}"
        printf '0|0'
    fi
}

plugin_collect() {
    local stats
    stats=$(_get_network_stats) || return 1

    IFS='|' read -r rx_rate tx_rate <<< "${stats}"

    plugin_data_set "rx_rate" "${rx_rate}"
    plugin_data_set "tx_rate" "${tx_rate}"
}

_fixed_speed() {
    local rate="${1:-0}" width formatted
    width=$(get_option "speed_width")
    [[ "${width}" =~ ^[0-9]+$ && "${width}" -gt 0 ]] || width=7

    formatted=$(format_speed "${rate}" 1)
    if (( ${#formatted} > width )); then
        printf '%s' "${formatted:0:width}"
    else
        printf "%${width}s" "${formatted}"
    fi
}

plugin_render() {
    local display separator
    display=$(get_option "display")
    separator=$(get_option "separator")

    local rx_rate tx_rate
    rx_rate=$(plugin_data_get "rx_rate")
    tx_rate=$(plugin_data_get "tx_rate")

    local icon_download icon_upload
    icon_download=$(get_option "icon_download")
    icon_upload=$(get_option "icon_upload")

    local parts=()

    if [[ "${display}" == "both" || "${display}" == "download" ]]; then
        parts+=("${icon_download} $(_fixed_speed "${rx_rate}")")
    fi

    if [[ "${display}" == "both" || "${display}" == "upload" ]]; then
        parts+=("${icon_upload} $(_fixed_speed "${tx_rate}")")
    fi

    [[ ${#parts[@]} -gt 0 ]] && join_with_separator "${separator}" "${parts[@]}"
}
