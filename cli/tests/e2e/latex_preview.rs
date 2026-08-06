//! Full LaTeX build-watch and preview-sidecar lifecycle on the SSH companion.

use serial_test::serial;

use super::runner::E2eRunner;

const PREVIEW_PORT: u16 = 18765;

#[test]
#[serial(companion_runtime)]
#[ignore = "resource-heavy LaTeX image build; run through scripts/run-e2e-shards.sh latex or all"]
#[ntest::timeout(1_800_000)]
fn latex_watcher_builds_and_preview_sidecar_serves_updated_pdf() {
    let runner = E2eRunner::new();
    let test = "latex-watch-preview";
    let workspace = format!("/workspaces/{test}");
    let runtime = runner.runtime_bin();
    runner.cleanup(test);

    let init = runner.aibox(
        test,
        &[
            "init",
            test,
            "--base",
            "debian",
            "--context",
            "managed",
            "--processkit-version",
            "unset",
            "--addon",
            "latex",
        ],
    );
    assert_output_ok("init LaTeX project", &init);

    let Some(published_version) = runner.latest_published_image_version(test) else {
        eprintln!("skipping LaTeX lifecycle: no pullable published Debian runtime image");
        runner.cleanup(test);
        return;
    };

    let configure = runner.exec(&format!(
        r#"cd {workspace} && \
sed -i 's/^release_version = .*/release_version = "{published_version}"/' aibox.toml && \
sed -i '/^\[latex.preview\]/i [[latex.documents]]\nname = "overview"\nsource = "docs/overview.tex"\noutput_dir = ".latex-cache/overview"\n' aibox.toml && \
sed -i '/^\[latex.preview\]/,/^allow_public =/ {{ s/^enabled = false/enabled = true/; s/^port = .*/port = {PREVIEW_PORT}  # E2E host port/; }}' aibox.toml"#
    ));
    assert_output_ok("configure LaTeX document and preview", &configure);

    runner.write_file(
        test,
        "docs/overview.tex",
        &latex_document("WATCHREVISIONONE"),
    );

    let apply = runner.aibox(test, &["apply", "--no-container"]);
    assert_output_ok("generate LaTeX project", &apply);

    // The derived image normally inherits this helper from the published base
    // image. Overlay the checkout's helper so this pre-release test exercises
    // the implementation under test rather than the previous release.
    let overlay = runner.exec(&format!(
        "cp /opt/aibox/aibox-latex-preview.py {workspace}/.devcontainer/ && \
         printf '\nCOPY --chmod=755 aibox-latex-preview.py /usr/local/bin/aibox-latex-preview\n' >> \
           {workspace}/.devcontainer/Dockerfile"
    ));
    assert_output_ok("overlay preview helper into derived image", &overlay);

    let compose_file = format!("{workspace}/.devcontainer/docker-compose.yml");
    let build = runner.exec(&format!(
        "cd {workspace} && {runtime} compose -f {compose_file} build {test}"
    ));
    assert_output_ok("build derived LaTeX image", &build);

    let up = runner.exec(&format!(
        "cd {workspace} && {runtime} compose -f {compose_file} up -d \
         {test} {test}-latex-preview"
    ));
    assert_output_ok("start development container and preview sidecar", &up);

    let ready = runner.exec(&format!(
        "for i in $(seq 1 60); do \
           {runtime} exec {test} sh -c 'command -v latexmk >/dev/null && command -v aibox-latex-watch >/dev/null' \
             >/dev/null 2>&1 && \
           curl -fsS http://127.0.0.1:{PREVIEW_PORT}/health >/dev/null 2>&1 && exit 0; \
           sleep 2; \
         done; \
         {runtime} compose -f {compose_file} ps; \
         {runtime} logs {test}-latex-preview; \
         exit 1"
    ));
    assert_output_ok("wait for LaTeX services", &ready);

    let watch = runner.exec(&format!(
        "{runtime} exec -d {test} sh -c \
         'aibox-latex-watch overview >/tmp/aibox-latex-watch.log 2>&1'"
    ));
    assert_output_ok("start in-container LaTeX watcher", &watch);

    wait_for_pdf_text(&runner, &runtime, test, "WATCHREVISIONONE");

    runner.write_file(
        test,
        "docs/overview.tex",
        &latex_document("WATCHREVISIONTWO"),
    );
    wait_for_pdf_text(&runner, &runtime, test, "WATCHREVISIONTWO");

    let served_pdf = format!("/tmp/{test}-served.pdf");
    let served = runner.exec(&format!(
        "curl -fsS http://127.0.0.1:{PREVIEW_PORT}/documents/overview/document.pdf \
           -o {served_pdf} && \
         test \"$(head -c 5 {served_pdf})\" = '%PDF-' && \
         cmp {served_pdf} {workspace}/.latex-cache/overview/overview.pdf && \
         {runtime} exec {test} sh -c \
           'pgrep -f \"latexmk.*overview.tex\" >/dev/null'"
    ));
    assert_output_ok("verify watched PDF is served byte-for-byte", &served);

    let down = runner.exec(&format!(
        "cd {workspace} && {runtime} compose -f {compose_file} down -v"
    ));
    assert_output_ok("stop LaTeX project", &down);
    runner.cleanup(test);
}

fn latex_document(marker: &str) -> String {
    format!("\\documentclass{{article}}\n\\begin{{document}}\n{marker}\n\\end{{document}}\n")
}

fn wait_for_pdf_text(runner: &E2eRunner, runtime: &str, test: &str, marker: &str) {
    let output = runner.exec(&format!(
        "for i in $(seq 1 90); do \
           {runtime} exec {test} sh -c \
             'test -s /workspace/.latex-cache/overview/overview.pdf && \
              pdftotext /workspace/.latex-cache/overview/overview.pdf - 2>/dev/null | grep -q {marker}' \
             >/dev/null 2>&1 && exit 0; \
           sleep 2; \
         done; \
         {runtime} exec {test} sh -c \
           'cat /tmp/aibox-latex-watch.log 2>/dev/null || true; \
            tail -80 /workspace/.latex-cache/overview/overview.log 2>/dev/null || true'; \
         exit 1"
    ));
    assert_output_ok(&format!("wait for PDF marker {marker}"), &output);
}

fn assert_output_ok(step: &str, output: &std::process::Output) {
    assert!(
        output.status.success(),
        "{step} failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
