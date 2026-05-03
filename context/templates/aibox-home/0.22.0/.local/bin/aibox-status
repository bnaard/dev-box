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

count_processes() {
  local count=0
  local entry
  for entry in /proc/[0-9]*; do
    [ -e "$entry" ] || continue
    count=$((count + 1))
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

print_status() {
  local memory_current memory_max oom_kill processes processkit_mcp
  memory_current="$(read_memory_current)"
  memory_max="$(read_memory_max)"
  oom_kill="$(read_oom_kill)"
  processes="$(count_processes)"
  processkit_mcp="$(count_processkit_mcp_python)"

  printf 'mem %s / %s | oom %s | proc %s | pk-mcp %s' \
    "$(format_bytes "$memory_current")" \
    "$memory_max" \
    "$oom_kill" \
    "$processes" \
    "$processkit_mcp"
}

if [ "${1:-}" = "--watch" ]; then
  interval="${AIBOX_STATUS_INTERVAL:-5}"
  while true; do
    printf '\r\033[2K'
    print_status
    sleep "$interval"
  done
else
  print_status
  printf '\n'
fi
