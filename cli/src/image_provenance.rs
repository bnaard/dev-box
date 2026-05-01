use anyhow::Result;
use serde::Serialize;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AiboxConfig;

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
        assert_eq!(policy.runtime_markers.version_file, "/etc/aibox-version");
        assert_eq!(
            policy.selected_addons,
            vec!["alpha".to_string(), "zeta".to_string()]
        );
    }
}
