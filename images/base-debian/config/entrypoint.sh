#!/bin/bash
# entrypoint.sh — UID/GID remapping for non-root aibox user
#
# On Linux: pass AIBOX_UID/AIBOX_GID env vars (or -u UID:GID) to match host user.
# On macOS (Docker Desktop/OrbStack): no remapping needed — VM layer handles it.

set -e

TARGET_UID="${AIBOX_UID:-1000}"
TARGET_GID="${AIBOX_GID:-1000}"
CURRENT_UID=$(id -u aibox 2>/dev/null || echo 1000)
CURRENT_GID=$(id -g aibox 2>/dev/null || echo 1000)

# Remap UID/GID if they differ from the built-in defaults
if [ "$TARGET_GID" != "$CURRENT_GID" ]; then
    groupmod -g "$TARGET_GID" aibox 2>/dev/null || true
fi
if [ "$TARGET_UID" != "$CURRENT_UID" ]; then
    usermod -u "$TARGET_UID" aibox 2>/dev/null || true
fi

# Ensure home directory ownership matches (fast — only top-level)
chown "$TARGET_UID:$TARGET_GID" /home/aibox 2>/dev/null || true

# Stamp this container start with a stable runtime session id. Runtime status
# and log readers use this to distinguish current-container logs from stale
# entries left in the project volume by older containers.
if [ -d /workspace ] || mkdir -p /workspace 2>/dev/null; then
    mkdir -p /workspace/.aibox 2>/dev/null || true
    session_id="$(cat /proc/sys/kernel/random/uuid 2>/dev/null || true)"
    if [ -z "$session_id" ]; then
        session_id="$(date +%s)-$$"
    fi
    started_at="$(date -u +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || date +%s)"
    hostname_value="$(hostname 2>/dev/null || printf 'unknown')"
    cat > /workspace/.aibox/runtime-session.json 2>/dev/null <<EOF || true
{"runtime_session_id":"${session_id}","container_started_at":"${started_at}","container_hostname":"${hostname_value}"}
EOF
    chown "$TARGET_UID:$TARGET_GID" /workspace/.aibox /workspace/.aibox/runtime-session.json 2>/dev/null || true
fi

# Drop to aibox user and exec the command
exec gosu aibox "$@"
