//! Generated visual matrix tests for layouts, themes, tools, harnesses, and Yazi previews.
//!
//! These run on the SSH companion with asciinema. They intentionally use the
//! generated `.aibox-home` files from a fresh project instead of hand-written
//! test layouts so regressions in the generator show up in Phase 1.

use serde_json::json;
use serial_test::serial;
use std::fs;
use std::path::PathBuf;

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

fn extract_cast_output(cast_content: &str) -> String {
    cast_content
        .lines()
        .skip(1)
        .filter_map(|line| {
            let parsed: serde_json::Value = serde_json::from_str(line).ok()?;
            let arr = parsed.as_array()?;
            if arr.len() >= 3 && arr[1].as_str() == Some("o") {
                arr[2].as_str().map(ToString::to_string)
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

fn contains_rgb(output: &str, r: u8, g: u8, b: u8) -> bool {
    let fg = format!("38;2;{r};{g};{b}");
    let bg = format!("48;2;{r};{g};{b}");
    output.contains(&fg) || output.contains(&bg)
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
        "native",
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
        "LC_ALL=C.UTF-8 LANG=C.UTF-8 asciinema rec --cols 160 --rows 45 --overwrite \
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
        "LC_ALL=C.UTF-8 LANG=C.UTF-8 asciinema rec --cols 160 --rows 45 --overwrite \
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
#[ntest::timeout(420_000)]
fn visual_generated_layouts_render_across_all_themes() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    for &(theme, r, g, b) in THEMES {
        let test_name = format!("visual-matrix-theme-{theme}");
        init_project(&runner, &test_name, theme, false, &["git-ui"]);
        install_visual_fixtures(&runner, &test_name);

        for layout in LAYOUTS {
            let (recording, logs) = record_layout_status(&runner, &test_name, layout);
            assert_no_zellij_runtime_errors(&logs, &format!("{theme}/{layout}"));
            let output = extract_cast_output(&recording);
            assert!(
                contains_rgb(&output, r, g, b),
                "{theme}/{layout}: expected theme RGB({r},{g},{b}) in generated layout recording"
            );
            assert!(
                recording.contains("LEADER") || recording.contains("PANES"),
                "{theme}/{layout}: native aibox key rows should be visible"
            );
        }

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
        let (recording, logs) = record_generated_layout(&runner, test_name, layout, 4);
        assert_no_zellij_runtime_errors(&logs, layout);
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
        assert!(
            recording.contains("AIBOX-VIM-READY"),
            "{layout}: expected Vim/editor surface to render"
        );
        for (_, _, marker) in HARNESSES {
            assert!(
                recording.contains(&format!("AIBOX-HARNESS-{marker}")),
                "{layout}: expected enabled harness marker {marker} to render"
            );
        }
    }

    runner.cleanup(test_name);
}

#[test]
#[serial]
#[ignore = "visual Yazi preview e2e is release-gated; run explicitly via scripts/maintain.sh test-e2e-visual-yazi or test-e2e-visual"]
#[ntest::timeout(180_000)]
fn visual_yazi_previews_git_symbols_and_optional_plugins_render() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let test_name = "visual-matrix-yazi-previews";
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

    for (label, entry, marker) in entries {
        runner.exec(&format!(
            "rm -rf /tmp/zellij-*; sudo rm -rf /workspace; sudo ln -s {workspace} /workspace"
        ));
        runner.write_file(
            test_name,
            &format!("yazi-preview-{label}.kdl"),
            &format!(
                r#"layout {{
    pane {{
        command "yazi"
        args "{workspace}/{entry}"
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
(
  sleep 3
  export ZELLIJ_SESSION_NAME=$(zellij list-sessions --no-formatting 2>/dev/null | grep -v EXITED | head -1 | awk '{{print $1}}')
  zellij action write 27 >/dev/null 2>&1 || true
  sleep 0.4
  zellij action dump-screen > "{workspace}/yazi-preview-{label}.screen" 2>/dev/null || true
  timeout 2s pkill -x zellij >/dev/null 2>&1 || true
) &
driver_pid=$!
timeout --kill-after=2s 15s zellij --config "$HOME/.config/zellij/config.kdl" \
       --config-dir "$HOME/.config/zellij" \
       --layout "{workspace}/yazi-preview-{label}.kdl" 2>/dev/null || true
wait "$driver_pid" 2>/dev/null || true
timeout 2s pkill -x zellij >/dev/null 2>&1 || true
true
"#
            ),
        );
        runner.exec(&format!("chmod +x {workspace}/yazi-preview-{label}.sh"));
        runner.exec(&format!(
            "LC_ALL=C.UTF-8 LANG=C.UTF-8 asciinema rec --cols 160 --rows 45 --overwrite \
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
            "Yazi {label} preview should contain marker {marker:?}:\n{recording}"
        );
    }

    runner.cleanup(test_name);
}
