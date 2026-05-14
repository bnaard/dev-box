//! Appearance integration tests — verify the `aibox init` / `aibox apply`
//! CLI flow produces seeded artifacts at expected paths.
//!
//! Per-theme and per-preset content assertions live in the **unit suite**
//! (`cli/src/themes.rs::tests`). What stays here is strictly integration:
//! - That CLI flags (`--theme`, `--prompt`) flow into the seeded files.
//! - That `aibox apply` after a config edit regenerates the right files.
//! - That CLI preset aliases resolve correctly.
//!
//! These are Tier 1 tests: they run `aibox init` + `aibox apply` locally
//! and inspect the generated/seeded config files. No container needed.

use std::fs;
use std::process::Command;

fn aibox_bin() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{}/target/debug/aibox", manifest_dir)
}

fn addons_dir() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{}/../addons", manifest_dir)
}

fn run_in(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(aibox_bin())
        .args(args)
        .current_dir(dir)
        .env("AIBOX_ADDONS_DIR", addons_dir())
        .output()
        .expect("failed to execute aibox")
}

/// Initialize with specific theme and prompt.
fn init_with_appearance(dir: &std::path::Path, theme: &str, prompt: &str) {
    let output = run_in(
        dir,
        &[
            "init",
            "appearance-test",
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
        "init with theme={} prompt={} failed: {}",
        theme,
        prompt,
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Replace the customization section in aibox.toml and re-apply.
fn change_appearance(dir: &std::path::Path, theme: &str, prompt: &str) {
    let toml_path = dir.join("aibox.toml");
    let content = fs::read_to_string(&toml_path).unwrap();

    // Replace [customization] section — search for header at start of line to avoid matching comments.
    let section_header = "[customization]";
    let needle = format!("\n{}", section_header);
    if let Some(needle_pos) = content.find(&needle) {
        let start = needle_pos + 1; // position of '[' in section header
        let rest = &content[start + section_header.len()..];
        let end = rest
            .find("\n[")
            .map(|i| start + section_header.len() + i)
            .unwrap_or(content.len());
        let new_content = format!(
            "{}{}\ntheme = \"{}\"\nprompt = \"{}\"\n{}",
            &content[..start],
            section_header,
            theme,
            prompt,
            &content[end..]
        );
        fs::write(&toml_path, new_content).unwrap();
    }

    let output = run_in(dir, &["apply"]);
    assert!(
        output.status.success(),
        "apply after appearance change failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Check that no template placeholders survive in seeded config files.
///
/// Matches whole tokens — a placeholder `AIBOX_THEME` does NOT match the
/// legitimate env-var name `AIBOX_THEME_CONFIRM_RESTART_TUIS`. Tokens are
/// considered identifiers delimited by characters outside `[A-Za-z0-9_]`.
fn assert_no_placeholders(dir: &std::path::Path) {
    let aibox_home = dir.join(".aibox-home");
    let placeholders = ["AIBOX_THEME", "AIBOX_VIM_COLORSCHEME", "AIBOX_VIM_BG"];

    let files_to_check = [
        ".vim/vimrc",
        ".config/tmux/tmux.conf",
        ".config/starship.toml",
    ];

    fn contains_token(haystack: &str, needle: &str) -> bool {
        for (start, _) in haystack.match_indices(needle) {
            let end = start + needle.len();
            let prev_ok = start == 0
                || !haystack.as_bytes()[start - 1].is_ascii_alphanumeric()
                    && haystack.as_bytes()[start - 1] != b'_';
            let next_ok = end == haystack.len()
                || !haystack.as_bytes()[end].is_ascii_alphanumeric()
                    && haystack.as_bytes()[end] != b'_';
            if prev_ok && next_ok {
                return true;
            }
        }
        false
    }

    for file in &files_to_check {
        let path = aibox_home.join(file);
        if path.exists() {
            let content = fs::read_to_string(&path).unwrap();
            for placeholder in &placeholders {
                assert!(
                    !contains_token(&content, placeholder),
                    "placeholder '{}' found in {}: {}",
                    placeholder,
                    file,
                    content
                        .lines()
                        .find(|l| contains_token(l, placeholder))
                        .unwrap_or("???")
                );
            }
        }
    }
}

// ─── Theme Tests ─────────────────────────────────────────────────────────────

/// Single-init integration smoke: `aibox init --theme <X>` writes the seeded
/// files at the expected paths. Per-theme content is checked exhaustively in
/// `themes::tests` unit tests — they run in ~10 ms instead of ~12 s.
#[test]
fn init_seeds_themed_files_at_expected_paths() {
    let dir = tempfile::tempdir().unwrap();
    init_with_appearance(dir.path(), "dracula", "default");
    let aibox_home = dir.path().join(".aibox-home");
    for rel in [
        ".vim/vimrc",
        ".vim/colors/aibox.vim",
        ".config/tmux/tmux.conf",
        ".config/tmux/aibox-powerkit-theme.sh",
        ".config/yazi/theme.toml",
        ".config/aibox/theme-env.sh",
        ".config/lnav/config.json",
        ".config/git/config",
        ".config/starship.toml",
    ] {
        let path = aibox_home.join(rel);
        assert!(path.exists(), "expected seeded file missing: {rel}");
        let body = fs::read_to_string(&path).unwrap();
        assert!(!body.is_empty(), "seeded {rel} should be non-empty");
    }
    // CLI flag plumbing: the dracula accent must reach the tmux config.
    let tmux = fs::read_to_string(aibox_home.join(".config/tmux/tmux.conf")).unwrap();
    assert!(
        tmux.contains("#BD93F9"),
        "--theme dracula must flow into tmux.conf:\n{tmux}"
    );
    assert_no_placeholders(dir.path());
}

#[test]
fn theme_change_auto_applies_untouched_runtime_files() {
    let dir = tempfile::tempdir().unwrap();
    init_with_appearance(dir.path(), "gruvbox-dark", "default");

    let aibox_home = dir.path().join(".aibox-home");

    let tmux_before = fs::read_to_string(aibox_home.join(".config/tmux/tmux.conf")).unwrap();
    assert!(tmux_before.contains("#D79921"));
    let aibox_vim_before = fs::read_to_string(aibox_home.join(".vim/colors/aibox.vim")).unwrap();
    assert!(aibox_vim_before.contains("#D79921"));
    let yazi_before = fs::read_to_string(aibox_home.join(".config/yazi/theme.toml")).unwrap();
    assert!(!yazi_before.is_empty(), "yazi theme should not be empty");

    change_appearance(dir.path(), "dracula", "default");

    // ChangedUpstreamOnly files are now auto-applied (the user hasn't
    // touched them, only the config changed), so the live files should
    // already reflect the new theme.
    let tmux_after = fs::read_to_string(aibox_home.join(".config/tmux/tmux.conf")).unwrap();
    assert!(
        tmux_after.contains("#BD93F9"),
        "tmux config should be auto-updated to the new theme"
    );
    let aibox_vim_after = fs::read_to_string(aibox_home.join(".vim/colors/aibox.vim")).unwrap();
    assert!(
        aibox_vim_after.contains("#BD93F9"),
        "aibox.vim should be regenerated with the dracula accent on theme change"
    );
    let yazi_after = fs::read_to_string(aibox_home.join(".config/yazi/theme.toml")).unwrap();
    assert_ne!(
        yazi_after, yazi_before,
        "yazi theme should be auto-updated to the new theme"
    );

    let pending_dir = dir.path().join("context/migrations/pending");
    let runtime_docs: Vec<_> = fs::read_dir(&pending_dir)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(|entry| entry.ok()))
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|name| name.starts_with("MIG-RUNTIME-"))
        })
        .collect();
    assert!(
        runtime_docs.is_empty(),
        "untouched theme runtime files should be auto-applied without a pending migration"
    );
}

// Per-theme content alignment (palette accent showing up in every themed
// artifact, no cross-theme palette drift, Yazi 26 schema compliance) is now
// covered by the `themes::tests::every_theme_aligns_tools_to_its_accent` and
// `every_theme_yazi_uses_current_schema` unit tests — they run in milliseconds
// against pure functions instead of ~7 `aibox init` subprocess invocations.

// ─── Keymap Tests ────────────────────────────────────────────────────────────

/// Verify that seeded yazi keymap includes the "e" binding for open-in-editor.
#[test]
fn yazi_keymap_includes_edit_in_pane_binding() {
    let dir = tempfile::tempdir().unwrap();
    init_with_appearance(dir.path(), "gruvbox-dark", "default");

    let keymap =
        fs::read_to_string(dir.path().join(".aibox-home/.config/yazi/keymap.toml")).unwrap();

    assert!(
        keymap.contains(r#"on = "e""#),
        "yazi keymap should contain 'e' keybinding for open-in-editor"
    );
    assert!(
        keymap.contains("open-in-editor"),
        "yazi keymap 'e' binding should invoke open-in-editor"
    );
}

// ─── Prompt Tests ────────────────────────────────────────────────────────────

// Per-preset content shape (ASCII-only for plain, chevrons + single-line for
// pastel/powerline-pastel, directory+git_branch for default) is covered by the
// `themes::tests::starship_presets_emit_their_expected_shape` unit test. The
// `--prompt` CLI alias resolution stays an integration test below since it
// exercises clap arg parsing, which is not pure-function territory.

#[test]
fn prompt_pastel_powerline_alias_is_still_accepted() {
    let dir = tempfile::tempdir().unwrap();
    init_with_appearance(dir.path(), "gruvbox-dark", "pastel-powerline");

    let content = fs::read_to_string(dir.path().join(".aibox-home/.config/starship.toml")).unwrap();

    assert!(content.contains("pastel powerline preset"));
    assert!(content.contains(""));
    assert!(
        !content.contains("$line_break"),
        "pastel-powerline alias should render as the one-line powerline-pastel prompt"
    );
}
