//! File preview tests — verify yazi previewer configuration is correctly seeded.
//!
//! Checks that after `aibox init`:
//!   - The yazi plugin files for SVG and EPS are present in .aibox-home
//!   - The yazi.toml contains the expected [plugin] prepend_previewers entries
//!   - preview-enhanced seeds rich markdown preview, PDF watch, and no-wrap pager hooks
//!
//! These are Tier 1 tests: no running container needed.

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

/// Initialize a project in a temp dir with default settings.
fn init_project(dir: &std::path::Path, name: &str) {
    let output = run_in(
        dir,
        &["init", name, "--base", "debian", "--context", "managed"],
    );
    assert!(
        output.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn enable_preview_enhanced(dir: &std::path::Path) {
    let output = run_in(
        dir,
        &["set", "addon", "preview-enhanced", "enabled", "--apply"],
    );
    assert!(
        output.status.success(),
        "set addon preview-enhanced enabled --apply failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn read_yazi_toml(dir: &std::path::Path) -> String {
    fs::read_to_string(dir.join(".aibox-home/.config/yazi/yazi.toml"))
        .unwrap_or_else(|e| panic!("failed to read yazi.toml: {}", e))
}

fn read_yazi_keymap(dir: &std::path::Path) -> String {
    fs::read_to_string(dir.join(".aibox-home/.config/yazi/keymap.toml"))
        .unwrap_or_else(|e| panic!("failed to read keymap.toml: {}", e))
}

// ─── Plugin File Presence Tests ───────────────────────────────────────────────

/// After `aibox init`, svg.yazi/init.lua must be seeded into .aibox-home.
#[test]
fn svg_yazi_plugin_seeded() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "preview-svg");

    let plugin_path = dir
        .path()
        .join(".aibox-home/.config/yazi/plugins/svg.yazi/init.lua");

    assert!(
        plugin_path.exists(),
        "svg.yazi/init.lua should be seeded at {}",
        plugin_path.display()
    );
}

/// After `aibox init`, eps.yazi/init.lua must be seeded into .aibox-home.
#[test]
fn eps_yazi_plugin_seeded() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "preview-eps");

    let plugin_path = dir
        .path()
        .join(".aibox-home/.config/yazi/plugins/eps.yazi/init.lua");

    assert!(
        plugin_path.exists(),
        "eps.yazi/init.lua should be seeded at {}",
        plugin_path.display()
    );
}

/// The svg.yazi plugin must reference resvg for SVG → PNG conversion.
#[test]
fn svg_yazi_plugin_uses_resvg() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "preview-svg-content");

    let plugin_path = dir
        .path()
        .join(".aibox-home/.config/yazi/plugins/svg.yazi/init.lua");
    let content = fs::read_to_string(&plugin_path)
        .unwrap_or_else(|e| panic!("failed to read svg.yazi/init.lua: {}", e));

    assert!(
        content.contains("resvg"),
        "svg.yazi/init.lua should invoke resvg for SVG conversion"
    );
}

/// The eps.yazi plugin must reference ghostscript (gs) for EPS → PNG conversion.
#[test]
fn eps_yazi_plugin_uses_ghostscript() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "preview-eps-content");

    let plugin_path = dir
        .path()
        .join(".aibox-home/.config/yazi/plugins/eps.yazi/init.lua");
    let content = fs::read_to_string(&plugin_path)
        .unwrap_or_else(|e| panic!("failed to read eps.yazi/init.lua: {}", e));

    assert!(
        content.contains("\"gs\"") || content.contains("'gs'"),
        "eps.yazi/init.lua should invoke gs (ghostscript) for EPS conversion"
    );
}

/// preview-enhanced seeds the rich-preview plugin used for rendered Markdown.
#[test]
fn rich_preview_plugin_seeded_with_preview_enhanced() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "preview-rich-plugin");
    enable_preview_enhanced(dir.path());

    let plugin_path = dir
        .path()
        .join(".aibox-home/.config/yazi/plugins/rich-preview.yazi/main.lua");

    assert!(
        plugin_path.exists(),
        "rich-preview.yazi/main.lua should be seeded at {}",
        plugin_path.display()
    );

    let plugin = fs::read_to_string(&plugin_path)
        .unwrap_or_else(|e| panic!("failed to read rich-preview plugin: {e}"));
    assert!(
        plugin.contains("h1 = (h1 * 33 + byte)")
            && plugin.contains("h2 = (h2 * 65599 + byte)")
            && !plugin.contains(":sub(1, 32)"),
        "rich-preview must distinguish files that share a long directory prefix"
    );
    assert!(
        plugin.contains("def split_front_matter(value):")
            && plugin.contains("Syntax(front_matter, lexer")
            && plugin.contains("console.print(Markdown(body))"),
        "rich-preview must render Hugo front matter verbatim and Markdown separately"
    );

    let yazi_toml = read_yazi_toml(dir.path());
    assert!(
        yazi_toml.contains("rich-preview"),
        "preview-enhanced should register rich-preview entries for rendered previews"
    );
}

// ─── yazi.toml [plugin] Section Tests ────────────────────────────────────────

/// yazi.toml must have a [plugin] section with prepend_previewers after init.
#[test]
fn yazi_toml_has_plugin_section() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "preview-plugin-section");

    let yazi_toml = fs::read_to_string(dir.path().join(".aibox-home/.config/yazi/yazi.toml"))
        .unwrap_or_else(|e| panic!("failed to read yazi.toml: {}", e));

    assert!(
        yazi_toml.contains("[plugin]"),
        "yazi.toml should contain a [plugin] section"
    );
    assert!(
        yazi_toml.contains("prepend_previewers"),
        "yazi.toml [plugin] section should define prepend_previewers"
    );
}

/// yazi.toml must route *.svg files through the svg previewer plugin.
#[test]
fn yazi_toml_svg_previewer_entry() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "preview-svg-entry");

    let yazi_toml = fs::read_to_string(dir.path().join(".aibox-home/.config/yazi/yazi.toml"))
        .unwrap_or_else(|e| panic!("failed to read yazi.toml: {}", e));

    assert!(
        yazi_toml.contains("\"*.svg\"") || yazi_toml.contains("'*.svg'"),
        "yazi.toml should contain a prepend_previewers entry matching *.svg"
    );
    // The svg entry must invoke the "svg" plugin run target
    assert!(
        yazi_toml.contains(r#"run = "svg""#),
        "yazi.toml svg entry should set run = \"svg\""
    );
}

/// yazi.toml must route *.eps files through the eps previewer plugin.
#[test]
fn yazi_toml_eps_previewer_entry() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "preview-eps-entry");

    let yazi_toml = fs::read_to_string(dir.path().join(".aibox-home/.config/yazi/yazi.toml"))
        .unwrap_or_else(|e| panic!("failed to read yazi.toml: {}", e));

    assert!(
        yazi_toml.contains("\"*.eps\"") || yazi_toml.contains("'*.eps'"),
        "yazi.toml should contain a prepend_previewers entry matching *.eps"
    );
    // The eps entry must invoke the "eps" plugin run target
    assert!(
        yazi_toml.contains(r#"run = "eps""#),
        "yazi.toml eps entry should set run = \"eps\""
    );
}

/// preview-enhanced routes Markdown files through the rich previewer.
#[test]
fn yazi_toml_markdown_rich_preview_entries() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "preview-rich-entries");
    enable_preview_enhanced(dir.path());

    let yazi_toml = read_yazi_toml(dir.path());

    assert!(
        yazi_toml.contains("\"*.md\"") || yazi_toml.contains("'*.md'"),
        "yazi.toml should contain a prepend_previewers entry matching *.md"
    );
    assert!(
        yazi_toml.contains("\"*.markdown\"") || yazi_toml.contains("'*.markdown'"),
        "yazi.toml should contain a prepend_previewers entry matching *.markdown"
    );
    assert!(
        yazi_toml.contains(r#"run = "rich-preview""#),
        "yazi.toml markdown entries should set run = \"rich-preview\""
    );
}

/// The Yazi keymap should expose a no-wrap pager for horizontally scrolling previews.
#[test]
fn yazi_keymap_has_horizontal_scroll_pager() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "preview-pager");

    let keymap_toml = read_yazi_keymap(dir.path());

    assert!(
        keymap_toml.contains("less -R -S"),
        "keymap.toml should expose a preview pager command using less -R -S"
    );
}

/// Yazi should expose whole-file host-copy and a selectable read-only preview.
#[test]
fn yazi_keymap_has_content_copy_and_selectable_preview() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "preview-copy-select");

    let keymap_toml = read_yazi_keymap(dir.path());

    assert!(
        keymap_toml.contains(r#"{ on = [ "c", "c" ], run = "shell 'aibox-copy < \"$1\"'"#),
        "keymap.toml should copy the hovered file contents through aibox-copy"
    );
    assert!(
        keymap_toml.contains(r#"{ on = [ "w", "v" ], run = "shell 'vim -R \"$1\"' --block"#),
        "keymap.toml should expose a read-only Vim surface for selecting preview text"
    );
}

/// The image fallback must match the generated Yazi clipboard/selection bindings.
#[test]
fn image_yazi_keymap_matches_content_copy_and_selectable_preview() {
    let image_keymap = fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../images/base-debian/config/yazi/keymap.toml"),
    )
    .expect("read image Yazi keymap");

    assert!(image_keymap.contains(r#"aibox-copy < \"$1\""#));
    assert!(image_keymap.contains(r#"vim -R \"$1\""#));
}

/// The Yazi keymap should expose the PDF live-watch helper for selected PDFs.
#[test]
fn yazi_keymap_has_pdf_watch_binding() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "preview-pdf-watch");

    let keymap_toml = read_yazi_keymap(dir.path());

    assert!(
        keymap_toml.contains("pdf-watch"),
        "keymap.toml should expose a PDF live-watch binding invoking pdf-watch"
    );
}

/// The Yazi keymap should expose a full-pane preview binding backed by a helper.
#[test]
fn yazi_keymap_has_full_pane_preview_binding() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "preview-full-pane");

    let keymap_toml = read_yazi_keymap(dir.path());

    assert!(
        keymap_toml.contains("aibox-preview"),
        "keymap.toml should expose the full-pane preview helper"
    );
}

/// The full-pane preview helper should be available in the generated runtime home.
#[test]
fn aibox_preview_helper_seeded() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "preview-helper");

    let helper_path = dir.path().join(".aibox-home/.local/bin/aibox-preview");

    assert!(
        helper_path.exists(),
        "aibox-preview helper should be seeded at {}",
        helper_path.display()
    );

    let content = fs::read_to_string(&helper_path)
        .unwrap_or_else(|e| panic!("failed to read aibox-preview helper: {}", e));
    assert!(
        content.contains("glow -s")
            && content.contains("bat --paging=never")
            && content.contains("--mouse")
            && content.contains("pdf-watch"),
        "aibox-preview helper should dispatch Markdown, text/code, and PDF previews through a mouse-aware pager"
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mode = fs::metadata(&helper_path)
            .unwrap_or_else(|e| panic!("failed to stat aibox-preview helper: {}", e))
            .permissions()
            .mode();
        assert_ne!(mode & 0o111, 0, "aibox-preview helper should be executable");
    }
}

/// The PDF watch helper should be available in the generated runtime home.
#[test]
fn pdf_watch_helper_seeded() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "preview-pdf-watch-helper");

    let helper_path = dir.path().join(".aibox-home/.local/bin/pdf-watch");

    assert!(
        helper_path.exists(),
        "pdf-watch helper should be seeded at {}",
        helper_path.display()
    );

    let content = fs::read_to_string(&helper_path)
        .unwrap_or_else(|e| panic!("failed to read pdf-watch helper: {}", e));
    assert!(
        content.contains("mutool draw") && content.contains("entr") && content.contains("timg"),
        "pdf-watch helper should wrap mutool, entr, and timg"
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mode = fs::metadata(&helper_path)
            .unwrap_or_else(|e| panic!("failed to stat pdf-watch helper: {}", e))
            .permissions()
            .mode();
        assert_ne!(mode & 0o111, 0, "pdf-watch helper should be executable");
    }
}

/// The runtime image must include the server used by the generated LaTeX sidecar.
#[test]
fn latex_preview_sidecar_helper_is_baked_into_runtime_image() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../images/base-debian");
    let helper_path = root.join("config/bin/aibox-latex-preview.py");
    let dockerfile = fs::read_to_string(root.join("Dockerfile"))
        .expect("failed to read base runtime Dockerfile");
    let helper = fs::read_to_string(&helper_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", helper_path.display()));

    assert!(
        dockerfile.contains(
            "COPY --chmod=755 config/bin/aibox-latex-preview.py /usr/local/bin/aibox-latex-preview"
        ),
        "runtime Dockerfile must install the LaTeX preview helper"
    );
    assert!(helper.contains("ThreadingHTTPServer"));
    assert!(helper.contains("EMBEDPDF_VERSION = \"2.14.3\""));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mode = fs::metadata(&helper_path)
            .unwrap_or_else(|error| panic!("failed to stat {}: {error}", helper_path.display()))
            .permissions()
            .mode();
        assert_ne!(mode & 0o111, 0, "source helper should be executable");
    }
}

/// SVG and EPS entries must appear before the built-in image/pdf entries
/// (prepend_previewers semantics: first match wins).
#[test]
fn yazi_toml_svg_and_eps_precede_builtin_previewers() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "preview-order");

    let yazi_toml = fs::read_to_string(dir.path().join(".aibox-home/.config/yazi/yazi.toml"))
        .unwrap_or_else(|e| panic!("failed to read yazi.toml: {}", e));

    let svg_pos = yazi_toml
        .find("\"*.svg\"")
        .or_else(|| yazi_toml.find("'*.svg'"))
        .expect("*.svg entry not found in yazi.toml");
    let eps_pos = yazi_toml
        .find("\"*.eps\"")
        .or_else(|| yazi_toml.find("'*.eps'"))
        .expect("*.eps entry not found in yazi.toml");
    let jpg_pos = yazi_toml
        .find("\"*.jpg\"")
        .or_else(|| yazi_toml.find("'*.jpg'"))
        .expect("*.jpg entry not found in yazi.toml");

    assert!(
        svg_pos < jpg_pos,
        "*.svg entry (pos {}) should appear before *.jpg entry (pos {}) in prepend_previewers",
        svg_pos,
        jpg_pos
    );
    assert!(
        eps_pos < jpg_pos,
        "*.eps entry (pos {}) should appear before *.jpg entry (pos {}) in prepend_previewers",
        eps_pos,
        jpg_pos
    );
}

// ─── Fixture File Sanity Tests ────────────────────────────────────────────────
//
// These tests verify the sample files in tests/e2e/fixtures/ are readable
// and have the expected content markers. They serve as a baseline to confirm
// the fixture files copied from assets/placeholder-package/ are intact.

/// The sample SVG fixture starts with an <svg> or <?xml ...> declaration.
#[test]
fn fixture_sample_svg_is_valid_xml() {
    let fixture =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/e2e/fixtures/sample.svg");

    assert!(
        fixture.exists(),
        "sample.svg fixture should exist at {}",
        fixture.display()
    );

    let content = fs::read_to_string(&fixture)
        .unwrap_or_else(|e| panic!("failed to read sample.svg fixture: {}", e));

    assert!(
        content.contains("<svg") || content.contains("<?xml"),
        "sample.svg should contain SVG/XML markup"
    );
}

/// The sample EPS fixture starts with the standard %!PS-Adobe EPS header.
#[test]
fn fixture_sample_eps_has_eps_header() {
    let fixture =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/e2e/fixtures/sample.eps");

    assert!(
        fixture.exists(),
        "sample.eps fixture should exist at {}",
        fixture.display()
    );

    let content = fs::read_to_string(&fixture)
        .unwrap_or_else(|e| panic!("failed to read sample.eps fixture: {}", e));

    assert!(
        content.starts_with("%!PS-Adobe") || content.contains("%%BoundingBox"),
        "sample.eps should start with a valid PostScript/EPS header"
    );
}
