//! Reusable least-privilege harness for CLI and generated-file E2E contracts.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

pub struct LocalProject {
    _tempdir: tempfile::TempDir,
    root: PathBuf,
}

impl Default for LocalProject {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalProject {
    pub fn new() -> Self {
        Self::empty()
    }

    pub fn empty() -> Self {
        let tempdir = tempfile::tempdir().expect("create local E2E workspace");
        let root = tempdir.path().to_path_buf();
        Self {
            _tempdir: tempdir,
            root,
        }
    }

    pub fn initialized(name: &str, context: &str, addons: &[&str]) -> Self {
        let project = Self::empty();
        let mut args = vec![
            "init",
            name,
            "--base",
            "debian",
            "--context",
            context,
            "--processkit-version",
            "unset",
            "--no-container",
        ];
        for addon in addons {
            args.extend(["--addon", addon]);
        }
        let output = project.run(&args);
        project.assert_success("init", &output);
        project
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn tmux_socket(&self) -> PathBuf {
        self.root.join("runtime/tmux.sock")
    }

    pub fn run(&self, args: &[&str]) -> Output {
        let mut command =
            Command::new(format!("{}/target/debug/aibox", env!("CARGO_MANIFEST_DIR")));
        self.sanitize(&mut command);
        command
            .args(args)
            .output()
            .expect("execute aibox in local E2E workspace")
    }

    pub fn shell(&self, script: &str) -> Output {
        let mut command = Command::new("bash");
        self.sanitize(&mut command);
        command
            .args(["-c", script])
            .output()
            .expect("execute local E2E probe")
    }

    /// Compatibility surface for tests being migrated from the SSH runner.
    /// The workspace name is intentionally ignored because this project owns
    /// one unique temporary root rather than a shared `/workspaces` namespace.
    pub fn aibox(&self, _workspace_name: &str, args: &[&str]) -> Output {
        self.run(args)
    }

    pub fn exec(&self, script: &str) -> Output {
        self.shell(script)
    }

    pub fn cleanup(&self, _workspace_name: &str) {
        // TempDir removes only this project's unique workspace on drop.
    }

    pub fn ensure_deployed(&self) {
        // The test binary and addon catalog are used directly from this checkout.
    }

    pub fn read_file(&self, _workspace_name: &str, path: &str) -> String {
        self.read(path)
    }

    pub fn write_file(&self, _workspace_name: &str, path: &str, content: &str) {
        self.write(path, content);
    }

    pub fn file_exists(&self, _workspace_name: &str, path: &str) -> bool {
        self.exists(path)
    }

    fn sanitize(&self, command: &mut Command) {
        command
            .current_dir(&self.root)
            .env(
                "AIBOX_ADDONS_DIR",
                format!("{}/../addons", env!("CARGO_MANIFEST_DIR")),
            )
            .env("AIBOX_NO_CONTAINER", "1")
            .env("AIBOX_TMUX_SOCKET", self.tmux_socket())
            .env("TMUX_TMPDIR", self.root.join("runtime/tmux-tmp"))
            .env_remove("DOCKER_HOST")
            .env_remove("CONTAINER_HOST")
            .env_remove("E2E_HOST")
            .env_remove("TMUX")
            .env_remove("TMUX_PANE");
    }

    pub fn assert_success(&self, label: &str, output: &Output) {
        assert!(
            output.status.success(),
            "{label} failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    pub fn exists(&self, path: &str) -> bool {
        self.root.join(path).exists()
    }

    pub fn is_dir(&self, path: &str) -> bool {
        self.root.join(path).is_dir()
    }

    pub fn read(&self, path: &str) -> String {
        fs::read_to_string(self.root.join(path)).expect("read local E2E file")
    }

    pub fn write(&self, path: &str, content: &str) {
        let destination = self.root.join(path);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).expect("create local E2E parent directory");
        }
        fs::write(destination, content).expect("write local E2E file");
    }
}

#[test]
fn local_tmux_cleanup_cannot_reach_an_existing_server() {
    let external = tempfile::tempdir().expect("create external tmux socket directory");
    let external_socket = external.path().join("existing.sock");
    let external_status = Command::new("tmux")
        .args(["-S"])
        .arg(&external_socket)
        .args(["new-session", "-d", "-s", "existing"])
        .status()
        .expect("start external tmux server");
    assert!(external_status.success());

    let project = LocalProject::new();
    let isolated = project.shell(
        "mkdir -p \"$(dirname \"$AIBOX_TMUX_SOCKET\")\" \"$TMUX_TMPDIR\" && \
         tmux -S \"$AIBOX_TMUX_SOCKET\" new-session -d -s isolated && \
         tmux -S \"$AIBOX_TMUX_SOCKET\" kill-server",
    );
    project.assert_success("isolated tmux cleanup", &isolated);

    let external_survived = Command::new("tmux")
        .args(["-S"])
        .arg(&external_socket)
        .args(["has-session", "-t", "existing"])
        .status()
        .expect("probe external tmux server");
    assert!(external_survived.success());

    let _ = Command::new("tmux")
        .args(["-S"])
        .arg(&external_socket)
        .args(["kill-server"])
        .status();
}
