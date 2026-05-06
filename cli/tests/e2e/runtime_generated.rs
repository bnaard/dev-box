//! Generated runtime smoke tests on the SSH companion.
//!
//! These are Phase 1 release tests: they run before host-side image publication,
//! using the current CLI binary and generated `.aibox-home` files on the
//! aibox-e2e-testrunner companion. Host Phase 2 still runs the full released
//! image smoke against GHCR.

use serial_test::serial;

use super::runner::E2eRunner;

#[test]
#[serial]
#[ntest::timeout(180_000)]
fn generated_runtime_yazi_lazygit_and_status_are_usable() {
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
            "--zellij-status",
            "shell",
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
zellij --version || fail=1
yazi --version || fail=1
lazygit --version || fail=1

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
fn generated_runtime_zellij_status_rows_are_visible() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let test_name = "runtime-generated-zellij";
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
            "--zellij-status",
            "native",
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

sudo rm -rf /workspace
sudo ln -s {workspace} /workspace
rm -rf /tmp/zellij-*
cat > {workspace}/driver.sh <<'DRIVER'
#!/usr/bin/env bash
set -u
export HOME="{workspace}/.aibox-home"
export TERM=xterm-256color
export COLORTERM=truecolor
export PATH="$HOME/.local/bin:/usr/local/bin:$PATH"
(sleep 6 && pkill -x zellij 2>/dev/null) &
zellij --config "$HOME/.config/zellij/config.kdl" \
       --config-dir "$HOME/.config/zellij" \
       --layout dev 2>/dev/null
true
DRIVER
chmod +x {workspace}/driver.sh
LC_ALL=C.UTF-8 LANG=C.UTF-8 asciinema rec --cols 160 --rows 45 --overwrite \
  -c {workspace}/driver.sh {workspace}/recording.cast 2>/dev/null || true
cat /tmp/zellij-*/zellij-log/zellij.log >/tmp/{test_name}-zellij.log 2>/dev/null || true
if grep -E 'ERROR IN PLUGIN|failed to load plugin|Panic occured|panicked|Unknown component: z' /tmp/{test_name}-zellij.log >/tmp/{test_name}-zellij-errors.txt 2>&1; then
  cat /tmp/{test_name}-zellij-errors.txt
  exit 1
fi
if grep -aE 'ERROR IN PLUGIN|failed to load plugin|could not find exported function' {workspace}/recording.cast >/tmp/{test_name}-zellij-cast-errors.txt 2>&1; then
  cat /tmp/{test_name}-zellij-cast-errors.txt
  exit 1
fi
if ! grep -aE 'LEADER|PANES' {workspace}/recording.cast >/tmp/{test_name}-zellij-visible.txt 2>&1; then
  echo "expected generated key/status rows were not visible"
  sed -n '1,80p' {workspace}/recording.cast
  exit 1
fi
if ! grep -aE 'MEM .+OOM kills [0-9]+|PROC .+total [0-9]+|MCP (gateway|granular|none) [0-9]+' {workspace}/recording.cast >/tmp/{test_name}-zellij-status-values.txt 2>&1; then
  echo "expected refreshed generated runtime details were not visible"
  sed -n '1,80p' {workspace}/recording.cast
  exit 1
fi
"#
    );

    let output = runner.exec(&probe);
    assert!(
        output.status.success(),
        "generated Zellij status plugin probe failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    runner.cleanup(test_name);
}
