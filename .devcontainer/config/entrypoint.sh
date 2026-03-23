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

# Drop to aibox user and exec the command
exec gosu aibox "$@"
