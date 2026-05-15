//! Visual tests for the tmux runtime.
//!
//! These tests keep the existing sidecar/asciinema visual paradigm, but drive
//! tmux directly. They assert that generated homes contain tmux configuration,
//! render visible panes/status text, and do not carry legacy zellij artifacts.

use serial_test::serial;

use super::runner::E2eRunner;

// Family names (clap --theme value enum). Each resolves to its canonical
// dark variant under default mode=auto with no host detection. The light
// variant of catppuccin (latte) is unit-tested in themes::tests but cannot
// be asserted here because `aibox init` has no --mode flag yet.
const THEME_SIGNATURES: &[(&str, u8, u8, u8)] = &[
    ("gruvbox", 152, 151, 26),
    ("catppuccin", 166, 227, 161),
    ("dracula", 80, 250, 123),
    ("tokyo-night", 158, 206, 106),
    ("nord", 163, 190, 140),
];

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

fn contains_rgb(output: &str, r: u8, g: u8, b: u8) -> bool {
    output.contains(&format!("38;2;{r};{g};{b}")) || output.contains(&format!("48;2;{r};{g};{b}"))
}

fn nearest_xterm_level(value: u8) -> u8 {
    const LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];
    LEVELS
        .iter()
        .enumerate()
        .min_by_key(|(_, level)| value.abs_diff(**level))
        .map(|(idx, _)| idx as u8)
        .unwrap_or(0)
}

fn xterm_256_index(r: u8, g: u8, b: u8) -> u8 {
    16 + (36 * nearest_xterm_level(r)) + (6 * nearest_xterm_level(g)) + nearest_xterm_level(b)
}

fn contains_signature_color(output: &str, r: u8, g: u8, b: u8) -> bool {
    if contains_rgb(output, r, g, b) {
        return true;
    }
    let idx = xterm_256_index(r, g, b);
    output.contains(&format!("38;5;{idx}")) || output.contains(&format!("48;5;{idx}"))
}

fn rgb_hex(r: u8, g: u8, b: u8) -> String {
    format!("#{r:02X}{g:02X}{b:02X}")
}

fn visible_text(output: &str) -> String {
    let mut text = String::with_capacity(output.len());
    let mut chars = output.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            match chars.peek().copied() {
                Some(']') => {
                    chars.next();
                    while let Some(next) = chars.next() {
                        if next == '\x07' {
                            break;
                        }
                        if next == '\x1b' && chars.peek() == Some(&'\\') {
                            chars.next();
                            break;
                        }
                    }
                }
                Some('[') => {
                    chars.next();
                    for next in chars.by_ref() {
                        if ('@'..='~').contains(&next) {
                            break;
                        }
                    }
                }
                Some('(') | Some(')') => {
                    chars.next();
                    chars.next();
                }
                _ => {}
            }
        } else if ch.is_control() {
            text.push(' ');
        } else {
            text.push(ch);
        }
    }
    text
}

fn init_project(runner: &E2eRunner, test_name: &str, theme: &str) {
    runner.cleanup(test_name);
    let output = runner.aibox(
        test_name,
        &[
            "init",
            test_name,
            "--base",
            "debian",
            "--context",
            "managed",
            "--theme",
            theme,
            "--no-container",
        ],
    );
    assert!(
        output.status.success(),
        "{test_name}: init failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_generated_tmux_config(runner: &E2eRunner, test_name: &str, theme: &str) {
    let workspace = format!("/workspaces/{test_name}");
    let theme_pattern = theme.replace('-', "[- ]");
    let probe = runner.exec(&format!(
        r#"cd {workspace}
test -f .aibox-home/.tmux.conf -o -f .aibox-home/.config/tmux/tmux.conf
! find .aibox-home -path '*zellij*' -print -quit | grep -q .
! grep -Rli --exclude-dir=.git 'zellij' .aibox-home .devcontainer aibox.toml >/tmp/{test_name}-legacy-zellij.txt 2>/dev/null
grep -Rli --exclude-dir=.git --exclude=claude 'tmux' .aibox-home .devcontainer aibox.toml >/tmp/{test_name}-tmux-files.txt
grep -REi --exclude-dir=.git --exclude=claude '{theme_pattern}' .aibox-home >/tmp/{test_name}-theme.txt
grep -F '@powerkit_line1_right "aibox_log,aibox_oom,aibox_proc,aibox_ai,aibox_mcp,aibox_mig,weather,uptime,datetime"' .aibox-home/.config/tmux/tmux.conf
grep -F '@powerkit_line2_left "git,github,kubernetes,terraform,cloud"' .aibox-home/.config/tmux/tmux.conf
grep -F '@powerkit_line2_right "hostname,externalip,ssh,netspeed,ping,cpu,loadavg,memory,swap,disk,gpu"' .aibox-home/.config/tmux/tmux.conf
grep -F '@powerkit_plugin_netspeed_speed_width "7"' .aibox-home/.config/tmux/tmux.conf
grep -F 'status-format[0]' .aibox-home/.config/tmux/tmux.conf | grep -F 'align=right'
grep -F 'status-format[1]' .aibox-home/.config/tmux/tmux.conf | grep -F 'align=left' | grep -F 'align=right'
"#
    ));
    assert!(
        probe.status.success(),
        "{test_name}: generated runtime should be tmux-only and reference theme {theme}\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&probe.stdout),
        String::from_utf8_lossy(&probe.stderr)
    );
}

fn assert_theme_signature_config(
    runner: &E2eRunner,
    test_name: &str,
    theme: &str,
    r: u8,
    g: u8,
    b: u8,
) {
    let workspace = format!("/workspaces/{test_name}");
    let expected = rgb_hex(r, g, b);
    let probe = runner.exec(&format!(
        "cd {workspace} && grep -Ri --exclude-dir=.git --exclude=claude '{expected}' .aibox-home >/tmp/{test_name}-theme-rgb.txt"
    ));
    assert!(
        probe.status.success(),
        "{theme}: generated tmux theme/config should contain signature color {expected}"
    );
}

fn record_tmux(runner: &E2eRunner, test_name: &str, script: &str) -> String {
    let workspace = format!("/workspaces/{test_name}");
    runner.exec(
        "tmux kill-server >/dev/null 2>&1 || true; rm -rf /tmp/tmux-* >/dev/null 2>&1 || true",
    );
    runner.write_file(test_name, "driver.sh", script);
    runner.exec(&format!("chmod +x {workspace}/driver.sh"));
    let output = runner.exec(&format!(
        "LC_ALL=C.UTF-8 LANG=C.UTF-8 asciinema rec --cols 160 --rows 45 --overwrite \
         -c {workspace}/driver.sh {workspace}/recording.cast 2>/dev/null; true"
    ));
    assert!(
        output.status.success(),
        "{test_name}: asciinema command failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    runner.read_file(test_name, "recording.cast")
}

fn tmux_driver(workspace: &str, session: &str, body: &str) -> String {
    format!(
        r#"#!/usr/bin/env bash
set -u
export HOME="{workspace}/.aibox-home"
export TERM=xterm-256color
export COLORTERM=truecolor
export PATH="$HOME/.local/bin:/usr/local/bin:$PATH"
tmux_conf="$HOME/.tmux.conf"
[ -f "$tmux_conf" ] || tmux_conf="$HOME/.config/tmux/tmux.conf"
if [ ! -f "$tmux_conf" ]; then
  echo "missing generated tmux config" >&2
  exit 90
fi
tmux kill-session -t "{session}" >/dev/null 2>&1 || true
(
  for _ in $(seq 1 40); do
    tmux has-session -t "{session}" >/dev/null 2>&1 && break
    sleep 0.1
  done
{body}
  sleep 0.5
  tmux capture-pane -e -p -t "{session}:1.1" > "{workspace}/final-pane.txt" 2>/dev/null || true
  tmux display-message -p -t "{session}" '#S #W #{{window_panes}} #{{status-left}} #{{status-right}}' > "{workspace}/tmux-status.txt" 2>/dev/null || true
  tmux kill-session -t "{session}" >/dev/null 2>&1 || true
) &
driver_pid=$!
tmux -f "$tmux_conf" new-session -A -s "{session}" -n dev -c "{workspace}" \
  "printf 'AIBOX-TMUX-SHELL\n'; exec bash"
wait "$driver_pid" 2>/dev/null || true
true
"#
    )
}

#[test]
#[serial]
#[ntest::timeout(180_000)]
fn visual_themes_produce_tmux_signature_colors() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    for &(theme, r, g, b) in THEME_SIGNATURES {
        let test_name = format!("visual-theme-{theme}");
        init_project(&runner, &test_name, theme);
        assert_generated_tmux_config(&runner, &test_name, theme);
        assert_theme_signature_config(&runner, &test_name, theme, r, g, b);

        let workspace = format!("/workspaces/{test_name}");
        let hex = rgb_hex(r, g, b);
        let body = format!(
            r##"  tmux set-option -t "{test_name}" -g status on
  tmux set-option -t "{test_name}" -ga terminal-overrides ",*:Tc"
  tmux set-option -t "{test_name}" -g status-left-length 100
  tmux set-option -t "{test_name}" -g status-right-length 100
  tmux set-option -t "{test_name}" -g status-format[0] "#[fg={hex},bold] AIBOX-TMUX-THEME {theme} #[default]"
  tmux set-option -t "{test_name}" -g status-format[1] ""
  sleep 2
"##
        );
        let cast = record_tmux(
            &runner,
            &test_name,
            &tmux_driver(&workspace, &test_name, &body),
        );
        assert!(cast.lines().count() > 5, "{theme}: cast too small");
        let output = extract_cast_output(&cast);
        let text = visible_text(&output);
        assert!(
            text.contains("AIBOX-TMUX-SHELL"),
            "{theme}: expected tmux shell marker in visible recording:\n{}",
            text.chars().take(1600).collect::<String>()
        );
        assert!(
            contains_signature_color(&output, r, g, b),
            "{theme}: expected tmux status to render RGB({r},{g},{b}) or its xterm-256 fallback"
        );
        runner.cleanup(&test_name);
    }
}

#[test]
#[serial]
#[ntest::timeout(90_000)]
fn visual_tmux_status_and_panes_render_without_legacy_artifacts() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let test_name = "visual-tmux-status";
    init_project(&runner, test_name, "projectious");
    assert_generated_tmux_config(&runner, test_name, "projectious");

    let workspace = format!("/workspaces/{test_name}");
    let body = format!(
        r#"  tmux set-option -t "{test_name}" -g status on
  tmux set-option -t "{test_name}" -g status-left-length 80
  tmux set-option -t "{test_name}" -g status-right-length 100
  aibox-status --once > "{workspace}/status-once.txt" 2>/dev/null || true
  tmux split-window -h -t "{test_name}:1" -c "{workspace}" "printf 'AIBOX-TMUX-RIGHT-PANE\n'; exec bash"
  tmux split-window -v -t "{test_name}:1.1" -c "{workspace}" "printf 'AIBOX-TMUX-LOWER-PANE\n'; exec bash"
  tmux select-pane -t "{test_name}:1.1"
  sleep 3
"#
    );
    let cast = record_tmux(
        &runner,
        test_name,
        &tmux_driver(&workspace, test_name, &body),
    );
    let output = extract_cast_output(&cast);
    let text = visible_text(&output);
    assert!(
        text.contains("AIBOX-TMUX")
            && text.contains("AIBOX-TMUX-RIGHT-PANE")
            && text.contains("AIBOX-TMUX-LOWER-PANE"),
        "expected tmux status and panes to render:\n{}",
        text.chars().take(2400).collect::<String>()
    );
    let status_once = runner.read_file(test_name, "status-once.txt");
    assert!(
        status_once.contains("MEM ")
            && status_once.contains("PROC ")
            && status_once.contains("MCP "),
        "expected aibox-status output to contain current status metrics:\n{status_once}"
    );
    assert!(
        !text.to_ascii_lowercase().contains("zellij"),
        "tmux recording should not mention legacy multiplexer artifacts:\n{text}"
    );
    runner.cleanup(test_name);
}

#[test]
#[serial]
#[ntest::timeout(90_000)]
fn visual_yazi_renders_in_tmux_pane() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let test_name = "visual-yazi-tmux";
    init_project(&runner, test_name, "tokyo-night");
    runner.write_file(test_name, "project-files/README.md", "# Test\n");
    runner.write_file(test_name, "project-files/main.rs", "fn main() {}\n");

    let workspace = format!("/workspaces/{test_name}");
    let body = format!(
        r#"  tmux rename-window -t "{test_name}:1" files
  tmux send-keys -t "{test_name}:1.1" "cd {workspace}/project-files && exec yazi ." C-m
  for _ in $(seq 1 30); do
    tmux capture-pane -p -t "{test_name}:1.1" > "{workspace}/yazi-screen.txt" 2>/dev/null || true
    grep -Eq 'README|main.rs' "{workspace}/yazi-screen.txt" && break
    sleep 0.25
  done
"#
    );
    let cast = record_tmux(
        &runner,
        test_name,
        &tmux_driver(&workspace, test_name, &body),
    );
    let output = extract_cast_output(&cast);
    let screen = runner.read_file(test_name, "yazi-screen.txt");
    assert!(cast.lines().count() > 5, "cast too small");
    assert!(
        output.contains("README")
            || output.contains("main.rs")
            || screen.contains("README")
            || screen.contains("main.rs"),
        "expected Yazi to render file names in tmux pane:\n{screen}"
    );
    runner.cleanup(test_name);
}
