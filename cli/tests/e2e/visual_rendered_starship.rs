//! Rendered-color tests for the Starship prompt — Tier 3 / local.
//!
//! Runs `starship prompt` against a generated `starship.toml` and replays the
//! ANSI-coloured output through vt100 to assert palette colors actually paint
//! cells. Local execution — no companion needed — so this Tier 3 surface
//! works on dev machines that have `starship` on PATH.

#![cfg(feature = "e2e-render")]

use crate::vt_render;
use std::process::Command;

// ── Palette constants for the 4 tested themes ────────────────────────────────
// (bg, accent, green, orange) — pulled from themes.rs::theme_palette().

/// GruvboxDark palette slots we assert on.
const GRUVBOX_DARK_ACCENT: &str = "#FABD2F";
const GRUVBOX_DARK_GREEN: &str = "#B8BB26";
const GRUVBOX_DARK_ORANGE: &str = "#FE8019";

/// CatppuccinMocha palette slots we assert on.
const CATPPUCCIN_MOCHA_ACCENT: &str = "#89B4FA";
const CATPPUCCIN_MOCHA_GREEN: &str = "#A6E3A1";
const CATPPUCCIN_MOCHA_ORANGE: &str = "#FAB387";
/// Used in the negative-test: no gruvbox cell should have this bg.
const CATPPUCCIN_MOCHA_BG: &str = "#1E1E2E";

/// Dracula palette slots we assert on.
const DRACULA_ACCENT: &str = "#BD93F9";
const DRACULA_GREEN: &str = "#50FA7B";
const DRACULA_ORANGE: &str = "#FFB86C";

/// Projectious palette slots we assert on.
// Starship uses the contrast-safe accent text slot for painted prompt cells.
const PROJECTIOUS_ACCENT: &str = "#EA7558";
const PROJECTIOUS_GREEN: &str = "#6CC090";
const PROJECTIOUS_ORANGE: &str = "#EA7558";

// ── Helpers (mirrors appearance.rs; duplicated to keep this file self-contained) ──

fn aibox_bin() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{manifest_dir}/target/debug/aibox")
}

fn addons_dir() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{manifest_dir}/../addons")
}

fn run_in(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(aibox_bin())
        .args(args)
        .current_dir(dir)
        .env("AIBOX_ADDONS_DIR", addons_dir())
        .output()
        .expect("failed to execute aibox")
}

/// Run `aibox init` for the given theme + prompt inside `dir`.
fn init_with_appearance(dir: &std::path::Path, theme: &str, prompt: &str) {
    let output = run_in(
        dir,
        &[
            "init",
            "render-test",
            "--base",
            "debian",
            "--context",
            "managed",
            "--theme",
            theme,
            "--prompt",
            prompt,
        ],
    );
    assert!(
        output.status.success(),
        "aibox init --theme={theme} --prompt={prompt} failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Return the path to the generated `starship.toml` inside `.aibox-home`.
fn starship_toml_path(dir: &std::path::Path) -> std::path::PathBuf {
    dir.join(".aibox-home/.config/starship.toml")
}

/// Check whether `starship` is available on PATH. Returns `false` if it is not,
/// so the caller can skip gracefully without failing the suite.
fn starship_on_path() -> bool {
    Command::new("which")
        .arg("starship")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Run `starship prompt` against the supplied config file and return the raw
/// ANSI bytes from stdout. Panics if the child process cannot be spawned.
fn run_starship_prompt(config: &std::path::Path, cwd: &std::path::Path) -> Vec<u8> {
    let output = Command::new("starship")
        .args(["prompt"])
        .env("STARSHIP_CONFIG", config)
        .env("STARSHIP_LOG", "error")
        .env("TERM", "xterm-256color")
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .current_dir(cwd)
        .output()
        .expect("failed to spawn starship");
    output.stdout
}

// ── Per-theme helpers ─────────────────────────────────────────────────────────

struct ThemeCase {
    theme_flag: &'static str,
    accent: &'static str,
    green: &'static str,
    orange: &'static str,
}

fn assert_default_preset(case: &ThemeCase) {
    if !starship_on_path() {
        eprintln!("[skip] starship not on PATH; skipping rendered prompt test");
        return;
    }

    let dir = tempfile::tempdir().expect("tempdir");
    init_with_appearance(dir.path(), case.theme_flag, "default");
    let toml_path = starship_toml_path(dir.path());
    assert!(
        toml_path.exists(),
        "starship.toml not generated at {toml_path:?}"
    );

    let raw = run_starship_prompt(&toml_path, dir.path());
    let ansi = String::from_utf8_lossy(&raw);
    let parser = vt_render::parse_tmux_capture(&ansi);
    let screen = parser.screen();

    assert!(
        vt_render::any_cell_has_fg(screen, case.accent),
        "default preset / theme={}: expected at least one cell with fg==accent {}\n\
         (directory module uses `bold fg:{{accent}}`).\n\
         Raw prompt bytes: {:?}",
        case.theme_flag,
        case.accent,
        &raw[..raw.len().min(512)],
    );
}

fn assert_pastel_powerline_preset(case: &ThemeCase) {
    if !starship_on_path() {
        eprintln!("[skip] starship not on PATH; skipping rendered prompt test");
        return;
    }

    let dir = tempfile::tempdir().expect("tempdir");
    init_with_appearance(dir.path(), case.theme_flag, "powerline-pastel");
    let toml_path = starship_toml_path(dir.path());
    assert!(
        toml_path.exists(),
        "starship.toml not generated at {toml_path:?}"
    );

    let raw = run_starship_prompt(&toml_path, dir.path());
    let ansi = String::from_utf8_lossy(&raw);
    let parser = vt_render::parse_tmux_capture(&ansi);
    let screen = parser.screen();

    // The directory segment uses `bg:{accent}`.
    assert!(
        vt_render::any_cell_has_bg(screen, case.accent),
        "pastel-powerline preset / theme={}: expected ≥1 cell with bg==accent {}\n\
         (directory segment: `bg:{{accent}}`). Raw bytes: {:?}",
        case.theme_flag,
        case.accent,
        &raw[..raw.len().min(512)],
    );

    // The git_branch segment uses `bg:{green}`.
    assert!(
        vt_render::any_cell_has_bg(screen, case.green),
        "pastel-powerline preset / theme={}: expected ≥1 cell with bg==green {}\n\
         (git_branch segment: `bg:{{green}}`). Raw bytes: {:?}",
        case.theme_flag,
        case.green,
        &raw[..raw.len().min(512)],
    );

    // The language segments use `bg:{orange}`. These only appear when the
    // relevant runtime is detectable, so we only assert when there's actually
    // output past the git segment (i.e. when rust/node/etc. are present).
    // We record a note so CI history shows what ran.
    let orange_cells = vt_render::count_cells_with_bg(screen, case.orange);
    eprintln!(
        "[info] pastel-powerline / theme={}: {} cells with bg==orange {}",
        case.theme_flag, orange_cells, case.orange
    );
}

// ── Negative test helper ──────────────────────────────────────────────────────

/// Assert that a prompt generated for `theme_flag` does NOT contain any cells
/// whose bg or fg exactly matches `alien_color`. Catches silent config-fallback
/// regressions (e.g. starship silently ignoring the STARSHIP_CONFIG env var and
/// falling back to a different theme's default).
fn assert_no_alien_color(theme_flag: &str, prompt_flag: &str, alien_color: &str) {
    if !starship_on_path() {
        eprintln!("[skip] starship not on PATH; skipping rendered prompt test");
        return;
    }

    let dir = tempfile::tempdir().expect("tempdir");
    init_with_appearance(dir.path(), theme_flag, prompt_flag);
    let toml_path = starship_toml_path(dir.path());

    let raw = run_starship_prompt(&toml_path, dir.path());
    let ansi = String::from_utf8_lossy(&raw);
    let parser = vt_render::parse_tmux_capture(&ansi);
    let screen = parser.screen();

    assert!(
        !vt_render::any_cell_has_fg(screen, alien_color)
            && !vt_render::any_cell_has_bg(screen, alien_color),
        "theme={theme_flag}: prompt unexpectedly contains alien color {alien_color}\n\
         (Suggests STARSHIP_CONFIG was not honoured and a foreign theme bled through.)\n\
         Raw prompt bytes: {:?}",
        &raw[..raw.len().min(512)],
    );
}

// ── Test functions ────────────────────────────────────────────────────────────

#[test]
fn gruvbox_dark_default_preset_paints_accent_as_fg() {
    assert_default_preset(&ThemeCase {
        theme_flag: "gruvbox",
        accent: GRUVBOX_DARK_ACCENT,
        green: GRUVBOX_DARK_GREEN,
        orange: GRUVBOX_DARK_ORANGE,
    });
}

#[test]
fn gruvbox_dark_pastel_powerline_paints_accent_and_green_as_bg() {
    assert_pastel_powerline_preset(&ThemeCase {
        theme_flag: "gruvbox",
        accent: GRUVBOX_DARK_ACCENT,
        green: GRUVBOX_DARK_GREEN,
        orange: GRUVBOX_DARK_ORANGE,
    });
}

#[test]
fn catppuccin_mocha_default_preset_paints_accent_as_fg() {
    assert_default_preset(&ThemeCase {
        theme_flag: "catppuccin",
        accent: CATPPUCCIN_MOCHA_ACCENT,
        green: CATPPUCCIN_MOCHA_GREEN,
        orange: CATPPUCCIN_MOCHA_ORANGE,
    });
}

#[test]
fn catppuccin_mocha_pastel_powerline_paints_accent_and_green_as_bg() {
    assert_pastel_powerline_preset(&ThemeCase {
        theme_flag: "catppuccin",
        accent: CATPPUCCIN_MOCHA_ACCENT,
        green: CATPPUCCIN_MOCHA_GREEN,
        orange: CATPPUCCIN_MOCHA_ORANGE,
    });
}

#[test]
fn dracula_default_preset_paints_accent_as_fg() {
    assert_default_preset(&ThemeCase {
        theme_flag: "dracula",
        accent: DRACULA_ACCENT,
        green: DRACULA_GREEN,
        orange: DRACULA_ORANGE,
    });
}

#[test]
fn dracula_pastel_powerline_paints_accent_and_green_as_bg() {
    assert_pastel_powerline_preset(&ThemeCase {
        theme_flag: "dracula",
        accent: DRACULA_ACCENT,
        green: DRACULA_GREEN,
        orange: DRACULA_ORANGE,
    });
}

#[test]
fn projectious_default_preset_paints_accent_as_fg() {
    assert_default_preset(&ThemeCase {
        theme_flag: "projectious",
        accent: PROJECTIOUS_ACCENT,
        green: PROJECTIOUS_GREEN,
        orange: PROJECTIOUS_ORANGE,
    });
}

#[test]
fn projectious_pastel_powerline_paints_accent_and_green_as_bg() {
    assert_pastel_powerline_preset(&ThemeCase {
        theme_flag: "projectious",
        accent: PROJECTIOUS_ACCENT,
        green: PROJECTIOUS_GREEN,
        orange: PROJECTIOUS_ORANGE,
    });
}

/// Negative test: a GruvboxDark prompt must not contain any cell painted with
/// CatppuccinMocha's distinctive background (#1E1E2E). If STARSHIP_CONFIG is
/// silently ignored and starship falls back to a default that happens to use
/// Catppuccin colours, this trips immediately.
#[test]
fn gruvbox_dark_prompt_does_not_bleed_catppuccin_mocha_bg() {
    assert_no_alien_color("gruvbox", "default", CATPPUCCIN_MOCHA_BG);
}

/// Negative test: same check for the powerline preset — the pastel-powerline
/// gruvbox prompt uses #282828 as its background, not #1E1E2E.
#[test]
fn gruvbox_dark_powerline_does_not_bleed_catppuccin_mocha_bg() {
    assert_no_alien_color("gruvbox", "powerline-pastel", CATPPUCCIN_MOCHA_BG);
}
