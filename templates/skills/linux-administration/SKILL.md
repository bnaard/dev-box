---
name: linux-administration
description: Essential Linux system administration for developers. File permissions, process management, systemd, journald, cron, and disk management. Use when managing Linux servers, debugging system issues, or writing system scripts.
allowed-tools: Bash Read Write
---

# Linux Administration

## When to Use

- Managing file permissions and ownership
- Investigating and managing running processes
- Creating or editing systemd service units
- Querying logs with journalctl
- Setting up cron jobs or systemd timers
- Diagnosing disk space, filesystem, or mount issues
- Managing users and groups
- Installing or managing packages with apt or dnf

## Instructions

### File Permissions and Ownership

Linux permissions use the rwx model for owner, group, and others. Use numeric (chmod 755) or symbolic (chmod u+x) notation:

```bash
chmod 755 script.sh          # rwxr-xr-x
chmod u+x,g+r,o-w file      # symbolic mode
chown app:app /srv/data      # change owner and group
chown -R app:app /srv/data   # recursive ownership change
```

Key permission values: 644 (files, rw-r--r--), 755 (executables/dirs, rwxr-xr-x), 600 (secrets, rw-------), 700 (private dirs). Directories need execute permission for traversal.

Special bits: setuid (4000), setgid (2000), sticky (1000). Use `stat -c '%a %U:%G %n' file` to inspect permissions concisely.

### Process Management

Find and manage processes:

```bash
ps aux                        # all processes with details
ps aux --sort=-%mem           # sorted by memory usage
pgrep -af "pattern"           # find processes by name
kill -TERM <pid>              # graceful termination
kill -KILL <pid>              # force kill (last resort)
top -b -n 1 -o %MEM          # batch mode, single snapshot, sort by memory
```

Use `SIGTERM` (15) first, `SIGKILL` (9) only when the process does not respond. Check open files with `lsof -p <pid>` and network connections with `ss -tlnp`.

### User and Group Management

```bash
useradd -m -s /bin/bash -G sudo newuser    # create user with home dir
usermod -aG docker existinguser            # add to group (keep existing groups)
passwd username                             # set password
userdel -r username                         # remove user and home dir
groups username                             # list user's groups
id username                                 # uid, gid, groups
```

Prefer `useradd` over `adduser` for scripting (consistent across distros). Always use `-aG` (append) with `usermod` to avoid removing existing group memberships.

### Systemd Services

Create service units in `/etc/systemd/system/`. See `references/systemd-reference.md` for the full unit file format.

```bash
systemctl start myapp              # start service
systemctl stop myapp               # stop service
systemctl restart myapp            # restart
systemctl enable myapp             # start on boot
systemctl enable --now myapp       # enable and start immediately
systemctl status myapp             # status with recent logs
systemctl daemon-reload            # reload after editing unit files
```

Always run `daemon-reload` after creating or modifying unit files.

### Journald Logging

Query logs with journalctl:

```bash
journalctl -u myapp -f                 # follow logs for a unit
journalctl -u myapp --since "1 hour ago"  # recent logs
journalctl -u myapp -p err             # only errors and above
journalctl -b -1                       # logs from previous boot
journalctl --disk-usage                # check journal size
journalctl --vacuum-size=500M          # reclaim space
```

See `references/systemd-reference.md` for journalctl patterns and filtering.

### Cron Jobs

Edit crontabs with `crontab -e`. Format: `minute hour day month weekday command`.

```bash
crontab -l                          # list current user's crontab
crontab -e                          # edit crontab
crontab -l -u www-data              # list another user's crontab
```

Common schedules:

```cron
0 * * * *    /usr/local/bin/hourly-task.sh        # every hour
0 2 * * *    /usr/local/bin/nightly-backup.sh     # daily at 2 AM
*/5 * * * *  /usr/local/bin/health-check.sh       # every 5 minutes
0 0 * * 0    /usr/local/bin/weekly-cleanup.sh     # weekly on Sunday
```

Always redirect output in cron: `command >> /var/log/task.log 2>&1`. Consider systemd timers as a modern alternative (see systemd-reference.md).

### Disk and Filesystem Management

```bash
df -h                    # filesystem usage (human-readable)
du -sh /var/log/*        # directory sizes
lsblk                    # block device tree
mount | column -t        # current mounts
findmnt --target /data   # find mount point for a path
```

For disk space emergencies: check `/var/log`, `/tmp`, old kernels (`apt autoremove`), Docker (`docker system prune`), and journal logs (`journalctl --vacuum-size`).

### Package Management

Debian/Ubuntu (apt):

```bash
apt update && apt upgrade -y         # update package lists and upgrade
apt install -y --no-install-recommends package   # minimal install
apt search keyword                   # search packages
apt autoremove -y                    # remove unused dependencies
dpkg -l | grep package               # check installed version
```

RHEL/Fedora (dnf):

```bash
dnf check-update                     # check for updates
dnf install -y package               # install
dnf search keyword                   # search
dnf autoremove                       # remove unused dependencies
```

See `references/commands-cheatsheet.md` for a quick-reference organized by task.

## Examples

### Diagnose high disk usage on a server

```bash
# 1. Check filesystem usage
df -h
# 2. Find the largest directories under the full partition
du -sh /* 2>/dev/null | sort -rh | head -20
# 3. Drill into the largest directory
du -sh /var/log/* | sort -rh | head -10
# 4. Clean up old logs and reclaim journal space
journalctl --vacuum-size=200M
apt autoremove -y
```

### Create a systemd service for a Python application

```bash
# 1. Create the unit file
cat > /etc/systemd/system/myapp.service << 'EOF'
[Unit]
Description=My Python Application
After=network.target

[Service]
Type=simple
User=app
WorkingDirectory=/srv/myapp
ExecStart=/srv/myapp/.venv/bin/python -m myapp
Restart=on-failure
RestartSec=5
Environment=PYTHONUNBUFFERED=1

[Install]
WantedBy=multi-user.target
EOF

# 2. Enable and start
systemctl daemon-reload
systemctl enable --now myapp
systemctl status myapp
```

### Investigate a process consuming excessive memory

```bash
# 1. Identify the process
ps aux --sort=-%mem | head -10
# 2. Get details (open files, connections, memory map)
PID=12345
ls -la /proc/$PID/fd | wc -l     # open file descriptors
ss -tlnp | grep $PID              # listening ports
cat /proc/$PID/status | grep -i vmrss   # resident memory
# 3. If needed, send graceful termination
kill -TERM $PID
# 4. Verify it stopped
sleep 2 && ps -p $PID || echo "Process terminated"
```
