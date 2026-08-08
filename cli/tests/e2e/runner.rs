//! E2E test runner — SSH harness for the companion container.
//!
//! Connects to the `aibox-e2e-testrunner` companion container via SSH and executes
//! aibox commands in isolated workspace directories.
//!
//! The main devcontainer does not need a local docker or podman binary to talk
//! to the companion. Reachability is SSH/SCP-based; docker/podman are only used
//! as a best-effort convenience when starting the default companion locally.
//!
//! The aibox binary and addon definitions are deployed to the companion
//! via SCP — no shared volumes. This makes the companion a realistic
//! simulation of a user's host machine.
//!
//! The companion runtime is intentionally runtime-neutral: tests detect a
//! responsive `docker` or `podman` binary on the remote host instead of
//! assuming Podman specifically.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::{Once, OnceLock};

/// Remote paths on the companion container.
const REMOTE_AIBOX_BIN: &str = "/usr/local/bin/aibox";
const REMOTE_ADDONS_DIR: &str = "/opt/aibox/addons";
const EXPECTED_YAZI_VERSION: &str = "26.5.6";

/// Ensure the binary + addons are deployed exactly once per test run.
static DEPLOY_ONCE: Once = Once::new();
static COMPANION_START_ONCE: Once = Once::new();
static COMPANION_START_ERROR: OnceLock<String> = OnceLock::new();
static RUNTIME_BIN: OnceLock<String> = OnceLock::new();

/// SSH-based runner for executing commands on the aibox-e2e-testrunner companion container.
pub struct E2eRunner {
    ssh_key: String,
    host: String,
    port: u16,
    user: String,
}

impl E2eRunner {
    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("cli crate should live under repo root")
            .to_path_buf()
    }

    fn is_default_companion_target(&self) -> bool {
        self.host == "aibox-e2e-testrunner" && self.port == 22
    }

    fn companion_start_hint() -> String {
        COMPANION_START_ERROR.get().map_or_else(
            || {
                "No automatic companion startup was attempted, or the startup result was unavailable."
                    .to_string()
            },
            |error| format!("Automatic companion startup failed: {error}"),
        )
    }

    fn maybe_start_default_companion(&self) {
        if !self.is_default_companion_target() {
            return;
        }

        COMPANION_START_ONCE.call_once(|| {
            if self.ssh_echo_ok() {
                return;
            }

            // Direct `cargo test --features e2e ...` bypasses maintain.sh,
            // so start the companion here as a best-effort preflight.
            let repo_root = Self::repo_root();
            let compose_file = repo_root.join(".devcontainer/docker-compose.yml");
            let compose_override = repo_root.join(".devcontainer/docker-compose.override.yml");
            let bins = ["docker", "podman"];
            let mut errors = Vec::new();

            for bin in bins {
                let args = [
                    "compose".to_string(),
                    "-f".to_string(),
                    compose_file.to_string_lossy().to_string(),
                    "-f".to_string(),
                    compose_override.to_string_lossy().to_string(),
                    "up".to_string(),
                    "-d".to_string(),
                    "aibox-e2e-testrunner".to_string(),
                ];
                let out = Command::new(bin).args(&args).output();
                match out {
                    Ok(output) if output.status.success() => return,
                    Ok(output) => errors.push(format!(
                        "{} {} failed with status {}: {}",
                        bin,
                        args.join(" "),
                        output
                            .status
                            .code()
                            .map_or_else(|| "signal".to_string(), |c| c.to_string()),
                        String::from_utf8_lossy(&output.stderr).trim()
                    )),
                    Err(err) => errors.push(format!("{} unavailable: {}", bin, err)),
                }
            }

            let _ = COMPANION_START_ERROR.set(errors.join(" | "));
        });
    }

    fn ssh_echo_ok(&self) -> bool {
        let mut args = self.ssh_opts();
        args.extend([
            "-p".to_string(),
            self.port.to_string(),
            format!("{}@{}", self.user, self.host),
            "echo ok".to_string(),
        ]);
        Command::new("ssh")
            .args(&args)
            .output()
            .is_ok_and(|output| output.status.success() && output.stdout == b"ok\n")
    }

    fn command_exists(bin: &str) -> bool {
        Command::new(bin).arg("-V").output().is_ok()
    }

    fn assert_local_prereqs(&self) {
        assert!(
            Self::command_exists("ssh"),
            "missing local prerequisite: `ssh` not found on PATH. For this repo, rebuild the devcontainer so its repo-only test layer installs `openssh-client`, or install an SSH client manually."
        );
        assert!(
            Self::command_exists("scp"),
            "missing local prerequisite: `scp` not found on PATH. For this repo, rebuild the devcontainer so its repo-only test layer installs `openssh-client`, or install an SSH client manually."
        );
        assert!(
            Path::new(&self.ssh_key).exists(),
            "missing local prerequisite: E2E SSH key not found at {}. The repo expects the pre-seeded key under `.aibox-e2e-runner-home/.ssh/`.",
            self.ssh_key
        );
    }

    /// Create a runner pointing at the companion container.
    ///
    /// By default, connects to `aibox-e2e-testrunner:22` using the pre-seeded test SSH key.
    /// The `aibox-e2e-testrunner` hostname is resolved via Docker DNS (same compose network).
    pub fn new() -> Self {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        Self {
            ssh_key: std::env::var("AIBOX_E2E_SSH_KEY").unwrap_or_else(|_| {
                format!("{}/../.aibox-e2e-runner-home/.ssh/id_ed25519", manifest_dir)
            }),
            host: std::env::var("E2E_HOST").unwrap_or_else(|_| "aibox-e2e-testrunner".to_string()),
            port: std::env::var("E2E_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(22),
            user: "testuser".to_string(),
        }
    }

    /// Common SSH args (reused by exec and scp).
    ///
    /// Enables ControlMaster multiplexing so subsequent `ssh`/`scp` calls reuse
    /// the first TCP+TLS handshake. With ~10 exec calls per test averaging
    /// ~50 ms of handshake cost each, this saves ~500 ms per test.
    fn ssh_opts(&self) -> Vec<String> {
        let control_path = format!(
            "{}/aibox-ssh-{}-%h-%p-%r",
            std::env::temp_dir().display(),
            std::process::id()
        );
        vec![
            "-i".to_string(),
            self.ssh_key.clone(),
            "-o".to_string(),
            "StrictHostKeyChecking=no".to_string(),
            "-o".to_string(),
            "UserKnownHostsFile=/dev/null".to_string(),
            "-o".to_string(),
            "ConnectTimeout=5".to_string(),
            "-o".to_string(),
            "LogLevel=ERROR".to_string(),
            "-o".to_string(),
            "ControlMaster=auto".to_string(),
            "-o".to_string(),
            format!("ControlPath={control_path}"),
            "-o".to_string(),
            "ControlPersist=60s".to_string(),
        ]
    }

    /// Execute a raw command on the companion container via SSH.
    pub fn exec(&self, cmd: &str) -> Output {
        self.assert_local_prereqs();
        self.maybe_start_default_companion();
        let mut args = self.ssh_opts();
        args.extend([
            "-p".to_string(),
            self.port.to_string(),
            format!("{}@{}", self.user, self.host),
            cmd.to_string(),
        ]);
        Command::new("ssh")
            .args(&args)
            .output()
            .unwrap_or_else(|err| {
                panic!(
                    "SSH command failed ({err}). {}",
                    Self::companion_start_hint()
                )
            })
    }

    /// Return the responsive container runtime available on the companion.
    ///
    /// Prefers docker when both are present to match the CLI's main runtime
    /// detection policy (OrbStack / Docker Desktop first, Podman fallback).
    /// Detection requires an SSH round trip, so the result is cached for the
    /// process; a companion runtime cannot change during one suite run.
    pub fn runtime_bin(&self) -> String {
        RUNTIME_BIN
            .get_or_init(|| {
                if let Ok(explicit) = std::env::var("E2E_RUNTIME") {
                    let trimmed = explicit.trim();
                    if !trimmed.is_empty() {
                        return trimmed.to_string();
                    }
                }

                for candidate in ["docker", "podman"] {
                    let output = self.exec(&format!("{candidate} info >/dev/null 2>&1"));
                    if output.status.success() {
                        return candidate.to_string();
                    }
                }

                panic!(
                    "no responsive container runtime found on aibox-e2e-testrunner; tried docker and podman"
                );
            })
            .clone()
    }

    /// Copy a local file to the companion container via SCP.
    fn scp(&self, local_path: &str, remote_path: &str) {
        self.assert_local_prereqs();
        let mut args = self.ssh_opts();
        args.extend([
            "-P".to_string(),
            self.port.to_string(),
            local_path.to_string(),
            format!("{}@{}:{}", self.user, self.host, remote_path),
        ]);
        let output = Command::new("scp")
            .args(&args)
            .output()
            .expect("SCP command failed — is aibox-e2e-testrunner running?");
        assert!(
            output.status.success(),
            "scp {} -> {} failed: {}",
            local_path,
            remote_path,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    /// Recursively copy a local directory to the companion via SCP.
    fn scp_recursive(&self, local_path: &str, remote_path: &str) {
        self.assert_local_prereqs();
        let mut args = self.ssh_opts();
        args.extend([
            "-r".to_string(),
            "-P".to_string(),
            self.port.to_string(),
            local_path.to_string(),
            format!("{}@{}:{}", self.user, self.host, remote_path),
        ]);
        let output = Command::new("scp")
            .args(&args)
            .output()
            .expect("SCP command failed — is aibox-e2e-testrunner running?");
        assert!(
            output.status.success(),
            "scp -r {} -> {} failed: {}",
            local_path,
            remote_path,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    /// Deploy the aibox binary and addon definitions to the companion.
    ///
    /// Called once per test run (guarded by `Once`). SCPs the freshly-built
    /// binary to `/usr/local/bin/aibox`, the addon YAMLs to `/opt/aibox/addons/`,
    /// and container image assets (vimrc, bin scripts) to `/opt/aibox/`.
    pub fn deploy(&self) {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let binary = format!("{}/target/debug/aibox", manifest_dir);
        let addons = format!("{}/../addons", manifest_dir);

        assert!(
            Path::new(&binary).exists(),
            "aibox binary not found at {}. Run `cargo build` first.",
            binary
        );

        // Create remote directories
        self.exec(&format!(
            "sudo mkdir -p {} && sudo chown testuser:testuser {}",
            REMOTE_ADDONS_DIR, REMOTE_ADDONS_DIR
        ));

        // Deploy binary
        let tmp_bin = "/tmp/aibox";
        self.scp(&binary, tmp_bin);
        self.exec(&format!(
            "sudo mv {} {} && sudo chmod +x {}",
            tmp_bin, REMOTE_AIBOX_BIN, REMOTE_AIBOX_BIN
        ));

        // Deploy addons (recursive copy)
        self.exec(&format!("rm -rf {}/*", REMOTE_ADDONS_DIR));
        self.scp_recursive(&addons, "/opt/aibox/");

        // Deploy container image assets for visual keybinding tests.
        // The full vimrc (with leader key mappings) and bin scripts live in the
        // container image, not in the seeded .aibox-home. Deploy them so the
        // aibox-e2e-testrunner can simulate the full container environment.
        let image_config = format!("{}/../images/base-debian/config", manifest_dir);
        self.deploy_image_asset(
            &format!("{}/vimrc", image_config),
            "/opt/aibox/vimrc",
            false,
        );
        for (src, dst) in &[
            ("bin/open-in-editor.sh", "open-in-editor"),
            ("bin/vim-loop.sh", "vim-loop"),
            ("bin/aibox-status-toggle.sh", "aibox-status-toggle"),
        ] {
            self.deploy_image_asset(
                &format!("{}/{}", image_config, src),
                &format!("/usr/local/bin/{}", dst),
                true,
            );
        }
        self.deploy_image_asset(
            &format!("{}/bin/aibox-latex-preview.py", image_config),
            "/opt/aibox/aibox-latex-preview.py",
            true,
        );
        let runtime_tools = compile_runtime_tool_binaries(&image_config);
        self.deploy_image_asset(
            &runtime_tools.join("aibox-status").to_string_lossy(),
            "/usr/local/bin/aibox-status",
            true,
        );
        self.deploy_image_asset(
            &runtime_tools.join("aibox-diagnostics").to_string_lossy(),
            "/usr/local/bin/aibox-diagnostics",
            true,
        );
        self.exec("sudo rm -f /usr/local/bin/zellij");

        // Verify deployment
        let output = self.exec(&format!("{} --version", REMOTE_AIBOX_BIN));
        assert!(
            output.status.success(),
            "deployed aibox binary is not executable: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    /// Deploy a single file from a local path to a remote path on the companion.
    /// Skips silently if the local file does not exist.
    fn deploy_image_asset(&self, local_path: &str, remote_path: &str, executable: bool) {
        if !Path::new(local_path).exists() {
            return;
        }
        let file_name = Path::new(local_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "asset".to_string());
        let tmp = format!("/tmp/aibox_asset_{}", file_name);
        self.scp(local_path, &tmp);
        let chmod = if executable { "755" } else { "644" };
        self.exec(&format!(
            "sudo mv {} {} && sudo chmod {} {}",
            tmp, remote_path, chmod, remote_path
        ));
    }

    /// Ensure the binary is deployed (called automatically, once per test run).
    pub fn ensure_deployed(&self) {
        // We need to capture `self` for the closure, but Once::call_once
        // requires a static lifetime. Work around by checking a file marker.
        DEPLOY_ONCE.call_once(|| {
            self.assert_companion_tool_versions();
            self.deploy();
        });
    }

    /// Execute an aibox command in an isolated workspace directory.
    ///
    /// Creates `/workspaces/<test_name>/` on the companion if it doesn't exist.
    /// Automatically ensures the binary is deployed on first call.
    pub fn aibox(&self, test_name: &str, args: &[&str]) -> Output {
        self.ensure_deployed();
        let workspace = format!("/workspaces/{}", test_name);
        let cmd = format!(
            "mkdir -p {workspace} && cd {workspace} && AIBOX_ADDONS_DIR={} {} {}",
            REMOTE_ADDONS_DIR,
            REMOTE_AIBOX_BIN,
            args.join(" ")
        );
        self.exec(&cmd)
    }

    /// Read a file from the companion container.
    pub fn read_file(&self, test_name: &str, path: &str) -> String {
        let cmd = format!("cat /workspaces/{}/{}", test_name, path);
        let output = self.exec(&cmd);
        String::from_utf8_lossy(&output.stdout).to_string()
    }

    /// Write content to a file on the companion container.
    pub fn write_file(&self, test_name: &str, path: &str, content: &str) {
        let workspace = format!("/workspaces/{}", test_name);
        let full_path = format!("{}/{}", workspace, path);
        let cmd = format!(
            "mkdir -p {workspace} && mkdir -p $(dirname {full_path}) && rm -f {full_path} && cat > {full_path} << 'AIBOX_E2E_EOF'\n{content}\nAIBOX_E2E_EOF"
        );
        let output = self.exec(&cmd);
        assert!(
            output.status.success(),
            "write_file failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    /// Check if a file exists on the companion container.
    pub fn file_exists(&self, test_name: &str, path: &str) -> bool {
        let cmd = format!("test -f /workspaces/{}/{}", test_name, path);
        self.exec(&cmd).status.success()
    }

    /// Check if a directory exists on the companion container.
    pub fn dir_exists(&self, test_name: &str, path: &str) -> bool {
        let cmd = format!("test -d /workspaces/{}/{}", test_name, path);
        self.exec(&cmd).status.success()
    }

    /// Resolve the newest published base image version for lifecycle tests.
    ///
    /// During Phase 1 of a release the CLI version can be ahead of GHCR images,
    /// because versioned base images are pushed by the host-side Phase 2. Tests
    /// that only need a runnable published image should pin the published tag.
    pub fn latest_published_image_version(&self, test_name: &str) -> Option<String> {
        let workspace = format!("/workspaces/{test_name}");
        let output = self.exec(&format!(
            "cd {workspace} && \
             sed -i 's/^release_version = .*/release_version = \"latest\"/' aibox.toml && \
             AIBOX_ADDONS_DIR={REMOTE_ADDONS_DIR} {REMOTE_AIBOX_BIN} self update --check"
        ));
        assert!(
            output.status.success(),
            "failed to resolve latest published base image:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{stdout}{stderr}");
        if combined.contains("No usable published tags found for flavor")
            && combined.contains("incomplete manifests")
        {
            return None;
        }
        let version = combined.lines().find_map(|line| {
            let (_, version) = line.rsplit_once(" -> ")?;
            let version = version.split_whitespace().next().unwrap_or("");
            if version.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                Some(version.to_string())
            } else {
                None
            }
        });
        assert!(
            version.is_some(),
            "failed to parse latest published base image from `aibox self update --check` output:\n{combined}"
        );
        let version = version?;
        let parts: Vec<u32> = version
            .split('.')
            .filter_map(|part| part.parse::<u32>().ok())
            .collect();
        let is_runtime_tag = parts
            .first()
            .zip(parts.get(1))
            .is_some_and(|(major, minor)| *major > 0 || (*major == 0 && *minor >= 27));
        let image_tag = if is_runtime_tag {
            format!("ghcr.io/projectious-work/aibox:base-debian-runtime-v{version}")
        } else {
            format!("ghcr.io/projectious-work/aibox:base-debian-v{version}")
        };
        let runtime = self.runtime_bin();
        let pull = self.exec(&format!("{runtime} pull {image_tag} >/dev/null 2>&1"));
        if !pull.status.success() {
            eprintln!("published image prerequisite is not pullable: {image_tag}");
            return None;
        }
        Some(version)
    }

    /// Clean up a test workspace directory.
    pub fn cleanup(&self, test_name: &str) {
        let runtime = self.runtime_bin();
        let workspace = format!("/workspaces/{test_name}");
        let quoted_workspace = shell_quote(&workspace);
        let quoted_project = shell_quote(test_name);
        let cmd = format!(
            "workspace={quoted_workspace}; project={quoted_project}; runtime={runtime}; \
             if [ -f \"$workspace/.devcontainer/docker-compose.yml\" ]; then \
               cd \"$workspace\" && \"$runtime\" compose -f .devcontainer/docker-compose.yml down -v --remove-orphans >/dev/null 2>&1 || true; \
             fi; \
             ids=$(\"$runtime\" ps -aq --filter \"label=com.docker.compose.project=$project\" 2>/dev/null || true); \
             if [ -n \"$ids\" ]; then \"$runtime\" rm -f $ids >/dev/null 2>&1 || true; fi; \
             \"$runtime\" rm -f \"$project\" \"$project-diagnostics\" >/dev/null 2>&1 || true; \
             rm -rf \"$workspace\" 2>/dev/null || sudo rm -rf \"$workspace\""
        );
        let output = self.exec(&cmd);
        assert!(
            output.status.success(),
            "cleanup failed for {test_name}:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    /// Remove E2E-owned nested runtime state on the companion.
    ///
    /// This is intentionally suite-scoped. It removes compose containers whose
    /// working directory is under the E2E workspace root, removes their project
    /// volumes, and preserves images and BuildKit cache for subsequent runs.
    pub fn prune_companion_storage(&self) {
        let runtime = self.runtime_bin();
        let cmd = format!(
            "runtime={runtime}; \
             for workspace in /workspaces/*; do \
               [ -d \"$workspace\" ] || continue; \
               if [ -f \"$workspace/.devcontainer/docker-compose.yml\" ]; then \
                 (cd \"$workspace\" && \"$runtime\" compose -f .devcontainer/docker-compose.yml down -v --remove-orphans >/dev/null 2>&1) || true; \
               fi; \
             done; \
             for id in $(\"$runtime\" ps -aq --filter label=com.docker.compose.project.working_dir 2>/dev/null || true); do \
               working_dir=$(\"$runtime\" inspect --format '{{{{ index .Config.Labels \"com.docker.compose.project.working_dir\" }}}}' \"$id\" 2>/dev/null || true); \
               case \"$working_dir\" in /workspaces/*) \"$runtime\" rm -f \"$id\" >/dev/null 2>&1 || true ;; esac; \
             done; \
             find /workspaces -mindepth 1 -maxdepth 1 -exec rm -rf {{}} + 2>/dev/null || \
               sudo find /workspaces -mindepth 1 -maxdepth 1 -exec rm -rf {{}} +"
        );
        let output = self.exec(&cmd);
        assert!(
            output.status.success(),
            "companion prune failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    /// Execute a command inside a running aibox container.
    ///
    /// Used for smoke tests that verify tools are installed and functional.
    pub fn container_exec(&self, container_name: &str, cmd: &str) -> Output {
        let runtime = self.runtime_bin();
        self.exec(&format!("{} exec {} {}", runtime, container_name, cmd))
    }

    /// Assert the companion container is reachable.
    pub fn assert_reachable(&self) {
        let output = self.exec("echo ok");
        assert!(
            output.status.success(),
            "aibox-e2e-testrunner is not reachable via SSH. Is the companion container running?\n\
             {}\n\
             stderr: {}",
            Self::companion_start_hint(),
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.trim() == "ok",
            "unexpected response from aibox-e2e-testrunner: '{}'",
            stdout.trim()
        );
        self.assert_companion_tool_versions();
    }

    /// Assert the companion image has the runtime tools expected by the tests.
    ///
    /// tmux rendering and Yazi config grammar are version-sensitive.
    /// A stale companion image can otherwise fail visual tests with misleading
    /// low-level terminal or TOML errors.
    pub fn assert_companion_tool_versions(&self) {
        let output = self.exec(
            "tmux -V && \
             yazi --version && \
             command -v ya && \
             ya --version && \
             bwrap --version && \
             command -v newuidmap || { echo missing-newuidmap; exit 1; }; \
             command -v newgidmap || { echo missing-newgidmap; exit 1; }; \
             unshare --user --map-root-user true && \
             bwrap --unshare-user --uid 0 --gid 0 --ro-bind / / --dev /dev --proc /proc /bin/true && \
             cat /usr/local/share/aibox/e2e-companion-contract && \
             test \"$(cat /usr/local/share/aibox/e2e-companion-contract)\" = \"aibox-e2e-companion-contract=3\" && \
             command -v fuse-overlayfs && \
             podman info --format 'podman-storage-driver={{ .Store.GraphDriverName }}' && \
             echo bwrap-ok",
        );
        assert!(
            output.status.success(),
            "failed to inspect aibox-e2e-testrunner tool versions:\n{}\nstdout:\n{}\nstderr:\n{}",
            Self::companion_start_hint(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        let expected_yazi = format!("Yazi {EXPECTED_YAZI_VERSION}");
        assert!(
            stdout.contains("tmux ")
                && stdout.matches(&expected_yazi).count() >= 2
                && stdout.contains("/usr/local/bin/ya")
                && stdout.contains("bubblewrap ")
                && stdout.contains("newuidmap")
                && stdout.contains("newgidmap")
                && stdout.contains("aibox-e2e-companion-contract=3")
                && stdout.contains("fuse-overlayfs")
                && (stdout.contains("podman-storage-driver=overlay")
                    || stdout.contains("podman-storage-driver=vfs"))
                && stdout.contains("bwrap-ok"),
            "aibox-e2e-testrunner image is stale or has an unsupported Podman storage driver; expected tmux, Yazi {EXPECTED_YAZI_VERSION}, the ya companion entrypoint, uidmap helpers, fuse-overlayfs, companion contract v3, Podman overlay or safe vfs fallback, and a working bubblewrap user-namespace smoke probe.\n\
             Rebuild/recreate the companion service from .devcontainer/Dockerfile.e2e, then rerun `./scripts/maintain.sh test-e2e`.\n\
             observed:\n{stdout}"
        );
    }
}

fn compile_runtime_tool_binaries(image_config: &str) -> PathBuf {
    // Hash all source files together so any change to either invalidates the
    // cached pair atomically. Caching shaves ~5–10 s off every test invocation
    // since these binaries change rarely but were previously recompiled per
    // `cargo test` run.
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Hash every source file under the image config bin/ dir that the tools
    // depend on. `aibox_status_core.rs` is shared by both binaries, so it
    // contributes to the cache key even though we don't compile it directly.
    let hashed_sources = [
        "bin/aibox_status_core.rs",
        "bin/aibox-status.rs",
        "bin/aibox-diagnostics.rs",
    ];

    let mut hasher = DefaultHasher::new();
    for src in &hashed_sources {
        let path = format!("{image_config}/{src}");
        if let Ok(bytes) = std::fs::read(&path) {
            src.hash(&mut hasher);
            bytes.hash(&mut hasher);
        }
    }
    let cache_key = format!("{:016x}", hasher.finish());

    let cache_root = std::env::temp_dir()
        .join("aibox-e2e-runtime-tools")
        .join(&cache_key);

    // Only the two installable binaries matter for cache validation.
    let installable_bins = ["aibox-status", "aibox-diagnostics"];
    let all_cached = installable_bins.iter().all(|bin| {
        let p = cache_root.join(bin);
        p.is_file() && std::fs::metadata(&p).map(|m| m.len() > 0).unwrap_or(false)
    });
    if all_cached {
        return cache_root;
    }

    std::fs::create_dir_all(&cache_root).expect("failed to create runtime tool build dir");
    for (src, bin) in [
        ("bin/aibox-status.rs", "aibox-status"),
        ("bin/aibox-diagnostics.rs", "aibox-diagnostics"),
    ] {
        let source = format!("{image_config}/{src}");
        let output = Command::new("rustc")
            .args([
                "--edition=2021",
                "-D",
                "warnings",
                "-C",
                "opt-level=2",
                &source,
                "-o",
                &cache_root.join(bin).to_string_lossy(),
            ])
            .output()
            .expect("failed to compile runtime tool");
        assert!(
            output.status.success(),
            "runtime tool build failed for {src}:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    cache_root
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

impl Default for E2eRunner {
    fn default() -> Self {
        Self::new()
    }
}
