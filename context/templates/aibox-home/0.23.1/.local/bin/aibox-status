#!/usr/bin/env bash
set -euo pipefail

format_bytes() {
  local bytes="${1:-0}"
  awk -v bytes="$bytes" 'BEGIN {
    split("B KiB MiB GiB TiB", units, " ");
    value = bytes + 0;
    idx = 1;
    while (value >= 1024 && idx < 5) {
      value = value / 1024;
      idx++;
    }
    if (idx == 1) {
      printf "%d %s", value, units[idx];
    } else {
      printf "%.1f %s", value, units[idx];
    }
  }'
}

read_memory_current() {
  if [ -r /sys/fs/cgroup/memory.current ]; then
    cat /sys/fs/cgroup/memory.current
  else
    printf '0'
  fi
}

read_memory_max() {
  if [ ! -r /sys/fs/cgroup/memory.max ]; then
    printf 'unavailable'
    return
  fi
  local max
  max="$(cat /sys/fs/cgroup/memory.max)"
  if [ "$max" = "max" ]; then
    printf 'unlimited'
  else
    format_bytes "$max"
  fi
}

read_oom_kill() {
  if [ -r /sys/fs/cgroup/memory.events ]; then
    awk '$1 == "oom_kill" { print $2; found=1 } END { if (!found) print 0 }' /sys/fs/cgroup/memory.events
  else
    printf '0'
  fi
}

read_memory_event() {
  local event_name="${1:-}"
  if [ -r /sys/fs/cgroup/memory.events ]; then
    awk -v name="$event_name" '$1 == name { print $2; found=1 } END { if (!found) print 0 }' /sys/fs/cgroup/memory.events
  else
    printf '0'
  fi
}

read_cpu_throttling() {
  if [ -r /sys/fs/cgroup/cpu.stat ]; then
    awk '
      $1 == "nr_throttled" { throttled=$2 }
      $1 == "throttled_usec" { throttled_usec=$2 }
      END {
        if (throttled == "") throttled = 0;
        if (throttled_usec == "") throttled_usec = 0;
        printf "%s/%ss", throttled, int(throttled_usec / 1000000);
      }
    ' /sys/fs/cgroup/cpu.stat
  else
    printf 'n/a'
  fi
}

read_container_uptime() {
  if [ -r /proc/uptime ] && [ -r /proc/1/stat ]; then
    awk -v hz="$(getconf CLK_TCK 2>/dev/null || printf 100)" '
      NR == FNR { uptime=$1; next }
      {
        elapsed = int(uptime - ($22 / hz));
        if (elapsed < 0) elapsed = 0;
        days = int(elapsed / 86400);
        hours = int((elapsed % 86400) / 3600);
        mins = int((elapsed % 3600) / 60);
        if (days > 0) printf "%dd%dh", days, hours;
        else if (hours > 0) printf "%dh%dm", hours, mins;
        else printf "%dm", mins;
      }
    ' /proc/uptime /proc/1/stat
  else
    printf 'n/a'
  fi
}

count_processes() {
  local count=0
  local entry
  for entry in /proc/[0-9]*; do
    [ -e "$entry" ] || continue
    count=$((count + 1))
  done
  printf '%s' "$count"
}

count_ai_agents() {
  local count=0
  local entry cmdline
  for entry in /proc/[0-9]*; do
    [ -r "$entry/cmdline" ] || continue
    cmdline="$(tr '\0' ' ' < "$entry/cmdline" 2>/dev/null || true)"
    case "$(printf '%s' "$cmdline" | tr '[:upper:]' '[:lower:]')" in
      *'/codex '*|*' codex '*|*'claude '*|*'gemini '*|*'aider '*|*'copilot '*|*'opencode '*|*'hermes '*) count=$((count + 1)) ;;
    esac
  done
  printf '%s' "$count"
}

count_processkit_mcp_python() {
  local count=0
  local entry cmdline
  for entry in /proc/[0-9]*; do
    [ -r "$entry/cmdline" ] || continue
    cmdline="$(tr '\0' ' ' < "$entry/cmdline" 2>/dev/null || true)"
    case "$(printf '%s' "$cmdline" | tr '[:upper:]' '[:lower:]')" in
      *python*processkit*mcp*|*processkit*mcp*python*) count=$((count + 1)) ;;
    esac
  done
  printf '%s' "$count"
}

read_processkit_mode() {
  local gateway=0 granular=0
  local entry cmdline
  for entry in /proc/[0-9]*; do
    [ -r "$entry/cmdline" ] || continue
    cmdline="$(tr '\0' ' ' < "$entry/cmdline" 2>/dev/null || true)"
    case "$(printf '%s' "$cmdline" | tr '[:upper:]' '[:lower:]')" in
      *processkit-gateway*mcp*server.py*) gateway=1 ;;
      *processkit*mcp*server.py*) granular=$((granular + 1)) ;;
    esac
  done
  if [ "$gateway" -eq 1 ]; then
    printf 'gateway'
  elif [ "$granular" -gt 0 ]; then
    printf 'granular'
  else
    printf 'none'
  fi
}

read_disk_available() {
  df -h /workspace 2>/dev/null | awk 'NR == 2 { print $4; found=1 } END { if (!found) print "n/a" }'
}

read_git_state() {
  if git -C /workspace rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    if [ -n "$(git -C /workspace status --porcelain --untracked-files=normal 2>/dev/null | sed -n '1p')" ]; then
      printf 'dirty'
    else
      printf 'clean'
    fi
  else
    printf 'n/a'
  fi
}

count_migrations() {
  local count=0
  local dir file
  for dir in /workspace/context/migrations/pending /workspace/context/migrations/in-progress; do
    [ -d "$dir" ] || continue
    for file in "$dir"/*.md; do
      [ -e "$file" ] || continue
      count=$((count + 1))
    done
  done
  printf '%s' "$count"
}

read_status_values() {
  status_memory_current="$(format_bytes "$(read_memory_current)")"
  status_memory_max="$(read_memory_max)"
  status_oom_kill="$(read_oom_kill)"
  status_memory_high="$(read_memory_event high)"
  status_memory_max_events="$(read_memory_event max)"
  status_cpu_throttling="$(read_cpu_throttling)"
  status_processes="$(count_processes)"
  status_ai_agents="$(count_ai_agents)"
  status_processkit_mcp="$(count_processkit_mcp_python)"
  status_processkit_mode="$(read_processkit_mode)"
  status_disk_available="$(read_disk_available)"
  status_container_uptime="$(read_container_uptime)"
}

read_project_values() {
  status_git_state="$(read_git_state)"
  status_migrations="$(count_migrations)"
}

print_status_plain() {
  printf 'MEM %s/%s oom%s hi%s max%s | CPU thr%s | PROC %s ai%s pk:%s/%s | FS %s | UP %s | PROJ git:%s mig%s' \
    "$status_memory_current" \
    "$status_memory_max" \
    "$status_oom_kill" \
    "$status_memory_high" \
    "$status_memory_max_events" \
    "$status_cpu_throttling" \
    "$status_processes" \
    "$status_ai_agents" \
    "$status_processkit_mode" \
    "$status_processkit_mcp" \
    "$status_disk_available" \
    "$status_container_uptime" \
    "$status_git_state" \
    "$status_migrations"
}

print_status_styled() {
  if [ "${AIBOX_STATUS_STYLE:-bar}" = "plain" ] || [ -n "${NO_COLOR:-}" ]; then
    print_status_plain
    return
  fi

  printf '\033[7m AIBOX \033[27m \033[2m MEM \033[22m\033[1m%s\033[22m/%s oom\033[1m%s\033[22m hi\033[1m%s\033[22m max\033[1m%s\033[22m  \033[2m CPU \033[22mthr\033[1m%s\033[22m  \033[2m PROC \033[22m\033[1m%s\033[22m ai\033[1m%s\033[22m pk:\033[1m%s/%s\033[22m  \033[2m FS \033[22m\033[1m%s\033[22m  \033[2m UP \033[22m\033[1m%s\033[22m  \033[2m PROJ \033[22mgit:\033[1m%s\033[22m mig\033[1m%s\033[22m' \
    "$status_memory_current" \
    "$status_memory_max" \
    "$status_oom_kill" \
    "$status_memory_high" \
    "$status_memory_max_events" \
    "$status_cpu_throttling" \
    "$status_processes" \
    "$status_ai_agents" \
    "$status_processkit_mode" \
    "$status_processkit_mcp" \
    "$status_disk_available" \
    "$status_container_uptime" \
    "$status_git_state" \
    "$status_migrations"
}

print_status() {
  read_status_values
  read_project_values
  print_status_plain
}

if [ "${1:-}" = "--watch" ]; then
  interval="${AIBOX_STATUS_INTERVAL:-5}"
  project_interval="${AIBOX_STATUS_PROJECT_INTERVAL:-60}"
  project_refresh_after="$(($(date +%s) + project_interval))"
  status_git_state="..."
  status_migrations="..."
  previous_width=0
  while true; do
    read_status_values
    now="$(date +%s)"
    if [ "$now" -ge "$project_refresh_after" ]; then
      read_project_values
      project_refresh_after="$((now + project_interval))"
    fi
    plain_line="$(print_status_plain)"
    line="$(print_status_styled)"
    printf '\r%s' "$line"
    line_width="${#plain_line}"
    if [ "$previous_width" -gt "$line_width" ]; then
      printf '%*s' "$((previous_width - line_width))" ''
    fi
    previous_width="$line_width"
    sleep "$interval"
  done
else
  print_status
  printf '\n'
fi
