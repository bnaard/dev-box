//! Release-gated visual matrix tests for the tmux runtime.
//!
//! The matrix keeps the sidecar/asciinema artifact flow and verifies generated
//! tmux configuration, layouts, tool windows, harness windows, and Yazi previews.

use serde_json::json;
use serial_test::serial;
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

use super::runner::E2eRunner;

// Per-theme broad coverage (ANSI status-bar invariants I1–I4) is now in
// `scripts/test-screencasts.sh` — it exercises all 61 theme slugs on every
// CI run without spinning up containers. This companion test is a deployment
// smoke test: it only needs to prove the end-to-end pipeline (init → apply →
// tmux config → asciinema cast) works for one representative theme.
//
// To restore the full 7-theme sweep (e.g. on a release branch), set:
//   AIBOX_E2E_VISUAL_FULL_MATRIX=1
// The full list is preserved in FULL_MATRIX_THEMES below.
const THEMES: &[(&str, u8, u8, u8)] = &[("projectious", 45, 106, 79)];

/// Full theme list used when AIBOX_E2E_VISUAL_FULL_MATRIX=1 is set.
/// Accent RGB values are sampled from each theme's `green` palette entry,
/// which is what the generated tmux config embeds for colour verification.
const FULL_MATRIX_THEMES: &[(&str, u8, u8, u8)] = &[
    ("gruvbox-dark", 152, 151, 26),
    ("catppuccin-mocha", 166, 227, 161),
    ("catppuccin-latte", 64, 160, 43),
    ("dracula", 80, 250, 123),
    ("tokyo-night", 158, 206, 106),
    ("nord", 163, 190, 140),
    ("projectious", 45, 106, 79),
];

const LAYOUTS: &[&str] = &["ai", "dev", "focus", "cowork"];

const HARNESSES: &[(&str, &str, &str)] = &[
    ("claude", "claude", "CLAUDE"),
    ("codex", "codex", "CODEX"),
    ("gemini", "gemini", "GEMINI"),
    ("aider", "aider", "AIDER"),
    ("continue", "cn", "CONTINUE"),
    ("cursor", "cursor", "CURSOR"),
    ("copilot", "copilot", "COPILOT"),
    ("opencode", "opencode", "OPENCODE"),
    ("hermes", "hermes", "HERMES"),
];

const DEFAULT_STATUS_THEME: &str = "projectious";
const DEFAULT_STATUS_LAYOUT: &str = "dev";
const DEFAULT_TAB_LAYOUTS: &[&str] = &["ai", "dev", "cowork"];

fn full_visual_matrix_enabled() -> bool {
    matches!(
        std::env::var("AIBOX_E2E_VISUAL_FULL_MATRIX")
            .ok()
            .as_deref(),
        Some("1" | "true" | "yes" | "full")
    )
}

/// Returns the active theme list: full 7-theme sweep when
/// `AIBOX_E2E_VISUAL_FULL_MATRIX` is set, otherwise the 1-theme smoke list.
fn active_themes() -> &'static [(&'static str, u8, u8, u8)] {
    if full_visual_matrix_enabled() {
        FULL_MATRIX_THEMES
    } else {
        THEMES
    }
}

fn status_matrix_layouts_for_theme(theme: &str) -> Vec<&'static str> {
    if full_visual_matrix_enabled() || theme == DEFAULT_STATUS_THEME {
        LAYOUTS.to_vec()
    } else {
        vec![DEFAULT_STATUS_LAYOUT]
    }
}

fn status_matrix_total_cases() -> usize {
    let themes = active_themes();
    if full_visual_matrix_enabled() {
        themes.len() * LAYOUTS.len()
    } else {
        LAYOUTS.len() + themes.len() - 1
    }
}

fn tab_matrix_layouts() -> &'static [&'static str] {
    if full_visual_matrix_enabled() {
        LAYOUTS
    } else {
        DEFAULT_TAB_LAYOUTS
    }
}

fn rgb_hex(r: u8, g: u8, b: u8) -> String {
    format!("#{r:02X}{g:02X}{b:02X}")
}

fn log_visual_progress(message: impl AsRef<str>) {
    eprintln!("[visual-e2e] {}", message.as_ref());
}

struct VisualProgressStep {
    label: String,
    started_at: Instant,
    stop: Option<mpsc::Sender<()>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl VisualProgressStep {
    fn start(label: impl Into<String>) -> Self {
        let label = label.into();
        log_visual_progress(format!("start: {label}"));
        let started_at = Instant::now();
        let heartbeat_started_at = started_at;
        let heartbeat_label = label.clone();
        let (stop, stopped) = mpsc::channel();
        let handle = thread::spawn(move || {
            loop {
                match stopped.recv_timeout(Duration::from_secs(15)) {
                    Ok(()) | Err(RecvTimeoutError::Disconnected) => break,
                    Err(RecvTimeoutError::Timeout) => {
                        log_visual_progress(format!(
                            "alive: {heartbeat_label} elapsed={}s",
                            heartbeat_started_at.elapsed().as_secs()
                        ));
                    }
                }
            }
        });
        Self {
            label,
            started_at,
            stop: Some(stop),
            handle: Some(handle),
        }
    }
}

impl Drop for VisualProgressStep {
    fn drop(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
        log_visual_progress(format!(
            "done: {} elapsed={}s",
            self.label,
            self.started_at.elapsed().as_secs()
        ));
    }
}

fn visual_artifact_dir() -> Option<PathBuf> {
    std::env::var_os("AIBOX_E2E_VISUAL_ARTIFACT_DIR")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn write_visual_artifacts(
    test_name: &str,
    stem: &str,
    cast: &str,
    screen: Option<&str>,
    logs: &str,
    metadata: serde_json::Value,
) {
    let Some(root) = visual_artifact_dir() else {
        return;
    };
    let dir = root.join(test_name);
    fs::create_dir_all(&dir)
        .unwrap_or_else(|err| panic!("failed to create visual artifact dir {dir:?}: {err}"));
    fs::write(dir.join(format!("{stem}.cast")), cast)
        .unwrap_or_else(|err| panic!("failed to write visual artifact cast {stem}: {err}"));
    if let Some(screen) = screen {
        fs::write(dir.join(format!("{stem}.screen.txt")), screen)
            .unwrap_or_else(|err| panic!("failed to write visual artifact screen {stem}: {err}"));
    }
    fs::write(dir.join(format!("{stem}.tmux.log")), logs)
        .unwrap_or_else(|err| panic!("failed to write visual artifact logs {stem}: {err}"));
    fs::write(
        dir.join(format!("{stem}.metadata.json")),
        serde_json::to_string_pretty(&metadata).expect("visual metadata should serialize"),
    )
    .unwrap_or_else(|err| panic!("failed to write visual artifact metadata {stem}: {err}"));
}

fn init_project(
    runner: &E2eRunner,
    test_name: &str,
    theme: &str,
    all_harnesses: bool,
    addons: &[&str],
) {
    let _progress = VisualProgressStep::start(format!(
        "init project={test_name} theme={theme} addons={}",
        addons.join(",")
    ));
    runner.cleanup(test_name);
    let mut args = vec![
        "init",
        test_name,
        "--base",
        "debian",
        "--context",
        "managed",
        "--processkit-version",
        "unset",
        "--theme",
        theme,
        "--harness",
    ];
    if all_harnesses {
        args.extend(HARNESSES.iter().map(|(name, _, _)| *name));
    } else {
        args.push("claude");
    }
    for addon in addons {
        args.extend(["--addon", addon]);
    }
    args.push("--no-container");

    let init = runner.aibox(test_name, &args);
    assert!(
        init.status.success(),
        "{test_name}: init failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&init.stdout),
        String::from_utf8_lossy(&init.stderr)
    );
    let apply = runner.aibox(test_name, &["apply", "--no-container"]);
    assert!(
        apply.status.success(),
        "{test_name}: apply failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&apply.stdout),
        String::from_utf8_lossy(&apply.stderr)
    );
}

fn assert_generated_tmux_config(
    runner: &E2eRunner,
    test_name: &str,
    theme: &str,
    r: u8,
    g: u8,
    b: u8,
) {
    let workspace = format!("/workspaces/{test_name}");
    let expected = rgb_hex(r, g, b);
    let theme_pattern = theme.replace('-', "[- ]");
    let probe = runner.exec(&format!(
        r#"cd {workspace}
	test -f .aibox-home/.tmux.conf -o -f .aibox-home/.config/tmux/tmux.conf
	grep -Rli --exclude-dir=.git --exclude=claude 'tmux' .aibox-home .devcontainer aibox.toml >/tmp/{test_name}-tmux-files.txt
	grep -REi --exclude-dir=.git --exclude=claude '{theme_pattern}' .aibox-home >/tmp/{test_name}-theme.txt
	grep -Ri --exclude-dir=.git --exclude=claude '{expected}' .aibox-home >/tmp/{test_name}-theme-rgb.txt
	grep -F '@powerkit_line1_right "aibox_log,aibox_oom,aibox_proc,aibox_ai,aibox_mcp,aibox_mig,weather,uptime,datetime"' .aibox-home/.config/tmux/tmux.conf
	grep -F '@powerkit_line2_left "git,github,kubernetes,terraform,cloud"' .aibox-home/.config/tmux/tmux.conf
	grep -F '@powerkit_line2_right "hostname,externalip,ssh,netspeed,ping,cpu,loadavg,memory,swap,disk,gpu"' .aibox-home/.config/tmux/tmux.conf
	grep -F '@powerkit_plugin_netspeed_speed_width "7"' .aibox-home/.config/tmux/tmux.conf
	grep -F 'status-format[0]' .aibox-home/.config/tmux/tmux.conf | grep -F 'align=right'
	grep -F 'status-format[1]' .aibox-home/.config/tmux/tmux.conf | grep -F 'align=left' | grep -F 'align=right'
! find .aibox-home -path '*zellij*' -print -quit | grep -q .
! grep -Rli --exclude-dir=.git 'zellij' .aibox-home .devcontainer aibox.toml >/tmp/{test_name}-legacy-zellij.txt 2>/dev/null
"#
    ));
    assert!(
        probe.status.success(),
        "{test_name}: expected tmux-only generated config for {theme}/{expected}\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&probe.stdout),
        String::from_utf8_lossy(&probe.stderr)
    );
}

fn generated_layout(runner: &E2eRunner, test_name: &str, layout: &str) -> String {
    let workspace = format!("/workspaces/{test_name}");
    let output = runner.exec(&format!(
        "cd {workspace} && cat .aibox-home/.config/tmux/layouts/{layout}.sh 2>/dev/null"
    ));
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn assert_generated_tmux_layout(layout: &str, body: &str) {
    assert!(
        body.contains("tmux") || body.contains("new-window") || body.contains("split-window"),
        "{layout}: expected generated tmux layout/script, got:\n{body}"
    );
    assert!(
        !body.to_ascii_lowercase().contains("zellij"),
        "{layout}: generated tmux layout should not reference zellij:\n{body}"
    );
}

fn install_visual_fixtures(runner: &E2eRunner, test_name: &str) {
    let _progress =
        VisualProgressStep::start(format!("install visual fixtures project={test_name}"));
    let workspace = format!("/workspaces/{test_name}");
    runner.write_file(
        test_name,
        "setup-visual-fixtures.sh",
        &format!(
            r#"#!/usr/bin/env bash
set -eu
cd {workspace}
export HOME="{workspace}/.aibox-home"
mkdir -p "$HOME/.local/bin" visual-fixtures/nested

cat > visual-fixtures/MATRIX_README.md <<'EOF'
# AIBOX YAZI RICH PREVIEW
Preview matrix marker.
EOF
cat > visual-fixtures/source.rs <<'EOF'
fn main() {{
    println!("AIBOX_VIM_SOURCE_MARKER");
}}
EOF
cat > visual-fixtures/data.csv <<'EOF'
city,count
Berlin,3
Lisbon,5
EOF
cat > visual-fixtures/data.tsv <<'EOF'
city	count
Oslo	8
EOF
cat > visual-fixtures/nested/child.txt <<'EOF'
AIBOX_DIRECTORY_PREVIEW_CHILD
EOF

cat > "$HOME/.local/bin/csvlook" <<'EOF'
#!/usr/bin/env bash
cat "${{@: -1}}"
EOF
chmod +x "$HOME/.local/bin/csvlook"

cat > "$HOME/.local/bin/in2csv" <<'EOF'
#!/usr/bin/env bash
cat "${{@: -1}}"
EOF
chmod +x "$HOME/.local/bin/in2csv"

cat > "$HOME/.local/bin/sqlite3" <<'EOF'
#!/usr/bin/env bash
case "$1" in
  -*) echo "SQLite database: 3.0"; echo "table people people" ;;
  *) [ -n "${{1:-}}" ] && : > "$1" ;;
esac
EOF
chmod +x "$HOME/.local/bin/sqlite3"

for bin in csvlook in2csv sqlite3; do
  sudo install -m 0755 "$HOME/.local/bin/$bin" "/usr/local/bin/$bin"
done

git init -q
git config user.email matrix@example.test
git config user.name "Matrix Test"
printf 'base\n' > modified.txt
printf 'delete me\n' > deleted.txt
printf 'conflict base\n' > conflict.txt
git add modified.txt deleted.txt conflict.txt
git commit -m baseline >/dev/null
printf 'changed\n' > modified.txt
rm deleted.txt
printf 'new\n' > added.txt
git add added.txt
printf 'loose\n' > untracked.txt
printf 'ignored.txt\n' > .gitignore
printf 'ignored\n' > ignored.txt
base_branch="$(git branch --show-current)"
git checkout -q -b side
printf 'side\n' > conflict.txt
git commit -am side >/dev/null
git checkout -q "$base_branch"
printf 'main\n' > conflict.txt
git commit -am main >/dev/null
git merge side >/dev/null 2>&1 || true

sqlite3 visual-fixtures/sample.sqlite 'CREATE TABLE people (name TEXT, count INTEGER); INSERT INTO people VALUES ("Aibox", 42);' || true

cat > "$HOME/.bashrc" <<'EOF'
echo AIBOX-SHELL-READY
EOF

cat > "$HOME/.local/bin/lazygit" <<'EOF'
#!/usr/bin/env bash
printf '\033[2J\033[H'
echo AIBOX-LAZYGIT-READY
while true; do sleep 1; done
EOF
chmod +x "$HOME/.local/bin/lazygit"

cat > "$HOME/.local/bin/vim-loop" <<'EOF'
#!/usr/bin/env bash
printf '\033[2J\033[H'
echo AIBOX-VIM-READY
while true; do sleep 1; done
EOF
chmod +x "$HOME/.local/bin/vim-loop"

cat > "$HOME/.local/bin/vim" <<'EOF'
#!/usr/bin/env bash
exec vim-loop
EOF
chmod +x "$HOME/.local/bin/vim"
"#
        ),
    );
    for (tab, bin, marker) in HARNESSES {
        let script = format!(
            r#"#!/usr/bin/env bash
printf '\033[2J\033[H'
echo AIBOX-HARNESS-{marker}
echo "window={tab} binary={bin}"
while true; do sleep 1; done
"#
        );
        runner.write_file(test_name, &format!(".aibox-home/.local/bin/{bin}"), &script);
        runner.exec(&format!(
            "chmod +x {workspace}/.aibox-home/.local/bin/{bin}"
        ));
    }
    let output = runner.exec(&format!("bash {workspace}/setup-visual-fixtures.sh"));
    assert!(
        output.status.success(),
        "{test_name}: fixture setup failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn tmux_conf_and_start(session: &str, workspace: &str, layout: &str, setup: &str) -> String {
    format!(
        r#"#!/usr/bin/env bash
set -u
export HOME="{workspace}/.aibox-home"
export TERM=xterm-256color
export COLORTERM=truecolor
export PATH="$HOME/.local/bin:/usr/local/bin:$PATH"
tmux_socket="${{AIBOX_TMUX_SOCKET:-$HOME/.tmux/aibox.sock}}"
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
ln -sf "$tmux_conf" "$HOME/.tmux.conf"
layout_script="$HOME/.config/tmux/layouts/{layout}.sh"
if [ ! -x "$layout_script" ]; then
  echo "missing executable generated tmux layout: $layout_script" >&2
  exit 91
fi
tmux kill-session -t "{session}" >/dev/null 2>&1 || true
(
  for _ in $(seq 1 50); do
    tmux has-session -t "{session}" >/dev/null 2>&1 && break
    sleep 0.1
  done
  if ! tmux has-session -t "{session}" >/dev/null 2>&1; then
    echo "generated tmux layout did not create session {session}" > "{workspace}/{session}.driver-error"
    exit 1
  fi
  {{
    echo "--- generated windows ---"
    tmux list-windows -t "{session}" -F '#I #{{window_name}} #{{window_panes}}'
    echo "--- generated panes ---"
    tmux list-panes -t "{session}:" -F '#I.#P #{{pane_current_command}} #{{pane_title}}'
  }} > "{workspace}/{session}.generated-state" 2>&1 || true
{setup}
  sleep 0.5
	  tmux list-windows -t "{session}" > "{workspace}/{session}.windows" 2>/dev/null || true
	  tmux list-panes -a -F '#S:#I.#P #{{pane_current_command}} #{{pane_title}}' > "{workspace}/{session}.panes" 2>/dev/null || true
	  tmux display-message -p -t "{session}" '#S #W #{{window_panes}} #{{status-left}} #{{status-right}}' > "{workspace}/{session}.status" 2>/dev/null || true
	  first_pane="$(tmux list-panes -t "{session}:" -F '#{{window_index}}.#{{pane_index}}' | head -1 || true)"
  if [ -n "$first_pane" ]; then
    tmux capture-pane -p -t "{session}:$first_pane" > "{workspace}/{session}.screen" 2>/dev/null || true
  fi
  tmux kill-session -t "{session}" >/dev/null 2>&1 || true
) &
driver_pid=$!
AIBOX_TMUX_SESSION="{session}" AIBOX_WORKSPACE="{workspace}" AIBOX_TMUX_CONFIG="$tmux_conf" AIBOX_TMUX_SOCKET="$tmux_socket" "$layout_script"
wait "$driver_pid" 2>/dev/null || true
if [ -s "{workspace}/{session}.driver-error" ]; then
  cat "{workspace}/{session}.driver-error" >&2
  exit 92
fi
true
"#
    )
}

fn record_layout_status(runner: &E2eRunner, test_name: &str, layout: &str) -> (String, String) {
    let _progress =
        VisualProgressStep::start(format!("record status project={test_name} layout={layout}"));
    let workspace = format!("/workspaces/{test_name}");
    let stem = format!("recording-status-{layout}");
    let setup = format!(
        r#"  tmux set-option -t "{stem}" -g status on
  tmux set-option -t "{stem}" -g status-left-length 80
  tmux set-option -t "{stem}" -g status-right-length 100
  tmux set-option -t "{stem}" -g status-left " AIBOX-TMUX {layout} #S:#I.#P "
  tmux set-option -t "{stem}" -g status-right " #(aibox-status 2>/dev/null | cut -c1-80) "
  sleep 3
"#
    );
    runner.write_file(
        test_name,
        &format!("driver-status-{layout}.sh"),
        &tmux_conf_and_start(&stem, &workspace, layout, &setup),
    );
    runner.exec(&format!("chmod +x {workspace}/driver-status-{layout}.sh"));
    runner.exec(&format!(
        "LC_ALL=C.UTF-8 LANG=C.UTF-8 timeout --kill-after=2s 45s asciinema rec --cols 160 --rows 45 --overwrite \
         -c {workspace}/driver-status-{layout}.sh {workspace}/{stem}.cast 2>/dev/null; true"
    ));
    let cast = runner.read_file(test_name, &format!("{stem}.cast"));
    let screen = runner.read_file(test_name, &format!("{stem}.screen"));
    let generated_state = runner.read_file(test_name, &format!("{stem}.generated-state"));
    let logs = format!(
        "{generated_state}\n--- final windows ---\n{}--- final status ---\n{}",
        runner.read_file(test_name, &format!("{stem}.windows")),
        runner.read_file(test_name, &format!("{stem}.status"))
    );
    write_visual_artifacts(
        test_name,
        &stem,
        &cast,
        Some(&screen),
        &logs,
        json!({"kind":"status-theme","layout":layout,"cols":160,"rows":45}),
    );
    (format!("{cast}\n{screen}"), logs)
}

fn record_generated_layout(runner: &E2eRunner, test_name: &str, layout: &str) -> (String, String) {
    let _progress =
        VisualProgressStep::start(format!("record tabs project={test_name} layout={layout}"));
    let workspace = format!("/workspaces/{test_name}");
    let stem = format!("recording-{layout}");
    let setup = format!(
        r#"  tmux new-window -t "{stem}" -n synthetic-files -c "{workspace}" "cd {workspace} && exec yazi ."
  tmux new-window -t "{stem}" -n synthetic-editor -c "{workspace}" "exec vim-loop"
  tmux new-window -t "{stem}" -n synthetic-git -c "{workspace}" "exec lazygit"
  tmux new-window -t "{stem}" -n synthetic-shell -c "{workspace}" "bash -lc 'echo AIBOX-SHELL-READY; exec bash'"
"#
    );
    let mut harness_windows = String::new();
    for (tab, bin, _) in HARNESSES {
        harness_windows.push_str(&format!(
            "  tmux new-window -t \"{stem}\" -n synthetic-{tab} -c \"{workspace}\" \"exec {bin}\"\n"
        ));
    }
    let capture = format!(
        r#"{setup}{harness_windows}
  : > "{workspace}/{stem}.screens"
  for win in work files ai lazygit shell claude codex gemini aider continue cursor copilot opencode hermes synthetic-files synthetic-editor synthetic-git synthetic-shell synthetic-claude synthetic-codex synthetic-gemini synthetic-aider synthetic-continue synthetic-cursor synthetic-copilot synthetic-opencode synthetic-hermes; do
    tmux select-window -t "{stem}:$win" >/dev/null 2>&1 || continue
    sleep 0.4
    printf '\n--- window:%s ---\n' "$win" >> "{workspace}/{stem}.screens"
    pane_target="$(tmux list-panes -t "{stem}:$win" -F '#{{window_index}}.#{{pane_index}}' | head -1 || true)"
    [ -n "$pane_target" ] || continue
    tmux capture-pane -p -t "{stem}:$pane_target" >> "{workspace}/{stem}.screens" 2>/dev/null || true
  done
"#
    );
    runner.write_file(
        test_name,
        &format!("driver-{layout}.sh"),
        &tmux_conf_and_start(&stem, &workspace, layout, &capture),
    );
    runner.exec(&format!("chmod +x {workspace}/driver-{layout}.sh"));
    runner.exec(&format!(
        "LC_ALL=C.UTF-8 LANG=C.UTF-8 timeout --kill-after=2s 90s asciinema rec --cols 160 --rows 45 --overwrite \
         -c {workspace}/driver-{layout}.sh {workspace}/{stem}.cast 2>/dev/null; true"
    ));
    let cast = runner.read_file(test_name, &format!("{stem}.cast"));
    let screens = runner.read_file(test_name, &format!("{stem}.screens"));
    let generated_state = runner.read_file(test_name, &format!("{stem}.generated-state"));
    let logs = format!(
        "{generated_state}\n--- final windows ---\n{}--- final panes ---\n{}",
        runner.read_file(test_name, &format!("{stem}.windows")),
        runner.read_file(test_name, &format!("{stem}.panes"))
    );
    write_visual_artifacts(
        test_name,
        &stem,
        &cast,
        Some(&screens),
        &logs,
        json!({"kind":"tab-traversal","layout":layout,"cols":160,"rows":45}),
    );
    (format!("{cast}\n{screens}"), logs)
}

fn expected_generated_window(layout: &str) -> &str {
    match layout {
        "focus" => "files",
        "ai" | "dev" | "cowork" => "work",
        _ => "work",
    }
}

fn assert_generated_layout_created_real_tmux_surfaces(layout: &str, logs: &str) {
    let expected_window = expected_generated_window(layout);
    assert!(
        logs.contains("--- generated windows ---") && logs.contains("--- generated panes ---"),
        "{layout}: missing generated tmux state capture before synthetic windows:\n{logs}"
    );
    assert!(
        logs.lines().any(|line| {
            line.split_whitespace()
                .nth(1)
                .is_some_and(|window| window == expected_window)
        }),
        "{layout}: expected generated window {expected_window:?} before synthetic windows:\n{logs}"
    );
    let generated_pane_count = logs
        .lines()
        .skip_while(|line| *line != "--- generated panes ---")
        .skip(1)
        .take_while(|line| *line != "--- final windows ---")
        .filter(|line| !line.trim().is_empty())
        .count();
    assert!(
        generated_pane_count > 0,
        "{layout}: expected generated layout to create real tmux panes before synthetic windows:\n{logs}"
    );
}

#[test]
#[serial]
#[ignore = "visual e2e matrix is release-gated; run explicitly via scripts/maintain.sh test-e2e-visual-status or test-e2e-visual"]
#[ntest::timeout(720_000)]
fn visual_generated_layouts_render_across_all_themes() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let total_cases = status_matrix_total_cases();
    let themes = active_themes();
    let mut case = 0;
    for (theme_index, &(theme, r, g, b)) in themes.iter().enumerate() {
        let test_name = format!("visual-matrix-theme-{theme}");
        log_visual_progress(format!(
            "status matrix: init theme {}/{} ({theme})",
            theme_index + 1,
            themes.len()
        ));
        init_project(&runner, &test_name, theme, false, &["git-ui"]);
        install_visual_fixtures(&runner, &test_name);
        assert_generated_tmux_config(&runner, &test_name, theme, r, g, b);

        for layout in status_matrix_layouts_for_theme(theme) {
            case += 1;
            log_visual_progress(format!(
                "status matrix [{case}/{total_cases}]: recording theme={theme} layout={layout}"
            ));
            let layout_body = generated_layout(&runner, &test_name, layout);
            assert_generated_tmux_layout(layout, &layout_body);
            let (recording, logs) = record_layout_status(&runner, &test_name, layout);
            assert_generated_layout_created_real_tmux_surfaces(layout, &logs);
            let status_evidence = format!("{recording}\n{logs}");
            assert!(
                status_evidence.contains("AIBOX-TMUX")
                    || status_evidence.contains("MEM ")
                    || status_evidence.contains("PROC "),
                "{theme}/{layout}: expected visible tmux status output:\n{recording}"
            );
            assert!(
                !recording.to_ascii_lowercase().contains("zellij"),
                "{theme}/{layout}: recording should not mention legacy multiplexer:\n{recording}"
            );
        }
        runner.cleanup(&test_name);
    }
}

#[test]
#[serial]
#[ignore = "visual tab-traversal e2e is release-gated; run explicitly via scripts/maintain.sh test-e2e-visual-tabs or test-e2e-visual"]
#[ntest::timeout(300_000)]
fn visual_generated_tools_and_harness_windows_render_when_enabled() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let test_name = "visual-matrix-tools-harnesses";
    init_project(
        &runner,
        test_name,
        "projectious",
        true,
        &["git-ui", "data-preview", "preview-enhanced", "yazi-omp"],
    );
    install_visual_fixtures(&runner, test_name);

    for layout in LAYOUTS {
        let layout_body = generated_layout(&runner, test_name, layout);
        assert_generated_tmux_layout(layout, &layout_body);
        assert!(
            layout_body.contains("yazi") || layout_body.contains("\"vim\""),
            "{layout}: expected generated tmux layout to include real tool surfaces:\n{layout_body}"
        );
    }

    let visual_layouts = tab_matrix_layouts();
    for (index, layout) in visual_layouts.iter().enumerate() {
        let case = index + 1;
        let total = visual_layouts.len();
        log_visual_progress(format!(
            "tabs matrix [{case}/{total}]: recording layout={layout}"
        ));
        let (recording, logs) = record_generated_layout(&runner, test_name, layout);
        assert_generated_layout_created_real_tmux_surfaces(layout, &logs);
        assert!(
            recording.contains("AIBOX-LAZYGIT-READY"),
            "{layout}: missing lazygit"
        );
        assert!(
            recording.contains("AIBOX-SHELL-READY"),
            "{layout}: missing shell"
        );
        assert!(
            recording.contains("AIBOX-VIM-READY"),
            "{layout}: missing editor"
        );
        for (_, _, marker) in HARNESSES {
            assert!(
                recording.contains(&format!("AIBOX-HARNESS-{marker}")),
                "{layout}: expected harness marker {marker}"
            );
        }
    }

    runner.cleanup(test_name);
}

#[test]
#[serial]
#[ignore = "visual Yazi preview e2e is release-gated; run explicitly via scripts/maintain.sh test-e2e-visual-yazi or test-e2e-visual"]
#[ntest::timeout(300_000)]
fn visual_yazi_previews_git_symbols_and_optional_plugins_render() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let test_name = "visual-matrix-yazi-previews";
    runner.exec(
        "timeout 2s tmux kill-server >/dev/null 2>&1 || true; \
         timeout 2s pkill -x yazi >/dev/null 2>&1 || true; \
         timeout 2s pkill -x asciinema >/dev/null 2>&1 || true",
    );
    init_project(
        &runner,
        test_name,
        "tokyo-night",
        false,
        &["git-ui", "data-preview", "preview-enhanced"],
    );
    install_visual_fixtures(&runner, test_name);

    let workspace = format!("/workspaces/{test_name}");
    let home = format!("{workspace}/.aibox-home");
    let config_probe = runner.exec(&format!(
        "cd {workspace} && \
         grep -E 'rich-preview|sqlite-preview|tabular-preview|svg|eps|pdf|image|dir-preview' {home}/.config/yazi/yazi.toml && \
         grep -E 'modified = \"M\"|added = \"A\"|deleted = \"D\"|updated = \"U\"|untracked = \"\\?\"|ignored = \"I\"' {home}/.config/yazi/init.lua && \
         test -f {home}/.config/yazi/plugins/rich-preview.yazi/main.lua && \
         test -f {home}/.config/yazi/plugins/sqlite-preview.yazi/main.lua && \
         test -f {home}/.config/yazi/plugins/tabular-preview.yazi/main.lua && \
         test ! -e {home}/.config/yazi/plugins/omp.yazi/main.lua && \
         test ! -e {home}/.config/yazi/yazi-prompt.omp.json"
    ));
    assert!(
        config_probe.status.success(),
        "Yazi preview/git config probe failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&config_probe.stdout),
        String::from_utf8_lossy(&config_probe.stderr)
    );

    let entries = [
        ("dir", "visual-fixtures", "MATRIX_README"),
        (
            "markdown",
            "visual-fixtures/MATRIX_README.md",
            "AIBOX YAZI RICH PREVIEW",
        ),
        ("csv", "visual-fixtures/data.csv", "Berlin"),
        ("tsv", "visual-fixtures/data.tsv", "Oslo"),
        ("sqlite", "visual-fixtures/sample.sqlite", "people"),
    ];

    for (index, (label, entry, marker)) in entries.iter().enumerate() {
        let case = index + 1;
        log_visual_progress(format!(
            "yazi matrix [{case}/{}]: recording preview label={label} entry={entry}",
            entries.len()
        ));
        let session = format!("yazi-preview-{label}");
        let setup = format!(
            r#"  tmux new-window -t "{session}" -n "preview-{label}" -c "{workspace}" "exec yazi {workspace}/{entry}"
  tmux select-window -t "{session}:preview-{label}" >/dev/null 2>&1 || true
  for _ in $(seq 1 30); do
    preview_pane="$(tmux list-panes -t "{session}:preview-{label}" -F '#{{window_index}}.#{{pane_index}}' | head -1 || true)"
    [ -n "$preview_pane" ] && tmux capture-pane -p -t "{session}:$preview_pane" > "{workspace}/yazi-preview-{label}.screen" 2>/dev/null || true
    grep -Fq "{marker}" "{workspace}/yazi-preview-{label}.screen" && break
    sleep 0.5
  done
"#
        );
        runner.write_file(
            test_name,
            &format!("yazi-preview-{label}.sh"),
            &tmux_conf_and_start(&session, &workspace, "focus", &setup),
        );
        runner.exec(&format!("chmod +x {workspace}/yazi-preview-{label}.sh"));
        runner.exec(&format!(
            "LC_ALL=C.UTF-8 LANG=C.UTF-8 timeout --kill-after=2s 45s asciinema rec --cols 160 --rows 45 --overwrite \
             -c {workspace}/yazi-preview-{label}.sh {workspace}/yazi-preview-{label}.cast 2>/dev/null; true"
        ));
        let cast = runner.read_file(test_name, &format!("yazi-preview-{label}.cast"));
        let screen = runner.read_file(test_name, &format!("yazi-preview-{label}.screen"));
        write_visual_artifacts(
            test_name,
            &format!("yazi-preview-{label}"),
            &cast,
            Some(&screen),
            "",
            json!({"kind":"yazi-preview","entry":entry,"marker":marker,"cols":160,"rows":45}),
        );
        let recording = format!("{cast}\n{screen}");
        assert!(
            recording.contains(marker),
            "Yazi {label} preview should contain marker {marker:?}:\n{recording}"
        );
    }

    runner.cleanup(test_name);
}
