use anyhow::{Context, Result, bail};
use std::process::Command;

/// Container state as detected by runtime inspect.
#[derive(Debug, Clone, PartialEq)]
pub enum ContainerState {
    Running,
    Stopped,
    Missing,
}

impl std::fmt::Display for ContainerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContainerState::Running => write!(f, "running"),
            ContainerState::Stopped => write!(f, "stopped"),
            ContainerState::Missing => write!(f, "missing"),
        }
    }
}

/// Abstraction over podman/docker runtime.
#[derive(Debug, Clone)]
pub struct Runtime {
    pub compose_bin: Vec<String>,
    pub runtime_bin: String,
}

impl Runtime {
    /// Detect available container runtime.
    ///
    /// Checks both presence on PATH and whether the daemon is actually
    /// responsive (`docker info` / `podman info`). Prefers docker over
    /// podman because OrbStack (common on macOS) provides docker and
    /// podman may be installed but not running.
    pub fn detect() -> Result<Self> {
        // Prefer docker — OrbStack and Docker Desktop both provide it
        if runtime_is_responsive("docker") {
            return Ok(Runtime {
                compose_bin: vec!["docker".to_string(), "compose".to_string()],
                runtime_bin: "docker".to_string(),
            });
        }

        // Fall back to podman if docker isn't responsive
        if runtime_is_responsive("podman") {
            return Ok(Runtime {
                compose_bin: vec!["podman".to_string(), "compose".to_string()],
                runtime_bin: "podman".to_string(),
            });
        }

        // Neither is responsive — give a helpful error
        let docker_present = command_exists("docker");
        let podman_present = command_exists("podman");

        match (docker_present, podman_present) {
            (true, _) => bail!(
                "docker found on PATH but not responding. \
                 Is Docker Desktop or OrbStack running?"
            ),
            (_, true) => bail!(
                "podman found on PATH but not responding. \
                 Try: podman machine start"
            ),
            _ => bail!("Neither podman nor docker found. Please install one of them."),
        }
    }

    /// Read the `aibox.version` label from a container image.
    ///
    /// Returns `None` if the container doesn't exist or the label is absent
    /// (e.g. containers built before BACK-060).
    pub fn get_container_image_version(&self, container_name: &str) -> Result<Option<String>> {
        let output = Command::new(&self.runtime_bin)
            .args([
                "inspect",
                "--format",
                "{{index .Config.Labels \"aibox.version\"}}",
                container_name,
            ])
            .output()
            .context("Failed to inspect container for version label")?;

        if !output.status.success() {
            return Ok(None);
        }

        let label = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if label.is_empty() {
            Ok(None)
        } else {
            Ok(Some(label))
        }
    }

    /// Read the Docker Compose project label from a container.
    ///
    /// Returns `None` if the container doesn't exist or wasn't created by
    /// Compose. This is used to detect old aibox runtimes whose generated
    /// compose project name predates the current `name:` field.
    pub fn get_container_compose_project(&self, container_name: &str) -> Result<Option<String>> {
        let output = Command::new(&self.runtime_bin)
            .args([
                "inspect",
                "--format",
                "{{index .Config.Labels \"com.docker.compose.project\"}}",
                container_name,
            ])
            .output()
            .context("Failed to inspect container for compose project label")?;

        if !output.status.success() {
            return Ok(None);
        }

        let label = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if label.is_empty() || label == "<no value>" {
            Ok(None)
        } else {
            Ok(Some(label))
        }
    }

    /// List all containers, including stopped ones, that belong to a Compose
    /// project label.
    pub fn list_containers_by_compose_project(&self, project: &str) -> Result<Vec<String>> {
        if project.trim().is_empty() {
            return Ok(vec![]);
        }

        let label_filter = format!("label=com.docker.compose.project={project}");
        let output = Command::new(&self.runtime_bin)
            .args([
                "ps",
                "-a",
                "--filter",
                &label_filter,
                "--format",
                "{{.Names}}",
            ])
            .output()
            .with_context(|| format!("Failed to list containers for compose project {project}"))?;

        if !output.status.success() {
            bail!(
                "Failed to list containers for compose project '{}'",
                project
            );
        }

        Ok(String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|line| line.trim().trim_start_matches('/').to_string())
            .filter(|line| !line.is_empty())
            .collect())
    }

    /// Get the container state by inspecting it directly.
    pub fn container_status(&self, name: &str) -> Result<ContainerState> {
        let output = Command::new(&self.runtime_bin)
            .args(["inspect", "--format", "{{.State.Status}}", name])
            .output()
            .context("Failed to run container inspect")?;

        if !output.status.success() {
            // Container doesn't exist
            return Ok(ContainerState::Missing);
        }

        let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
        match status.as_str() {
            "running" => Ok(ContainerState::Running),
            _ => Ok(ContainerState::Stopped),
        }
    }

    /// Resolve a compose file path to absolute.
    ///
    /// `podman compose` delegates to external providers (e.g. `podman-compose`)
    /// which may not inherit the working directory, so relative paths break.
    fn resolve_compose_path(compose_file: &str) -> Result<String> {
        let path = std::env::current_dir()
            .context("Failed to get current directory")?
            .join(compose_file);
        Ok(path.to_string_lossy().to_string())
    }

    /// Build the `-f` arguments for Docker Compose, automatically including
    /// `docker-compose.override.yml` if it exists alongside the main file.
    /// This ensures user-defined services (e.g. the e2e companion container)
    /// are always included in build/up/stop operations.
    fn compose_file_args(compose_file: &str) -> Result<Vec<String>> {
        let abs = Self::resolve_compose_path(compose_file)?;
        let mut args = vec!["-f".to_string(), abs.clone()];

        // Check for override file alongside the main compose file
        let main_path = std::path::Path::new(&abs);
        if let Some(parent) = main_path.parent() {
            let override_path = parent.join("docker-compose.override.yml");
            if compose_override_has_content(&override_path) {
                args.push("-f".to_string());
                args.push(override_path.to_string_lossy().to_string());
            }
        }

        Ok(args)
    }

    /// Run compose build.
    ///
    /// When `no_cache` is true, passes both `--no-cache` (rebuild all
    /// layers from scratch) and `--pull` (force fresh base image pulls)
    /// to ensure a fully clean rebuild.
    pub fn compose_build(&self, compose_file: &str, no_cache: bool) -> Result<()> {
        let file_args = Self::compose_file_args(compose_file)?;
        let mut args: Vec<&str> = file_args.iter().map(|s| s.as_str()).collect();
        args.push("build");
        if no_cache {
            args.push("--no-cache");
            args.push("--pull");
        }

        let status = self.run_compose(&args)?;
        if !status.success() {
            bail!("Compose build failed");
        }
        Ok(())
    }

    fn compose_up_args(compose_file: &str, service: &str, no_deps: bool) -> Result<Vec<String>> {
        let file_args = Self::compose_file_args(compose_file)?;
        let mut args = file_args;
        args.extend(["up".to_string(), "-d".to_string()]);
        if no_deps {
            args.push("--no-deps".to_string());
        }
        args.push(service.to_string());
        Ok(args)
    }

    /// Run compose up -d for a service.
    pub fn compose_up(&self, compose_file: &str, service: &str) -> Result<()> {
        let args = Self::compose_up_args(compose_file, service, false)?;
        let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let status = self.run_compose(&arg_refs)?;
        if !status.success() {
            bail!("Compose up failed");
        }
        Ok(())
    }

    /// Run compose up -d --no-deps for a service.
    pub fn compose_up_no_deps(&self, compose_file: &str, service: &str) -> Result<()> {
        let args = Self::compose_up_args(compose_file, service, true)?;
        let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let status = self.run_compose(&arg_refs)?;
        if !status.success() {
            bail!("Compose up failed");
        }
        Ok(())
    }

    /// Run compose stop for the whole compose project.
    pub fn compose_stop_all(&self, compose_file: &str) -> Result<()> {
        let file_args = Self::compose_file_args(compose_file)?;
        let mut args: Vec<&str> = file_args.iter().map(|s| s.as_str()).collect();
        args.push("stop");
        let status = self.run_compose(&args)?;
        if !status.success() {
            bail!("Compose stop failed");
        }
        Ok(())
    }

    /// Run compose down for the whole compose project.
    pub fn compose_down(&self, compose_file: &str) -> Result<()> {
        let file_args = Self::compose_file_args(compose_file)?;
        let mut args: Vec<&str> = file_args.iter().map(|s| s.as_str()).collect();
        args.push("down");
        let status = self.run_compose(&args)?;
        if !status.success() {
            bail!("Compose down failed");
        }
        Ok(())
    }

    /// Stop and remove one concrete container by name.
    ///
    /// Compose cleanup is project-scoped. When aibox changes the compose
    /// project name, an old same-name container can survive outside the current
    /// project. `aibox delete runtime` still owns the explicit configured
    /// container name, so it needs this direct fallback.
    pub fn remove_container_by_name(&self, name: &str) -> Result<()> {
        let status = Command::new(&self.runtime_bin)
            .args(["rm", "-f", name])
            .status()
            .context("Failed to remove container")?;

        if !status.success() {
            bail!("Failed to remove container '{}'", name);
        }
        Ok(())
    }

    /// Stop one concrete container by name.
    pub fn stop_container_by_name(&self, name: &str) -> Result<()> {
        let status = Command::new(&self.runtime_bin)
            .args(["stop", name])
            .status()
            .context("Failed to stop container")?;

        if !status.success() {
            bail!("Failed to stop container '{}'", name);
        }
        Ok(())
    }

    /// Exec interactively into a container as the specified user.
    pub fn exec_interactive(&self, container: &str, user: &str, cmd: &[&str]) -> Result<()> {
        let mut args = vec!["exec", "-it", "-u", user, container];
        args.extend_from_slice(cmd);

        let status = Command::new(&self.runtime_bin)
            .args(&args)
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status()
            .context("Failed to exec into container")?;

        if !status.success() {
            bail!(
                "Exec into container failed (exit code: {:?})",
                status.code()
            );
        }
        Ok(())
    }

    /// Exec a command non-interactively into a container as the specified user.
    pub fn exec_status(&self, container: &str, user: &str, cmd: &[&str]) -> Result<bool> {
        let mut args = vec!["exec", "-u", user, container];
        args.extend_from_slice(cmd);

        let status = Command::new(&self.runtime_bin)
            .args(&args)
            .status()
            .context("Failed to exec into container")?;

        Ok(status.success())
    }

    /// Wait for a container to reach running state, polling every 500ms.
    pub fn wait_for_running(&self, name: &str, timeout_ms: u64) -> Result<()> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);
        let poll_interval = std::time::Duration::from_millis(500);

        loop {
            if let Ok(ContainerState::Running) = self.container_status(name) {
                return Ok(());
            }

            if start.elapsed() > timeout {
                bail!(
                    "Timed out waiting for container '{}' to start ({}ms)",
                    name,
                    timeout_ms
                );
            }

            std::thread::sleep(poll_interval);
        }
    }

    /// Run a compose command, inheriting stdio.
    fn run_compose(&self, args: &[&str]) -> Result<std::process::ExitStatus> {
        let (program, base_args) = match self.compose_bin.len() {
            1 => (self.compose_bin[0].as_str(), vec![]),
            _ => (
                self.compose_bin[0].as_str(),
                self.compose_bin[1..]
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>(),
            ),
        };

        let mut full_args = base_args;
        full_args.extend_from_slice(args);

        Command::new(program)
            .args(&full_args)
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status()
            .context("Failed to run compose command")
    }
}

fn compose_override_has_content(path: &std::path::Path) -> bool {
    let Ok(content) = std::fs::read_to_string(path) else {
        return false;
    };
    content
        .lines()
        .map(str::trim)
        .any(|line| !line.is_empty() && !line.starts_with('#'))
}

/// Check if a command exists on PATH.
pub(crate) fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if a container runtime is on PATH AND responsive.
/// Runs `<runtime> info` with a short timeout to verify the daemon is up.
fn runtime_is_responsive(cmd: &str) -> bool {
    if !command_exists(cmd) {
        return false;
    }

    // `docker info` / `podman info` exits 0 if the daemon is reachable
    Command::new(cmd)
        .arg("info")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_state_display_running() {
        assert_eq!(format!("{}", ContainerState::Running), "running");
    }

    #[test]
    fn container_state_display_stopped() {
        assert_eq!(format!("{}", ContainerState::Stopped), "stopped");
    }

    #[test]
    fn container_state_display_missing() {
        assert_eq!(format!("{}", ContainerState::Missing), "missing");
    }

    #[test]
    fn container_state_equality() {
        assert_eq!(ContainerState::Running, ContainerState::Running);
        assert_ne!(ContainerState::Running, ContainerState::Stopped);
        assert_ne!(ContainerState::Stopped, ContainerState::Missing);
    }

    #[test]
    fn command_exists_for_bash() {
        // bash should exist on any Linux system
        assert!(command_exists("bash"), "bash should be found on PATH");
    }

    #[test]
    fn command_exists_for_nonexistent() {
        assert!(!command_exists("this_command_does_not_exist_xyz123"));
    }

    #[test]
    fn runtime_struct_fields() {
        let rt = Runtime {
            compose_bin: vec!["docker".to_string(), "compose".to_string()],
            runtime_bin: "docker".to_string(),
        };
        assert_eq!(rt.runtime_bin, "docker");
        assert_eq!(rt.compose_bin.len(), 2);
    }

    #[test]
    fn compose_file_args_include_override_when_present() {
        let dir = tempfile::tempdir().unwrap();
        let devcontainer = dir.path().join(".devcontainer");
        std::fs::create_dir_all(&devcontainer).unwrap();
        let compose_file = devcontainer.join("docker-compose.yml");
        std::fs::write(&compose_file, "services: {}\n").unwrap();
        std::fs::write(
            devcontainer.join("docker-compose.override.yml"),
            "services: {}\n",
        )
        .unwrap();

        let compose_file = compose_file.to_string_lossy();
        let args = Runtime::compose_file_args(&compose_file).unwrap();

        assert_eq!(args.len(), 4);
        assert_eq!(args[0], "-f");
        assert!(args[1].ends_with(".devcontainer/docker-compose.yml"));
        assert_eq!(args[2], "-f");
        assert!(args[3].ends_with(".devcontainer/docker-compose.override.yml"));
    }

    #[test]
    fn compose_file_args_skip_comment_only_override() {
        let dir = tempfile::tempdir().unwrap();
        let devcontainer = dir.path().join(".devcontainer");
        std::fs::create_dir_all(&devcontainer).unwrap();
        let compose_file = devcontainer.join("docker-compose.yml");
        std::fs::write(&compose_file, "services: {}\n").unwrap();
        std::fs::write(
            devcontainer.join("docker-compose.override.yml"),
            "# project-specific overrides\n\n# services:\n",
        )
        .unwrap();

        let compose_file = compose_file.to_string_lossy();
        let args = Runtime::compose_file_args(&compose_file).unwrap();

        assert_eq!(args.len(), 2);
        assert_eq!(args[0], "-f");
        assert!(args[1].ends_with(".devcontainer/docker-compose.yml"));
    }

    #[test]
    fn compose_up_args_can_skip_dependencies() {
        let dir = tempfile::tempdir().unwrap();
        let devcontainer = dir.path().join(".devcontainer");
        std::fs::create_dir_all(&devcontainer).unwrap();
        let compose_file = devcontainer.join("docker-compose.yml");
        std::fs::write(&compose_file, "services: {}\n").unwrap();

        let compose_file = compose_file.to_string_lossy();
        let args = Runtime::compose_up_args(&compose_file, "aibox-diagnostics", true).unwrap();

        assert!(args.ends_with(&[
            "up".to_string(),
            "-d".to_string(),
            "--no-deps".to_string(),
            "aibox-diagnostics".to_string(),
        ]));
    }

    #[test]
    fn compose_up_args_include_dependencies_by_default() {
        let dir = tempfile::tempdir().unwrap();
        let devcontainer = dir.path().join(".devcontainer");
        std::fs::create_dir_all(&devcontainer).unwrap();
        let compose_file = devcontainer.join("docker-compose.yml");
        std::fs::write(&compose_file, "services: {}\n").unwrap();

        let compose_file = compose_file.to_string_lossy();
        let args = Runtime::compose_up_args(&compose_file, "aibox", false).unwrap();

        assert!(args.ends_with(&["up".to_string(), "-d".to_string(), "aibox".to_string(),]));
        assert!(!args.contains(&"--no-deps".to_string()));
    }

    #[test]
    fn get_container_image_version_for_nonexistent_container() {
        let rt = if runtime_is_responsive("docker") {
            Runtime {
                compose_bin: vec!["docker".to_string(), "compose".to_string()],
                runtime_bin: "docker".to_string(),
            }
        } else if runtime_is_responsive("podman") {
            Runtime {
                compose_bin: vec!["podman".to_string(), "compose".to_string()],
                runtime_bin: "podman".to_string(),
            }
        } else {
            return;
        };
        // Nonexistent container should return None (inspect fails)
        let version = rt
            .get_container_image_version("nonexistent_container_xyz123")
            .unwrap();
        assert_eq!(version, None);
    }

    #[test]
    fn container_status_for_nonexistent_container() {
        // This test requires docker or podman to be available
        let rt = if runtime_is_responsive("docker") {
            Runtime {
                compose_bin: vec!["docker".to_string(), "compose".to_string()],
                runtime_bin: "docker".to_string(),
            }
        } else if runtime_is_responsive("podman") {
            Runtime {
                compose_bin: vec!["podman".to_string(), "compose".to_string()],
                runtime_bin: "podman".to_string(),
            }
        } else {
            // No responsive runtime — skip
            return;
        };
        let state = rt.container_status("nonexistent_container_xyz123").unwrap();
        assert_eq!(state, ContainerState::Missing);
    }
}
