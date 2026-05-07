//! Generated visual matrix tests for layouts, themes, tools, harnesses, and Yazi previews.
//!
//! These run on the SSH companion with asciinema. They intentionally use the
//! generated `.aibox-home` files from a fresh project instead of hand-written
//! test layouts so regressions in the generator show up in Phase 1.

use serde_json::json;
use serial_test::serial;
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

use super::runner::E2eRunner;

const THEMES: &[(&str, u8, u8, u8)] = &[
    ("gruvbox-dark", 152, 151, 26),
    ("catppuccin-mocha", 166, 227, 161),
    ("catppuccin-latte", 64, 160, 43),
    ("dracula", 80, 250, 123),
    ("tokyo-night", 158, 206, 106),
    ("nord", 163, 190, 140),
    ("projectious", 45, 106, 79),
];

const LAYOUTS: &[&str] = &["dev", "focus", "cowork", "cowork-swap", "browse", "ai"];

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
const DEFAULT_TAB_LAYOUTS: &[&str] = &["dev", "cowork-swap", "ai"];

fn full_visual_matrix_enabled() -> bool {
    matches!(
        std::env::var("AIBOX_E2E_VISUAL_FULL_MATRIX")
            .ok()
            .as_deref(),
        Some("1" | "true" | "yes" | "full")
    )
}

fn status_matrix_layouts_for_theme(theme: &str) -> Vec<&'static str> {
    if full_visual_matrix_enabled() {
        return LAYOUTS.to_vec();
    }

    if theme == DEFAULT_STATUS_THEME {
        LAYOUTS.to_vec()
    } else {
        vec![DEFAULT_STATUS_LAYOUT]
    }
}

fn status_matrix_total_cases() -> usize {
    if full_visual_matrix_enabled() {
        THEMES.len() * LAYOUTS.len()
    } else {
        LAYOUTS.len() + THEMES.len() - 1
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

fn assert_generated_theme_config(
    runner: &E2eRunner,
    test_name: &str,
    theme: &str,
    r: u8,
    g: u8,
    b: u8,
) {
    let config = runner.read_file(test_name, ".aibox-home/.config/zellij/config.kdl");
    assert!(
        config.contains(&format!("theme \"{theme}\"")),
        "{theme}: generated Zellij config should select the requested theme:\n{config}"
    );

    let theme_file = runner.read_file(
        test_name,
        &format!(".aibox-home/.config/zellij/themes/{theme}.kdl"),
    );
    let expected = rgb_hex(r, g, b);
    assert!(
        theme_file.contains(&expected),
        "{theme}: generated theme file should contain expected RGB {expected}:\n{theme_file}"
    );
}

fn generated_layout(runner: &E2eRunner, test_name: &str, layout: &str) -> String {
    runner.read_file(
        test_name,
        &format!(".aibox-home/.config/zellij/layouts/{layout}.kdl"),
    )
}

fn layout_has_top_level_editor_tab(layout_kdl: &str) -> bool {
    layout_kdl.contains("aibox-tab name=\"editor\"")
}

fn layout_has_top_level_tab(layout_kdl: &str, tab: &str) -> bool {
    layout_kdl.contains(&format!("aibox-tab name=\"{tab}\""))
}

fn assert_generated_sidecar_status_layout(layout: &str, layout_kdl: &str) {
    assert!(
        layout_kdl.contains("role \"keys\"") && layout_kdl.contains("role \"status\""),
        "{layout}: expected generated layout to wire sidecar-backed aibox key/status rows:\n{layout_kdl}"
    );
}

fn assert_no_zellij_runtime_errors(logs: &str, label: &str) {
    for bad in [
        "ERROR IN PLUGIN",
        "failed to load plugin",
        "could not find exported function",
        "Panic occured",
        "panicked",
        "Unknown component: z",
    ] {
        assert!(
            !logs.contains(bad),
            "{label}: Zellij logs contain {bad:?}:\n{logs}"
        );
    }
}

fn assert_no_zellij_permission_prompt(recording: &str, label: &str) {
    for bad in [
        "This plugin asks permission",
        "ReadApplicationState",
        "RunCommands",
        "Allow? (y/n)",
    ] {
        assert!(
            !recording.contains(bad),
            "{label}: sidecar status plugin permission prompt leaked into the visual recording ({bad:?}):\n{recording}"
        );
    }
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
    fs::write(dir.join(format!("{stem}.zellij.log")), logs)
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
        "--zellij-status",
        "sidecar",
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
file=""
for arg in "$@"; do
  case "$arg" in
    --*) ;;
    *) file="$arg" ;;
  esac
done
if [ -n "$file" ] && [ -f "$file" ]; then
  cat "$file"
else
  cat
fi
EOF
chmod +x "$HOME/.local/bin/csvlook"

cat > "$HOME/.local/bin/in2csv" <<'EOF'
#!/usr/bin/env bash
file=""
for arg in "$@"; do
  file="$arg"
done
if [ -n "$file" ] && [ -f "$file" ]; then
  cat "$file"
fi
EOF
chmod +x "$HOME/.local/bin/in2csv"

cat > "$HOME/.local/bin/sqlite3" <<'EOF'
#!/usr/bin/env bash
case "$1" in
  -*)
    echo "SQLite database: 3.0"
    echo "table people people"
    ;;
  *)
    [ -n "$1" ] && : > "$1"
    ;;
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

if command -v sqlite3 >/dev/null 2>&1; then
  sqlite3 visual-fixtures/sample.sqlite 'CREATE TABLE people (name TEXT, count INTEGER); INSERT INTO people VALUES ("Aibox", 42);'
fi

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

cat > "$HOME/.local/bin/oh-my-posh" <<'EOF'
#!/usr/bin/env bash
echo "AIBOX-OMP-PROMPT"
EOF
chmod +x "$HOME/.local/bin/oh-my-posh"
"#
        ),
    );
    for (tab, bin, marker) in HARNESSES {
        let script = format!(
            r#"#!/usr/bin/env bash
printf '\033[2J\033[H'
echo AIBOX-HARNESS-{marker}
echo "tab={tab} binary={bin}"
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

fn record_generated_layout(
    runner: &E2eRunner,
    test_name: &str,
    layout: &str,
    seconds: u8,
) -> (String, String) {
    let _progress =
        VisualProgressStep::start(format!("record tabs project={test_name} layout={layout}"));
    let workspace = format!("/workspaces/{test_name}");
    let stem = format!("recording-{layout}");
    runner.exec(&format!(
        "sudo rm -rf /workspace; sudo ln -s {workspace} /workspace; rm -rf /tmp/zellij-*"
    ));
    runner.write_file(
        test_name,
        &format!("driver-{layout}.sh"),
        &format!(
            r#"#!/usr/bin/env bash
set -u
export HOME="{workspace}/.aibox-home"
export TERM=xterm-256color
export COLORTERM=truecolor
export PATH="$HOME/.local/bin:/usr/local/bin:$PATH"
(
  sleep {seconds}
  export ZELLIJ_SESSION_NAME=$(zellij list-sessions --no-formatting 2>/dev/null | grep -v EXITED | head -1 | awk '{{print $1}}')
  zellij action write 27 >/dev/null 2>&1 || true
  sleep 0.2
  : > "{workspace}/{stem}.screens"
  capture_tab() {{
    tab="$1"
    marker="${{2:-}}"
    zellij action go-to-tab-name "$tab" >/dev/null 2>&1 || true
    for i in $(seq 1 16); do
      sleep 0.25
      tmp="{workspace}/{stem}.${{tab}}.screen"
      zellij action dump-screen > "$tmp" 2>/dev/null || true
      printf '\n--- tab:%s attempt:%s ---\n' "$tab" "$i" >> "{workspace}/{stem}.screens"
      cat "$tmp" >> "{workspace}/{stem}.screens"
      if [ -z "$marker" ] || grep -qF "$marker" "$tmp"; then
        return 0
      fi
      zellij action go-to-tab-name "$tab" >/dev/null 2>&1 || true
    done
  }}
  capture_tab dev " NOR "
  zellij action move-focus right >/dev/null 2>&1 || true
  for i in $(seq 1 16); do
    sleep 0.25
    tmp="{workspace}/{stem}.dev-editor.screen"
    zellij action dump-screen > "$tmp" 2>/dev/null || true
    printf '\n--- focus:dev-editor attempt:%s ---\n' "$i" >> "{workspace}/{stem}.screens"
    cat "$tmp" >> "{workspace}/{stem}.screens"
    grep -qF "AIBOX-VIM-READY" "$tmp" && break
  done
  zellij action move-focus left >/dev/null 2>&1 || true
  capture_tab files " NOR "
  capture_tab cowork ""
  capture_tab cowork-swap ""
  capture_tab browse ""
  capture_tab ai ""
  capture_tab editor "AIBOX-VIM-READY"
  capture_tab git "AIBOX-LAZYGIT-READY"
  capture_tab shell "AIBOX-SHELL-READY"
  capture_tab claude "AIBOX-HARNESS-CLAUDE"
  capture_tab codex "AIBOX-HARNESS-CODEX"
  capture_tab gemini "AIBOX-HARNESS-GEMINI"
  capture_tab aider "AIBOX-HARNESS-AIDER"
  capture_tab continue "AIBOX-HARNESS-CONTINUE"
  capture_tab cursor "AIBOX-HARNESS-CURSOR"
  capture_tab copilot "AIBOX-HARNESS-COPILOT"
  capture_tab opencode "AIBOX-HARNESS-OPENCODE"
  capture_tab hermes "AIBOX-HARNESS-HERMES"
  zellij action move-focus right >/dev/null 2>&1 || true
  sleep 0.2
  printf '\n--- focus:right ---\n' >> "{workspace}/{stem}.screens"
  zellij action dump-screen >> "{workspace}/{stem}.screens" 2>/dev/null || true
  timeout 2s pkill -x zellij >/dev/null 2>&1 || true
) &
driver_pid=$!
timeout --kill-after=2s 60s zellij --config "$HOME/.config/zellij/config.kdl" \
       --config-dir "$HOME/.config/zellij" \
       --layout "$HOME/.config/zellij/layouts/{layout}.kdl" 2>/dev/null || true
wait "$driver_pid" 2>/dev/null || true
timeout 2s pkill -x zellij >/dev/null 2>&1 || true
true
"#
        ),
    );
    runner.exec(&format!("chmod +x {workspace}/driver-{layout}.sh"));
    runner.exec(&format!(
        "LC_ALL=C.UTF-8 LANG=C.UTF-8 timeout --kill-after=2s 90s asciinema rec --cols 160 --rows 45 --overwrite \
         -c {workspace}/driver-{layout}.sh {workspace}/{stem}.cast 2>/dev/null; true"
    ));
    let logs = runner.exec("cat /tmp/zellij-*/zellij-log/zellij.log 2>/dev/null || true");
    let log_text = String::from_utf8_lossy(&logs.stdout).to_string();
    let cast = runner.read_file(test_name, &format!("{stem}.cast"));
    let screens = runner.read_file(test_name, &format!("{stem}.screens"));
    write_visual_artifacts(
        test_name,
        &stem,
        &cast,
        Some(&screens),
        &log_text,
        json!({
            "kind": "tab-traversal",
            "test": "visual_generated_tools_and_harness_tabs_render_when_enabled",
            "layout": layout,
            "cols": 160,
            "rows": 45,
            "docs_source": true,
        }),
    );
    (format!("{cast}\n{screens}"), log_text)
}

fn record_layout_status(runner: &E2eRunner, test_name: &str, layout: &str) -> (String, String) {
    let _progress =
        VisualProgressStep::start(format!("record status project={test_name} layout={layout}"));
    let workspace = format!("/workspaces/{test_name}");
    let stem = format!("recording-status-{layout}");
    runner.exec(&format!(
        "sudo rm -rf /workspace; sudo ln -s {workspace} /workspace; rm -rf /tmp/zellij-*"
    ));
    runner.write_file(
        test_name,
        &format!("driver-status-{layout}.sh"),
        &format!(
            r#"#!/usr/bin/env bash
set -u
export HOME="{workspace}/.aibox-home"
export TERM=xterm-256color
export COLORTERM=truecolor
export PATH="$HOME/.local/bin:/usr/local/bin:$PATH"
(
  sleep 2
  export ZELLIJ_SESSION_NAME=$(zellij list-sessions --no-formatting 2>/dev/null | grep -v EXITED | head -1 | awk '{{print $1}}')
  zellij action write 27 >/dev/null 2>&1 || true
  sleep 0.2
  zellij action dump-screen > "{workspace}/{stem}.screen" 2>/dev/null || true
  timeout 2s pkill -x zellij >/dev/null 2>&1 || true
) &
driver_pid=$!
timeout --kill-after=2s 15s zellij --config "$HOME/.config/zellij/config.kdl" \
       --config-dir "$HOME/.config/zellij" \
       --layout "$HOME/.config/zellij/layouts/{layout}.kdl" 2>/dev/null || true
wait "$driver_pid" 2>/dev/null || true
timeout 2s pkill -x zellij >/dev/null 2>&1 || true
true
"#
        ),
    );
    runner.exec(&format!("chmod +x {workspace}/driver-status-{layout}.sh"));
    runner.exec(&format!(
        "LC_ALL=C.UTF-8 LANG=C.UTF-8 timeout --kill-after=2s 30s asciinema rec --cols 160 --rows 45 --overwrite \
         -c {workspace}/driver-status-{layout}.sh {workspace}/{stem}.cast 2>/dev/null; true"
    ));
    let logs = runner.exec("cat /tmp/zellij-*/zellij-log/zellij.log 2>/dev/null || true");
    let log_text = String::from_utf8_lossy(&logs.stdout).to_string();
    let cast = runner.read_file(test_name, &format!("{stem}.cast"));
    let screen = runner.read_file(test_name, &format!("{stem}.screen"));
    write_visual_artifacts(
        test_name,
        &stem,
        &cast,
        Some(&screen),
        &log_text,
        json!({
            "kind": "status-theme",
            "test": "visual_generated_layouts_render_across_all_themes",
            "layout": layout,
            "cols": 160,
            "rows": 45,
            "docs_source": true,
        }),
    );
    (format!("{cast}\n{screen}"), log_text)
}

#[test]
#[serial]
#[ignore = "visual e2e matrix is release-gated; run explicitly via scripts/maintain.sh test-e2e-visual-status or test-e2e-visual"]
#[ntest::timeout(720_000)]
fn visual_generated_layouts_render_across_all_themes() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let total_cases = status_matrix_total_cases();
    let mut case = 0;
    for (theme_index, &(theme, r, g, b)) in THEMES.iter().enumerate() {
        let test_name = format!("visual-matrix-theme-{theme}");
        log_visual_progress(format!(
            "status matrix: init theme {}/{} ({theme})",
            theme_index + 1,
            THEMES.len()
        ));
        init_project(&runner, &test_name, theme, false, &["git-ui"]);
        install_visual_fixtures(&runner, &test_name);

        for layout in status_matrix_layouts_for_theme(theme) {
            case += 1;
            log_visual_progress(format!(
                "status matrix [{case}/{total_cases}]: recording theme={theme} layout={layout}"
            ));
            let (recording, logs) = record_layout_status(&runner, &test_name, layout);
            let label = format!("{theme}/{layout}");
            assert_no_zellij_runtime_errors(&logs, &label);
            assert_no_zellij_permission_prompt(&recording, &label);
            assert_generated_theme_config(&runner, &test_name, theme, r, g, b);
            let layout_kdl = generated_layout(&runner, &test_name, layout);
            assert_generated_sidecar_status_layout(layout, &layout_kdl);
            log_visual_progress(format!(
                "status matrix [{case}/{total_cases}]: passed theme={theme} layout={layout}"
            ));
        }

        log_visual_progress(format!(
            "status matrix: cleanup theme {}/{} ({theme})",
            theme_index + 1,
            THEMES.len()
        ));
        runner.cleanup(&test_name);
    }
}

#[test]
#[serial]
#[ignore = "visual tab-traversal e2e is release-gated; run explicitly via scripts/maintain.sh test-e2e-visual-tabs or test-e2e-visual"]
#[ntest::timeout(300_000)]
fn visual_generated_tools_and_harness_tabs_render_when_enabled() {
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
        let layout_kdl = generated_layout(&runner, test_name, layout);
        assert!(
            layout_kdl.contains("command \"yazi\"")
                || layout_kdl.contains("exec yazi")
                || layout_kdl.contains("name=\"files\""),
            "{layout}: expected generated layout to include a files/Yazi surface:\n{layout_kdl}"
        );
        assert!(
            layout_kdl.contains("command \"vim-loop\"") || layout_kdl.contains("exec vim-loop"),
            "{layout}: expected generated layout to include an editor pane using vim-loop:\n{layout_kdl}"
        );
        for (tab, bin, _) in HARNESSES {
            assert!(
                layout_has_top_level_tab(&layout_kdl, tab)
                    || layout_kdl.contains(&format!("command \"{bin}\"")),
                "{layout}: expected generated layout to include harness {tab}/{bin}:\n{layout_kdl}"
            );
        }
    }

    let visual_layouts = tab_matrix_layouts();
    for (index, layout) in visual_layouts.iter().enumerate() {
        let case = index + 1;
        let total = visual_layouts.len();
        log_visual_progress(format!(
            "tabs matrix [{case}/{total}]: recording layout={layout}"
        ));
        let (recording, logs) = record_generated_layout(&runner, test_name, layout, 4);
        assert_no_zellij_runtime_errors(&logs, layout);
        let layout_kdl = generated_layout(&runner, test_name, layout);
        assert!(
            recording.contains("--- tab:files") && recording.contains(" NOR "),
            "{layout}: expected Yazi/file pane surface in generated layout recording:\n{recording}"
        );
        assert!(
            recording.contains("AIBOX-LAZYGIT-READY"),
            "{layout}: expected lazygit tab to render when git-ui:lazygit is enabled"
        );
        assert!(
            recording.contains("AIBOX-SHELL-READY"),
            "{layout}: expected shell tab to render"
        );
        if layout_has_top_level_editor_tab(&layout_kdl) {
            assert!(
                recording.contains("AIBOX-VIM-READY"),
                "{layout}: expected Vim/editor tab to render"
            );
        } else {
            assert!(
                layout_kdl.contains("name=\"editor\"")
                    && (layout_kdl.contains("command \"vim-loop\"")
                        || layout_kdl.contains("exec vim-loop")),
                "{layout}: expected generated layout to include an editor pane using vim-loop:\n{layout_kdl}"
            );
        }
        for (tab, bin, marker) in HARNESSES {
            if layout_has_top_level_tab(&layout_kdl, tab) {
                assert!(
                    recording.contains(&format!("AIBOX-HARNESS-{marker}")),
                    "{layout}: expected enabled harness marker {marker} to render"
                );
            } else {
                assert!(
                    layout_kdl.contains(&format!("command \"{bin}\"")),
                    "{layout}: expected generated layout to include embedded harness {tab}/{bin}:\n{layout_kdl}"
                );
            }
        }
        log_visual_progress(format!(
            "tabs matrix [{case}/{total}]: passed layout={layout}"
        ));
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
        "timeout 2s pkill -x zellij >/dev/null 2>&1 || true; \
         timeout 2s pkill -x yazi >/dev/null 2>&1 || true; \
         timeout 2s pkill -x asciinema >/dev/null 2>&1 || true",
    );
    init_project(
        &runner,
        test_name,
        "tokyo-night",
        false,
        &["git-ui", "data-preview", "preview-enhanced", "yazi-omp"],
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
         test -f {home}/.config/yazi/plugins/omp.yazi/main.lua && \
         git status --porcelain --ignored=matching | grep '^M  modified.txt\\|^ M modified.txt' && \
         git status --porcelain --ignored=matching | grep '^D  deleted.txt\\|^ D deleted.txt' && \
         git status --porcelain --ignored=matching | grep '^A  added.txt' && \
         git status --porcelain --ignored=matching | grep '^UU conflict.txt' && \
         git status --porcelain --ignored=matching | grep '^?? untracked.txt' && \
         git status --porcelain --ignored=matching | grep '^!! ignored.txt'"
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

    let total_cases = entries.len();
    for (index, (label, entry, marker)) in entries.iter().enumerate() {
        let case = index + 1;
        log_visual_progress(format!(
            "yazi matrix [{case}/{total_cases}]: recording preview label={label} entry={entry}"
        ));
        let _progress = VisualProgressStep::start(format!(
            "record yazi preview project={test_name} case={case}/{total_cases} label={label}"
        ));
        runner.exec(&format!(
            "rm -rf /tmp/zellij-*; sudo rm -rf /workspace; sudo ln -s {workspace} /workspace; timeout 2s zellij delete-session aibox-yazi-preview-{label} --force >/dev/null 2>&1 || true"
        ));
        runner.write_file(
            test_name,
            &format!(".aibox-home/.config/zellij/layouts/yazi-preview-{label}.kdl"),
            &format!(
                r#"layout {{
    pane {{
        command "bash"
        args "-lc" "exec yazi {workspace}/{entry}"
        cwd "{workspace}"
    }}
}}
"#
            ),
        );
        runner.write_file(
            test_name,
            &format!("yazi-preview-{label}.sh"),
            &format!(
                r#"#!/usr/bin/env bash
set -u
export HOME="{home}"
export TERM=xterm-256color
export COLORTERM=truecolor
export PATH="$HOME/.local/bin:/usr/local/bin:$PATH"
marker="{marker}"
(
  for attempt in $(seq 1 10); do
    export ZELLIJ_SESSION_NAME=$(zellij list-sessions --no-formatting 2>/dev/null | grep -v EXITED | head -1 | awk '{{print $1}}')
    [ -n "$ZELLIJ_SESSION_NAME" ] && break
    sleep 0.25
  done
  sleep 1
  zellij action write 27 >/dev/null 2>&1 || true
  for attempt in $(seq 1 24); do
    zellij action dump-screen > "{workspace}/yazi-preview-{label}.screen.tmp" 2>/dev/null || true
    if [ -s "{workspace}/yazi-preview-{label}.screen.tmp" ]; then
      mv "{workspace}/yazi-preview-{label}.screen.tmp" "{workspace}/yazi-preview-{label}.screen"
      grep -Fq "$marker" "{workspace}/yazi-preview-{label}.screen" && break
    fi
    sleep 0.5
  done
  if [ ! -s "{workspace}/yazi-preview-{label}.screen" ]; then
    zellij action dump-screen > "{workspace}/yazi-preview-{label}.screen" 2>/dev/null || true
  fi
  timeout 2s pkill -x zellij >/dev/null 2>&1 || true
) &
driver_pid=$!
timeout --kill-after=2s 18s zellij --config "$HOME/.config/zellij/config.kdl" \
       --config-dir "$HOME/.config/zellij" \
       --new-session-with-layout "$HOME/.config/zellij/layouts/yazi-preview-{label}.kdl" \
       --session "aibox-yazi-preview-{label}" 2>/dev/null || true
wait "$driver_pid" 2>/dev/null || true
timeout 2s pkill -x zellij >/dev/null 2>&1 || true
true
"#
            ),
        );
        runner.exec(&format!("chmod +x {workspace}/yazi-preview-{label}.sh"));
        runner.exec(&format!(
            "LC_ALL=C.UTF-8 LANG=C.UTF-8 timeout --kill-after=2s 35s asciinema rec --cols 160 --rows 45 --overwrite \
             -c {workspace}/yazi-preview-{label}.sh {workspace}/yazi-preview-{label}.cast 2>/dev/null; true"
        ));
        let logs = runner.exec("cat /tmp/zellij-*/zellij-log/zellij.log 2>/dev/null || true");
        let log_text = String::from_utf8_lossy(&logs.stdout).to_string();
        assert_no_zellij_runtime_errors(&log_text, label);
        let cast = runner.read_file(test_name, &format!("yazi-preview-{label}.cast"));
        let screen = runner.read_file(test_name, &format!("yazi-preview-{label}.screen"));
        write_visual_artifacts(
            test_name,
            &format!("yazi-preview-{label}"),
            &cast,
            Some(&screen),
            &log_text,
            json!({
                "kind": "yazi-preview",
                "test": "visual_yazi_previews_git_symbols_and_optional_plugins_render",
                "entry": entry,
                "marker": marker,
                "cols": 160,
                "rows": 45,
                "docs_source": true,
            }),
        );
        let recording = format!("{cast}\n{screen}");
        assert!(
            recording.contains(marker),
            "Yazi {label} preview should contain marker {marker:?}:\n{recording}\nZellij logs:\n{log_text}"
        );
        log_visual_progress(format!(
            "yazi matrix [{case}/{total_cases}]: passed preview label={label}"
        ));
    }

    runner.cleanup(test_name);
}
