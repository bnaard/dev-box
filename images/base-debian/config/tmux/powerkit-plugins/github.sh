#!/usr/bin/env bash
# PowerKit plugin: aibox GitHub repository status.
#
# Reads local git state synchronously and polls GitHub CLI only for cached issue
# and PR counts. If gh is unavailable or unauthenticated, the segment still
# renders the current GitHub repository branch without blocking the status line.

POWERKIT_ROOT="${POWERKIT_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
. "${POWERKIT_ROOT}/src/contract/plugin_contract.sh"

plugin_get_metadata() {
    metadata_set "id" "github"
    metadata_set "name" "GitHub"
    metadata_set "description" "Display GitHub repository branch plus open issue and PR counts"
}

plugin_declare_options() {
    declare_option "icon" "string" "GH" "Segment label"
    declare_option "show_branch" "bool" "true" "Show the current git branch"
    declare_option "show_counts" "bool" "true" "Show open issue and PR counts via gh"
    declare_option "timeout" "number" "3" "GitHub CLI timeout in seconds"
    declare_option "limit" "number" "1000" "Maximum issues/PRs counted by gh"
    declare_option "cache_ttl" "number" "120" "Cache duration in seconds"
}

plugin_get_content_type() { printf 'dynamic'; }
plugin_get_presence() { printf 'conditional'; }

_repo_dir() {
    local dir="${PWD:-/workspace}" pane_dir
    if has_cmd tmux; then
        pane_dir="$(tmux display-message -p "#{pane_current_path}" 2>/dev/null)" || pane_dir=""
        [[ -n "$pane_dir" && -d "$pane_dir" ]] && dir="$pane_dir"
    fi
    printf '%s' "$dir"
}

_git() {
    git -C "$(_repo_dir)" "$@" 2>/dev/null
}

_github_repo_from_remote() {
    local url="$1" owner name rest
    case "$url" in
        https://github.com/*/*)
            url="${url#https://github.com/}"
            ;;
        http://github.com/*/*)
            url="${url#http://github.com/}"
            ;;
        git@github.com:*)
            url="${url#git@github.com:}"
            ;;
        ssh://git@github.com/*/*)
            url="${url#ssh://git@github.com/}"
            ;;
        *)
            return 1
            ;;
    esac
    url="${url%.git}"
    IFS='/' read -r owner name rest <<< "$url"
    [[ -n "$owner" && -n "$name" ]] || return 1
    name="${name%.git}"
    printf '%s/%s' "$owner" "$name"
}

_current_branch() {
    local branch
    branch="$(_git branch --show-current)"
    if [[ -n "$branch" ]]; then
        printf '%s' "$branch"
        return 0
    fi
    _git rev-parse --short HEAD
}

_gh_count() {
    local kind="$1" repo="$2" timeout_s="$3" limit="$4" count
    [[ -n "$repo" ]] || return 1
    has_cmd gh || return 1

    case "$kind" in
        issues)
            if has_cmd timeout; then
                count="$(timeout "${timeout_s}s" gh issue list --repo "$repo" --state open --limit "$limit" --json number --jq 'length' 2>/dev/null)" || return 1
            else
                count="$(gh issue list --repo "$repo" --state open --limit "$limit" --json number --jq 'length' 2>/dev/null)" || return 1
            fi
            ;;
        prs)
            if has_cmd timeout; then
                count="$(timeout "${timeout_s}s" gh pr list --repo "$repo" --state open --limit "$limit" --json number --jq 'length' 2>/dev/null)" || return 1
            else
                count="$(gh pr list --repo "$repo" --state open --limit "$limit" --json number --jq 'length' 2>/dev/null)" || return 1
            fi
            ;;
        *)
            return 1
            ;;
    esac

    [[ "$count" =~ ^[0-9]+$ ]] || return 1
    printf '%s' "$count"
}

plugin_collect() {
    local remote repo branch show_counts timeout_s limit issues prs
    _git rev-parse --is-inside-work-tree >/dev/null || return 0

    remote="$(_git config --get remote.origin.url)"
    repo="$(_github_repo_from_remote "$remote")" || repo=""
    [[ -z "$repo" ]] && return 0

    branch="$(_current_branch)"
    plugin_data_set "repo" "$repo"
    plugin_data_set "branch" "$branch"

    show_counts="$(get_option show_counts)"
    [[ "$show_counts" == "true" ]] || return 0

    timeout_s="$(get_option timeout)"
    limit="$(get_option limit)"
    issues="$(_gh_count issues "$repo" "$timeout_s" "$limit")" || issues=""
    prs="$(_gh_count prs "$repo" "$timeout_s" "$limit")" || prs=""
    [[ -n "$issues" ]] && plugin_data_set "issues" "$issues"
    [[ -n "$prs" ]] && plugin_data_set "prs" "$prs"
}

plugin_get_state() {
    [[ -n "$(plugin_data_get repo)" ]] && printf 'active' || printf 'inactive'
}

plugin_get_health() { printf 'ok'; }

plugin_get_context() {
    local repo
    repo="$(plugin_data_get repo)"
    printf '%s' "${repo:-none}"
}

plugin_get_icon() { get_option "icon"; }

plugin_render() {
    local show_branch branch issues prs
    local -a parts=()

    [[ -n "$(plugin_data_get repo)" ]] || return 0

    show_branch="$(get_option show_branch)"
    branch="$(plugin_data_get branch)"
    if [[ "$show_branch" == "true" && -n "$branch" ]]; then
        parts+=("$branch")
    fi

    issues="$(plugin_data_get issues)"
    prs="$(plugin_data_get prs)"
    [[ -n "$issues" ]] && parts+=("I${issues}")
    [[ -n "$prs" ]] && parts+=("P${prs}")

    local IFS=' '
    printf '%s' "${parts[*]}"
}
