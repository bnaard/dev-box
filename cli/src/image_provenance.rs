use anyhow::Result;
use serde::Serialize;
use std::path::Path;

use crate::cli::OutputFormat;
use crate::config::{AiboxConfig, COMPOSE_FILE, DOCKERFILE, IMAGE_REGISTRY};

pub const IMAGE_PROVENANCE_POLICY_SCHEMA_VERSION: &str = "aibox.image-provenance-policy.v0-preview";

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct ImageProvenancePolicy {
    pub schema_version: &'static str,
    pub aibox_version: &'static str,
    pub image: ImageReference,
    pub generated_files: GeneratedImageFiles,
    pub runtime_markers: RuntimeImageMarkers,
    pub selected_addons: Vec<String>,
    pub release_phase: ReleasePhasePolicy,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct ImageReference {
    pub registry: &'static str,
    pub flavor: String,
    pub version_pin: String,
    pub tag: Option<String>,
    pub tag_template: String,
    pub mutable_version_pin: bool,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct GeneratedImageFiles {
    pub dockerfile: &'static str,
    pub compose_file: &'static str,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct RuntimeImageMarkers {
    pub docker_label: &'static str,
    pub profile_label: &'static str,
    pub version_file: &'static str,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct ReleasePhasePolicy {
    pub container_phase: &'static str,
    pub host_phase: &'static str,
    pub host_command_template: &'static str,
}

pub fn image_provenance_policy(config: &AiboxConfig) -> ImageProvenancePolicy {
    let flavor = format!("base-{}", config.aibox.base);
    let version_pin = config.aibox.version.clone();
    let mut selected_addons: Vec<String> = config.addons.addons.keys().cloned().collect();
    selected_addons.sort();

    let mutable_version_pin = version_pin == "latest";

    ImageProvenancePolicy {
        schema_version: IMAGE_PROVENANCE_POLICY_SCHEMA_VERSION,
        aibox_version: env!("CARGO_PKG_VERSION"),
        image: ImageReference {
            registry: IMAGE_REGISTRY,
            tag: (!mutable_version_pin).then(|| format!("{}-v{}", flavor, version_pin)),
            tag_template: format!("{}-v{{version}}", flavor),
            flavor,
            mutable_version_pin,
            version_pin,
        },
        generated_files: GeneratedImageFiles {
            dockerfile: DOCKERFILE,
            compose_file: COMPOSE_FILE,
        },
        runtime_markers: RuntimeImageMarkers {
            docker_label: "aibox.version",
            profile_label: "aibox.profile",
            version_file: "/etc/aibox-version",
        },
        selected_addons,
        release_phase: ReleasePhasePolicy {
            container_phase: "./scripts/maintain.sh release <version>",
            host_phase: "./scripts/maintain.sh release-host <version>",
            host_command_template: "./scripts/maintain.sh release-host {version}",
        },
    }
}

pub fn cmd_image_provenance_policy(
    config_path: &Option<String>,
    format: OutputFormat,
) -> Result<()> {
    let config = AiboxConfig::from_cli_option(config_path)?;
    let policy = image_provenance_policy(&config);

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&policy)?);
        }
        OutputFormat::Yaml => {
            print!("{}", serde_yaml::to_string(&policy)?);
        }
        OutputFormat::Table => {
            println!("Image provenance policy");
            println!("  Schema:      {}", policy.schema_version);
            println!("  aibox:       {}", policy.aibox_version);
            println!(
                "  Image:       {}",
                policy
                    .image
                    .tag
                    .as_deref()
                    .unwrap_or(&policy.image.tag_template)
            );
            println!("  Registry:    {}", policy.image.registry);
            println!("  Label:       {}", policy.runtime_markers.docker_label);
            println!("  Profile:     {}", policy.runtime_markers.profile_label);
            println!("  Version:     {}", policy.runtime_markers.version_file);
            println!("  Addons:      {}", policy.selected_addons.len());
            println!();
            println!(
                "Use `aibox describe image-provenance-policy -o json` for the machine-readable projection."
            );
        }
    }

    Ok(())
}

pub fn image_provenance_warnings(config: &AiboxConfig, project_root: &Path) -> Vec<String> {
    let policy = image_provenance_policy(config);
    let mut warnings = Vec::new();

    if policy.image.mutable_version_pin {
        warnings.push(
            "image-provenance-mutable-version: [aibox].version is \"latest\"; generated Dockerfile must resolve this to a concrete image tag before build"
                .to_string(),
        );
    }

    let dockerfile_path = project_root.join(policy.generated_files.dockerfile);
    let Ok(dockerfile) = std::fs::read_to_string(&dockerfile_path) else {
        warnings.push(format!(
            "image-provenance-dockerfile-missing: {} is missing; run 'aibox apply'",
            policy.generated_files.dockerfile
        ));
        return warnings;
    };

    if dockerfile.contains("-vlatest") {
        warnings.push(
            "image-provenance-mutable-tag-written: generated Dockerfile references a mutable vlatest tag; run 'aibox apply' with network access or pin [aibox].version"
                .to_string(),
        );
    }

    if let Some(tag) = policy.image.tag.as_deref() {
        let expected_from = format!("FROM {}:{}", policy.image.registry, tag);
        if !dockerfile.contains(&expected_from) {
            warnings.push(format!(
                "image-provenance-tag-mismatch: generated Dockerfile does not reference expected image tag {}",
                tag
            ));
        }
    }

    let label_prefix = format!("LABEL {}=", policy.runtime_markers.docker_label);
    let label_line = dockerfile
        .lines()
        .find(|line| line.trim_start().starts_with(&label_prefix));
    match label_line {
        Some(line)
            if !policy.image.mutable_version_pin
                && !line.contains(&format!("\"{}\"", policy.image.version_pin)) =>
        {
            warnings.push(format!(
                "image-provenance-label-mismatch: generated Dockerfile label {} does not match [aibox].version {}",
                policy.runtime_markers.docker_label, policy.image.version_pin
            ));
        }
        Some(_) => {}
        None => warnings.push(format!(
            "image-provenance-label-missing: generated Dockerfile is missing LABEL {}",
            policy.runtime_markers.docker_label
        )),
    }

    let profile_label_prefix = format!("LABEL {}=", policy.runtime_markers.profile_label);
    let profile_label_line = dockerfile
        .lines()
        .find(|line| line.trim_start().starts_with(&profile_label_prefix));
    match profile_label_line {
        Some(line) if !line.contains(&format!("\"{}\"", config.aibox.profile)) => {
            warnings.push(format!(
                "image-provenance-profile-label-mismatch: generated Dockerfile label {} does not match [aibox].profile {}",
                policy.runtime_markers.profile_label, config.aibox.profile
            ));
        }
        Some(_) => {}
        None => warnings.push(format!(
            "image-provenance-profile-label-missing: generated Dockerfile is missing LABEL {}",
            policy.runtime_markers.profile_label
        )),
    }

    if !dockerfile.contains(policy.runtime_markers.version_file) {
        warnings.push(format!(
            "image-provenance-version-file-missing: generated Dockerfile does not write {}",
            policy.runtime_markers.version_file
        ));
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AiboxConfig;
    use std::fs;

    #[test]
    fn image_provenance_policy_reports_tag_markers_and_sorted_addons() {
        let config = AiboxConfig::from_str(
            r#"[aibox]
version = "0.22.0"
base = "debian"

[container]
name = "demo"

[ai]
harnesses = []

[addons.zeta.tools]
z = {}

[addons.alpha.tools]
a = {}
"#,
        )
        .unwrap();

        let policy = image_provenance_policy(&config);

        assert_eq!(
            policy.schema_version,
            IMAGE_PROVENANCE_POLICY_SCHEMA_VERSION
        );
        assert_eq!(policy.image.registry, "ghcr.io/projectious-work/aibox");
        assert_eq!(policy.image.flavor, "base-debian");
        assert_eq!(policy.image.tag.as_deref(), Some("base-debian-v0.22.0"));
        assert_eq!(policy.image.tag_template, "base-debian-v{version}");
        assert!(!policy.image.mutable_version_pin);
        assert_eq!(policy.runtime_markers.docker_label, "aibox.version");
        assert_eq!(policy.runtime_markers.profile_label, "aibox.profile");
        assert_eq!(policy.runtime_markers.version_file, "/etc/aibox-version");
        assert_eq!(
            policy.selected_addons,
            vec!["alpha".to_string(), "zeta".to_string()]
        );
    }

    #[test]
    fn image_provenance_warnings_detect_dockerfile_drift() {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join(".devcontainer")).unwrap();
        fs::write(
            tmp.path().join(".devcontainer/Dockerfile"),
            r#"FROM ghcr.io/projectious-work/aibox:base-debian-v0.21.0 AS aibox
LABEL aibox.version="0.21.0"
LABEL aibox.profile="headless-runner"
"#,
        )
        .unwrap();
        let config = AiboxConfig::from_str(
            r#"[aibox]
version = "0.22.0"

[container]
name = "demo"

[ai]
harnesses = []
"#,
        )
        .unwrap();

        let warnings = image_provenance_warnings(&config, tmp.path());

        assert!(
            warnings
                .iter()
                .any(|warning| warning.contains("image-provenance-tag-mismatch"))
        );
        assert!(
            warnings
                .iter()
                .any(|warning| warning.contains("image-provenance-label-mismatch"))
        );
        assert!(
            warnings
                .iter()
                .any(|warning| warning.contains("image-provenance-profile-label-mismatch"))
        );
        assert!(
            warnings
                .iter()
                .any(|warning| warning.contains("image-provenance-version-file-missing"))
        );
    }

    #[test]
    fn image_provenance_warnings_detect_latest_written_to_dockerfile() {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join(".devcontainer")).unwrap();
        fs::write(
            tmp.path().join(".devcontainer/Dockerfile"),
            r#"FROM ghcr.io/projectious-work/aibox:base-debian-vlatest AS aibox
LABEL aibox.version="latest"
RUN echo "latest" > /etc/aibox-version
"#,
        )
        .unwrap();
        let config = AiboxConfig::from_str(
            r#"[aibox]
version = "latest"

[container]
name = "demo"

[ai]
harnesses = []
"#,
        )
        .unwrap();

        let warnings = image_provenance_warnings(&config, tmp.path());

        assert!(
            warnings
                .iter()
                .any(|warning| warning.contains("image-provenance-mutable-version"))
        );
        assert!(
            warnings
                .iter()
                .any(|warning| warning.contains("image-provenance-mutable-tag-written"))
        );
    }
}
