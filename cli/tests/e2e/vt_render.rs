//! VT-100 cell-level rendered-color assertion helpers.
//!
//! Replays a terminal byte stream through a vt100 parser and exposes
//! ergonomic checks against a theme palette. Unlike grepping `*.conf` files,
//! these helpers prove that the **actual cells painted on the terminal**
//! carry the colors the theme demands — closing the historical gap where a
//! tmux-background-not-themed regression slipped past the visual e2e suite.
//!
//! Two flavors of input are supported:
//!
//! - `tmux capture-pane -p -e` output → line-based ANSI with each line
//!   prefixed by SGR codes. Use [`parse_tmux_capture`].
//! - asciinema cast JSONL → each "o" event carries ANSI bytes. Use
//!   [`parse_asciinema_cast`].
//!
//! Both produce a [`vt100::Parser`] whose `screen()` exposes per-cell colors.
//!
//! The helpers operate on hex strings (e.g. `"#1E1E2E"`) to match the rest of
//! the theme infrastructure. RGB equality is exact — vt100 normalizes
//! direct-color SGRs (`48;2;R;G;B`) to its `Color::Rgb` variant, so any byte
//! drift trips the assertion.

#![cfg(feature = "e2e-render")]
#![allow(dead_code)] // tests in this module are gated; helpers are surface-by-surface.

use std::path::Path;
use vt100::{Color, Parser, Screen};

/// Build a parser sized for typical terminal captures. 200×80 leaves
/// headroom for status-bar rows + a few panes without truncation.
pub fn make_parser() -> Parser {
    Parser::new(80, 200, 0)
}

/// Build a parser sized to match an asciinema recording's declared geometry.
/// Falls back to [`make_parser`] dimensions if parsing the header fails.
pub fn make_parser_for_cast(cast: &str) -> Parser {
    let header = cast.lines().next().unwrap_or("");
    let (rows, cols) = parse_cast_geometry(header).unwrap_or((80, 200));
    Parser::new(rows, cols, 0)
}

fn parse_cast_geometry(header: &str) -> Option<(u16, u16)> {
    // Header is JSON like `{"version":2,"width":160,"height":45,...}`.
    let v: serde_json::Value = serde_json::from_str(header).ok()?;
    let w = v.get("width")?.as_u64()? as u16;
    let h = v.get("height")?.as_u64()? as u16;
    Some((h, w))
}

/// Parse the output of `tmux capture-pane -p -e -t <session>:<pane>` into a
/// vt100 parser. The capture is a pre-painted snapshot: each visible line is
/// emitted with the SGRs needed to recreate its cells, separated by `\n`.
pub fn parse_tmux_capture(capture: &str) -> Parser {
    let mut parser = make_parser();
    // Reset to a known state, then feed the capture. capture-pane uses LF
    // line endings; the parser handles them as cursor-down moves.
    parser.process(b"\x1bc"); // RIS — full reset
    parser.process(capture.as_bytes());
    parser
}

/// Parse an asciinema v2 cast file and return a parser whose `screen()`
/// reflects the final rendered state.
pub fn parse_asciinema_cast(path: impl AsRef<Path>) -> Parser {
    let body = std::fs::read_to_string(path.as_ref())
        .unwrap_or_else(|e| panic!("failed to read cast {}: {e}", path.as_ref().display()));
    parse_asciinema_cast_str(&body)
}

pub fn parse_asciinema_cast_str(cast: &str) -> Parser {
    let mut parser = make_parser_for_cast(cast);
    parser.process(b"\x1bc");
    for line in cast.lines().skip(1) {
        // Each event line is JSON: [timestamp, "o"|"i", "<data>"].
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        let Some(arr) = v.as_array() else { continue };
        if arr.len() < 3 {
            continue;
        }
        if arr[1].as_str() != Some("o") {
            continue;
        }
        let Some(data) = arr[2].as_str() else {
            continue;
        };
        parser.process(data.as_bytes());
    }
    parser
}

/// Parse `#RRGGBB` → `(R, G, B)`.
pub fn hex_rgb(hex: &str) -> (u8, u8, u8) {
    let h = hex.strip_prefix('#').unwrap_or(hex);
    assert_eq!(h.len(), 6, "expected #RRGGBB, got {hex}");
    let r = u8::from_str_radix(&h[0..2], 16).unwrap_or_else(|_| panic!("bad hex: {hex}"));
    let g = u8::from_str_radix(&h[2..4], 16).unwrap_or_else(|_| panic!("bad hex: {hex}"));
    let b = u8::from_str_radix(&h[4..6], 16).unwrap_or_else(|_| panic!("bad hex: {hex}"));
    (r, g, b)
}

/// Match a vt100 [`Color`] against a hex string. `Color::Default` never
/// matches anything (the test wants a *concrete* color present on screen).
pub fn color_eq_hex(color: Color, hex: &str) -> bool {
    let (er, eg, eb) = hex_rgb(hex);
    matches!(color, Color::Rgb(r, g, b) if r == er && g == eg && b == eb)
}

/// Returns true if any cell in the screen has `bg == hex`. Useful to assert
/// a theme surface (e.g. tmux status bar bg) actually rendered somewhere.
pub fn any_cell_has_bg(screen: &Screen, hex: &str) -> bool {
    iter_cells(screen).any(|(_r, _c, cell)| color_eq_hex(cell.bgcolor(), hex))
}

/// Returns true if any cell has `fg == hex`.
pub fn any_cell_has_fg(screen: &Screen, hex: &str) -> bool {
    iter_cells(screen).any(|(_r, _c, cell)| color_eq_hex(cell.fgcolor(), hex))
}

/// Count cells whose bg matches `hex`. Distinguishes "the bg appears once
/// (likely a stray sequence in a fixture)" from "the bg actually paints a
/// surface".
pub fn count_cells_with_bg(screen: &Screen, hex: &str) -> usize {
    iter_cells(screen)
        .filter(|(_r, _c, cell)| color_eq_hex(cell.bgcolor(), hex))
        .count()
}

/// Yield every cell on the screen as `(row, col, cell)`.
pub fn iter_cells(screen: &Screen) -> impl Iterator<Item = (u16, u16, &vt100::Cell)> {
    let (rows, cols) = screen.size();
    (0..rows).flat_map(move |r| {
        (0..cols).filter_map(move |c| screen.cell(r, c).map(|cell| (r, c, cell)))
    })
}

/// Returns true if a cell carries any visible content or styled background.
/// Used by `last_content_row` to skip the empty rows below a tmux capture.
pub fn cell_is_rendered(cell: &vt100::Cell) -> bool {
    if !matches!(cell.bgcolor(), Color::Default) {
        return true;
    }
    if !matches!(cell.fgcolor(), Color::Default) {
        return true;
    }
    let contents = cell.contents();
    !contents.is_empty() && contents.chars().any(|c| !c.is_whitespace())
}

/// Returns the index of the last row that has any rendered content. Useful
/// when a 200x80 parser receives a 24-row tmux capture: the parser is
/// generously sized but only the first 24 rows hold content; the rest are
/// `Color::Default` and would skew "bottom N rows" assertions.
///
/// Returns `None` if the screen is entirely empty.
pub fn last_content_row(screen: &Screen) -> Option<u16> {
    let (rows, cols) = screen.size();
    (0..rows).rev().find(|&r| {
        (0..cols)
            .filter_map(|c| screen.cell(r, c))
            .any(cell_is_rendered)
    })
}

/// Returns `(start_row, end_row_exclusive)` covering the bottom `n` *rendered*
/// rows — i.e. the bottom `n` rows of the captured content, not the bottom
/// `n` rows of the parser grid. Panics if the screen is empty (caller's bug).
pub fn bottom_content_rows(screen: &Screen, n: u16) -> (u16, u16) {
    let last = last_content_row(screen).expect("bottom_content_rows called on an empty screen");
    let end = last + 1;
    let start = end.saturating_sub(n);
    (start, end)
}

/// Count cells with `bg == hex` within a single row. Used by per-row checks
/// that must distinguish "row 22 fully painted with surface" from "row 23
/// fully painted, row 22 empty".
pub fn count_bg_in_row(screen: &Screen, row: u16, hex: &str) -> usize {
    let (_rows, cols) = screen.size();
    (0..cols)
        .filter_map(|c| screen.cell(row, c))
        .filter(|cell| color_eq_hex(cell.bgcolor(), hex))
        .count()
}

/// Count cells in a row that are `Color::Default` background — i.e. cells
/// the renderer never touched. A two-line status bar where line 1 is empty
/// leaves an entire row of `Color::Default` bgs above the painted line 2.
pub fn count_default_bg_in_row(screen: &Screen, row: u16) -> usize {
    let (_rows, cols) = screen.size();
    (0..cols)
        .filter_map(|c| screen.cell(row, c))
        .filter(|cell| matches!(cell.bgcolor(), Color::Default))
        .count()
}

/// Return the visible text content of a row as a single string. Used by
/// tests that need to assert specific labels (e.g. `customization.tmux
/// .status.labels.aibox_log`) appear on the expected status line. Trailing
/// whitespace is trimmed.
pub fn row_text(screen: &Screen, row: u16) -> String {
    let (_rows, cols) = screen.size();
    let mut s = String::with_capacity(cols as usize);
    for c in 0..cols {
        if let Some(cell) = screen.cell(row, c) {
            let contents = cell.contents();
            if contents.is_empty() {
                s.push(' ');
            } else {
                s.push_str(contents);
            }
        } else {
            s.push(' ');
        }
    }
    s.trim_end().to_string()
}

/// Capture a tmux pane on the companion. Wraps `capture-pane -p -e` so the
/// emitted ANSI sequences survive the SSH round-trip. The base64 transport
/// is important: raw escapes through ssh+exec lose escape interpretation in
/// some shells.
pub fn capture_pane_ansi(runner: &super::runner::E2eRunner, session: &str, target: &str) -> String {
    let socket = "$HOME/.tmux/aibox.sock";
    let cmd = format!(
        r#"export AIBOX_TMUX_SOCKET="{socket}"; \
           tmux -S "$AIBOX_TMUX_SOCKET" capture-pane -p -e -t "{session}:{target}" | base64 -w0"#
    );
    let output = runner.exec(&cmd);
    let b64 = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let decoded = base64_decode(&b64);
    String::from_utf8_lossy(&decoded).to_string()
}

fn base64_decode(input: &str) -> Vec<u8> {
    // Minimal RFC-4648 decoder; std doesn't ship one. The companion's
    // `base64 -w0` output is well-formed (no whitespace, no URL-safe alphabet),
    // so this is sufficient. We don't reach for the `base64` crate to keep
    // dev-deps small.
    const ALPHA: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut lut = [255u8; 256];
    for (i, &c) in ALPHA.iter().enumerate() {
        lut[c as usize] = i as u8;
    }
    let bytes: Vec<u8> = input
        .bytes()
        .filter(|b| *b != b'\n' && *b != b'\r' && *b != b' ')
        .collect();
    let mut out = Vec::with_capacity(bytes.len() / 4 * 3);
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
    out
}

#[cfg(test)]
mod self_tests {
    use super::*;

    #[test]
    fn hex_rgb_round_trips() {
        assert_eq!(hex_rgb("#1E1E2E"), (0x1E, 0x1E, 0x2E));
        assert_eq!(hex_rgb("#FFFFFF"), (0xFF, 0xFF, 0xFF));
        assert_eq!(hex_rgb("#000000"), (0, 0, 0));
    }

    #[test]
    fn color_eq_hex_rejects_default_and_palette() {
        assert!(!color_eq_hex(Color::Default, "#000000"));
        // Indexed color is *not* equal to any hex — we want concrete RGB.
        assert!(!color_eq_hex(Color::Idx(0), "#000000"));
        assert!(color_eq_hex(Color::Rgb(0x1E, 0x1E, 0x2E), "#1E1E2E"));
    }

    #[test]
    fn parse_tmux_capture_sets_cell_bg_from_sgr() {
        // `\e[48;2;30;30;46m` sets bg to #1E1E2E (catppuccin mocha base).
        // Followed by a space character that should now carry that bg.
        let capture = "\u{1b}[48;2;30;30;46m \u{1b}[0m\n";
        let parser = parse_tmux_capture(capture);
        let screen = parser.screen();
        assert!(
            any_cell_has_bg(screen, "#1E1E2E"),
            "expected at least one cell with bg #1E1E2E"
        );
    }

    #[test]
    fn parse_asciinema_cast_str_handles_header_and_events() {
        let cast = "\
{\"version\":2,\"width\":80,\"height\":24}
[0.1, \"o\", \"\\u001b[48;2;14;23;32m \\u001b[0m\"]
[0.2, \"o\", \"hello\"]
";
        let parser = parse_asciinema_cast_str(cast);
        let screen = parser.screen();
        assert!(
            any_cell_has_bg(screen, "#0E1720"),
            "projectious bg #0E1720 should paint a cell"
        );
    }

    #[test]
    fn base64_decode_round_trips() {
        // "hello" → "aGVsbG8="
        assert_eq!(base64_decode("aGVsbG8="), b"hello".to_vec());
        // multi-line should be stripped
        assert_eq!(base64_decode("aGVs\nbG8="), b"hello".to_vec());
    }
}
