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
    '        github_hosts) printf "%s" "${TEST_GITHUB_HOSTS:-github.com}" ;;' \
    '        gitea_hosts|forgejo_hosts) printf "" ;;' \
    '        *) printf "" ;;' \
    '    esac' \
    '}' > "$tmpdir/src/contract/plugin_contract.sh"

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
[[ "$(<"$TEST_GH_ARGS")" == "issue list --repo bnaard/internal --state open --json number --jq length" ]]

TEST_GITHUB_HOSTS="github.com"
_provider="" _label="" _owner="" _repo_name="" _api_base=""
if _detect_provider "git@unconfigured.example:owner/repo.git"; then
    echo "unconfigured hosts must not be identified as GitHub" >&2
    exit 1
fi

echo "forge plugin tests passed"
