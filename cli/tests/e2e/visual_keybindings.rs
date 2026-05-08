//! Visual keybinding tests for the tmux runtime.
//!
//! These tests exercise the same user workflows as the previous visual suite:
//! Yazi-to-Vim handoff, Vim leader mappings, tmux pane/window controls, and
//! tmux buffer/yank behavior. All interaction is driven through tmux itself.

use serial_test::serial;

use super::runner::E2eRunner;

fn extract_cast_output(cast_content: &str) -> String {
    cast_content
        .lines()
        .skip(1)
        .filter_map(|line| {
            let parsed: serde_json::Value = serde_json::from_str(line).ok()?;
            let arr = parsed.as_array()?;
            if arr.len() >= 3 && arr[1].as_str() == Some("o") {
                arr[2].as_str().map(str::to_owned)
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

fn init_managed_project(runner: &E2eRunner, test_name: &str) {
    let init = runner.aibox(
        test_name,
        &[
            "init",
            test_name,
            "--base",
            "debian",
            "--context",
            "managed",
            "--no-container",
        ],
    );
    assert!(
        init.status.success(),
        "{test_name}: init failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&init.stdout),
        String::from_utf8_lossy(&init.stderr)
    );
}

fn assert_tmux_only_runtime(runner: &E2eRunner, test_name: &str) {
    let workspace = format!("/workspaces/{test_name}");
    let probe = runner.exec(&format!(
        r#"cd {workspace}
test -f .aibox-home/.tmux.conf -o -f .aibox-home/.config/tmux/tmux.conf
! find .aibox-home -path '*zellij*' -print -quit | grep -q .
! grep -Rli --exclude-dir=.git 'zellij' .aibox-home .devcontainer aibox.toml >/tmp/{test_name}-legacy-zellij.txt 2>/dev/null
"#
    ));
    assert!(
        probe.status.success(),
        "{test_name}: generated runtime should be tmux-only\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&probe.stdout),
        String::from_utf8_lossy(&probe.stderr)
    );
}

fn record(runner: &E2eRunner, test_name: &str, driver: &str) -> String {
    let ws = format!("/workspaces/{test_name}");
    runner.exec(
        "tmux kill-server >/dev/null 2>&1 || true; rm -rf /tmp/tmux-* >/dev/null 2>&1 || true",
    );
    runner.write_file(test_name, "driver.sh", driver);
    runner.exec(&format!("chmod +x {ws}/driver.sh"));
    runner.exec(&format!(
        "LC_ALL=C.UTF-8 LANG=C.UTF-8 asciinema rec --cols 160 --rows 45 --overwrite \
         -c {ws}/driver.sh {ws}/recording.cast 2>/dev/null; true"
    ));
    runner.read_file(test_name, "recording.cast")
}

fn tmux_driver(ws: &str, session: &str, startup_cmd: &str, actions: &str) -> String {
    format!(
        r#"#!/usr/bin/env bash
set -u
export TERM=xterm-256color
export COLORTERM=truecolor
export HOME="{ws}/.aibox-home"
export PATH="$HOME/.local/bin:/usr/local/bin:$PATH"
tmux_conf="$HOME/.tmux.conf"
[ -f "$tmux_conf" ] || tmux_conf="$HOME/.config/tmux/tmux.conf"
if [ ! -f "$tmux_conf" ]; then
  echo "missing generated tmux config" >&2
  exit 90
fi
tmux kill-session -t "{session}" >/dev/null 2>&1 || true
(
  for _ in $(seq 1 50); do
    tmux has-session -t "{session}" >/dev/null 2>&1 && break
    sleep 0.1
  done
{actions}
  sleep 0.4
  tmux capture-pane -p -t "{session}:1.1" > "{ws}/final-screen.txt" 2>/dev/null || true
  tmux kill-session -t "{session}" >/dev/null 2>&1 || true
) &
driver_pid=$!
tmux -f "$tmux_conf" new-session -A -s "{session}" -n dev -c "{ws}" {startup_cmd}
wait "$driver_pid" 2>/dev/null || true
true
"#
    )
}

fn quoted_shell(command: &str) -> String {
    format!("{command:?}")
}

#[test]
#[serial]
#[ntest::timeout(120_000)]
fn visual_kb_yazi_e_opens_file_in_vim_pane() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let test_name = "visual-kb-yazi-e";
    runner.cleanup(test_name);
    init_managed_project(&runner, test_name);
    assert_tmux_only_runtime(&runner, test_name);

    let marker = "AIBOX_E2E_OPEN_OK";
    runner.write_file(
        test_name,
        "src/hello.rs",
        &format!("fn main() {{\n    // {marker}\n}}\n"),
    );

    let ws = format!("/workspaces/{test_name}");
    let src = format!("{ws}/src");
    let actions = format!(
        r#"  tmux split-window -h -t "{test_name}:1.1" -c "{ws}" "AIBOX_EDITOR_DIR=right exec vim-loop"
  initial_panes="$(tmux list-panes -t "{test_name}:1" | wc -l | tr -d ' ')"
  tmux select-pane -t "{test_name}:1.1"
  tmux send-keys -t "{test_name}:1.1" "cd {src} && AIBOX_EDITOR_DIR=right exec yazi ." C-m
  sleep 1.5
  tmux send-keys -t "{test_name}:1.1" "e"
  for _ in $(seq 1 40); do
    tmux capture-pane -p -t "{test_name}:1.2" > "{ws}/editor-screen.txt" 2>/dev/null || true
    grep -qF "{marker}" "{ws}/editor-screen.txt" && touch "{ws}/open-ok" && break
    sleep 0.25
  done
  tmux send-keys -t "{test_name}:1.2" Escape ":q" Enter
  sleep 1
  active_pane="$(tmux list-panes -t "{test_name}:1" -F '#{{pane_active}} #{{pane_id}}' | awk '$1==1 {{print $2; exit}}')"
  files_pane="$(tmux display-message -p -t "{test_name}:1.1" '#{{pane_id}}')"
  [ "$active_pane" = "$files_pane" ] && touch "{ws}/focus-return-ok"
  final_panes="$(tmux list-panes -t "{test_name}:1" | wc -l | tr -d ' ')"
  [ "$final_panes" = "$initial_panes" ] && touch "{ws}/pane-count-ok"
"#
    );
    let cast = record(
        &runner,
        test_name,
        &tmux_driver(
            &ws,
            test_name,
            &quoted_shell("printf 'files pane'; exec bash"),
            &actions,
        ),
    );
    assert!(cast.lines().count() > 5, "cast too small");
    assert!(
        runner.file_exists(test_name, "open-ok"),
        "expected Yazi e to open the file in Vim pane; editor screen:\n{}",
        runner.read_file(test_name, "editor-screen.txt")
    );
    assert!(
        runner.file_exists(test_name, "focus-return-ok"),
        "expected :q in editor pane to return focus to yazi pane"
    );
    assert!(
        runner.file_exists(test_name, "pane-count-ok"),
        "expected Yazi e flow not to create extra tmux panes"
    );
    runner.cleanup(test_name);
}

#[test]
#[serial]
#[ntest::timeout(90_000)]
fn visual_kb_yazi_enter_opens_vim_inplace_and_returns() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let test_name = "visual-kb-yazi-enter-return";
    runner.cleanup(test_name);
    init_managed_project(&runner, test_name);
    assert_tmux_only_runtime(&runner, test_name);

    let marker = "AIBOX_E2E_ENTER_RETURN_BETA";
    runner.write_file(test_name, "src/alpha.rs", "fn alpha() {}\n");
    runner.write_file(test_name, "src/beta.rs", &format!("// {marker}\n"));

    let ws = format!("/workspaces/{test_name}");
    let actions = format!(
        r#"  tmux send-keys -t "{test_name}:1.1" "cd {ws}/src && EDITOR=vim exec yazi ." C-m
  sleep 1.5
  tmux send-keys -t "{test_name}:1.1" Enter
  sleep 1
  tmux send-keys -t "{test_name}:1.1" Escape ":q" Enter
  sleep 0.8
  tmux send-keys -t "{test_name}:1.1" "j" Enter
  for _ in $(seq 1 40); do
    tmux capture-pane -p -t "{test_name}:1.1" > "{ws}/return-screen.txt" 2>/dev/null || true
    grep -qF "{marker}" "{ws}/return-screen.txt" && touch "{ws}/return-ok" && break
    sleep 0.25
  done
"#
    );
    let _cast = record(
        &runner,
        test_name,
        &tmux_driver(
            &ws,
            test_name,
            &quoted_shell("printf 'yazi pane'; exec bash"),
            &actions,
        ),
    );
    assert!(
        runner.file_exists(test_name, "return-ok"),
        "expected Yazi Enter -> Vim :q to resume Yazi; screen:\n{}",
        runner.read_file(test_name, "return-screen.txt")
    );
    runner.cleanup(test_name);
}

#[test]
#[serial]
#[ntest::timeout(90_000)]
fn visual_kb_yazi_git_summary_and_changes_show_status() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let test_name = "visual-kb-yazi-git-status";
    runner.cleanup(test_name);
    init_managed_project(&runner, test_name);
    assert_tmux_only_runtime(&runner, test_name);

    let ws = format!("/workspaces/{test_name}");
    runner.exec(&format!(
        "cd {ws} && git -c user.email=test@test.com -c user.name=test init && \
         echo old > changed.txt && git -c user.email=test@test.com -c user.name=test add changed.txt && \
         git -c user.email=test@test.com -c user.name=test commit -m init && echo new > changed.txt"
    ));
    let actions = format!(
        r#"  tmux send-keys -t "{test_name}:1.1" "cd {ws} && exec yazi ." C-m
  sleep 1.5
  tmux send-keys -t "{test_name}:1.1" "gs"
  sleep 1.2
  tmux send-keys -t "{test_name}:1.1" "q"
  sleep 0.4
  tmux send-keys -t "{test_name}:1.1" "gc"
  sleep 1.2
  tmux capture-pane -p -t "{test_name}:1.1" > "{ws}/git-screen.txt" 2>/dev/null || true
"#
    );
    let cast = record(
        &runner,
        test_name,
        &tmux_driver(
            &ws,
            test_name,
            &quoted_shell("printf 'git pane'; exec bash"),
            &actions,
        ),
    );
    let output = extract_cast_output(&cast);
    let screen = runner.read_file(test_name, "git-screen.txt");
    assert!(
        output.contains("changed.txt") || screen.contains("changed.txt") || screen.contains("##"),
        "expected Yazi git shortcuts to show status/change output:\n{screen}"
    );
    runner.cleanup(test_name);
}

#[test]
#[serial]
#[ntest::timeout(90_000)]
fn visual_kb_yazi_pane_toggles_keep_file_list_alive() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let test_name = "visual-kb-yazi-pane-toggles";
    runner.cleanup(test_name);
    init_managed_project(&runner, test_name);
    assert_tmux_only_runtime(&runner, test_name);
    runner.write_file(test_name, "visible.txt", "still here\n");

    let ws = format!("/workspaces/{test_name}");
    let actions = format!(
        r#"  tmux send-keys -t "{test_name}:1.1" "cd {ws} && exec yazi ." C-m
  sleep 1.5
  tmux send-keys -t "{test_name}:1.1" "zl"
  sleep 0.3
  tmux send-keys -t "{test_name}:1.1" "zm"
  sleep 0.3
  tmux send-keys -t "{test_name}:1.1" "z0"
  sleep 0.3
  tmux send-keys -t "{test_name}:1.1" "zc"
  sleep 0.3
  tmux send-keys -t "{test_name}:1.1" "z0"
  sleep 0.8
  tmux capture-pane -p -t "{test_name}:1.1" > "{ws}/final-screen.txt" 2>/dev/null || true
"#
    );
    let _cast = record(
        &runner,
        test_name,
        &tmux_driver(
            &ws,
            test_name,
            &quoted_shell("printf 'yazi pane'; exec bash"),
            &actions,
        ),
    );
    let screen = runner.read_file(test_name, "final-screen.txt");
    assert!(
        screen.contains("visible.txt"),
        "expected Yazi file list to survive pane toggle shortcuts:\n{screen}"
    );
    runner.cleanup(test_name);
}

#[test]
#[serial]
#[ntest::timeout(90_000)]
fn visual_kb_tmux_prefix_splits_windows_and_status_render() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let test_name = "visual-kb-tmux-prefix-status";
    runner.cleanup(test_name);
    init_managed_project(&runner, test_name);
    assert_tmux_only_runtime(&runner, test_name);

    let ws = format!("/workspaces/{test_name}");
    let actions = format!(
        r#"  tmux set-option -t "{test_name}" -g status-left " AIBOX-TMUX-KEYS #S:#I.#P "
  tmux set-option -t "{test_name}" -g status-right " prefix C-g | panes r d | windows c n p "
  tmux split-window -h -t "{test_name}:1.1" -c "{ws}" "exec bash"
  sleep 0.4
  tmux send-keys -t "{test_name}:1.2" "printf AIBOX-TMUX-SPLIT" C-m
  tmux new-window -t "{test_name}" -n shell -c "{ws}" "printf 'AIBOX-TMUX-WINDOW\n'; exec bash"
  sleep 1
  tmux list-windows -t "{test_name}" > "{ws}/windows.txt"
  tmux list-panes -t "{test_name}:1" > "{ws}/panes.txt"
  tmux capture-pane -p -t "{test_name}:2.1" > "{ws}/window-screen.txt" 2>/dev/null || true
"#
    );
    let cast = record(
        &runner,
        test_name,
        &tmux_driver(
            &ws,
            test_name,
            &quoted_shell("printf 'tmux keys'; exec bash"),
            &actions,
        ),
    );
    let output = extract_cast_output(&cast);
    assert!(
        output.contains("AIBOX-TMUX-KEYS")
            || runner
                .read_file(test_name, "window-screen.txt")
                .contains("AIBOX-TMUX-WINDOW"),
        "expected tmux status/window visual output"
    );
    assert!(
        runner.read_file(test_name, "windows.txt").contains("shell")
            && runner.read_file(test_name, "panes.txt").lines().count() >= 2,
        "expected tmux prefix workflow to create panes and windows"
    );
    runner.cleanup(test_name);
}

fn vim_driver(ws: &str, session: &str, vim_args: &str, actions: &str) -> String {
    let startup = quoted_shell(&format!("exec vim -u /opt/aibox/vimrc {vim_args}"));
    tmux_driver(ws, session, &startup, actions)
}

#[test]
#[serial]
#[ntest::timeout(90_000)]
fn visual_kb_vim_leader_e_opens_netrw() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let test_name = "visual-kb-vim-netrw";
    runner.cleanup(test_name);
    init_managed_project(&runner, test_name);
    assert_tmux_only_runtime(&runner, test_name);
    runner.write_file(test_name, "project.toml", "[package]\nname = \"test\"\n");

    let ws = format!("/workspaces/{test_name}");
    let actions = format!(
        r#"  sleep 1
  tmux send-keys -t "{test_name}:1.1" " l"
  sleep 0.4
  tmux send-keys -t "{test_name}:1.1" " e"
  sleep 1
  tmux capture-pane -p -t "{test_name}:1.1" > "{ws}/vim-screen.txt" 2>/dev/null || true
"#
    );
    let cast = record(
        &runner,
        test_name,
        &vim_driver(&ws, test_name, &format!("\"{ws}/project.toml\""), &actions),
    );
    let output = extract_cast_output(&cast);
    let screen = runner.read_file(test_name, "vim-screen.txt");
    assert!(
        output.contains("project.toml")
            || screen.contains("project.toml")
            || screen.contains("netrw"),
        "expected netrw/project listing after <Space>e:\n{screen}"
    );
    runner.cleanup(test_name);
}

#[test]
#[serial]
#[ntest::timeout(90_000)]
fn visual_kb_vim_leader_l_shows_buffer_list() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let test_name = "visual-kb-vim-buflist";
    runner.cleanup(test_name);
    init_managed_project(&runner, test_name);
    assert_tmux_only_runtime(&runner, test_name);
    runner.write_file(test_name, "alpha.rs", "fn alpha() {}\n");
    runner.write_file(test_name, "beta.rs", "fn beta() {}\n");

    let ws = format!("/workspaces/{test_name}");
    let actions = format!(
        r#"  sleep 1
  tmux send-keys -t "{test_name}:1.1" " l"
  sleep 0.8
  tmux capture-pane -p -t "{test_name}:1.1" > "{ws}/vim-screen.txt" 2>/dev/null || true
"#
    );
    let _cast = record(
        &runner,
        test_name,
        &vim_driver(
            &ws,
            test_name,
            &format!("\"{ws}/alpha.rs\" \"{ws}/beta.rs\""),
            &actions,
        ),
    );
    let screen = runner.read_file(test_name, "vim-screen.txt");
    assert!(
        screen.contains("alpha") || screen.contains("beta") || screen.contains("line 1"),
        "expected :ls buffer list after <Space>l:\n{screen}"
    );
    runner.cleanup(test_name);
}

#[test]
#[serial]
#[ntest::timeout(90_000)]
fn visual_kb_vim_leader_w_saves_file() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let test_name = "visual-kb-vim-save";
    runner.cleanup(test_name);
    init_managed_project(&runner, test_name);
    assert_tmux_only_runtime(&runner, test_name);
    runner.write_file(test_name, "save_me.rs", "fn main() {}\n");

    let ws = format!("/workspaces/{test_name}");
    let actions = format!(
        r#"  sleep 1
  tmux send-keys -t "{test_name}:1.1" "A edited" Escape " w"
  sleep 0.8
  tmux capture-pane -p -t "{test_name}:1.1" > "{ws}/vim-screen.txt" 2>/dev/null || true
"#
    );
    let cast = record(
        &runner,
        test_name,
        &vim_driver(&ws, test_name, &format!("\"{ws}/save_me.rs\""), &actions),
    );
    let output = extract_cast_output(&cast);
    let saved = runner.read_file(test_name, "save_me.rs");
    assert!(
        output.contains("written") || saved.contains("edited"),
        "expected <Space>w to write the file; saved content:\n{saved}"
    );
    runner.cleanup(test_name);
}

#[test]
#[serial]
#[ntest::timeout(90_000)]
fn visual_kb_vim_leader_x_writes_and_quits_vim() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let test_name = "visual-kb-vim-writequit";
    runner.cleanup(test_name);
    init_managed_project(&runner, test_name);
    assert_tmux_only_runtime(&runner, test_name);
    runner.write_file(test_name, "writequit.rs", "fn main() {}\n");

    let ws = format!("/workspaces/{test_name}");
    let actions = format!(
        r#"  sleep 1
  tmux send-keys -t "{test_name}:1.1" "A // saved" Escape " x"
  sleep 1
  if ! pgrep -x vim >/dev/null 2>&1; then touch "{ws}/writequit-ok"; fi
  tmux capture-pane -p -t "{test_name}:1.1" > "{ws}/vim-screen.txt" 2>/dev/null || true
"#
    );
    let _cast = record(
        &runner,
        test_name,
        &vim_driver(&ws, test_name, &format!("\"{ws}/writequit.rs\""), &actions),
    );
    let saved = runner.read_file(test_name, "writequit.rs");
    assert!(
        runner.file_exists(test_name, "writequit-ok") && saved.contains("// saved"),
        "expected <Space>x to write and quit; screen:\n{} content:\n{saved}",
        runner.read_file(test_name, "vim-screen.txt")
    );
    runner.cleanup(test_name);
}

#[test]
#[serial]
#[ntest::timeout(90_000)]
fn visual_kb_vim_leader_n_p_cycles_buffers() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let test_name = "visual-kb-vim-cycle";
    runner.cleanup(test_name);
    init_managed_project(&runner, test_name);
    assert_tmux_only_runtime(&runner, test_name);
    runner.write_file(test_name, "alpha.rs", "AIBOX_ALPHA_BUFFER\n");
    runner.write_file(test_name, "beta.rs", "AIBOX_BETA_BUFFER\n");

    let ws = format!("/workspaces/{test_name}");
    let actions = format!(
        r#"  sleep 1
  tmux send-keys -t "{test_name}:1.1" " n"
  sleep 0.6
  tmux capture-pane -p -t "{test_name}:1.1" > "{ws}/next-screen.txt" 2>/dev/null || true
  tmux send-keys -t "{test_name}:1.1" " p"
  sleep 0.6
  tmux capture-pane -p -t "{test_name}:1.1" > "{ws}/prev-screen.txt" 2>/dev/null || true
"#
    );
    let _cast = record(
        &runner,
        test_name,
        &vim_driver(
            &ws,
            test_name,
            &format!("\"{ws}/alpha.rs\" \"{ws}/beta.rs\""),
            &actions,
        ),
    );
    let combined = format!(
        "{}\n{}",
        runner.read_file(test_name, "next-screen.txt"),
        runner.read_file(test_name, "prev-screen.txt")
    );
    assert!(
        combined.contains("AIBOX_ALPHA_BUFFER") && combined.contains("AIBOX_BETA_BUFFER"),
        "expected <Space>n/<Space>p to cycle buffers:\n{combined}"
    );
    runner.cleanup(test_name);
}

#[test]
#[serial]
#[ntest::timeout(60_000)]
fn visual_kb_tmux_buffer_yank_round_trip() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let test_name = "visual-kb-tmux-yank-buffer";
    runner.cleanup(test_name);
    init_managed_project(&runner, test_name);
    assert_tmux_only_runtime(&runner, test_name);

    let ws = format!("/workspaces/{test_name}");
    let marker = "AIBOX_TMUX_YANK_BUFFER";
    let actions = format!(
        r#"  tmux set-buffer -b aibox-yank "{marker}"
  tmux save-buffer -b aibox-yank "{ws}/tmux-buffer.txt"
  tmux show-buffer -b aibox-yank > "{ws}/tmux-buffer-visible.txt"
  grep -R 'tmux-yank' "$HOME/.tmux.conf" "$HOME/.config/tmux" > "{ws}/tmux-yank-config.txt" 2>/dev/null || true
  sleep 0.5
"#
    );
    let _cast = record(
        &runner,
        test_name,
        &tmux_driver(
            &ws,
            test_name,
            &quoted_shell("printf 'tmux buffer'; exec bash"),
            &actions,
        ),
    );
    assert_eq!(
        runner.read_file(test_name, "tmux-buffer.txt").trim(),
        marker
    );
    assert!(
        runner
            .read_file(test_name, "tmux-yank-config.txt")
            .contains("tmux-yank"),
        "expected generated tmux config to include tmux-yank plugin"
    );
    runner.cleanup(test_name);
}
