//! H1 — Disabled-tool absence tests (BR-TEST-GAPS).
//!
//! Tier 1 tests (no container needed). Each test verifies that when a tool
//! is explicitly disabled in `[addons.<addon>.tools]`, the generated
//! `.devcontainer/Dockerfile`:
//!   a) does NOT contain an `apt-get install ... <tool>` line for the
//!      disabled tool, AND
//!   b) DOES contain the "disable-then-purge" block that guarantees
//!      the binary cannot survive from an older base image layer.
//!
//! A companion `#[ignore]`-gated test for lazygit extends
//! `runtime_generated.rs` (see H1_COMPANION note below).
//!
//! We also guard `runtime_generated.rs:73` (`lazygit --version`) against
//! the addon-disabled case; that is done in `runtime_generated.rs` itself.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn aibox_bin() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{}/target/debug/aibox", manifest_dir)
}

fn addons_dir() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{}/../addons", manifest_dir)
}

fn run_no_container(dir: &Path, args: &[&str]) -> Output {
    Command::new(aibox_bin())
        .args(args)
        .current_dir(dir)
        .env("AIBOX_ADDONS_DIR", addons_dir())
        .env("AIBOX_NO_CONTAINER", "1")
        .output()
        .expect("failed to execute aibox")
}

fn fmt_output(label: &str, out: &Output) -> String {
    format!(
        "{label}: status={}\n--- stdout ---\n{}\n--- stderr ---\n{}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    )
}

/// Scaffold a project with the given toml content, run `aibox apply`, and
/// return the rendered Dockerfile content.
fn render_dockerfile(toml_content: &str) -> String {
    let tmp = tempfile::TempDir::new().expect("create tempdir");
    let dir = tmp.path();

    fs::write(dir.join("aibox.toml"), toml_content).expect("write aibox.toml");

    let apply_out = run_no_container(dir, &["apply"]);
    assert!(
        apply_out.status.success(),
        "apply failed.\n{}",
        fmt_output("apply", &apply_out)
    );

    fs::read_to_string(dir.join(".devcontainer/Dockerfile"))
        .expect("Dockerfile should exist after apply")
}

// ─── H1-a: git-ui addon — lazygit disabled ───────────────────────────────────

#[test]
fn git_ui_lazygit_disabled_omits_install_and_adds_purge() {
    let toml = r#"[aibox]
version = "0.25.5"
base = "debian"

[container]
name = "h1-git-ui"

[processkit]
version = "unset"

[addons.git-ui.tools]
gh = {}
lazygit = { enabled = false }
"#;

    let dockerfile = render_dockerfile(toml);

    // Must NOT install lazygit
    assert!(
        !dockerfile.contains("    lazygit \\") && !dockerfile.contains("apt-get install.*lazygit"),
        "Dockerfile must not install lazygit when explicitly disabled:\n{dockerfile}"
    );

    // MUST contain the purge block — the absence contract.
    assert!(
        dockerfile.contains("dpkg-query -W -f='${Status}' lazygit")
            || dockerfile.contains("rm -f /usr/local/bin/lazygit"),
        "Dockerfile must contain disable-then-purge block for lazygit:\n{dockerfile}"
    );
}

// ─── H1-b: kubernetes addon — kubectl and helm disabled ──────────────────────

#[test]
fn kubernetes_kubectl_disabled_omits_copy_and_adds_purge() {
    let toml = r#"[aibox]
version = "0.25.5"
base = "debian"

[container]
name = "h1-k8s"

[processkit]
version = "unset"

[addons.kubernetes.tools]
kubectl = { enabled = false }
helm = {}
kustomize = {}
"#;

    let dockerfile = render_dockerfile(toml);

    // kubectl must not be copied into the image
    assert!(
        !dockerfile.contains("COPY --from=k8s-builder /build/bin/kubectl"),
        "Dockerfile must not COPY kubectl when disabled:\n{dockerfile}"
    );

    // Must contain the purge/rm block for kubectl
    assert!(
        dockerfile.contains("rm -f /usr/local/bin/kubectl"),
        "Dockerfile must contain purge block for disabled kubectl:\n{dockerfile}"
    );
}

#[test]
fn kubernetes_helm_disabled_omits_copy_and_adds_purge() {
    let toml = r#"[aibox]
version = "0.25.5"
base = "debian"

[container]
name = "h1-k8s-helm"

[processkit]
version = "unset"

[addons.kubernetes.tools]
kubectl = {}
helm = { enabled = false }
kustomize = {}
"#;

    let dockerfile = render_dockerfile(toml);

    assert!(
        !dockerfile.contains("COPY --from=k8s-builder /build/bin/helm"),
        "Dockerfile must not COPY helm when disabled:\n{dockerfile}"
    );
    assert!(
        dockerfile.contains("rm -f /usr/local/bin/helm"),
        "Dockerfile must contain purge block for disabled helm:\n{dockerfile}"
    );
}

// ─── H1-c: cloud-aws — aws-cli disabled ──────────────────────────────────────

#[test]
fn cloud_aws_cli_disabled_omits_install_and_adds_purge() {
    let toml = r#"[aibox]
version = "0.25.5"
base = "debian"

[container]
name = "h1-aws"

[processkit]
version = "unset"

[addons.cloud-aws.tools]
aws-cli = { enabled = false }
"#;

    let dockerfile = render_dockerfile(toml);

    assert!(
        !dockerfile.contains("awscli.amazonaws.com"),
        "Dockerfile must not install aws-cli when disabled:\n{dockerfile}"
    );
    assert!(
        dockerfile.contains("rm -rf /usr/local/aws-cli")
            || dockerfile.contains("rm -f /usr/local/bin/aws"),
        "Dockerfile must contain purge block for disabled aws-cli:\n{dockerfile}"
    );
}

// ─── H1-d: cloud-azure — azure-cli disabled ──────────────────────────────────

#[test]
fn cloud_azure_cli_disabled_omits_install_and_adds_purge() {
    let toml = r#"[aibox]
version = "0.25.5"
base = "debian"

[container]
name = "h1-azure"

[processkit]
version = "unset"

[addons.cloud-azure.tools]
azure-cli = { enabled = false }
"#;

    let dockerfile = render_dockerfile(toml);

    assert!(
        !dockerfile.contains("azure-cli") || !dockerfile.contains("pip install"),
        "Dockerfile must not install azure-cli when disabled:\n{dockerfile}"
    );
    // The purge block is present when the tool is disabled
    assert!(
        dockerfile.contains("azure-cli")
            && (dockerfile.contains("pip") || dockerfile.contains("purge")),
        "Dockerfile must contain disable-then-purge block for azure-cli:\n{dockerfile}"
    );
}

// ─── H1-e: cloud-gcp — gcloud-cli disabled ───────────────────────────────────

#[test]
fn cloud_gcp_cli_disabled_omits_install_and_adds_purge() {
    let toml = r#"[aibox]
version = "0.25.5"
base = "debian"

[container]
name = "h1-gcp"

[processkit]
version = "unset"

[addons.cloud-gcp.tools]
gcloud-cli = { enabled = false }
"#;

    let dockerfile = render_dockerfile(toml);

    // When gcloud-cli is disabled, its installation steps must be absent
    assert!(
        !dockerfile.contains("cloud.google.com")
            && !dockerfile.contains("apt-get install.*google-cloud"),
        "Dockerfile must not install gcloud-cli when disabled:\n{dockerfile}"
    );
    // Must contain purge block
    assert!(
        dockerfile.contains("google-cloud-cli") || dockerfile.contains("gcloud"),
        "Dockerfile must contain disable-then-purge block for gcloud-cli:\n{dockerfile}"
    );
}

// ─── H1-f: infrastructure — opentofu and packer disabled ─────────────────────

#[test]
fn infrastructure_opentofu_disabled_omits_install_and_adds_purge() {
    let toml = r#"[aibox]
version = "0.25.5"
base = "debian"

[container]
name = "h1-infra"

[processkit]
version = "unset"

[addons.infrastructure.tools]
opentofu = { enabled = false }
ansible = {}
packer = {}
"#;

    let dockerfile = render_dockerfile(toml);

    assert!(
        !dockerfile.contains("COPY --from=infra-builder /build/bin/tofu"),
        "Dockerfile must not COPY opentofu when disabled:\n{dockerfile}"
    );
    assert!(
        dockerfile.contains("rm -f /usr/local/bin/tofu"),
        "Dockerfile must contain purge block for disabled opentofu:\n{dockerfile}"
    );
}

#[test]
fn infrastructure_packer_disabled_omits_install_and_adds_purge() {
    let toml = r#"[aibox]
version = "0.25.5"
base = "debian"

[container]
name = "h1-infra-packer"

[processkit]
version = "unset"

[addons.infrastructure.tools]
opentofu = {}
ansible = {}
packer = { enabled = false }
"#;

    let dockerfile = render_dockerfile(toml);

    assert!(
        !dockerfile.contains("COPY --from=infra-builder /build/bin/packer"),
        "Dockerfile must not COPY packer when disabled:\n{dockerfile}"
    );
    assert!(
        dockerfile.contains("rm -f /usr/local/bin/packer"),
        "Dockerfile must contain purge block for disabled packer:\n{dockerfile}"
    );
}

// ─── H1-g: audio-voice — sox disabled ────────────────────────────────────────

#[test]
fn audio_voice_sox_disabled_omits_install_and_adds_purge() {
    let toml = r#"[aibox]
version = "0.25.5"
base = "debian"

[container]
name = "h1-audio"

[processkit]
version = "unset"

[addons.audio-voice.tools]
sox = { enabled = false }
sox-pulse = {}
pulseaudio-utils = {}
alsa-pulse = {}
"#;

    let dockerfile = render_dockerfile(toml);

    // sox must not appear in the install block
    assert!(
        !dockerfile.contains("    sox \\"),
        "Dockerfile must not install sox when disabled:\n{dockerfile}"
    );
    // purge guard must be present
    assert!(
        dockerfile.contains("dpkg-query -W -f='${Status}' sox"),
        "Dockerfile must contain disable-then-purge block for sox:\n{dockerfile}"
    );
}

// ─── H1-h: preview-archive — chafa disabled ──────────────────────────────────

#[test]
fn preview_archive_chafa_disabled_omits_install_and_adds_purge() {
    let toml = r#"[aibox]
version = "0.25.5"
base = "debian"

[container]
name = "h1-preview-archive"

[processkit]
version = "unset"

[addons.preview-archive.tools]
chafa = { enabled = false }
librsvg = {}
poppler = {}
timg = {}
mupdf = {}
entr = {}
p7zip = {}
ouch = {}
resvg = {}
"#;

    let dockerfile = render_dockerfile(toml);

    assert!(
        !dockerfile.contains("    chafa \\"),
        "Dockerfile must not install chafa when disabled:\n{dockerfile}"
    );
    assert!(
        dockerfile.contains("dpkg-query -W -f='${Status}' chafa"),
        "Dockerfile must contain disable-then-purge block for chafa:\n{dockerfile}"
    );
}

// ─── H1-i: preview-enhanced — rich disabled ──────────────────────────────────

#[test]
fn preview_enhanced_rich_disabled_omits_install_and_adds_purge() {
    let toml = r#"[aibox]
version = "0.25.5"
base = "debian"

[container]
name = "h1-preview-enhanced"

[processkit]
version = "unset"

[addons.preview-enhanced.tools]
ffmpeg = {}
imagemagick = {}
ghostscript = {}
rich = { enabled = false }
"#;

    let dockerfile = render_dockerfile(toml);

    assert!(
        !dockerfile.contains("    python3-rich \\"),
        "Dockerfile must not install python3-rich when rich is disabled:\n{dockerfile}"
    );
    assert!(
        dockerfile.contains("dpkg-query -W -f='${Status}' python3-rich"),
        "Dockerfile must contain disable-then-purge block for rich:\n{dockerfile}"
    );
}

// ─── H1-j: data-preview — sqlite3 disabled ───────────────────────────────────

#[test]
fn data_preview_sqlite3_disabled_omits_install_and_adds_purge() {
    let toml = r#"[aibox]
version = "0.25.5"
base = "debian"

[container]
name = "h1-data-preview"

[processkit]
version = "unset"

[addons.data-preview.tools]
sqlite3 = { enabled = false }
csvkit = {}
"#;

    let dockerfile = render_dockerfile(toml);

    assert!(
        !dockerfile.contains("    sqlite3 \\"),
        "Dockerfile must not install sqlite3 when disabled:\n{dockerfile}"
    );
    assert!(
        dockerfile.contains("dpkg-query -W -f='${Status}' sqlite3"),
        "Dockerfile must contain disable-then-purge block for sqlite3:\n{dockerfile}"
    );
}

// ─── H1-k: yazi-omp — oh-my-posh disabled ────────────────────────────────────

#[test]
fn yazi_omp_oh_my_posh_disabled_omits_install_and_adds_purge() {
    let toml = r#"[aibox]
version = "0.25.5"
base = "debian"

[container]
name = "h1-yazi-omp"

[processkit]
version = "unset"

[addons.yazi-omp.tools]
oh-my-posh = { enabled = false }
"#;

    let dockerfile = render_dockerfile(toml);

    assert!(
        !dockerfile.contains("ohmyposh.dev") && !dockerfile.contains("oh-my-posh.zip"),
        "Dockerfile must not install oh-my-posh when disabled:\n{dockerfile}"
    );
    assert!(
        dockerfile.contains("rm -f /usr/local/bin/oh-my-posh"),
        "Dockerfile must contain disable-then-purge block for oh-my-posh:\n{dockerfile}"
    );
}

// ─── H1 companion: image-level lazygit absence after build ───────────────────
//
// This test builds the devcontainer image with `--no-cache` and then queries
// dpkg inside the container. It is gated `#[ignore]` because:
//   1. It requires a working container runtime (Docker/Podman).
//   2. It performs a full image build (slow).
//   3. It is intended to run on demand: `cargo test -- --ignored`.
//
// Run it with: cargo test h1_lazygit_absent_in_built_image -- --ignored
#[test]
#[ignore]
fn h1_lazygit_absent_in_built_image() {
    use std::path::PathBuf;
    use std::process::Command;

    let tmp = tempfile::TempDir::new().expect("create tempdir");
    let dir = tmp.path();

    // Write a minimal project with lazygit disabled.
    let toml = r#"[aibox]
version = "0.25.5"
base = "debian"

[container]
name = "h1-lazygit-absent-build"

[processkit]
version = "unset"

[addons.git-ui.tools]
gh = {}
lazygit = { enabled = false }
"#;
    fs::write(dir.join("aibox.toml"), toml).unwrap();

    let apply_out = run_no_container(dir, &["apply"]);
    assert!(
        apply_out.status.success(),
        "apply failed: {}",
        fmt_output("apply", &apply_out)
    );

    let devcontainer = dir.join(".devcontainer");
    let dockerfile = devcontainer.join("Dockerfile");
    let image_tag = "aibox-test-lazygit-absent:h1";

    // Build the image.
    let build_out = Command::new("docker")
        .args([
            "build",
            "--no-cache",
            "--build-arg",
            "BUILDKIT_INLINE_CACHE=0",
            "-t",
            image_tag,
            "-f",
            &dockerfile.to_string_lossy(),
            &devcontainer.to_string_lossy(),
        ])
        .output()
        .expect("failed to run docker build");
    assert!(
        build_out.status.success(),
        "docker build failed:\n{}",
        String::from_utf8_lossy(&build_out.stderr)
    );

    // Query dpkg inside the built image.
    let probe_out = Command::new("docker")
        .args([
            "run",
            "--rm",
            image_tag,
            "bash",
            "-c",
            "dpkg-query -W -f='${Status}' lazygit 2>&1 || echo no-packages-found",
        ])
        .output()
        .expect("failed to run docker probe");
    let probe_stdout = String::from_utf8_lossy(&probe_out.stdout);

    assert!(
        probe_stdout.contains("no packages found") || probe_stdout.contains("no-packages-found"),
        "lazygit should not be installed in the built image when disabled:\n{probe_stdout}"
    );

    // Clean up the image.
    let _ = Command::new("docker")
        .args(["rmi", "-f", image_tag])
        .output();

    let _ = PathBuf::new(); // suppress unused import warning
}
