//! Rendered-color tests for the Yazi file manager — Tier 3 / companion.
//!
//! Launches Yazi inside a real tmux session on the e2e companion, drives it
//! into a known view, captures the pane via `tmux capture-pane -p -e`, and
//! asserts per-cell colors against the active theme's palette using
//! `crate::vt_render`.

#![cfg(all(feature = "e2e", feature = "e2e-render"))]

use serial_test::serial;

use crate::runner::E2eRunner;
use crate::vt_render;

/// Per-theme palette triplets: (theme-slug, bg, surface, accent).
///
/// `bg`      — Yazi fills the body with this color; ≥50 cells expected.
/// `surface` — bottom status bar `[status] overall` bg.
/// `accent`  — cwd breadcrumb fg AND active-tab bg.
///
/// Hex values are cross-referenced from `cli/src/themes.rs` `theme_palette`
/// and `yazi_surface_color` functions. We hardcode them here because the bin
/// crate is not importable from tests/.
const THEMES: &[(&str, &str, &str, &str)] = &[
    // catppuccin-mocha: base=#1E1E2E  surface0=#313244  blue=#89B4FA
    ("catppuccin-mocha", "#1E1E2E", "#313244", "#89B4FA"),
    // projectious: bg=#0E1720  surface=#131E2B  red=#E05232
    ("projectious", "#0E1720", "#131E2B", "#E05232"),
    // gruvbox-light: bg=#FBF1C7  surface1=#EBDBB2  orange=#D65D0E
    ("gruvbox-light", "#FBF1C7", "#EBDBB2", "#D65D0E"),
];

/// Build the shell driver that:
///   1. Configures HOME so Yazi reads the aibox-generated theme config.
///   2. Launches a detached tmux session running `yazi <workspace>`.
///   3. Waits 5 s for Yazi to scan + paint its initial frame.
///   4. Captures the pane with SGR attributes preserved (`-e`) into a file.
///   5. Kills the session.
fn yazi_capture_driver(workspace: &str, session: &str) -> String {
    format!(
        r#"#!/usr/bin/env bash
set -eu
export HOME="{workspace}/.aibox-home"
export TERM=xterm-256color
export COLORTERM=truecolor
export PATH="$HOME/.local/bin:/usr/local/bin:$PATH"
tmux_socket="${{AIBOX_TMUX_SOCKET:-$HOME/.tmux/aibox.sock}}"
mkdir -p "$(dirname "$tmux_socket")"

# Kill any leftover session from a previous run.
command tmux -S "$tmux_socket" kill-session -t "{session}" >/dev/null 2>&1 || true

# Start a detached tmux session running Yazi.
command tmux -S "$tmux_socket" new-session -d -s "{session}" -x 200 -y 60 \
  "exec env TERM=xterm-256color COLORTERM=truecolor HOME={workspace}/.aibox-home yazi {workspace}"

# Give Yazi time to scan the directory and paint the full initial frame.
sleep 5

# Capture the rendered pane (ANSI SGR preserved) into a file.
command tmux -S "$tmux_socket" capture-pane -p -e -t "{session}:0" \
  > "{workspace}/yazi.ansi-capture" 2>/dev/null || true

# Tear down.
command tmux -S "$tmux_socket" kill-session -t "{session}" >/dev/null 2>&1 || true
"#
    )
}

#[test]
#[serial]
#[ignore = "rendered yazi e2e is companion-gated; run via cargo test --test e2e --features 'e2e e2e-render'"]
#[ntest::timeout(180_000)]
fn visual_yazi_theme_palette_paints_real_terminal_cells() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    // Kill any stale yazi / tmux processes from a previous aborted run.
    runner.exec(
        "timeout 2s tmux kill-server >/dev/null 2>&1 || true; \
         timeout 2s pkill -x yazi >/dev/null 2>&1 || true",
    );

    for &(theme, bg, surface, accent) in THEMES {
        let test_name = format!("yazi-render-{theme}");
        eprintln!("[yazi-render] theme={theme} bg={bg} surface={surface} accent={accent}");

        // ── 1. Provision workspace ──────────────────────────────────────────
        runner.cleanup(&test_name);

        let init = runner.aibox(
            &test_name,
            &[
                "init",
                &test_name,
                "--base",
                "debian",
                "--context",
                "managed",
                "--processkit-version",
                "unset",
                "--theme",
                theme,
                "--harness",
                "claude",
                "--no-container",
            ],
        );
        assert!(
            init.status.success(),
            "{test_name}: init failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&init.stdout),
            String::from_utf8_lossy(&init.stderr)
        );

        let apply = runner.aibox(&test_name, &["apply", "--no-container"]);
        assert!(
            apply.status.success(),
            "{test_name}: apply failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&apply.stdout),
            String::from_utf8_lossy(&apply.stderr)
        );

        // ── 2. Write fixture files so Yazi has files to list ────────────────
        let workspace = format!("/workspaces/{test_name}");
        runner.write_file(&test_name, "a.rs", "fn main() {}\n");
        runner.write_file(&test_name, "b.py", "print('hello')\n");
        runner.write_file(&test_name, "c.md", "# Hello\n");
        runner.write_file(&test_name, "README.md", "# README\n");

        // ── 3. Write + execute the capture driver ───────────────────────────
        let session = format!("yazi-render-{theme}");
        let driver_script = yazi_capture_driver(&workspace, &session);
        runner.write_file(&test_name, "yazi-capture-driver.sh", &driver_script);
        runner.exec(&format!("chmod +x {workspace}/yazi-capture-driver.sh"));

        // Run the driver; errors are non-fatal here because the capture file
        // may still have been written before any cleanup failure.
        let drive_out = runner.exec(&format!(
            "AIBOX_TMUX_SOCKET=/workspaces/{test_name}/.aibox-home/.tmux/aibox.sock \
             bash {workspace}/yazi-capture-driver.sh"
        ));
        if !drive_out.status.success() {
            eprintln!(
                "[yazi-render] driver stderr for {theme}:\n{}",
                String::from_utf8_lossy(&drive_out.stderr)
            );
        }

        // ── 4. Read the ANSI capture ─────────────────────────────────────────
        let capture = runner.read_file(&test_name, "yazi.ansi-capture");

        // Emit a length diagnostic so CI logs are informative.
        eprintln!(
            "[yazi-render] theme={theme}: capture length={} bytes",
            capture.len()
        );

        let parser = vt_render::parse_tmux_capture(&capture);
        let screen = parser.screen();

        // ── 5. Collect black-cell coordinates for the stray-black check ──────
        let black_cells: Vec<(u16, u16)> = vt_render::iter_cells(screen)
            .filter_map(|(r, c, cell)| {
                if vt_render::color_eq_hex(cell.bgcolor(), "#000000") {
                    Some((r, c))
                } else {
                    None
                }
            })
            .collect();

        // ── 6. Assertions ─────────────────────────────────────────────────────

        // 6a. Yazi body fills the pane with the theme bg.
        let bg_count = vt_render::count_cells_with_bg(screen, bg);
        assert!(
            bg_count >= 50,
            "{theme}: expected ≥50 cells with bg={bg} (Yazi body background) but found {bg_count}.\n\
             Likely the theme was not applied — check {workspace}/yazi.ansi-capture."
        );

        // 6b. The accent color appears as a foreground color somewhere.
        assert!(
            vt_render::any_cell_has_fg(screen, accent),
            "{theme}: expected at least one cell with fg={accent} (cwd breadcrumb / accent fg) \
             but found none."
        );

        // 6c. The surface color appears as a background color somewhere
        //     (bottom status bar uses the surface palette).
        assert!(
            vt_render::any_cell_has_bg(screen, surface),
            "{theme}: expected at least one cell with bg={surface} (status bar surface bg) \
             but found none."
        );

        // 6d. The accent appears as a background color somewhere
        //     (active tab indicator uses accent as bg).
        assert!(
            vt_render::any_cell_has_bg(screen, accent),
            "{theme}: expected at least one cell with bg={accent} (active tab bg) \
             but found none."
        );

        // 6e. Stray-black guard: very few cells should have a pure-black bg.
        //     If everything is black the theme config didn't load.
        if !black_cells.is_empty() {
            eprintln!(
                "[yazi-render] {theme}: {} black cell(s) found at: {:?}",
                black_cells.len(),
                &black_cells[..black_cells.len().min(20)]
            );
        }
        assert!(
            black_cells.len() < 5,
            "{theme}: found {} cells with bg=#000000 (stray-black regression — \
             theme palette was not applied). Black cell coordinates (up to 20): {:?}",
            black_cells.len(),
            &black_cells[..black_cells.len().min(20)]
        );

        // ── 7. Cross-theme negative checks ───────────────────────────────────
        // Ensure we aren't looking at a different theme's leftover palette.
        match theme {
            "catppuccin-mocha" => {
                assert!(
                    !vt_render::any_cell_has_bg(screen, "#0E1720"),
                    "catppuccin-mocha: found Projectious bg (#0E1720) on screen — \
                     cross-theme contamination detected."
                );
            }
            "projectious" => {
                assert!(
                    !vt_render::any_cell_has_bg(screen, "#1E1E2E"),
                    "projectious: found Catppuccin Mocha bg (#1E1E2E) on screen — \
                     cross-theme contamination detected."
                );
            }
            _ => {}
        }

        // ── 8. Per-theme cleanup ──────────────────────────────────────────────
        runner.cleanup(&test_name);
    }
}
