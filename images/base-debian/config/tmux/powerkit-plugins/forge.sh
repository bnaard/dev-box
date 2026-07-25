#!/usr/bin/env bash
# aibox-shipped PowerKit plugin: forge — multi-provider git forge status.
#
# Auto-detects the hosting provider from `git remote get-url origin` and
# renders a "<LABEL> <branch> I<issues> P<prs> D<discussions>" segment with graceful
# degradation when network calls fail or credentials are absent.
#
# Supported providers:
#   github.com          → label GH, API via `gh` CLI
#   @powerkit_plugin_forge_github_hosts (whitespace-separated) → label GH, API via `gh` CLI
#   gitlab.com / gitlab*→ label GL, API via `glab` CLI or curl
#   codeberg.org        → label CB, Gitea-compatible REST API
#   @powerkit_plugin_forge_gitea_hosts (whitespace-separated) → label FJ, Gitea REST API
#   @powerkit_plugin_forge_forgejo_hosts                      → label FJ, Forgejo REST API
#   unknown host        → segment hidden (presence=conditional)
#
# This plugin supersedes the upstream `git` and `github` plugins for default
# aibox installs.  Users who want to pin to the upstream behaviour can disable
# forge and enable git/github explicitly in their aibox.toml
# (customization.tmux.status.elements.forge = false).
#
# Configuration options (all via tmux set -g @powerkit_plugin_forge_<name>):
#   label         Override the provider label (e.g. "MY" for a custom Gitea).
#   show_branch   Show current branch name (default true).
#   show_counts   Fetch and show open issue/PR/discussion counts (default true).
#   timeout       Network call timeout in seconds (default 3).
#   cache_ttl     Cache lifetime in seconds (default 120).
#   github_hosts  Whitespace-separated GitHub hostnames and SSH aliases (default github.com).
#   gitea_hosts   Whitespace-separated hostnames treated as Gitea instances.
#   forgejo_hosts Whitespace-separated hostnames treated as Forgejo instances.
#
# Token environment variables for private Gitea/Forgejo/Codeberg repos:
#   GITEA_TOKEN, FORGEJO_TOKEN, CODEBERG_TOKEN

POWERKIT_ROOT="${POWERKIT_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
. "${POWERKIT_ROOT}/src/contract/plugin_contract.sh"

# =============================================================================
# Plugin Contract: Metadata
# =============================================================================

plugin_get_metadata() {
    metadata_set "id" "forge"
    metadata_set "name" "Forge"
    metadata_set "description" "Display git forge branch plus open issue, PR, and discussion counts (multi-provider)"
}

# =============================================================================
# Plugin Contract: Options
# =============================================================================

plugin_declare_options() {
    declare_option "label"         "string" ""    "Segment label override (empty = auto from provider)"
    declare_option "show_branch"   "bool"   "true" "Show the current git branch"
    declare_option "show_counts"   "bool"   "true" "Show open issue, PR, and discussion counts"
    declare_option "timeout"       "number" "3"   "Network call timeout in seconds"
    declare_option "cache_ttl"     "number" "120" "Cache duration in seconds"
    declare_option "github_hosts"  "string" "github.com" "Whitespace-separated GitHub hostnames and SSH aliases"
    declare_option "gitea_hosts"   "string" ""    "Whitespace-separated Gitea hostnames"
    declare_option "forgejo_hosts" "string" ""    "Whitespace-separated Forgejo hostnames"
}

# =============================================================================
# Plugin Contract: Presence / type
# =============================================================================

plugin_get_content_type() { printf 'dynamic'; }
plugin_get_presence()     { printf 'conditional'; }

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
    local lbl
    lbl="$(get_option label)"
    if [[ -n "$lbl" ]]; then
        printf '%s' "$lbl"
    else
        printf '%s' "$(plugin_data_get label)"
    fi
}

# =============================================================================
# Internal helpers
# =============================================================================

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

_current_branch() {
    local branch
    branch="$(_git branch --show-current)"
    if [[ -n "$branch" ]]; then
        printf '%s' "$branch"
        return 0
    fi
    _git rev-parse --short HEAD
}

# Detect provider from a remote URL.
# Sets globals: _provider, _label, _owner, _repo_name, _api_base
_detect_provider() {
    local url="$1"
    local github_hosts gitea_hosts forgejo_hosts host path

    github_hosts="$(get_option github_hosts)"
    gitea_hosts="$(get_option gitea_hosts)"
    forgejo_hosts="$(get_option forgejo_hosts)"

    # Normalise: strip trailing .git, extract host + path
    case "$url" in
        https://*/*)
            url="${url#https://}"
            ;;
        http://*/*)
            url="${url#http://}"
            ;;
        git@*:*)
            # git@host:owner/repo.git  →  host/owner/repo.git
            url="${url#git@}"
            url="${url/://}"
            ;;
        ssh://git@*/*)
            url="${url#ssh://git@}"
            ;;
        *)
            return 1
            ;;
    esac
    url="${url%.git}"

    # Split host from path
    host="${url%%/*}"
    path="${url#*/}"

    # owner/repo from path
    _owner="${path%%/*}"
    _repo_name="${path#*/}"
    _repo_name="${_repo_name%%/*}"
    [[ -n "$_owner" && -n "$_repo_name" ]] || return 1

    # Provider detection
    case "$host" in
        github.com)
            _provider="github"
            _label="GH"
            _api_base="https://api.github.com"
            ;;
        gitlab.com|gitlab.*)
            _provider="gitlab"
            _label="GL"
            _api_base="https://gitlab.com"
            ;;
        codeberg.org)
            _provider="codeberg"
            _label="CB"
            _api_base="https://codeberg.org/api/v1"
            ;;
        *)
            # Check user-configured lists before giving up
            local h
            for h in $github_hosts; do
                if [[ "$host" == "$h" ]]; then
                    _provider="github"
                    _label="GH"
                    # GitHub SSH aliases still use the public GitHub API and gh CLI.
                    _api_base="https://api.github.com"
                    return 0
                fi
            done
            for h in $forgejo_hosts; do
                if [[ "$host" == "$h" ]]; then
                    _provider="forgejo"
                    _label="FJ"
                    _api_base="https://${host}/api/v1"
                    return 0
                fi
            done
            for h in $gitea_hosts; do
                if [[ "$host" == "$h" ]]; then
                    _provider="gitea"
                    _label="GT"
                    _api_base="https://${host}/api/v1"
                    return 0
                fi
            done
            return 1
            ;;
    esac
    return 0
}

# URL-encode a string (slash-safe for path segments).
_urlencode() {
    local string="$1" encoded="" c
    local i=0
    while [[ $i -lt ${#string} ]]; do
        c="${string:$i:1}"
        case "$c" in
            [a-zA-Z0-9._~-]) encoded+="$c" ;;
            *) printf -v encoded '%s%%%02X' "$encoded" "'$c" ;;
        esac
        ((i++))
    done
    printf '%s' "$encoded"
}

# =============================================================================
# Count helpers
# =============================================================================

# GitHub: use independent GraphQL totalCount queries so the values are exact
# and one unavailable metric does not suppress the other two.
_gh_count_github() {
    local kind="$1" owner="$2" name="$3" timeout_s="$4"
    has_cmd gh || return 1
    local count query jq_filter
    case "$kind" in
        issues)
            query='query($owner:String!,$name:String!){repository(owner:$owner,name:$name){issues(states:OPEN){totalCount}}}'
            jq_filter='.data.repository.issues.totalCount'
            ;;
        prs)
            query='query($owner:String!,$name:String!){repository(owner:$owner,name:$name){pullRequests(states:OPEN){totalCount}}}'
            jq_filter='.data.repository.pullRequests.totalCount'
            ;;
        discussions)
            query='query($owner:String!,$name:String!){repository(owner:$owner,name:$name){discussions(first:1,states:OPEN){totalCount}}}'
            jq_filter='.data.repository.discussions.totalCount'
            ;;
        *) return 1 ;;
    esac
    if has_cmd timeout; then
        count="$(timeout "${timeout_s}s" gh api graphql \
            -f query="$query" -F owner="$owner" -F name="$name" \
            --jq "$jq_filter" 2>/dev/null)" || return 1
    else
        count="$(gh api graphql \
            -f query="$query" -F owner="$owner" -F name="$name" \
            --jq "$jq_filter" 2>/dev/null)" || return 1
    fi
    [[ "$count" =~ ^[0-9]+$ ]] || return 1
    printf '%s' "$count"
}

# GitLab: prefer glab CLI; fall back to curl against the REST API.
_gh_count_gitlab() {
    local kind="$1" owner="$2" name="$3" timeout_s="$4" api_base="$5"
    local count header_count

    if has_cmd glab; then
        case "$kind" in
            issues)
                if has_cmd timeout; then
                    count="$(timeout "${timeout_s}s" glab issue list --opened --output ids 2>/dev/null | wc -l)" || return 1
                else
                    count="$(glab issue list --opened --output ids 2>/dev/null | wc -l)" || return 1
                fi
                ;;
            prs)
                if has_cmd timeout; then
                    count="$(timeout "${timeout_s}s" glab mr list --opened --output ids 2>/dev/null | wc -l)" || return 1
                else
                    count="$(glab mr list --opened --output ids 2>/dev/null | wc -l)" || return 1
                fi
                ;;
            *) return 1 ;;
        esac
        count="${count//[^0-9]/}"
        [[ "$count" =~ ^[0-9]+$ ]] || return 1
        printf '%s' "$count"
        return 0
    fi

    # curl fallback: read X-Total response header
    local encoded_path project_path api_kind
    project_path="${owner}/${name}"
    encoded_path="$(_urlencode "$project_path")"
    case "$kind" in
        issues) api_kind="issues?state=opened&per_page=1" ;;
        prs)    api_kind="merge_requests?state=opened&per_page=1" ;;
        *) return 1 ;;
    esac
    header_count="$(curl -fsSL --max-time "${timeout_s}" -I \
        "${api_base}/api/v4/projects/${encoded_path}/${api_kind}" 2>/dev/null \
        | grep -i '^x-total:' | tr -dc '0-9')" || return 1
    [[ "$header_count" =~ ^[0-9]+$ ]] || return 1
    printf '%s' "$header_count"
}

# Gitea/Forgejo/Codeberg: curl against the Gitea-compatible REST API.
# Uses pagination header X-Total-Count; falls back to unauthenticated for
# public repos.
_gh_count_gitea() {
    local kind="$1" owner="$2" name="$3" timeout_s="$4" api_base="$5" provider="$6"
    local token api_url header_count auth_header

    case "$provider" in
        codeberg) token="${CODEBERG_TOKEN:-}" ;;
        forgejo)  token="${FORGEJO_TOKEN:-}" ;;
        *)        token="${GITEA_TOKEN:-}" ;;
    esac

    case "$kind" in
        issues) api_url="${api_base}/repos/${owner}/${name}/issues?state=open&type=issues&limit=1" ;;
        prs)    api_url="${api_base}/repos/${owner}/${name}/pulls?state=open&limit=1" ;;
        *) return 1 ;;
    esac

    local curl_args=(-fsSL --max-time "${timeout_s}" -I)
    if [[ -n "$token" ]]; then
        curl_args+=(-H "Authorization: token ${token}")
    fi

    header_count="$(curl "${curl_args[@]}" "$api_url" 2>/dev/null \
        | grep -i '^x-total-count:' | tr -dc '0-9')" || return 1
    [[ "$header_count" =~ ^[0-9]+$ ]] || return 1
    printf '%s' "$header_count"
}

# =============================================================================
# Plugin Contract: collect
# =============================================================================

plugin_collect() {
    local remote url provider label owner repo_name api_base
    local branch show_counts timeout_s issues prs discussions

    # Must be inside a git repo
    _git rev-parse --is-inside-work-tree >/dev/null || return 0

    remote="$(_git config --get remote.origin.url)"
    [[ -n "$remote" ]] || return 0

    # Declare globals used by _detect_provider
    local _provider="" _label="" _owner="" _repo_name="" _api_base=""
    _detect_provider "$remote" || return 0

    provider="$_provider"
    label="$(get_option label)"
    [[ -z "$label" ]] && label="$_label"
    owner="$_owner"
    repo_name="$_repo_name"
    api_base="$_api_base"

    branch="$(_current_branch)"

    plugin_data_set "provider"   "$provider"
    plugin_data_set "label"      "$label"
    plugin_data_set "branch"     "$branch"

    show_counts="$(get_option show_counts)"
    [[ "$show_counts" == "true" ]] || return 0

    timeout_s="$(get_option timeout)"

    case "$provider" in
        github)
            issues="$(_gh_count_github issues "$owner" "$repo_name" "$timeout_s")" || issues=""
            prs="$(_gh_count_github prs    "$owner" "$repo_name" "$timeout_s")" || prs=""
            discussions="$(_gh_count_github discussions "$owner" "$repo_name" "$timeout_s")" || discussions=""
            ;;
        gitlab)
            issues="$(_gh_count_gitlab issues "$owner" "$repo_name" "$timeout_s" "$api_base")" || issues=""
            prs="$(_gh_count_gitlab prs    "$owner" "$repo_name" "$timeout_s" "$api_base")" || prs=""
            ;;
        codeberg|forgejo|gitea)
            issues="$(_gh_count_gitea issues "$owner" "$repo_name" "$timeout_s" "$api_base" "$provider")" || issues=""
            prs="$(_gh_count_gitea prs    "$owner" "$repo_name" "$timeout_s" "$api_base" "$provider")" || prs=""
            ;;
    esac

    [[ -n "$issues" ]] && plugin_data_set "issues" "$issues"
    [[ -n "$prs" ]]    && plugin_data_set "prs"    "$prs"
    [[ -n "$discussions" ]] && plugin_data_set "discussions" "$discussions"
}

# =============================================================================
# Plugin Contract: render
# =============================================================================

plugin_render() {
    local label branch issues prs discussions show_branch
    local -a parts=()

    # Only render when a provider was detected
    [[ -n "$(plugin_data_get provider)" ]] || return 0

    label="$(plugin_data_get label)"
    [[ -n "$label" ]] && parts+=("$label")

    show_branch="$(get_option show_branch)"
    branch="$(plugin_data_get branch)"
    if [[ "$show_branch" == "true" && -n "$branch" ]]; then
        parts+=("$branch")
    fi

    issues="$(plugin_data_get issues)"
    prs="$(plugin_data_get prs)"
    discussions="$(plugin_data_get discussions)"
    [[ -n "$issues" ]] && parts+=("I${issues}")
    [[ -n "$prs" ]]    && parts+=("P${prs}")
    [[ -n "$discussions" ]] && parts+=("D${discussions}")

    local IFS=' '
    printf '%s' "${parts[*]}"
}
