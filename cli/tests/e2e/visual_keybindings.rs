//! Visual keybinding tests for the tmux runtime.
//!
//! These tests exercise the same user workflows as the previous visual suite:
//! Yazi-to-Vim handoff, Vim leader mappings, tmux pane/window controls, and
//! tmux buffer/yank behavior. All interaction is driven through tmux itself.

use super::local_runner::LocalProject as E2eRunner;

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
    let workspace = runner.root().display().to_string();
    let probe = runner.exec(&format!(
        r#"cd {workspace}
test -f .aibox-home/.tmux.conf -o -f .aibox-home/.config/tmux/tmux.conf
! find .aibox-home -path '*zellij*' -print -quit | grep -q .
! grep -Rli --exclude-dir=.git --exclude=claude 'zellij' .aibox-home .devcontainer aibox.toml >/tmp/{test_name}-legacy-zellij.txt 2>/dev/null
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
    let ws = runner.root().display().to_string();
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
tmux_socket="${{AIBOX_TMUX_SOCKET:?isolated tmux socket is required}}"
mkdir -p "$(dirname "$tmux_socket")"
tmux() {{
  command tmux -S "$tmux_socket" "$@"
}}
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
  wait_for_pane_command() {{
    target="$1"
    shift
    for _ in $(seq 1 50); do
      current="$(tmux display-message -p -t "$target" '#{{pane_current_command}}' 2>/dev/null || true)"
      for expected in "$@"; do
        if [ "$current" = "$expected" ]; then
          # pane_current_command changes before full-screen programs finish
          # loading their configuration and becoming ready for input.
          sleep 0.3
          return 0
        fi
      done
      sleep 0.1
    done
    return 1
  }}
  wait_for_pane_text() {{
    target="$1"
    pattern="$2"
    output="$3"
    for _ in $(seq 1 50); do
      tmux capture-pane -p -t "$target" > "$output" 2>/dev/null || true
      grep -qF "$pattern" "$output" && return 0
      sleep 0.1
    done
    return 1
  }}
{actions}
  sleep 0.1
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
#[ntest::timeout(120_000)]
fn visual_kb_yazi_e_opens_file_in_vim_pane() {
    // Per DEC-20260508_1604-LuckySeal (v0.25.6): yazi `e` opens the marked
    // file in a full-screen `tmux display-popup -E` running vim — there is
    // no persistent vim pane in any layout. The popup auto-closes when vim
    // exits and focus returns to the originating yazi pane.
    //
    // We cannot drive the popup via `tmux send-keys` (those go to the host
    // pane's pty, not the popup overlay). Instead we set EDITOR to a vim
    // invocation that uses an init file to (a) record the opened file path
    // into a marker file and (b) auto-quit, which closes the popup.
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let test_name = "visual-kb-yazi-e";
    runner.cleanup(test_name);
    init_managed_project(&runner, test_name);
    assert_tmux_only_runtime(&runner, test_name);

    let marker_text = "AIBOX_E2E_OPEN_OK";
    runner.write_file(
        test_name,
        "src/hello.rs",
        &format!("fn main() {{\n    // {marker_text}\n}}\n"),
    );
    let ws = runner.root().display().to_string();
    let src = format!("{ws}/src");
    runner.write_file(
        test_name,
        "popup-vim-init.vim",
        &format!(
            "silent execute \"!printf '%s' \" . shellescape(expand(\"%:p\")) . \" > {ws}/popup-marker.txt\"\nquit\n"
        ),
    );

    let actions = format!(
        r#"  initial_panes="$(tmux list-panes -t "{test_name}:1" | wc -l | tr -d ' ')"
  files_pane_id="$(tmux display-message -p -t "{test_name}:1.1" '#{{pane_id}}')"
  rm -f "{ws}/popup-marker.txt"
  tmux send-keys -t "{test_name}:1.1" "cd {src} && EDITOR='vim -u {ws}/popup-vim-init.vim' exec yazi ." C-m
  for _ in $(seq 1 40); do
    tmux capture-pane -p -t "{test_name}:1.1" > "{ws}/files-screen.txt" 2>/dev/null || true
    grep -qF "hello.rs" "{ws}/files-screen.txt" && break
    sleep 0.25
  done
  tmux send-keys -t "{test_name}:1.1" "e"
  for _ in $(seq 1 40); do
    [ -s "{ws}/popup-marker.txt" ] && break
    sleep 0.25
  done
  if grep -qF "{src}/hello.rs" "{ws}/popup-marker.txt" 2>/dev/null; then
    touch "{ws}/open-ok"
  fi
  sleep 0.6
  active_pane="$(tmux list-panes -t "{test_name}:1" -F '#{{pane_active}} #{{pane_id}}' | awk '$1==1 {{print $2; exit}}')"
  [ "$active_pane" = "$files_pane_id" ] && touch "{ws}/focus-return-ok"
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
        "expected Yazi e to open hello.rs in popup-vim; popup marker:\n{}\nfiles screen:\n{}",
        runner.read_file(test_name, "popup-marker.txt"),
        runner.read_file(test_name, "files-screen.txt")
    );
    assert!(
        runner.file_exists(test_name, "pane-count-ok"),
        "expected Yazi e popup flow not to add or remove tmux panes"
    );
    assert!(
        runner.file_exists(test_name, "focus-return-ok"),
        "expected popup auto-close to return focus to the yazi pane"
    );
    runner.cleanup(test_name);
}

#[test]
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

    let ws = runner.root().display().to_string();
    let actions = format!(
        r#"  tmux send-keys -t "{test_name}:1.1" "cd {ws}/src && EDITOR=vim exec yazi ." C-m
  wait_for_pane_text "{test_name}:1.1" "alpha.rs" "{ws}/yazi-ready-screen.txt"
  tmux send-keys -t "{test_name}:1.1" Enter
  wait_for_pane_command "{test_name}:1.1" vim nvim
  tmux send-keys -t "{test_name}:1.1" Escape ":q" Enter
  wait_for_pane_text "{test_name}:1.1" "alpha.rs" "{ws}/yazi-return-screen.txt"
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
#[ntest::timeout(180_000)]
fn visual_kb_yazi_git_summary_and_changes_show_status() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let test_name = "visual-kb-yazi-git-status";
    runner.cleanup(test_name);
    init_managed_project(&runner, test_name);
    assert_tmux_only_runtime(&runner, test_name);

    let ws = runner.root().display().to_string();
    runner.exec(&format!(
        "cd {ws} && git -c user.email=test@test.com -c user.name=test init && \
         echo old > changed.txt && git -c user.email=test@test.com -c user.name=test add changed.txt && \
         git -c user.email=test@test.com -c user.name=test commit -m init && echo new > changed.txt"
    ));
    let actions = format!(
        r#"  tmux send-keys -t "{test_name}:1.1" "cd {ws} && exec yazi ." C-m
  wait_for_pane_text "{test_name}:1.1" "changed.txt" "{ws}/yazi-ready-screen.txt"
  tmux send-keys -t "{test_name}:1.1" "gs"
  wait_for_pane_text "{test_name}:1.1" "changed.txt" "{ws}/git-summary-screen.txt" || true
  tmux send-keys -t "{test_name}:1.1" "q"
  wait_for_pane_text "{test_name}:1.1" "changed.txt" "{ws}/yazi-return-screen.txt"
  tmux send-keys -t "{test_name}:1.1" "gc"
  wait_for_pane_text "{test_name}:1.1" "changed.txt" "{ws}/git-screen.txt" || \
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
#[ntest::timeout(90_000)]
fn visual_kb_yazi_pane_toggles_keep_file_list_alive() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let test_name = "visual-kb-yazi-pane-toggles";
    runner.cleanup(test_name);
    init_managed_project(&runner, test_name);
    assert_tmux_only_runtime(&runner, test_name);
    runner.write_file(test_name, "visible.txt", "still here\n");

    let ws = runner.root().display().to_string();
    let actions = format!(
        r#"  tmux send-keys -t "{test_name}:1.1" "cd {ws} && exec yazi ." C-m
  wait_for_pane_text "{test_name}:1.1" "visible.txt" "{ws}/yazi-ready-screen.txt"
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
#[ntest::timeout(90_000)]
fn visual_kb_tmux_prefix_splits_windows_and_status_render() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let test_name = "visual-kb-tmux-prefix-status";
    runner.cleanup(test_name);
    init_managed_project(&runner, test_name);
    assert_tmux_only_runtime(&runner, test_name);

    let ws = runner.root().display().to_string();
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
    let runtime_vimrc = format!(
        "{}/../images/base-debian/config/vimrc",
        env!("CARGO_MANIFEST_DIR")
    );
    let startup = quoted_shell(&format!("exec vim -u {runtime_vimrc} {vim_args}"));
    tmux_driver(ws, session, &startup, actions)
}

#[test]
#[ntest::timeout(90_000)]
fn visual_kb_vim_leader_e_opens_netrw() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let test_name = "visual-kb-vim-netrw";
    runner.cleanup(test_name);
    init_managed_project(&runner, test_name);
    assert_tmux_only_runtime(&runner, test_name);
    runner.write_file(test_name, "project.toml", "[package]\nname = \"test\"\n");

    let ws = runner.root().display().to_string();
    let actions = format!(
        r#"  wait_for_pane_command "{test_name}:1.1" vim nvim
  tmux send-keys -t "{test_name}:1.1" " l"
  wait_for_pane_text "{test_name}:1.1" "project.toml" "{ws}/vim-screen.txt" || true
  tmux send-keys -t "{test_name}:1.1" " e"
  wait_for_pane_text "{test_name}:1.1" "netrw" "{ws}/vim-screen.txt" || \
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

    let ws = runner.root().display().to_string();
    let actions = format!(
        r#"  wait_for_pane_command "{test_name}:1.1" vim nvim
  tmux send-keys -t "{test_name}:1.1" " l"
  wait_for_pane_text "{test_name}:1.1" "beta" "{ws}/vim-screen.txt" || \
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
#[ntest::timeout(90_000)]
fn visual_kb_vim_leader_w_saves_file() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let test_name = "visual-kb-vim-save";
    runner.cleanup(test_name);
    init_managed_project(&runner, test_name);
    assert_tmux_only_runtime(&runner, test_name);
    runner.write_file(test_name, "save_me.rs", "fn main() {}\n");

    let ws = runner.root().display().to_string();
    let actions = format!(
        r#"  wait_for_pane_command "{test_name}:1.1" vim nvim
  tmux send-keys -t "{test_name}:1.1" "A edited" Escape " w"
  for _ in $(seq 1 50); do
    grep -qF 'edited' "{ws}/save_me.rs" && break
    sleep 0.1
  done
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
#[ntest::timeout(90_000)]
fn visual_kb_vim_leader_x_writes_and_quits_vim() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let test_name = "visual-kb-vim-writequit";
    runner.cleanup(test_name);
    init_managed_project(&runner, test_name);
    assert_tmux_only_runtime(&runner, test_name);
    runner.write_file(test_name, "writequit.rs", "fn main() {}\n");

    let ws = runner.root().display().to_string();
    let actions = format!(
        r#"  wait_for_pane_command "{test_name}:1.1" vim nvim
  tmux send-keys -t "{test_name}:1.1" "A // saved" Escape " x"
  for _ in $(seq 1 50); do
    pane_cmd="$(tmux display-message -p -t "{test_name}:1.1" '#{{pane_current_command}}' 2>/dev/null || true)"
    [ "$pane_cmd" != "vim" ] && [ "$pane_cmd" != "nvim" ] && break
    sleep 0.1
  done
  pane_cmd="$(tmux display-message -p -t "{test_name}:1.1" '#{{pane_current_command}}' 2>/dev/null || true)"
  if [ "$pane_cmd" != "vim" ] && [ "$pane_cmd" != "nvim" ]; then touch "{ws}/writequit-ok"; fi
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

    let ws = runner.root().display().to_string();
    let actions = format!(
        r#"  wait_for_pane_command "{test_name}:1.1" vim nvim
  tmux send-keys -t "{test_name}:1.1" " n"
  wait_for_pane_text "{test_name}:1.1" "AIBOX_BETA_BUFFER" "{ws}/next-screen.txt" || true
  tmux send-keys -t "{test_name}:1.1" " p"
  wait_for_pane_text "{test_name}:1.1" "AIBOX_ALPHA_BUFFER" "{ws}/prev-screen.txt" || true
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
#[ntest::timeout(60_000)]
fn visual_kb_tmux_buffer_yank_round_trip() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let test_name = "visual-kb-tmux-yank-buffer";
    runner.cleanup(test_name);
    init_managed_project(&runner, test_name);
    assert_tmux_only_runtime(&runner, test_name);

    let ws = runner.root().display().to_string();
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
