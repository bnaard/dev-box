#!/usr/bin/env bash
set -u

ROOT="${ROOT:-$(pwd)}"
OUT="${OUT:-$ROOT/.aibox/stale-runtime-diagnosis-$(date -u +%Y%m%dT%H%M%SZ).txt}"
CONTAINER="${CONTAINER:-aibox}"
IMAGE="${IMAGE:-aibox-devcontainer:latest}"

mkdir -p "$(dirname "$OUT")"

section() {
  printf '\n===== %s =====\n' "$1"
}

run() {
  printf '\n$ %s\n' "$*"
  "$@" 2>&1 || printf '[exit %s]\n' "$?"
}

show_file() {
  local path="$1"
  local lines="${2:-160}"
  printf '\n--- %s ---\n' "$path"
  if [ -f "$path" ]; then
    sed -n "1,${lines}p" "$path" 2>&1 || true
  else
    printf 'missing\n'
  fi
}

grep_file() {
  local path="$1"
  local pattern="$2"
  printf '\n--- grep %s in %s ---\n' "$pattern" "$path"
  if [ -f "$path" ]; then
    grep -nE "$pattern" "$path" 2>&1 || true
  else
    printf 'missing\n'
  fi
}

docker_available() {
  command -v docker >/dev/null 2>&1
}

{
  section "diagnostic metadata"
  printf 'timestamp_utc=%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  printf 'host=%s\n' "$(hostname 2>/dev/null || true)"
  printf 'user=%s\n' "$(id -un 2>/dev/null || true)"
  printf 'root=%s\n' "$ROOT"
  printf 'out=%s\n' "$OUT"
  printf 'container=%s\n' "$CONTAINER"
  printf 'image=%s\n' "$IMAGE"

  section "host aibox binaries"
  if command -v aibox >/dev/null 2>&1; then
    run command -v aibox
    run aibox --version
  else
    printf 'aibox not found on PATH\n'
  fi
  run which -a aibox
  run ls -l "$(command -v aibox 2>/dev/null || printf /nonexistent)"
  if [ -x "$ROOT/cli/target/debug/aibox" ]; then
    run "$ROOT/cli/target/debug/aibox" --version
    run file "$ROOT/cli/target/debug/aibox"
  else
    printf 'repo debug binary missing or not executable: %s\n' "$ROOT/cli/target/debug/aibox"
  fi

  section "git state"
  if [ -d "$ROOT/.git" ]; then
    run git -C "$ROOT" status --short --branch
    run git -C "$ROOT" log -8 --oneline --decorate
    run git -C "$ROOT" tag --points-at HEAD
  else
    printf 'no .git directory under root\n'
  fi

  section "project version files"
  show_file "$ROOT/aibox.lock" 80
  grep_file "$ROOT/aibox.toml" '^(apiVersion|kind|\[metadata\]|\[image\]|\[aibox\]|\[customization\.zellij_status\]|mode =|version =|base =|profile =|name =)'
  show_file "$ROOT/aibox.toml" 260
  show_file "$ROOT/.devcontainer/devcontainer.json" 140
  show_file "$ROOT/.devcontainer/docker-compose.yml" 180
  grep_file "$ROOT/.devcontainer/Dockerfile" 'FROM ghcr|LABEL aibox.version|Addon:|lazygit|cargo-audit|x86_64-cross|gcc-x86-64|mupdf|ffmpeg|imagemagick|ghostscript|RUN apt-get|COPY --from'
  show_file "$ROOT/.devcontainer/Dockerfile" 260

  section "runtime-home zellij projection on host mount"
  show_file "$ROOT/.aibox-home/.config/zellij/layouts/ai.kdl" 160
  show_file "$ROOT/.aibox-home/.config/zellij/layouts/aibox-status-visible.kdl" 80
  show_file "$ROOT/.aibox-home/.config/zellij/layouts/aibox-status-hidden.kdl" 80
  show_file "$ROOT/.aibox-home/.config/zellij/config.kdl" 220
  grep_file "$ROOT/.aibox-home/.config/zellij/layouts/ai.kdl" 'aibox-status|status-bar|wasm|/usr/local/share|/workspace/.aibox-home|command "bash"|--watch'
  grep_file "$ROOT/.aibox-home/.config/zellij/config.kdl" 'aibox_toggle_runtime|aibox-status|default_layout|theme '

  section "aibox command log tail"
  if [ -f "$ROOT/.aibox/aibox.log" ]; then
    tail -120 "$ROOT/.aibox/aibox.log" 2>&1 || true
  else
    printf 'missing %s\n' "$ROOT/.aibox/aibox.log"
  fi

  section "docker availability"
  if docker_available; then
    run docker version
    run docker context ls
    run docker ps -a --filter "name=^/${CONTAINER}$" --no-trunc
    run docker ps -a --filter "label=com.docker.compose.project=aibox" --no-trunc
    run docker images --digests "$IMAGE"
  else
    printf 'docker not found on host PATH\n'
  fi

  if docker_available; then
    section "docker inspect container"
    run docker inspect "$CONTAINER"
    section "docker inspect image"
    run docker image inspect "$IMAGE"
    section "docker history image"
    run docker history --no-trunc "$IMAGE"

    section "live container markers"
    run docker exec "$CONTAINER" /bin/sh -lc 'printf "whoami="; whoami; printf "pwd="; pwd; printf "aibox-version-file="; cat /etc/aibox-version 2>/dev/null || true; printf "\npid1="; tr "\0" " " </proc/1/cmdline; printf "\n"; printf "pids.current="; cat /sys/fs/cgroup/pids.current 2>/dev/null || true; printf "memory.events\n"; cat /sys/fs/cgroup/memory.events 2>/dev/null || true'
    run docker exec "$CONTAINER" /bin/sh -lc 'command -v zellij || true; zellij --version 2>/dev/null || true; command -v aibox-status || true; command -v lazygit || true; command -v gh || true; command -v cargo-audit || true; command -v x86_64-linux-gnu-gcc || true'
    run docker exec "$CONTAINER" /bin/sh -lc 'dpkg-query -W -f="${Package} ${Version} ${Status}\n" lazygit gh gcc gcc-x86-64-linux-gnu libc6-dev-amd64-cross imagemagick ghostscript ffmpeg mupdf-tools 2>/dev/null || true'
    run docker exec "$CONTAINER" /bin/sh -lc 'du -sh /usr /home/aibox/.cargo /home/aibox/.rustup /workspace/.aibox-home 2>/dev/null || true'

    section "live mounted zellij files from inside container"
    run docker exec "$CONTAINER" /bin/sh -lc 'sed -n "1,160p" /home/aibox/.config/zellij/layouts/ai.kdl 2>&1 || true'
    run docker exec "$CONTAINER" /bin/sh -lc 'sed -n "1,80p" /home/aibox/.config/zellij/layouts/aibox-status-visible.kdl 2>&1 || true'
    run docker exec "$CONTAINER" /bin/sh -lc 'sed -n "1,220p" /home/aibox/.config/zellij/config.kdl 2>&1 || true'
    run docker exec "$CONTAINER" /bin/sh -lc 'grep -RInE "aibox-status|status-bar|wasm|/usr/local/share|/workspace/.aibox-home|--watch" /home/aibox/.config/zellij 2>/dev/null || true'

    section "live zellij sessions and processes"
    run docker exec "$CONTAINER" /bin/sh -lc 'zellij list-sessions 2>/dev/null || true'
    run docker exec "$CONTAINER" /bin/sh -lc 'if command -v ps >/dev/null 2>&1; then ps -eo pid,ppid,stat,comm,args | grep -E "zellij|aibox-status|codex|bwrap" | grep -v grep; else for s in /proc/[0-9]*/stat; do [ -r "$s" ] || continue; line=$(cat "$s"); comm=${line#*(}; comm=${comm%%)*}; case "$comm" in zellij|aibox-status|codex|bwrap) pid=${s#/proc/}; pid=${pid%/stat}; printf "%s %s\n" "$pid" "$line";; esac; done; fi'
  fi

  section "done"
  printf 'wrote=%s\n' "$OUT"
} >"$OUT" 2>&1

perl -0pi -e 's/(GH_TOKEN=)[^",\n]+/${1}<redacted>/g; s/(GITHUB_TOKEN=)[^",\n]+/${1}<redacted>/g; s/("GH_TOKEN":\s*")[^"]+/${1}<redacted>/g; s/("GITHUB_TOKEN":\s*")[^"]+/${1}<redacted>/g' "$OUT" 2>/dev/null || true
printf '%s\n' "$OUT"
