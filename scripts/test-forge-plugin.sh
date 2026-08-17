#!/usr/bin/env bash
# Focused regression tests for the aibox-shipped PowerKit forge plugin.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmpdir="$(mktemp -d)"
trap 'rm -rf -- "$tmpdir"' EXIT

mkdir -p "$tmpdir/src/contract"
printf '%s\n' \
    'get_option() {' \
    '    case "$1" in' \
    '        show_branch) printf "true" ;;' \
    '        github_hosts) printf "%s" "${TEST_GITHUB_HOSTS:-github.com}" ;;' \
    '        gitea_hosts|forgejo_hosts) printf "" ;;' \
    '        *) printf "" ;;' \
    '    esac' \
    '}' \
    'declare -Ag TEST_PLUGIN_DATA=()' \
    'plugin_data_set() { TEST_PLUGIN_DATA["$1"]="$2"; }' \
    'plugin_data_get() { printf "%s" "${TEST_PLUGIN_DATA[$1]:-}"; }' \
    > "$tmpdir/src/contract/plugin_contract.sh"

POWERKIT_ROOT="$tmpdir"
export POWERKIT_ROOT
# shellcheck source=../images/base-debian/config/tmux/powerkit-plugins/forge.sh
source "$repo_root/images/base-debian/config/tmux/powerkit-plugins/forge.sh"

assert_provider() {
    local remote="$1" expected_provider="$2" expected_owner="$3" expected_repo="$4"
    _provider="" _label="" _owner="" _repo_name="" _api_base=""
    _detect_provider "$remote"
    [[ "$_provider" == "$expected_provider" ]]
    [[ "$_owner" == "$expected_owner" ]]
    [[ "$_repo_name" == "$expected_repo" ]]
    [[ "$_api_base" == "https://api.github.com" ]]
}

TEST_GITHUB_HOSTS="github.com"
assert_provider "git@github.com:owner/repo.git" github owner repo

TEST_GITHUB_HOSTS="github.com github-bnaard"
assert_provider "git@github-bnaard:bnaard/internal.git" github bnaard internal
assert_provider "ssh://git@github-bnaard/owner/repo.git" github owner repo

mkdir -p "$tmpdir/bin"
printf '%s\n' \
    '#!/usr/bin/env bash' \
    'printf "%s\\n" "$*" > "$TEST_GH_ARGS"' \
    'printf "7"' > "$tmpdir/bin/gh"
chmod +x "$tmpdir/bin/gh"
PATH="$tmpdir/bin:$PATH"
has_cmd() { command -v "$1" >/dev/null; }
TEST_GH_ARGS="$tmpdir/gh-args"
export TEST_GH_ARGS
[[ "$(_gh_count_github issues bnaard internal 1)" == "7" ]]
issue_args="$(<"$TEST_GH_ARGS")"
[[ "$issue_args" == *"api graphql"* ]]
[[ "$issue_args" == *"issues(states:OPEN)"* ]]
[[ "$issue_args" == *"--jq .data.repository.issues.totalCount"* ]]

[[ "$(_gh_count_github discussions bnaard internal 1)" == "7" ]]
discussion_args="$(<"$TEST_GH_ARGS")"
[[ "$discussion_args" == *"api graphql"* ]]
[[ "$discussion_args" == *"states:OPEN"* ]]
[[ "$discussion_args" == *"-F owner=bnaard -F name=internal"* ]]
[[ "$discussion_args" == *"--jq .data.repository.discussions.totalCount"* ]]

[[ "$(_gh_count_github prs bnaard internal 1)" == "7" ]]
pr_args="$(<"$TEST_GH_ARGS")"
[[ "$pr_args" == *"pullRequests(states:OPEN)"* ]]
[[ "$pr_args" == *"--jq .data.repository.pullRequests.totalCount"* ]]

_git() {
    [[ "$*" == "describe --tags --abbrev=0 --match v[0-9]* HEAD" ]] || return 1
    printf 'v0.33.0'
}
[[ "$(_current_release_tag)" == "v0.33.0" ]]

plugin_data_set provider github
plugin_data_set label GH
plugin_data_set branch main
plugin_data_set release_tag v0.33.0
plugin_data_set issues 2
plugin_data_set prs 3
plugin_data_set discussions 4
[[ "$(plugin_render)" == "GH main v0.33.0 I2 P3 D4" ]]

TEST_GITHUB_HOSTS="github.com"
_provider="" _label="" _owner="" _repo_name="" _api_base=""
if _detect_provider "git@unconfigured.example:owner/repo.git"; then
    echo "unconfigured hosts must not be identified as GitHub" >&2
    exit 1
fi

echo "forge plugin tests passed"
