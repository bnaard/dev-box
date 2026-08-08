//! Reusable least-privilege harness for CLI and generated-file E2E contracts.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

pub struct LocalProject {
    _tempdir: tempfile::TempDir,
    root: PathBuf,
}

impl LocalProject {
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

    fn sanitize(&self, command: &mut Command) {
        command
            .current_dir(&self.root)
            .env(
                "AIBOX_ADDONS_DIR",
                format!("{}/../addons", env!("CARGO_MANIFEST_DIR")),
            )
            .env("AIBOX_NO_CONTAINER", "1")
            .env_remove("DOCKER_HOST")
            .env_remove("CONTAINER_HOST")
            .env_remove("E2E_HOST");
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
