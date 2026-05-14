//! Rendered-color tests for the tmux status bar / panes — Tier 3 / companion.
//!
//! Starts a real tmux session inside the e2e companion, captures the rendered
//! panes via `tmux capture-pane -p -e`, then asserts per-cell `bgcolor`/`fgcolor`
//! against the active theme's palette using the vt100 helpers in
//! `crate::vt_render`.

#![cfg(all(feature = "e2e", feature = "e2e-render"))]

use serial_test::serial;

use crate::runner::E2eRunner;
use crate::vt_render;

/// Per-theme palette table hardcoded here (aibox is a bin crate, not a lib, so
/// `aibox::themes` is not importable from tests).
///
/// Fields: (theme_name, surface_hex, bg_hex, accent_hex)
///
/// `surface` is the tmux status-bar background color (slightly lighter/darker
/// than the base bg). `bg` is the overall terminal background. `accent` is the
/// highlight / session-name color used as fg.
struct ThemePalette {
    name: &'static str,
    surface: &'static str,
    bg: &'static str,
    accent: &'static str,
}

const THEMES: &[ThemePalette] = &[
    ThemePalette {
        name: "catppuccin-mocha",
        surface: "#313244",
        bg: "#1E1E2E",
        accent: "#89B4FA",
    },
    ThemePalette {
        name: "projectious",
        surface: "#131E2B",
        bg: "#0E1720",
        accent: "#E05232",
    },
    ThemePalette {
        name: "gruvbox-light",
        surface: "#EBDBB2",
        bg: "#FBF1C7",
        accent: "#D65D0E",
    },
];

// ─── helpers ─────────────────────────────────────────────────────────────────

/// Initialise a project on the companion with `--no-container` so we get the
/// seeded `.aibox-home/` without spinning up a container runtime.
fn init_project(runner: &E2eRunner, test_name: &str, theme: &str) {
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
            "--theme",
            theme,
            "--processkit-version",
            "unset",
            "--no-container",
        ],
    );
    assert!(
        init.status.success(),
        "[{test_name}] init failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&init.stdout),
        String::from_utf8_lossy(&init.stderr)
    );
    let apply = runner.aibox(test_name, &["apply", "--no-container"]);
    assert!(
        apply.status.success(),
        "[{test_name}] apply failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&apply.stdout),
        String::from_utf8_lossy(&apply.stderr)
    );
}

/// Build and write the driver shell script, start a tmux session via the
/// generated `dev.sh` layout, wait for the status bar to render, and capture
/// the pane ANSI output.
///
/// Returns the raw ANSI capture string.
fn start_tmux_and_capture(runner: &E2eRunner, test_name: &str) -> String {
    let workspace = format!("/workspaces/{test_name}");
    let session = format!("rendered-{test_name}");

    // Minimal driver: export env, source the generated dev layout, sleep for
    // the status bar, then use the vt_render helper (capture-pane base64 path)
    // to grab the pane. We write a tiny wrapper that sets HOME and delegates to
    // the layout script.
    let driver = format!(
        r#"#!/usr/bin/env bash
set -eu
export HOME="{workspace}/.aibox-home"
export TERM=xterm-256color
export COLORTERM=truecolor
export PATH="$HOME/.local/bin:/usr/local/bin:$PATH"
tmux_socket="${{AIBOX_TMUX_SOCKET:-$HOME/.tmux/aibox.sock}}"
mkdir -p "$(dirname "$tmux_socket")"
export AIBOX_TMUX_SOCKET="$tmux_socket"
tmux_alias() {{
  command tmux -S "$tmux_socket" "$@"
}}
tmux_conf="$HOME/.tmux.conf"
[ -f "$tmux_conf" ] || tmux_conf="$HOME/.config/tmux/tmux.conf"
if [ ! -f "$tmux_conf" ]; then
  echo "missing generated tmux config" >&2
  exit 90
fi
ln -sf "$tmux_conf" "$HOME/.tmux.conf"
layout_script="$HOME/.config/tmux/layouts/dev.sh"
if [ ! -x "$layout_script" ]; then
  echo "missing executable generated tmux layout: $layout_script" >&2
  exit 91
fi
# Kill any lingering session from a previous run.
tmux_alias kill-session -t "{session}" >/dev/null 2>&1 || true
# Launch the dev layout in a subshell; the layout script normally blocks.
AIBOX_TMUX_SESSION="{session}" \
AIBOX_WORKSPACE="{workspace}" \
AIBOX_TMUX_CONFIG="$tmux_conf" \
AIBOX_TMUX_SOCKET="$tmux_socket" \
  "$layout_script" &
layout_pid=$!
# Wait up to 10 s for the session to appear.
for _ in $(seq 1 100); do
  tmux_alias has-session -t "{session}" >/dev/null 2>&1 && break
  sleep 0.1
done
if ! tmux_alias has-session -t "{session}" >/dev/null 2>&1; then
  echo "tmux session {session} never started" >&2
  exit 92
fi
# Give the status bar 2 s to fully render.
sleep 2
# Capture with -e (SGR sequences) and -p (stdout). Encode via base64 so
# escape sequences survive the SSH transport without being mangled.
tmux_alias capture-pane -p -e -t "{session}:0" | base64 -w0 > "{workspace}/session.ansi-capture.b64"
# Tear down.
tmux_alias kill-session -t "{session}" >/dev/null 2>&1 || true
kill "$layout_pid" >/dev/null 2>&1 || true
wait "$layout_pid" 2>/dev/null || true
"#
    );

    runner.write_file(test_name, "driver-rendered-tmux.sh", &driver);
    let chmod = runner.exec(&format!("chmod +x {workspace}/driver-rendered-tmux.sh"));
    assert!(
        chmod.status.success(),
        "[{test_name}] chmod on driver script failed"
    );

    // Execute the driver.  Use a timeout to avoid blocking if something hangs.
    let run = runner.exec(&format!(
        "LC_ALL=C.UTF-8 LANG=C.UTF-8 timeout --kill-after=5s 30s \
         bash {workspace}/driver-rendered-tmux.sh 2>&1; true"
    ));
    // We don't assert success here — the capture step may emit a non-zero exit
    // if tmux starts asynchronously; what matters is the capture file.
    eprintln!(
        "[rendered-tmux/{test_name}] driver stdout+stderr:\n{}",
        String::from_utf8_lossy(&run.stdout)
    );

    // Read the base64-encoded capture.
    let b64 = runner.read_file(test_name, "session.ansi-capture.b64");
    assert!(
        !b64.trim().is_empty(),
        "[{test_name}] ANSI capture file is empty — tmux session may not have started.\n\
         Driver output:\n{}",
        String::from_utf8_lossy(&run.stdout)
    );

    // Decode manually via the same base64 helper that vt_render uses internally.
    // We re-use capture_pane_ansi in the real tests, but here we use the file
    // approach because the driver already ran and wrote the file.  We decode
    // in-process to avoid another SSH round-trip.
    decode_base64_capture(&b64)
}

/// Decode a base64 string the same way vt_render's internal helper does.
fn decode_base64_capture(b64: &str) -> String {
    const ALPHA: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut lut = [255u8; 256];
    for (i, &c) in ALPHA.iter().enumerate() {
        lut[c as usize] = i as u8;
    }
    let bytes: Vec<u8> = b64
        .bytes()
        .filter(|b| *b != b'\n' && *b != b'\r' && *b != b' ')
        .collect();
    let mut out = Vec::with_capacity(bytes.len() / 4 * 3 + 3);
    for chunk in bytes.chunks(4) {
        let pad = chunk.iter().filter(|&&b| b == b'=').count();
        let mut buf = [0u32; 4];
        for (i, &b) in chunk.iter().enumerate() {
            buf[i] = if b == b'=' { 0 } else { lut[b as usize] as u32 };
        }
        let n = (buf[0] << 18) | (buf[1] << 12) | (buf[2] << 6) | buf[3];
        out.push((n >> 16) as u8);
        if pad < 2 {
            out.push((n >> 8) as u8);
        }
        if pad < 1 {
            out.push(n as u8);
        }
    }
    String::from_utf8_lossy(&out).to_string()
}

/// Core assertion suite run for every theme.
///
/// Parameters
/// ----------
/// * `capture`      — raw ANSI string from `tmux capture-pane -p -e`
/// * `palette`      — the expected palette for this theme
/// * `not_bg_hex`   — a bg color from a *different* theme; asserted absent
fn assert_rendered_theme_palette(capture: &str, palette: &ThemePalette, not_bg_hex: &str) {
    let parser = vt_render::parse_tmux_capture(capture);
    let screen = parser.screen();
    let (rows, cols) = screen.size();

    // ── 1. Status-bar surface bg covers ≥ 20 cells ───────────────────────
    let surface_count = vt_render::count_cells_with_bg(screen, palette.surface);
    assert!(
        surface_count >= 20,
        "[{}] FAIL: status-bar surface color {} painted only {} cells (need ≥ 20). \
         The tmux status bar is not themed — bg may have fallen back to terminal default.\n\
         screen size: {rows}×{cols}",
        palette.name,
        palette.surface,
        surface_count
    );

    // ── 2. Accent appears as a fg on at least one cell ────────────────────
    assert!(
        vt_render::any_cell_has_fg(screen, palette.accent),
        "[{}] FAIL: accent color {} never appeared as a cell fg. \
         Session prefix / status indicators are not themed.\n\
         screen size: {rows}×{cols}",
        palette.name,
        palette.accent
    );

    // ── 3. Both status-bar rows are themed ─────────────────────────────────
    //       Locates the actual rendered region (the parser is 80x200 but the
    //       tmux pane is only ~24 rows tall, so the "bottom 2 rows of the
    //       parser grid" sits on empty space below the capture). Then for
    //       each of the bottom-2 *rendered* rows it asserts:
    //         a) no cell has bg=#000000   (historical "tmux bg black")
    //         b) <30% of cells have Color::Default bg
    //                                     (regression where line 1 of the
    //                                      two-line status bar rendered empty
    //                                      while line 2 painted fine)
    //         c) ≥60% of cells have bg == surface
    //                                     (positive: the row carries the
    //                                      themed surface bg)
    let (status_start, status_end) = vt_render::bottom_content_rows(screen, 2);
    let black_cells: Vec<(u16, u16)> = (status_start..status_end)
        .flat_map(|r| {
            (0..cols).filter_map(move |c| {
                screen.cell(r, c).and_then(|cell| {
                    if vt_render::color_eq_hex(cell.bgcolor(), "#000000") {
                        Some((r, c))
                    } else {
                        None
                    }
                })
            })
        })
        .collect();

    let row_dump = |start: u16, end: u16| -> String {
        (start..end)
            .map(|r| {
                let line: String = (0..cols)
                    .filter_map(|c| screen.cell(r, c))
                    .map(|cell| {
                        let ch = cell.contents();
                        if ch.is_empty() {
                            ' '
                        } else {
                            ch.chars().next().unwrap_or(' ')
                        }
                    })
                    .collect();
                let default_bgs = vt_render::count_default_bg_in_row(screen, r);
                let surface_bgs =
                    vt_render::count_bg_in_row(screen, r, palette.surface);
                format!(
                    "  row {r}: |{line}|  surface={surface_bgs}/{cols}  default={default_bgs}/{cols}"
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    if !black_cells.is_empty() {
        panic!(
            "[{}] REGRESSION (bg-black): {} cell(s) in status-bar rows {status_start}..{status_end} \
             have bg = #000000 (terminal black). The previous undetected regression: tmux is \
             painting the status bar black instead of using the theme's surface color ({}).\n\
             Black cells at: {black_cells:?}\nStatus rows:\n{}",
            palette.name,
            black_cells.len(),
            palette.surface,
            row_dump(status_start, status_end),
        );
    }

    // Per-row checks for `default-bg` (cells the renderer never touched).
    // A two-line status bar where line 1 silently didn't render leaves an
    // entire row of `Color::Default` bg cells above the painted line 2.
    let unstyled_threshold = (cols as usize * 30) / 100;
    let surface_threshold = (cols as usize * 60) / 100;
    for r in status_start..status_end {
        let default_bgs = vt_render::count_default_bg_in_row(screen, r);
        let surface_bgs = vt_render::count_bg_in_row(screen, r, palette.surface);
        assert!(
            default_bgs < unstyled_threshold,
            "[{}] REGRESSION (status-row-not-rendered): status-bar row {r} has {default_bgs}/{cols} \
             unstyled (Color::Default) bg cells — the renderer never painted this row. The previous \
             undetected regression: only the second of two status lines rendered, leaving the first \
             empty.\nStatus rows:\n{}",
            palette.name,
            row_dump(status_start, status_end),
        );
        assert!(
            surface_bgs >= surface_threshold,
            "[{}] REGRESSION (status-row-not-themed): status-bar row {r} has only {surface_bgs}/{cols} \
             cells with surface bg ({}); need ≥{surface_threshold} (60% of {cols}). The status line \
             may be partially rendered or themed with the wrong palette.\nStatus rows:\n{}",
            palette.name,
            palette.surface,
            row_dump(status_start, status_end),
        );
    }

    // ── 4. Cross-theme negative: no cell has bg from a *different* theme ──
    assert!(
        !vt_render::any_cell_has_bg(screen, not_bg_hex),
        "[{}] FAIL: found bg = {not_bg_hex} which belongs to a different theme. \
         The active theme silently fell back to a wrong palette (default palette bleed).\n\
         screen size: {rows}×{cols}",
        palette.name
    );
}

// ─── tests ───────────────────────────────────────────────────────────────────

#[test]
#[serial]
#[ignore = "rendered tmux e2e is companion-gated; run via cargo test --test e2e --features 'e2e e2e-render'"]
fn rendered_tmux_status_bar_paints_theme_palette_catppuccin_mocha() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let palette = &THEMES[0]; // catppuccin-mocha
    let test_name = "rendered-tmux-catppuccin-mocha";

    init_project(&runner, test_name, palette.name);
    let capture = start_tmux_and_capture(&runner, test_name);

    // Cross-theme negative: must not bleed Projectious bg.
    let not_bg = THEMES[1].bg; // #0E1720

    assert_rendered_theme_palette(&capture, palette, not_bg);

    runner.cleanup(test_name);
}

#[test]
#[serial]
#[ignore = "rendered tmux e2e is companion-gated; run via cargo test --test e2e --features 'e2e e2e-render'"]
fn rendered_tmux_status_bar_paints_theme_palette_projectious() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let palette = &THEMES[1]; // projectious
    let test_name = "rendered-tmux-projectious";

    init_project(&runner, test_name, palette.name);
    let capture = start_tmux_and_capture(&runner, test_name);

    // Cross-theme negative: must not bleed CatppuccinMocha bg.
    let not_bg = THEMES[0].bg; // #1E1E2E

    assert_rendered_theme_palette(&capture, palette, not_bg);

    runner.cleanup(test_name);
}

/// Append a `[customization.tmux.status.labels]` block to `aibox.toml` on
/// the companion so each label maps to a *unique, ASCII* test marker. We use
/// ASCII because Nerd Font glyphs may render as boxes in a headless terminal
/// — markers must survive the parser.
fn inject_test_labels(runner: &E2eRunner, test_name: &str) {
    let workspace = format!("/workspaces/{test_name}");
    let snippet = r#"

[customization.tmux.status.labels]
aibox_log  = "T1LOG"
aibox_oom  = "T1OOM"
aibox_proc = "T1PROC"
aibox_ai   = "T1AI"
aibox_mcp  = "T1MCP"
aibox_mig  = "T1MIG"
uptime     = "T1UP"
kubernetes = "T2K8S"
cloud      = "T2CLD"
netspeed   = "T2NET"
"#;
    // Append the section to the file. The default init does not include a
    // `labels` subsection, so a plain `>>` is sufficient.
    runner.write_file(test_name, ".aibox-toml-labels-snippet", snippet);
    let cmd = format!("cat {workspace}/.aibox-toml-labels-snippet >> {workspace}/aibox.toml");
    let out = runner.exec(&cmd);
    assert!(
        out.status.success(),
        "[{test_name}] failed to inject test labels: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Verify both status lines render and each carries the elements that the
/// active `aibox.toml` assigned to it. This is the regression guard for the
/// historical bug where line 1 silently never rendered while line 2 painted
/// fine and the unit tests passed.
#[test]
#[serial]
#[ignore = "rendered tmux e2e is companion-gated; run via cargo test --test e2e --features 'e2e e2e-render'"]
fn rendered_tmux_status_lines_match_aibox_toml_layout() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let palette = &THEMES[0]; // catppuccin-mocha — palette assertions piggyback on the existing test trio
    let test_name = "rendered-tmux-toml-layout";

    init_project(&runner, test_name, palette.name);
    inject_test_labels(&runner, test_name);
    // Re-apply so the new labels flow through into tmux.conf.
    let apply = runner.aibox(test_name, &["apply", "--no-container"]);
    assert!(
        apply.status.success(),
        "[{test_name}] second apply failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&apply.stdout),
        String::from_utf8_lossy(&apply.stderr)
    );

    let capture = start_tmux_and_capture(&runner, test_name);
    let parser = vt_render::parse_tmux_capture(&capture);
    let screen = parser.screen();
    let (status_start, status_end) = vt_render::bottom_content_rows(screen, 2);
    let line1 = vt_render::row_text(screen, status_start);
    let line2 = vt_render::row_text(screen, status_end - 1);

    eprintln!("[rendered-tmux-toml-layout] status line 1 (row {status_start}): {line1:?}");
    eprintln!(
        "[rendered-tmux-toml-layout] status line 2 (row {}): {line2:?}",
        status_end - 1
    );

    // Markers configured on line 1 (line1_right defaults: aibox_log, _oom,
    // _proc, _ai, _mcp, _mig, weather, uptime, datetime). The uptime label
    // is a stable witness because it's always present and we set it to
    // "T1UP" above.
    let line1_markers = ["T1LOG", "T1OOM", "T1PROC", "T1AI", "T1MCP", "T1MIG", "T1UP"];
    for marker in line1_markers {
        assert!(
            line1.contains(marker),
            "[catppuccin-mocha] REGRESSION: status line 1 missing marker {marker:?}. \
             The first status line is configured to carry aibox_*/uptime segments per \
             aibox.toml's [customization.tmux.status.layout]; not seeing them means line 1 \
             did not render with its configured elements.\nline 1: {line1:?}\nline 2: {line2:?}"
        );
    }

    // Markers configured on line 2 (line2_left: kubernetes, cloud; line2_right: netspeed).
    let line2_markers = ["T2K8S", "T2CLD", "T2NET"];
    for marker in line2_markers {
        assert!(
            line2.contains(marker),
            "[catppuccin-mocha] REGRESSION: status line 2 missing marker {marker:?}. \
             Line 2 is configured to carry kubernetes/cloud/netspeed per aibox.toml.\n\
             line 1: {line1:?}\nline 2: {line2:?}"
        );
    }

    // Cross-line negatives: a line 1 marker must NOT show up on line 2, and
    // vice versa. Catches "both lines render the same plugin list" regressions.
    for marker in line1_markers {
        assert!(
            !line2.contains(marker),
            "[catppuccin-mocha] REGRESSION: line 2 unexpectedly contains line-1 marker {marker:?}.\n\
             line 1: {line1:?}\nline 2: {line2:?}"
        );
    }
    for marker in line2_markers {
        assert!(
            !line1.contains(marker),
            "[catppuccin-mocha] REGRESSION: line 1 unexpectedly contains line-2 marker {marker:?}.\n\
             line 1: {line1:?}\nline 2: {line2:?}"
        );
    }

    // Piggy-back the palette assertions so this single test gates the full
    // rendering surface: both lines themed AND both lines carrying the right
    // elements.
    let not_bg = THEMES[1].bg; // #0E1720 — Projectious bg must not bleed in
    assert_rendered_theme_palette(&capture, palette, not_bg);

    runner.cleanup(test_name);
}

#[test]
#[serial]
#[ignore = "rendered tmux e2e is companion-gated; run via cargo test --test e2e --features 'e2e e2e-render'"]
fn rendered_tmux_status_bar_paints_theme_palette_gruvbox_light() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let palette = &THEMES[2]; // gruvbox-light
    let test_name = "rendered-tmux-gruvbox-light";

    init_project(&runner, test_name, palette.name);
    let capture = start_tmux_and_capture(&runner, test_name);

    // Cross-theme negative: must not bleed Projectious bg.
    let not_bg = THEMES[1].bg; // #0E1720

    assert_rendered_theme_palette(&capture, palette, not_bg);

    runner.cleanup(test_name);
}

// ─────────────────────────────────────────────────────────────────────
// Polish tier 3 tests for the live layout-switch and theme-switch
// mechanisms. They drive the *helpers* directly (skipping the
// display-menu UI which is unit-tested separately) so the assertions
// are deterministic and don't depend on send-keys timing.
// ─────────────────────────────────────────────────────────────────────

/// Driver that starts tmux via the dev layout, runs a user-supplied
/// shell snippet (`extra_action`) *while* the session is attached, then
/// captures the resulting pane. Returns `(capture_ansi, windows_before,
/// windows_after)`. `windows_before` is captured immediately before the
/// extra action, `windows_after` after it settles.
fn start_tmux_with_mid_action(
    runner: &E2eRunner,
    test_name: &str,
    extra_action: &str,
) -> (String, String, String) {
    let workspace = format!("/workspaces/{test_name}");
    let session = format!("rendered-{test_name}");
    let driver = format!(
        r#"#!/usr/bin/env bash
set -eu
export HOME="{workspace}/.aibox-home"
export TERM=xterm-256color
export COLORTERM=truecolor
export PATH="$HOME/.local/bin:/usr/local/bin:$PATH"
tmux_socket="${{AIBOX_TMUX_SOCKET:-$HOME/.tmux/aibox.sock}}"
mkdir -p "$(dirname "$tmux_socket")"
export AIBOX_TMUX_SOCKET="$tmux_socket"
tmux_alias() {{ command tmux -S "$tmux_socket" "$@"; }}
tmux_conf="$HOME/.tmux.conf"
[ -f "$tmux_conf" ] || tmux_conf="$HOME/.config/tmux/tmux.conf"
ln -sf "$tmux_conf" "$HOME/.tmux.conf"
layout_script="$HOME/.config/tmux/layouts/dev.sh"
tmux_alias kill-session -t "{session}" >/dev/null 2>&1 || true
AIBOX_TMUX_SESSION="{session}" \
AIBOX_WORKSPACE="{workspace}" \
AIBOX_TMUX_CONFIG="$tmux_conf" \
AIBOX_TMUX_SOCKET="$tmux_socket" \
  "$layout_script" &
layout_pid=$!
for _ in $(seq 1 100); do
  tmux_alias has-session -t "{session}" >/dev/null 2>&1 && break
  sleep 0.1
done
sleep 2
# Snapshot windows BEFORE the action.
tmux_alias list-windows -t "{session}" -F '#W' > "{workspace}/windows.before"
# The action runs against the live tmux server.
AIBOX_TMUX_SESSION="{session}" \
AIBOX_TMUX_SOCKET="$tmux_socket" \
{extra_action}
sleep 2
tmux_alias list-windows -t "{session}" -F '#W' > "{workspace}/windows.after"
tmux_alias capture-pane -p -e -t "{session}:0" | base64 -w0 > "{workspace}/session.ansi-capture.b64"
tmux_alias kill-session -t "{session}" >/dev/null 2>&1 || true
kill "$layout_pid" >/dev/null 2>&1 || true
wait "$layout_pid" 2>/dev/null || true
"#
    );
    runner.write_file(test_name, "driver-rendered-tmux-mid.sh", &driver);
    runner.exec(&format!("chmod +x {workspace}/driver-rendered-tmux-mid.sh"));
    let run = runner.exec(&format!(
        "LC_ALL=C.UTF-8 LANG=C.UTF-8 timeout --kill-after=5s 45s \
         bash {workspace}/driver-rendered-tmux-mid.sh 2>&1; true"
    ));
    eprintln!(
        "[rendered-tmux/{test_name}] driver stdout+stderr:\n{}",
        String::from_utf8_lossy(&run.stdout)
    );
    let b64 = runner.read_file(test_name, "session.ansi-capture.b64");
    let capture = decode_base64_capture(&b64);
    let windows_before = runner.read_file(test_name, "windows.before");
    let windows_after = runner.read_file(test_name, "windows.after");
    (capture, windows_before, windows_after)
}

#[test]
#[serial]
#[ignore = "rendered tmux e2e is companion-gated; run via cargo test --test e2e --features 'e2e e2e-render'"]
#[ntest::timeout(120_000)]
fn rendered_tmux_layout_switch_rebuilds_windows_to_focus() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let palette = &THEMES[0]; // catppuccin-mocha — palette is incidental here
    let test_name = "rendered-tmux-layout-switch";
    init_project(&runner, test_name, palette.name);

    // Mid-action: invoke the switcher helper directly. The session is
    // live, so the helper's rename/kill/respawn dance must execute
    // against the attached state.
    let (_capture, windows_before, windows_after) = start_tmux_with_mid_action(
        &runner,
        test_name,
        r#"AIBOX_LAYOUT_CONFIRM=false aibox-tmux-switch-layout focus"#,
    );

    eprintln!("[layout-switch] windows BEFORE:\n{windows_before}");
    eprintln!("[layout-switch] windows AFTER:\n{windows_after}");

    // Dev layout's first window is "work" (split: yazi + bash + harness).
    assert!(
        windows_before.lines().any(|w| w == "work"),
        "fresh dev layout should produce a 'work' window before switch:\n{windows_before}"
    );

    // Focus layout's first window is "files" (yazi only) + one window per
    // harness. After the switch, "work" must be gone, "files" present.
    assert!(
        windows_after.lines().any(|w| w == "files"),
        "after switch to focus layout, a 'files' window must exist:\n{windows_after}"
    );
    assert!(
        !windows_after.lines().any(|w| w == "work"),
        "after switch to focus layout, the dev-layout 'work' window must NOT survive:\n{windows_after}"
    );
    assert!(
        !windows_after.lines().any(|w| w == "_swap_"),
        "the helper's placeholder window must be cleaned up:\n{windows_after}"
    );

    runner.cleanup(test_name);
}

#[test]
#[serial]
#[ignore = "rendered tmux e2e is companion-gated; run via cargo test --test e2e --features 'e2e e2e-render'"]
#[ntest::timeout(120_000)]
fn rendered_tmux_theme_switch_changes_status_bar_surface() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let start_palette = &THEMES[0]; // catppuccin-mocha, surface #313244
    let end_palette = &THEMES[1]; // projectious, surface #131E2B
    let test_name = "rendered-tmux-theme-switch";
    init_project(&runner, test_name, start_palette.name);

    // Mid-action: switch theme via the CLI (regenerates managed files),
    // then re-source tmux.conf so the new powerkit custom-theme file is
    // loaded. We skip the live-refresh helper's send-keys path — only
    // the rendered status bar matters for this assertion.
    let workspace = format!("/workspaces/{test_name}");
    let target = end_palette.name;
    let extra = format!(
        r#"cd "{workspace}" && AIBOX_HOST_ROOT="$HOME" aibox theme --theme {target} --no-restart-session >/dev/null 2>&1 || true
tmux_alias source-file "$tmux_conf" >/dev/null 2>&1 || true
tmux_alias refresh-client >/dev/null 2>&1 || true"#,
    );

    let (capture, _windows_before, _windows_after) =
        start_tmux_with_mid_action(&runner, test_name, &extra);
    let parser = vt_render::parse_tmux_capture(&capture);
    let screen = parser.screen();

    // The rendered status bar should now carry the NEW theme's surface.
    let new_surface_count = vt_render::count_cells_with_bg(screen, end_palette.surface);
    assert!(
        new_surface_count >= 20,
        "after theme switch from {} → {}, expected ≥20 cells with new surface {} but found {}\n\
         (live theme switch did not re-render the status bar)",
        start_palette.name,
        end_palette.name,
        end_palette.surface,
        new_surface_count
    );

    // And the OLD theme's surface should now be absent (or trivially few
    // cells — accidental overlap from cursor or other rendering noise).
    let old_surface_count = vt_render::count_cells_with_bg(screen, start_palette.surface);
    assert!(
        old_surface_count < 5,
        "after theme switch from {} → {}, expected <5 cells with old surface {} but found {}\n\
         (the previous theme's palette did not fully clear)",
        start_palette.name,
        end_palette.name,
        start_palette.surface,
        old_surface_count
    );

    runner.cleanup(test_name);
}
