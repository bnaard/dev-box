//! Provisional, producer-gated processkit CLI protocol boundary.
//!
//! This module owns only opaque intent, typed command arguments, and result
//! provenance.  It deliberately has no knowledge of processkit layouts,
//! skills, packages, templates, migrations, or harness projections.  The
//! fixture schema is frozen for joint review with processkit#118; production
//! use remains disabled until a compatible producer release is available.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

pub const PROCESSKIT_INSTALL_PROTOCOL_V1ALPHA1: &str =
    "processkit.projectious.work/install/v1alpha1";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InstallRequest {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    #[serde(default)]
    pub harnesses: Vec<String>,
    pub root: PathBuf,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub environment: BTreeMap<String, String>,
}

impl InstallRequest {
    pub fn new(enabled: bool, root: PathBuf) -> Self {
        Self {
            api_version: PROCESSKIT_INSTALL_PROTOCOL_V1ALPHA1.into(),
            enabled,
            source: None,
            channel: None,
            version: None,
            profile: None,
            harnesses: Vec::new(),
            root,
            environment: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InstallResult {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub outcome: InstallOutcome,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ProtocolError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<InstallProvenance>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InstallOutcome {
    Succeeded,
    Noop,
    Failed,
    Interrupted,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProtocolError {
    pub code: String,
    pub retryable: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InstallProvenance {
    pub producer_version: String,
    pub invocation_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CliAvailability {
    Available(PathBuf),
    Unavailable,
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

/// Invokes the producer through an argv vector.  The request JSON is passed as
/// one argument; no shell is involved and no command string is persisted.
pub fn invoke(cli: &Path, request: &InstallRequest, retry_once: bool) -> Result<InstallResult> {
    let argv = request_argv(request)?;
    let attempts = if retry_once { 2 } else { 1 };
    for attempt in 0..attempts {
        let output = Command::new(cli)
            .args(&argv)
            .output()
            .with_context(|| format!("invoke processkit CLI at {}", cli.display()))?;
        let result = decode_output(&output.status, &output.stdout)?;
        if !(result.outcome == InstallOutcome::Failed
            && result.error.as_ref().is_some_and(|error| error.retryable)
            && attempt + 1 < attempts)
        {
            return Ok(result);
        }
    }
    unreachable!("attempt count is always positive")
}

pub fn request_argv(request: &InstallRequest) -> Result<Vec<OsString>> {
    if request.api_version != PROCESSKIT_INSTALL_PROTOCOL_V1ALPHA1 {
        bail!(
            "unsupported processkit install request apiVersion: {}",
            request.api_version
        );
    }
    // Environment facts are optional producer context, never a channel for
    // credentials. Preserve non-secret facts while excluding conventional
    // credential names before serialisation or process execution.
    let mut wire = request.clone();
    wire.environment
        .retain(|name, _| !is_secret_environment_name(name));
    Ok(vec![
        "install".into(),
        "--request-json".into(),
        serde_json::to_string(&wire)?.into(),
        "--output".into(),
        "json".into(),
    ])
}

fn is_secret_environment_name(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();
    ["TOKEN", "SECRET", "PASSWORD", "CREDENTIAL", "API_KEY"]
        .iter()
        .any(|marker| upper.contains(marker))
}

fn decode_output(status: &ExitStatus, stdout: &[u8]) -> Result<InstallResult> {
    let result: InstallResult =
        serde_json::from_slice(stdout).context("malformed processkit CLI result JSON")?;
    if result.api_version != PROCESSKIT_INSTALL_PROTOCOL_V1ALPHA1 {
        bail!(
            "incompatible processkit install result apiVersion: {}",
            result.api_version
        );
    }
    if !status.success()
        && result.outcome != InstallOutcome::Interrupted
        && result.outcome != InstallOutcome::Failed
    {
        bail!("processkit CLI exited unsuccessfully without a failed or interrupted result");
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    const CANARY: &str = "M5_SECRET_CANARY_DO_NOT_LEAK";
    fn fixture(name: &str) -> &'static str {
        match name {
            "valid" => include_str!("../tests/fixtures/processkit-protocol/valid-result.json"),
            "invalid" => include_str!("../tests/fixtures/processkit-protocol/invalid-result.json"),
            _ => unreachable!(),
        }
    }
    fn request() -> InstallRequest {
        let mut r = InstallRequest::new(true, PathBuf::from("/project"));
        r.source = Some("https://example.invalid/processkit".into());
        r.harnesses = vec!["codex".into()];
        r.environment.insert("PATH".into(), "/bin".into());
        r
    }
    fn fake_cli(dir: &TempDir, body: &str) -> PathBuf {
        let path = dir.path().join("processkit");
        fs::write(&path, format!("#!/bin/sh\n{}\n", body)).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        }
        path
    }

    #[test]
    fn valid_fixture_round_trips() {
        let got: InstallResult = serde_json::from_str(fixture("valid")).unwrap();
        assert_eq!(got.outcome, InstallOutcome::Succeeded);
    }
    #[test]
    fn invalid_fixture_is_rejected() {
        assert!(serde_json::from_str::<InstallResult>(fixture("invalid")).is_err());
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
    fn typed_argv_never_contains_secret_canary() {
        let mut r = request();
        r.environment.insert("TOKEN".into(), CANARY.into());
        let encoded = request_argv(&r).unwrap();
        assert!(
            !encoded
                .iter()
                .any(|arg| arg.to_string_lossy().contains(CANARY))
        );
    }
    #[test]
    fn success_noop_and_interruption_are_preserved() {
        let dir = TempDir::new().unwrap();
        for (outcome, exit) in [("succeeded", 0), ("noop", 0), ("interrupted", 130)] {
            let cli = fake_cli(
                &dir,
                &format!(
                    "printf '%s' '{{\"apiVersion\":\"{}\",\"outcome\":\"{}\"}}'; exit {}",
                    PROCESSKIT_INSTALL_PROTOCOL_V1ALPHA1, outcome, exit
                ),
            );
            assert_eq!(
                invoke(&cli, &request(), false).unwrap().outcome,
                match outcome {
                    "succeeded" => InstallOutcome::Succeeded,
                    "noop" => InstallOutcome::Noop,
                    _ => InstallOutcome::Interrupted,
                }
            );
        }
    }
    #[test]
    fn retryable_failure_retries_once() {
        let dir = TempDir::new().unwrap();
        let mark = dir.path().join("attempt");
        let cli = fake_cli(
            &dir,
            &format!(
                "if [ -e '{}' ]; then printf '%s' '{{\"apiVersion\":\"{}\",\"outcome\":\"succeeded\"}}'; else touch '{}'; printf '%s' '{{\"apiVersion\":\"{}\",\"outcome\":\"failed\",\"error\":{{\"code\":\"temporary\",\"retryable\":true}}}}'; exit 1; fi",
                mark.display(),
                PROCESSKIT_INSTALL_PROTOCOL_V1ALPHA1,
                mark.display(),
                PROCESSKIT_INSTALL_PROTOCOL_V1ALPHA1
            ),
        );
        assert_eq!(
            invoke(&cli, &request(), true).unwrap().outcome,
            InstallOutcome::Succeeded
        );
    }
    #[test]
    fn malformed_and_incompatible_results_fail() {
        let dir = TempDir::new().unwrap();
        for body in [
            "printf nope",
            "printf '%s' '{\"apiVersion\":\"other/v1\",\"outcome\":\"succeeded\"}'",
        ] {
            let cli = fake_cli(&dir, body);
            assert!(invoke(&cli, &request(), false).is_err());
        }
    }
}
