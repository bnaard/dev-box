//! Opaque processkit installer protocol boundary for aibox v1.
//!
//! The wire contract is owned by processkit. Aibox selects an operation and
//! supplies paths and user intent, then treats the structured result as opaque
//! evidence. It has no knowledge of processkit layouts, skills, packages,
//! templates, migrations, or harness projections.
//!
//! Consumer compatibility is validated against the exact-pinned processkit
//! v1.0.0-alpha.3 release assets. Stable-v1 remains gated on the rest of the
//! parity, migration, rollback, interruption, and secret-safety evidence.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tempfile::NamedTempFile;

pub const PROCESSKIT_INSTALLER_PROTOCOL_V1ALPHA1: &str =
    "processkit.projectious.work/installer/v1alpha1";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InstallerOperation {
    Plan,
    Install,
    Update,
    Verify,
    Uninstall,
    Recover,
}

impl InstallerOperation {
    fn needs_release(self) -> bool {
        matches!(self, Self::Plan | Self::Install | Self::Update)
    }

    fn mutates(self) -> bool {
        matches!(
            self,
            Self::Install | Self::Update | Self::Uninstall | Self::Recover
        )
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InstallerRequest {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub operation: InstallerOperation,
    pub root: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distribution_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub envelope_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trust_store_path: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub profiles: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub harnesses: Vec<String>,
    #[serde(default)]
    pub yes: bool,
}

impl InstallerRequest {
    pub fn development(
        operation: InstallerOperation,
        root: PathBuf,
        distribution_path: PathBuf,
    ) -> Self {
        Self {
            api_version: PROCESSKIT_INSTALLER_PROTOCOL_V1ALPHA1.into(),
            operation,
            root,
            distribution_path: Some(distribution_path),
            envelope_path: None,
            signature_path: None,
            trust_store_path: None,
            profiles: Vec::new(),
            harnesses: Vec::new(),
            yes: operation.mutates(),
        }
    }

    pub fn signed_release(
        operation: InstallerOperation,
        root: PathBuf,
        envelope_path: PathBuf,
        signature_path: PathBuf,
        trust_store_path: PathBuf,
    ) -> Self {
        Self {
            api_version: PROCESSKIT_INSTALLER_PROTOCOL_V1ALPHA1.into(),
            operation,
            root,
            distribution_path: None,
            envelope_path: Some(envelope_path),
            signature_path: Some(signature_path),
            trust_store_path: Some(trust_store_path),
            profiles: Vec::new(),
            harnesses: Vec::new(),
            yes: operation.mutates(),
        }
    }

    pub fn local(operation: InstallerOperation, root: PathBuf) -> Self {
        Self {
            api_version: PROCESSKIT_INSTALLER_PROTOCOL_V1ALPHA1.into(),
            operation,
            root,
            distribution_path: None,
            envelope_path: None,
            signature_path: None,
            trust_store_path: None,
            profiles: Vec::new(),
            harnesses: Vec::new(),
            yes: operation.mutates(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.api_version != PROCESSKIT_INSTALLER_PROTOCOL_V1ALPHA1 {
            bail!(
                "unsupported processkit installer request apiVersion: {}",
                self.api_version
            );
        }
        if self.root.as_os_str().is_empty() {
            bail!("processkit installer request root must not be empty");
        }
        let signed_count = [
            self.envelope_path.is_some(),
            self.signature_path.is_some(),
            self.trust_store_path.is_some(),
        ]
        .into_iter()
        .filter(|present| *present)
        .count();
        if !matches!(signed_count, 0 | 3) {
            bail!("signed processkit release requires envelope, signature, and trust-store paths");
        }
        if signed_count == 3 && self.distribution_path.is_some() {
            bail!("signed and development processkit release inputs are mutually exclusive");
        }
        if self.operation.needs_release() && self.distribution_path.is_none() && signed_count == 0 {
            bail!("processkit {:?} requires a release input", self.operation);
        }
        if !self.operation.needs_release()
            && (self.distribution_path.is_some() || signed_count != 0)
        {
            bail!(
                "processkit {:?} does not accept a release input",
                self.operation
            );
        }
        if self.operation.mutates() && !self.yes {
            bail!(
                "processkit {:?} requires explicit mutation acknowledgement",
                self.operation
            );
        }
        if self.profiles.iter().any(|value| value.trim().is_empty())
            || self.harnesses.iter().any(|value| value.trim().is_empty())
        {
            bail!("processkit profiles and harnesses must not contain empty values");
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InstallerStatus {
    Planned,
    Conflict,
    Invalid,
    Installed,
    Updated,
    Uninstalled,
    Recovered,
    Verified,
    Drifted,
}

impl InstallerStatus {
    pub fn is_success(&self) -> bool {
        matches!(
            self,
            Self::Planned
                | Self::Installed
                | Self::Updated
                | Self::Uninstalled
                | Self::Recovered
                | Self::Verified
        )
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallerResult {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub status: InstallerStatus,
    pub changes: Vec<Value>,
    pub conflicts: Vec<Value>,
    pub warnings: Vec<Value>,
    pub errors: Vec<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checked: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Value>,
    #[serde(flatten)]
    pub extensions: serde_json::Map<String, Value>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CliAvailability {
    Available(PathBuf),
    Unavailable,
}

/// The complete v1 integration choice. Disabled means no discovery, request
/// file, subprocess, or filesystem mutation. Direct delegates one already
/// validated request to the processkit-owned CLI and returns its opaque result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProcesskitExecution {
    Disabled,
    Direct {
        cli: PathBuf,
        request: Box<InstallerRequest>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum ProcesskitExecutionResult {
    Disabled,
    Delegated(Box<InstallerResult>),
}

pub fn execute(execution: &ProcesskitExecution) -> Result<ProcesskitExecutionResult> {
    match execution {
        ProcesskitExecution::Disabled => Ok(ProcesskitExecutionResult::Disabled),
        ProcesskitExecution::Direct { cli, request } => Ok(ProcesskitExecutionResult::Delegated(
            Box::new(invoke(cli, request)?),
        )),
    }
}

pub fn discover_cli(path: Option<&Path>) -> CliAvailability {
    match path {
        Some(path) if path.is_file() => CliAvailability::Available(path.to_path_buf()),
        Some(_) => CliAvailability::Unavailable,
        None => std::env::var_os("PATH")
            .and_then(|paths| {
                std::env::split_paths(&paths)
                    .map(|dir| dir.join("processkit"))
                    .find(|candidate| candidate.is_file())
            })
            .map_or(CliAvailability::Unavailable, CliAvailability::Available),
    }
}

/// Invoke the producer using its request-file boundary.
///
/// The temporary request is private, is deleted when this function returns,
/// and is never rendered into a shell command or retained as release evidence.
pub fn invoke(cli: &Path, request: &InstallerRequest) -> Result<InstallerResult> {
    invoke_with_environment(cli, request, &[])
}

fn invoke_with_environment(
    cli: &Path,
    request: &InstallerRequest,
    environment: &[(&str, &str)],
) -> Result<InstallerResult> {
    request.validate()?;
    let mut request_file =
        NamedTempFile::new().context("create private processkit installer request")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        request_file
            .as_file()
            .set_permissions(std::fs::Permissions::from_mode(0o600))
            .context("restrict processkit installer request permissions")?;
    }
    serde_json::to_writer(request_file.as_file_mut(), request)
        .context("write processkit installer request")?;
    request_file
        .as_file_mut()
        .sync_all()
        .context("flush processkit installer request")?;

    let output = Command::new(cli)
        .arg("execute")
        .arg("--request")
        .arg(request_file.path())
        .envs(environment.iter().copied())
        .output()
        .with_context(|| format!("invoke processkit CLI at {}", cli.display()))?;
    decode_output(&output.status, &output.stdout)
}

/// Recover an interrupted target and retry the original operation once.
pub fn recover_then_retry(
    cli: &Path,
    interrupted_request: &InstallerRequest,
) -> Result<InstallerResult> {
    let recovery = InstallerRequest::local(
        InstallerOperation::Recover,
        interrupted_request.root.clone(),
    );
    let recovery_result = invoke(cli, &recovery)?;
    if !recovery_result.status.is_success() {
        bail!(
            "processkit recovery did not succeed: {:?}",
            recovery_result.status
        );
    }
    invoke(cli, interrupted_request)
}

fn decode_output(status: &ExitStatus, stdout: &[u8]) -> Result<InstallerResult> {
    let result: InstallerResult =
        serde_json::from_slice(stdout).context("malformed processkit CLI result JSON")?;
    if result.api_version != PROCESSKIT_INSTALLER_PROTOCOL_V1ALPHA1 {
        bail!(
            "incompatible processkit installer result apiVersion: {}",
            result.api_version
        );
    }
    if status.success() != result.status.is_success() {
        bail!(
            "processkit exit status and structured status disagree: exit={} status={:?}",
            status,
            result.status
        );
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tempfile::TempDir;

    static FAKE_CLI_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

    fn request() -> InstallerRequest {
        let mut request = InstallerRequest::development(
            InstallerOperation::Install,
            PathBuf::from("/project"),
            PathBuf::from("/release"),
        );
        request.profiles = vec!["managed".into()];
        request.harnesses = vec!["codex".into()];
        request
    }

    fn fake_cli(dir: &TempDir, body: &str) -> PathBuf {
        let sequence = FAKE_CLI_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = dir.path().join(format!("processkit-{sequence}"));
        fs::write(&path, format!("#!/bin/sh\nset -eu\n{body}\n")).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        }
        path
    }

    fn result(status: &str) -> String {
        format!(
            "{{\"apiVersion\":\"{PROCESSKIT_INSTALLER_PROTOCOL_V1ALPHA1}\",\
             \"status\":\"{status}\",\"changes\":[],\"conflicts\":[],\
             \"warnings\":[],\"errors\":[]}}"
        )
    }

    fn execute_direct(cli: &Path, request: InstallerRequest) -> InstallerResult {
        match execute(&ProcesskitExecution::Direct {
            cli: cli.to_path_buf(),
            request: Box::new(request),
        })
        .unwrap()
        {
            ProcesskitExecutionResult::Delegated(result) => *result,
            ProcesskitExecutionResult::Disabled => panic!("direct execution was skipped"),
        }
    }

    #[test]
    fn request_matches_producer_execute_contract() {
        let request = request();
        request.validate().unwrap();
        let value = serde_json::to_value(request).unwrap();
        assert_eq!(value["apiVersion"], PROCESSKIT_INSTALLER_PROTOCOL_V1ALPHA1);
        assert_eq!(value["operation"], "install");
        assert_eq!(value["profiles"], serde_json::json!(["managed"]));
        assert_eq!(value["harnesses"], serde_json::json!(["codex"]));
        assert_eq!(value["yes"], true);
        assert!(value.get("enabled").is_none());
        assert!(value.get("source").is_none());
        assert!(value.get("environment").is_none());
    }

    #[test]
    fn request_rejects_mixed_or_incomplete_release_inputs() {
        let mut mixed = request();
        mixed.envelope_path = Some("/envelope".into());
        mixed.signature_path = Some("/signature".into());
        mixed.trust_store_path = Some("/trust".into());
        assert!(mixed.validate().is_err());

        let mut incomplete = request();
        incomplete.distribution_path = None;
        incomplete.envelope_path = Some("/envelope".into());
        assert!(incomplete.validate().is_err());
    }

    #[test]
    fn availability_discovery_is_explicit() {
        let dir = TempDir::new().unwrap();
        let cli = fake_cli(&dir, "exit 0");
        assert_eq!(discover_cli(Some(&cli)), CliAvailability::Available(cli));
        assert_eq!(
            discover_cli(Some(&dir.path().join("missing"))),
            CliAvailability::Unavailable
        );
    }

    #[test]
    fn invocation_uses_execute_request_file_and_removes_it() {
        let dir = TempDir::new().unwrap();
        let argv_log = dir.path().join("argv");
        let request_copy = dir.path().join("request.json");
        let cli = fake_cli(
            &dir,
            &format!(
                "printf '%s\\n' \"$@\" > '{}'\ncp \"$3\" '{}'\nprintf '%s' '{}'",
                argv_log.display(),
                request_copy.display(),
                result("installed")
            ),
        );
        let got = invoke(&cli, &request()).unwrap();
        assert_eq!(got.status, InstallerStatus::Installed);
        let argv = fs::read_to_string(argv_log).unwrap();
        let args: Vec<_> = argv.lines().collect();
        assert_eq!(args[0], "execute");
        assert_eq!(args[1], "--request");
        assert!(!Path::new(args[2]).exists());
        let wire: InstallerRequest =
            serde_json::from_slice(&fs::read(request_copy).unwrap()).unwrap();
        assert_eq!(wire, request());
    }

    #[test]
    fn disabled_boundary_performs_no_discovery_or_process_invocation() {
        let result = execute(&ProcesskitExecution::Disabled).unwrap();
        assert_eq!(result, ProcesskitExecutionResult::Disabled);
    }

    #[test]
    fn direct_boundary_matches_the_opaque_protocol_outcome() {
        let dir = TempDir::new().unwrap();
        let calls = dir.path().join("calls");
        let cli = fake_cli(
            &dir,
            &format!(
                "printf called >> '{}'\nprintf '%s' '{}'",
                calls.display(),
                result("installed")
            ),
        );
        let request = request();
        let delegated = execute(&ProcesskitExecution::Direct {
            cli,
            request: Box::new(request.clone()),
        })
        .unwrap();
        assert_eq!(
            delegated,
            ProcesskitExecutionResult::Delegated(Box::new(InstallerResult {
                api_version: PROCESSKIT_INSTALLER_PROTOCOL_V1ALPHA1.into(),
                status: InstallerStatus::Installed,
                changes: Vec::new(),
                conflicts: Vec::new(),
                warnings: Vec::new(),
                errors: Vec::new(),
                state: None,
                checked: None,
                provenance: None,
                extensions: serde_json::Map::new(),
            }))
        );
        assert_eq!(fs::read_to_string(calls).unwrap(), "called");
    }

    #[test]
    fn v1_boundary_contains_no_legacy_distribution_or_layout_policy() {
        let source = include_str!("processkit_protocol.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("production source precedes tests");
        for forbidden in [
            "processkit_vocab",
            "content_install",
            "content_source",
            "context/templates/processkit",
            ".processkit-version",
            "AGENTS.md",
            "SKILL.md",
            "mcp-config.json",
        ] {
            assert!(
                !production.contains(forbidden),
                "v1 opaque boundary must not contain legacy policy token {forbidden}"
            );
        }
    }

    #[test]
    fn result_accepts_forward_compatible_extensions() {
        let mut value: Value = serde_json::from_str(&result("verified")).unwrap();
        value["producerExtension"] = serde_json::json!({"new": true});
        let parsed: InstallerResult = serde_json::from_value(value).unwrap();
        assert_eq!(parsed.status, InstallerStatus::Verified);
        assert_eq!(
            parsed.extensions["producerExtension"],
            serde_json::json!({"new": true})
        );
    }

    #[test]
    fn malformed_incompatible_and_exit_status_mismatch_fail() {
        let dir = TempDir::new().unwrap();
        for body in [
            "printf nope",
            "printf '%s' '{\"apiVersion\":\"other/v1\",\"status\":\"installed\",\"changes\":[],\"conflicts\":[],\"warnings\":[],\"errors\":[]}'",
            &format!("printf '%s' '{}'; exit 3", result("installed")),
            &format!("printf '%s' '{}'", result("invalid")),
        ] {
            let cli = fake_cli(&dir, body);
            assert!(invoke(&cli, &request()).is_err());
        }
    }

    #[test]
    fn recover_then_retry_uses_the_only_supported_interruption_path() {
        let dir = TempDir::new().unwrap();
        let calls = dir.path().join("calls");
        let cli = fake_cli(
            &dir,
            &format!(
                "operation=$(sed -n 's/.*\"operation\":\"\\([^\"]*\\)\".*/\\1/p' \"$3\")\nprintf '%s\\n' \"$operation\" >> '{}'\nif [ \"$operation\" = recover ]; then printf '%s' '{}'; else printf '%s' '{}'; fi",
                calls.display(),
                result("recovered"),
                result("installed")
            ),
        );
        assert_eq!(
            recover_then_retry(&cli, &request()).unwrap().status,
            InstallerStatus::Installed
        );
        assert_eq!(fs::read_to_string(calls).unwrap(), "recover\ninstall\n");
    }

    #[test]
    fn real_producer_interruption_recovery_lock_and_secret_safety_when_configured() {
        let Some(cli) = std::env::var_os("AIBOX_PROCESSKIT_V1_TEST_CLI") else {
            return;
        };
        let (install, project) = signed_or_development_install();
        let cli = Path::new(&cli);
        let v0_bridge = project.path().join(".aibox-v0-bridge-canary");
        fs::write(&v0_bridge, "retain v0 bridge\n").unwrap();

        let interruption = invoke_with_environment(
            cli,
            &install,
            &[("PROCESSKIT_INSTALLER_FAIL_AFTER_ACTION", "0")],
        );
        assert!(
            interruption.is_err(),
            "the producer failpoint must interrupt mutation"
        );
        assert!(
            project.path().join(".processkit/transactions").is_dir(),
            "an interrupted mutation must leave durable recovery evidence"
        );
        assert_eq!(
            invoke(cli, &install).unwrap().status,
            InstallerStatus::Invalid,
            "normal retry must be refused until recovery"
        );
        assert_eq!(
            invoke(
                cli,
                &InstallerRequest::local(InstallerOperation::Recover, project.path().to_path_buf())
            )
            .unwrap()
            .status,
            InstallerStatus::Recovered
        );
        assert!(
            !project.path().join(".processkit/transactions").exists()
                || fs::read_dir(project.path().join(".processkit/transactions"))
                    .unwrap()
                    .next()
                    .is_none(),
            "recovery must leave no ambiguous transaction journal"
        );
        assert!(
            v0_bridge.is_file(),
            "v1 recovery must not modify the v0 bridge"
        );

        let installed = invoke_with_environment(
            cli,
            &install,
            &[("AIBOX_PROCESSKIT_SECRET_CANARY", "v1-secret-canary")],
        )
        .unwrap();
        assert_eq!(installed.status, InstallerStatus::Installed);
        let rendered_result = serde_json::to_string(&installed).unwrap();
        assert!(
            !rendered_result.contains("v1-secret-canary"),
            "producer result must not disclose inherited secret canaries"
        );
        let state = fs::read_to_string(project.path().join(".processkit/state.json")).unwrap();
        assert!(
            !state.contains("v1-secret-canary"),
            "installation state must not retain inherited secret canaries"
        );

        let lock = project.path().join(".processkit/lock");
        fs::write(&lock, format!("test:{}\n", std::process::id())).unwrap();
        let mut update = install.clone();
        update.operation = InstallerOperation::Update;
        let refused = invoke(cli, &update).unwrap();
        assert_eq!(refused.status, InstallerStatus::Invalid);
        assert!(
            serde_json::to_string(&refused)
                .unwrap()
                .contains("another processkit operation is already in progress"),
            "a held target lock must refuse a concurrent mutation"
        );
        fs::remove_file(lock).unwrap();

        assert_eq!(
            invoke(
                cli,
                &InstallerRequest::local(
                    InstallerOperation::Uninstall,
                    project.path().to_path_buf()
                )
            )
            .unwrap()
            .status,
            InstallerStatus::Uninstalled
        );
        assert!(
            v0_bridge.is_file(),
            "v1 uninstall must leave the v0 bridge intact"
        );
    }

    #[test]
    fn real_producer_lifecycle_when_configured() {
        let Some(cli) = std::env::var_os("AIBOX_PROCESSKIT_V1_TEST_CLI") else {
            return;
        };
        let (install, project) = signed_or_development_install();

        let mut plan = install.clone();
        plan.operation = InstallerOperation::Plan;
        plan.yes = false;
        let planned = invoke(Path::new(&cli), &plan).unwrap();
        assert_eq!(planned.status, InstallerStatus::Planned);
        assert!(
            !project.path().join(".processkit").exists(),
            "planning must not create installer state"
        );
        assert!(
            !project.path().join(".mcp.json").exists(),
            "planning must not project harness configuration"
        );

        assert_eq!(
            execute_direct(Path::new(&cli), install.clone()).status,
            InstallerStatus::Installed
        );
        assert!(project.path().join(".processkit/state.json").is_file());
        assert_eq!(
            execute_direct(
                Path::new(&cli),
                InstallerRequest::local(InstallerOperation::Verify, project.path().to_path_buf())
            )
            .status,
            InstallerStatus::Verified
        );

        let mut update = install.clone();
        update.operation = InstallerOperation::Update;
        let updated = execute_direct(Path::new(&cli), update);
        assert_eq!(updated.status, InstallerStatus::Updated);
        assert_eq!(
            updated.changes,
            vec![serde_json::json!({"count": 0})],
            "an unchanged producer update must report zero changes"
        );
        assert_eq!(
            execute_direct(
                Path::new(&cli),
                InstallerRequest::local(
                    InstallerOperation::Uninstall,
                    project.path().to_path_buf()
                )
            )
            .status,
            InstallerStatus::Uninstalled
        );
        assert!(!project.path().join(".mcp.json").exists());
    }

    fn signed_or_development_install() -> (InstallerRequest, TempDir) {
        let project = TempDir::new().unwrap();
        let mut install = match (
            std::env::var_os("AIBOX_PROCESSKIT_V1_TEST_ENVELOPE"),
            std::env::var_os("AIBOX_PROCESSKIT_V1_TEST_SIGNATURE"),
            std::env::var_os("AIBOX_PROCESSKIT_V1_TEST_TRUST_STORE"),
        ) {
            (Some(envelope), Some(signature), Some(trust_store)) => {
                InstallerRequest::signed_release(
                    InstallerOperation::Install,
                    project.path().to_path_buf(),
                    envelope.into(),
                    signature.into(),
                    trust_store.into(),
                )
            }
            (None, None, None) => {
                let distribution = std::env::var_os("AIBOX_PROCESSKIT_V1_TEST_DISTRIBUTION")
                    .expect(
                        "consumer gate requires signed release inputs or \
                         AIBOX_PROCESSKIT_V1_TEST_DISTRIBUTION",
                    );
                InstallerRequest::development(
                    InstallerOperation::Install,
                    project.path().to_path_buf(),
                    distribution.into(),
                )
            }
            _ => panic!("consumer gate signed release inputs must be provided together"),
        };
        install.profiles = vec!["minimal".into()];
        install.harnesses = vec!["codex".into()];
        (install, project)
    }
}
