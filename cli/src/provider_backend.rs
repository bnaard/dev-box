use anyhow::Result;
use serde::Serialize;
use std::collections::BTreeSet;

use crate::addon_loader::LoadedAddon;
use crate::cli::OutputFormat;
use crate::config::{AiHarness, AiboxConfig};

pub const PROVIDER_BACKEND_SCHEMA_VERSION: &str = "aibox.provider-backends.v0-preview";

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct ProviderBackendIndex {
    pub schema_version: &'static str,
    pub aibox_version: &'static str,
    pub selected_backends: Vec<String>,
    pub backends: Vec<ProviderBackend>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct ProviderBackend {
    pub name: String,
    pub display_name: &'static str,
    pub selected: bool,
    pub container_cli: bool,
    pub binary_name: Option<&'static str>,
    pub addon_name: Option<String>,
    pub addon_available: bool,
    pub mcp_client: bool,
    pub mcp_config_target: Option<&'static str>,
    pub permission_target: Option<&'static str>,
    pub notes: Vec<&'static str>,
}

pub fn provider_backend_index(
    config: &AiboxConfig,
    addons: &[LoadedAddon],
) -> ProviderBackendIndex {
    let selected: BTreeSet<String> = config
        .ai
        .effective_harnesses()
        .iter()
        .filter(|harness| harness.is_active())
        .map(ToString::to_string)
        .collect();
    let addon_names: BTreeSet<&str> = addons.iter().map(|addon| addon.name.as_str()).collect();

    let backends = AiHarness::all()
        .iter()
        .map(|harness| provider_backend(harness, &selected, &addon_names))
        .collect();

    ProviderBackendIndex {
        schema_version: PROVIDER_BACKEND_SCHEMA_VERSION,
        aibox_version: env!("CARGO_PKG_VERSION"),
        selected_backends: selected.into_iter().collect(),
        backends,
    }
}

pub fn cmd_provider_backends(config_path: &Option<String>, format: OutputFormat) -> Result<()> {
    let config = AiboxConfig::from_cli_option(config_path)?;
    let index = provider_backend_index(&config, crate::addon_loader::all_addons());

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&index)?);
        }
        OutputFormat::Yaml => {
            print!("{}", serde_yaml::to_string(&index)?);
        }
        OutputFormat::Table => {
            println!("Provider backends");
            println!("  Schema:      {}", index.schema_version);
            println!("  aibox:       {}", index.aibox_version);
            println!("  Backends:    {}", index.backends.len());
            println!("  Selected:    {}", index.selected_backends.len());
            println!();
            println!(
                "Use `aibox describe provider-backends -o json` for the machine-readable projection."
            );
        }
    }

    Ok(())
}

pub fn provider_backend_warnings(config: &AiboxConfig, addons: &[LoadedAddon]) -> Vec<String> {
    let index = provider_backend_index(config, addons);
    let mut warnings = Vec::new();

    for backend in index.backends.iter().filter(|backend| backend.selected) {
        if let Some(addon_name) = &backend.addon_name
            && !backend.addon_available
        {
            warnings.push(format!(
                "provider-backend-addon-missing: {} expects addon {} but it is not available",
                backend.name, addon_name
            ));
        }

        if !backend.mcp_client {
            warnings.push(format!(
                "provider-backend-mcp-unavailable: {} does not have a built-in MCP client; processkit MCP tools will not be available in that backend",
                backend.name
            ));
        }

        if backend.mcp_client && backend.permission_target.is_none() {
            warnings.push(format!(
                "provider-backend-permissions-missing: {} can receive MCP registrations but has no aibox permission projection yet",
                backend.name
            ));
        }

        if config.aibox.profile.as_str() == "headless-runner" && !backend.container_cli {
            warnings.push(format!(
                "provider-backend-headless-mismatch: {} is host-side only and cannot run inside a headless container",
                backend.name
            ));
        }
    }

    warnings
}

fn provider_backend(
    harness: &AiHarness,
    selected: &BTreeSet<String>,
    addon_names: &BTreeSet<&str>,
) -> ProviderBackend {
    let name = harness.to_string();
    let addon_name = addon_name(harness);
    let addon_available = addon_name
        .as_ref()
        .is_some_and(|name| addon_names.contains(name.as_str()));

    ProviderBackend {
        name: name.clone(),
        display_name: harness.display_name(),
        selected: selected.contains(&name),
        container_cli: has_container_cli(harness),
        binary_name: has_container_cli(harness).then_some(harness.binary_name()),
        addon_name,
        addon_available,
        mcp_client: has_mcp_client(harness),
        mcp_config_target: mcp_config_target(harness),
        permission_target: permission_target(harness),
        notes: notes(harness),
    }
}

fn addon_name(harness: &AiHarness) -> Option<String> {
    match harness {
        AiHarness::Cursor => None,
        _ => Some(harness.addon_name()).filter(|name| !name.is_empty()),
    }
}

fn has_container_cli(harness: &AiHarness) -> bool {
    !matches!(harness, AiHarness::Cursor)
}

fn has_mcp_client(harness: &AiHarness) -> bool {
    !matches!(harness, AiHarness::Aider)
}

fn mcp_config_target(harness: &AiHarness) -> Option<&'static str> {
    match harness {
        AiHarness::Claude | AiHarness::Copilot | AiHarness::OpenCode | AiHarness::Hermes => {
            Some(".mcp.json")
        }
        AiHarness::Cursor => Some(".cursor/mcp.json"),
        AiHarness::Gemini => Some(".gemini/settings.json"),
        AiHarness::Codex => Some(".codex/config.toml"),
        AiHarness::Continue => Some(".continue/mcpServers/"),
        AiHarness::Aider => None,
        AiHarness::Mistral => Some(".mcp.json"),
    }
}

fn permission_target(harness: &AiHarness) -> Option<&'static str> {
    match harness {
        AiHarness::Claude => Some(".claude/settings.local.json"),
        AiHarness::Codex => Some(".codex/config.toml"),
        AiHarness::Gemini => Some(".gemini/settings.json"),
        AiHarness::Aider => Some(".aider/mcp-permissions.json"),
        AiHarness::Continue => Some(".continue/config.json"),
        AiHarness::Cursor => Some(".cursor/settings.json"),
        AiHarness::Copilot => Some(".copilot-env"),
        AiHarness::OpenCode => Some(".opencode/config.toml"),
        AiHarness::Hermes | AiHarness::Mistral => None,
    }
}

fn notes(harness: &AiHarness) -> Vec<&'static str> {
    match harness {
        AiHarness::Aider => vec!["no built-in MCP client; aibox emits permission fallback only"],
        AiHarness::Cursor => vec!["host-side IDE backend; no in-container CLI addon"],
        AiHarness::Hermes => vec!["uses .mcp.json registration; no permission projection yet"],
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::addon_loader::{AddonExportSurface, AddonProfile, LoadedAddon};
    use crate::config::AiboxConfig;

    fn addon(name: &str) -> LoadedAddon {
        LoadedAddon {
            name: name.to_string(),
            category: String::new(),
            description: String::new(),
            addon_version: String::new(),
            requires: Vec::new(),
            profile_intent: None,
            usage_class: None,
            profiles: vec![AddonProfile::HumanDev],
            exported_surfaces: vec![AddonExportSurface::CliBinary],
            builder_weight: None,
            tools: Vec::new(),
            builder_template: None,
            runtime_template: None,
        }
    }

    #[test]
    fn provider_backend_index_marks_selected_and_host_only_backends() {
        let config = AiboxConfig::from_str(
            r#"[aibox]
version = "0.22.0"

[container]
name = "demo"

[ai]
harnesses = ["cursor", "codex", "aider"]
"#,
        )
        .unwrap();
        let addons = vec![addon("ai-codex"), addon("ai-aider")];
        let index = provider_backend_index(&config, &addons);

        assert_eq!(index.schema_version, PROVIDER_BACKEND_SCHEMA_VERSION);
        assert_eq!(
            index.selected_backends,
            vec![
                "aider".to_string(),
                "codex".to_string(),
                "cursor".to_string()
            ]
        );

        let cursor = index
            .backends
            .iter()
            .find(|backend| backend.name == "cursor")
            .unwrap();
        assert!(cursor.selected);
        assert!(!cursor.container_cli);
        assert_eq!(cursor.addon_name, None);
        assert_eq!(cursor.mcp_config_target, Some(".cursor/mcp.json"));

        let aider = index
            .backends
            .iter()
            .find(|backend| backend.name == "aider")
            .unwrap();
        assert!(aider.selected);
        assert!(!aider.mcp_client);
        assert_eq!(aider.permission_target, Some(".aider/mcp-permissions.json"));

        let codex = index
            .backends
            .iter()
            .find(|backend| backend.name == "codex")
            .unwrap();
        assert!(codex.addon_available);
        assert_eq!(codex.binary_name, Some("codex"));
    }

    #[test]
    fn provider_backend_warnings_detect_automation_mismatches() {
        let config = AiboxConfig::from_str(
            r#"[aibox]
version = "0.22.0"
profile = "headless-runner"

[container]
name = "demo"

[ai]
harnesses = ["cursor", "aider", "hermes"]
"#,
        )
        .unwrap();
        let addons = vec![addon("ai-aider"), addon("ai-hermes")];
        let warnings = provider_backend_warnings(&config, &addons);

        assert!(
            warnings
                .iter()
                .any(|warning| warning.contains("provider-backend-headless-mismatch: cursor"))
        );
        assert!(
            warnings
                .iter()
                .any(|warning| warning.contains("provider-backend-mcp-unavailable: aider"))
        );
        assert!(
            warnings
                .iter()
                .any(|warning| warning.contains("provider-backend-permissions-missing: hermes"))
        );
    }
}
