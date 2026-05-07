//! Generated runtime smoke tests on the SSH companion.
//!
//! These Phase 1 tests use the current CLI binary and generated `.aibox-home`
//! files on the aibox-e2e-testrunner companion. The runtime target is tmux-only.

use serial_test::serial;

use super::runner::E2eRunner;

#[test]
#[serial]
#[ntest::timeout(180_000)]
fn generated_runtime_yazi_lazygit_tmux_and_status_are_usable() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let test_name = "runtime-generated-smoke";
    runner.cleanup(test_name);

    let init = runner.aibox(
        test_name,
        &[
            "init",
            test_name,
            "--base",
            "debian",
            "--context",
            "managed",
            "--processkit-version",
            "unset",
            "--addon",
            "git-ui",
            "--theme",
            "tokyo-night",
            "--prompt",
            "arrow",
            "--no-container",
        ],
    );
    assert!(
        init.status.success(),
        "init failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&init.stdout),
        String::from_utf8_lossy(&init.stderr)
    );

    let apply = runner.aibox(
        test_name,
        &["apply", "--no-container", "--standardize-config"],
    );
    assert!(
        apply.status.success(),
        "apply failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&apply.stdout),
        String::from_utf8_lossy(&apply.stderr)
    );

    let workspace = format!("/workspaces/{test_name}");
    let probe = format!(
        r#"set -u
cd {workspace}
export HOME="{workspace}/.aibox-home"
export TERM=xterm-256color
export COLORTERM=truecolor
export PATH="$HOME/.local/bin:/usr/local/bin:$PATH"
export XDG_STATE_HOME="$HOME/.local/state"

fail=0

echo "== versions =="
tmux -V || fail=1
yazi --version || fail=1
lazygit --version || fail=1
if command -v zellij >/tmp/{test_name}-zellij-bin.txt 2>&1; then
  echo "zellij binary should not be present in generated runtime path"
  cat /tmp/{test_name}-zellij-bin.txt
  fail=1
fi

echo "== generated tmux runtime =="
test -f "$HOME/.tmux.conf" -o -f "$HOME/.config/tmux/tmux.conf" || fail=1
grep -Rli --exclude=claude 'tmux' "$HOME" .devcontainer aibox.toml >/tmp/{test_name}-tmux-files.txt || fail=1
if find "$HOME" -path '*zellij*' -print -quit | grep -q .; then
  echo "zellij path remains in generated home"
  find "$HOME" -path '*zellij*' -print
  fail=1
fi
if grep -Rli --exclude-dir=.git 'zellij' "$HOME" .devcontainer aibox.toml >/tmp/{test_name}-zellij-refs.txt 2>/dev/null; then
  echo "zellij references remain in generated runtime"
  cat /tmp/{test_name}-zellij-refs.txt
  fail=1
fi
tmux_config_targets="$HOME/.config/tmux"
[ -f "$HOME/.tmux.conf" ] && tmux_config_targets="$tmux_config_targets $HOME/.tmux.conf"
grep -R 'tmux-sensible\|tmux-powerkit\|tmux-yank\|vim-tmux-navigator' $tmux_config_targets >/tmp/{test_name}-tmux-plugins.txt 2>/dev/null || fail=1
cat /tmp/{test_name}-tmux-plugins.txt 2>/dev/null || true

echo "== yazi config =="
nl -ba "$HOME/.config/yazi/yazi.toml" | sed -n '1,140p'
nl -ba "$HOME/.config/yazi/theme.toml" | sed -n '1,160p'
if grep -R 'name = "' "$HOME/.config/yazi"/*.toml "$HOME/.config/yazi/themes"/*.toml >/tmp/{test_name}-yazi-invalid.txt 2>&1; then
  echo "invalid Yazi 26 name matcher remains"
  cat /tmp/{test_name}-yazi-invalid.txt
  fail=1
fi
if yazi --debug >/tmp/{test_name}-yazi-debug.txt 2>&1; then
  sed -n '1,100p' /tmp/{test_name}-yazi-debug.txt
else
  code=$?
  sed -n '1,160p' /tmp/{test_name}-yazi-debug.txt
  echo "yazi --debug failed with $code"
  fail=1
fi

echo "== lazygit state =="
git init -q
git -c user.email=test@test.com -c user.name=test add aibox.toml
git -c user.email=test@test.com -c user.name=test commit -m init >/dev/null
ls -ld "$HOME/.local" "$HOME/.local/state" "$HOME/.local/state/lazygit" "$HOME/.config/lazygit" || fail=1
timeout 8s lazygit --debug >/tmp/{test_name}-lazygit-debug.txt 2>&1
code=$?
sed -n '1,120p' /tmp/{test_name}-lazygit-debug.txt
if [ "$code" -ne 0 ] && [ "$code" -ne 124 ] && ! grep -q 'open /dev/tty' /tmp/{test_name}-lazygit-debug.txt; then
  echo "lazygit --debug failed with $code"
  fail=1
fi
if grep -q 'could not create any of the following paths' /tmp/{test_name}-lazygit-debug.txt; then
  echo "lazygit state directory creation failed"
  fail=1
fi

echo "== status helper =="
if aibox-status --plugin-json >/tmp/{test_name}-status.json 2>&1; then
  cat /tmp/{test_name}-status.json
  jq -e '.plain and .memory_current and .processes' /tmp/{test_name}-status.json >/dev/null || fail=1
else
  cat /tmp/{test_name}-status.json
  fail=1
fi

exit "$fail"
"#
    );

    let output = runner.exec(&probe);
    assert!(
        output.status.success(),
        "generated runtime probe failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    runner.cleanup(test_name);
}

#[test]
#[serial]
#[ntest::timeout(120_000)]
fn generated_runtime_tmux_status_panes_and_buffer_are_visible() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let test_name = "runtime-generated-tmux";
    runner.cleanup(test_name);

    let init = runner.aibox(
        test_name,
        &[
            "init",
            test_name,
            "--base",
            "debian",
            "--context",
            "managed",
            "--processkit-version",
            "unset",
            "--theme",
            "tokyo-night",
            "--no-container",
        ],
    );
    assert!(
        init.status.success(),
        "init failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&init.stdout),
        String::from_utf8_lossy(&init.stderr)
    );

    let apply = runner.aibox(test_name, &["apply", "--no-container"]);
    assert!(
        apply.status.success(),
        "apply failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&apply.stdout),
        String::from_utf8_lossy(&apply.stderr)
    );

    let workspace = format!("/workspaces/{test_name}");
    let probe = format!(
        r#"set -u
cd {workspace}
export HOME="{workspace}/.aibox-home"
export TERM=xterm-256color
export COLORTERM=truecolor
export PATH="$HOME/.local/bin:/usr/local/bin:$PATH"

tmux_conf="$HOME/.tmux.conf"
[ -f "$tmux_conf" ] || tmux_conf="$HOME/.config/tmux/tmux.conf"
test -f "$tmux_conf"
tmux kill-session -t {test_name} >/dev/null 2>&1 || true
cat > {workspace}/driver.sh <<'DRIVER'
#!/usr/bin/env bash
set -u
export HOME="{workspace}/.aibox-home"
export TERM=xterm-256color
export COLORTERM=truecolor
export PATH="$HOME/.local/bin:/usr/local/bin:$PATH"
tmux_conf="$HOME/.tmux.conf"
[ -f "$tmux_conf" ] || tmux_conf="$HOME/.config/tmux/tmux.conf"
(
  for _ in $(seq 1 50); do
    tmux has-session -t {test_name} >/dev/null 2>&1 && break
    sleep 0.1
  done
  tmux set-option -t {test_name} -g status on
  tmux set-option -t {test_name} -g status-left " AIBOX-TMUX #S:#I.#P "
  tmux set-option -t {test_name} -g status-right " #(aibox-status 2>/dev/null | cut -c1-80) "
  tmux split-window -h -t {test_name}:1 -c "{workspace}" "printf 'AIBOX-TMUX-RIGHT-PANE\n'; exec bash"
  tmux split-window -v -t {test_name}:1.1 -c "{workspace}" "printf 'AIBOX-TMUX-LOWER-PANE\n'; exec bash"
  tmux set-buffer -b aibox-yank "AIBOX_TMUX_BUFFER_MARKER"
  tmux save-buffer -b aibox-yank "{workspace}/tmux-buffer.txt"
  sleep 3
  tmux capture-pane -p -t {test_name}:1.1 > "{workspace}/screen-left.txt" 2>/dev/null || true
  tmux capture-pane -p -t {test_name}:1.2 > "{workspace}/screen-right.txt" 2>/dev/null || true
  tmux display-message -p -t {test_name} '#S #W #{{window_panes}} #{{status-left}} #{{status-right}}' > "{workspace}/status.txt" 2>/dev/null || true
  tmux kill-session -t {test_name} >/dev/null 2>&1 || true
) &
driver_pid=$!
tmux -f "$tmux_conf" new-session -A -s {test_name} -n dev -c "{workspace}" "printf 'AIBOX-TMUX-LEFT-PANE\n'; exec bash"
wait "$driver_pid" 2>/dev/null || true
true
DRIVER
chmod +x {workspace}/driver.sh
LC_ALL=C.UTF-8 LANG=C.UTF-8 timeout --kill-after=2s 35s asciinema rec --cols 160 --rows 45 --overwrite \
  -c {workspace}/driver.sh {workspace}/recording.cast 2>/dev/null || true

cat {workspace}/recording.cast {workspace}/screen-left.txt {workspace}/screen-right.txt {workspace}/status.txt > {workspace}/combined.txt
grep -aE 'AIBOX-TMUX|AIBOX-TMUX-RIGHT-PANE|AIBOX-TMUX-LOWER-PANE' {workspace}/combined.txt
grep -aE 'MEM |PROC |MCP ' {workspace}/combined.txt
grep -qF 'AIBOX_TMUX_BUFFER_MARKER' {workspace}/tmux-buffer.txt
if grep -ai 'zellij' {workspace}/combined.txt; then
  echo "legacy multiplexer text leaked into tmux runtime recording"
  exit 1
fi
"#
    );

    let output = runner.exec(&probe);
    assert!(
        output.status.success(),
        "generated tmux runtime visual probe failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    runner.cleanup(test_name);
}
