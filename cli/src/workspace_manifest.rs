use anyhow::Result;
use serde::Serialize;
use std::collections::BTreeMap;

use crate::cli::OutputFormat;
use crate::config::{AiboxConfig, ExtraMcpServer};

pub const WORKSPACE_MANIFEST_SCHEMA_VERSION: &str = "aibox.workspace-manifest.v0-preview";

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct WorkspaceManifest {
    pub schema_version: &'static str,
    pub aibox_version: &'static str,
    pub project: WorkspaceProject,
    pub processkit: WorkspaceProcessKit,
    pub context: WorkspaceContext,
    pub ai: WorkspaceAi,
    pub addons: Vec<WorkspaceAddon>,
    pub mcp: WorkspaceMcp,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct WorkspaceProject {
    pub name: String,
    pub base: String,
    pub profile: String,
    pub user: String,
    pub keepalive: bool,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct WorkspaceProcessKit {
    pub source: String,
    pub version: String,
    pub src_path: String,
    pub branch: Option<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct WorkspaceContext {
    pub schema_version: String,
    pub packages: Vec<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct WorkspaceAi {
    pub harnesses: Vec<String>,
    pub model_providers: Vec<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct WorkspaceAddon {
    pub name: String,
    pub tools: BTreeMap<String, Option<String>>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct WorkspaceMcp {
    pub extra_servers: Vec<WorkspaceMcpServer>,
    pub permissions: WorkspaceMcpPermissions,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct WorkspaceMcpServer {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env_keys: Vec<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct WorkspaceMcpPermissions {
    pub default_mode: String,
    pub allow_patterns: Vec<String>,
    pub deny_patterns: Vec<String>,
    pub harness_overrides: Vec<String>,
}

pub fn workspace_manifest(config: &AiboxConfig) -> WorkspaceManifest {
    let mut context_packages = config.context.packages.clone();
    context_packages.sort();

    let mut harnesses: Vec<String> = config
        .ai
        .effective_harnesses()
        .iter()
        .map(ToString::to_string)
        .collect();
    harnesses.sort();

    let mut model_providers: Vec<String> = config
        .ai
        .model_providers
        .iter()
        .map(ToString::to_string)
        .collect();
    model_providers.sort();

    let mut addons: Vec<WorkspaceAddon> = config
        .addons
        .addons
        .iter()
        .map(|(name, addon)| {
            let tools = addon
                .tools
                .iter()
                .map(|(tool, entry)| (tool.clone(), entry.version.clone()))
                .collect();
            WorkspaceAddon {
                name: name.clone(),
                tools,
            }
        })
        .collect();
    addons.sort_by(|a, b| a.name.cmp(&b.name));

    WorkspaceManifest {
        schema_version: WORKSPACE_MANIFEST_SCHEMA_VERSION,
        aibox_version: env!("CARGO_PKG_VERSION"),
        project: WorkspaceProject {
            name: config.container.name.clone(),
            base: config.aibox.base.to_string(),
            profile: config.aibox.profile.to_string(),
            user: config.container.user.clone(),
            keepalive: config.container.keepalive,
        },
        processkit: WorkspaceProcessKit {
            source: config.processkit.source.clone(),
            version: config.processkit.version.clone(),
            src_path: config.processkit.src_path.clone(),
            branch: config.processkit.branch.clone(),
        },
        context: WorkspaceContext {
            schema_version: config.context.schema_version.clone(),
            packages: context_packages,
        },
        ai: WorkspaceAi {
            harnesses,
            model_providers,
        },
        addons,
        mcp: WorkspaceMcp {
            extra_servers: mcp_servers(&config.mcp.servers),
            permissions: WorkspaceMcpPermissions {
                default_mode: mcp_default_mode(&config.mcp.permissions.default_mode),
                allow_patterns: sorted(config.mcp.permissions.allow_patterns.clone()),
                deny_patterns: sorted(config.mcp.permissions.deny_patterns.clone()),
                harness_overrides: config.mcp.permissions.harness.keys().cloned().collect(),
            },
        },
    }
}

pub fn cmd_workspace_manifest(config_path: &Option<String>, format: OutputFormat) -> Result<()> {
    let config = AiboxConfig::from_cli_option(config_path)?;
    let manifest = workspace_manifest(&config);

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&manifest)?);
        }
        OutputFormat::Yaml => {
            print!("{}", serde_yaml::to_string(&manifest)?);
        }
        OutputFormat::Table => {
            println!("Workspace manifest");
            println!("  Schema:       {}", manifest.schema_version);
            println!("  aibox:        {}", manifest.aibox_version);
            println!("  Project:      {}", manifest.project.name);
            println!("  Base:         {}", manifest.project.base);
            println!("  Profile:      {}", manifest.project.profile);
            println!("  Processkit:   {}", manifest.processkit.version);
            println!("  Harnesses:    {}", manifest.ai.harnesses.len());
            println!("  Addons:       {}", manifest.addons.len());
            println!("  MCP servers:  {}", manifest.mcp.extra_servers.len());
            println!();
            println!(
                "Use `aibox describe workspace-manifest -o json` for the machine-readable projection."
            );
        }
    }

    Ok(())
}

fn mcp_servers(servers: &[ExtraMcpServer]) -> Vec<WorkspaceMcpServer> {
    let mut out: Vec<WorkspaceMcpServer> = servers
        .iter()
        .map(|server| WorkspaceMcpServer {
            name: server.name.clone(),
            command: server.command.clone(),
            args: server.args.clone(),
            env_keys: server.env.keys().cloned().collect(),
        })
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

fn sorted(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values
}

fn mcp_default_mode(value: &str) -> String {
    if value.is_empty() {
        "ask".to_string()
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AiboxConfig;

    #[test]
    fn workspace_manifest_is_sorted_and_redacts_mcp_env_values() {
        let toml = r#"[aibox]
version = "0.22.0"
profile = "headless-runner"

[container]
name = "demo"
user = "agent"
keepalive = true

[context]
schema_version = "1.0.0"
packages = ["software", "managed"]

[ai]
harnesses = ["codex", "claude"]
model_providers = ["openai", "anthropic"]

[processkit]
source = "https://github.com/projectious-work/processkit"
version = "v0.24.0"
src_path = "src"
branch = "main"

[mcp.permissions]
default_mode = "allow"
allow_patterns = ["z-*", "a-*"]
deny_patterns = ["private-*"]

[[mcp.servers]]
name = "z-server"
command = "uv"
args = ["run", "server.py"]

[mcp.servers.env]
TOKEN = "secret"

[addons.zeta.tools]
z = { version = "1" }

[addons.alpha.tools]
a = {}
"#;
        let config = AiboxConfig::from_str(toml).unwrap();
        let manifest = workspace_manifest(&config);

        assert_eq!(manifest.schema_version, WORKSPACE_MANIFEST_SCHEMA_VERSION);
        assert_eq!(manifest.project.name, "demo");
        assert_eq!(manifest.project.profile, "headless-runner");
        assert_eq!(
            manifest.context.packages,
            vec!["managed".to_string(), "software".to_string()]
        );
        assert_eq!(
            manifest.ai.harnesses,
            vec!["claude".to_string(), "codex".to_string()]
        );
        let addon_names: Vec<&str> = manifest
            .addons
            .iter()
            .map(|addon| addon.name.as_str())
            .collect();
        assert!(addon_names.contains(&"alpha"));
        assert!(addon_names.contains(&"zeta"));
        assert!(
            addon_names.iter().position(|name| *name == "alpha")
                < addon_names.iter().position(|name| *name == "zeta")
        );
        assert_eq!(
            manifest.mcp.permissions.allow_patterns,
            vec!["a-*".to_string(), "z-*".to_string()]
        );
        assert_eq!(manifest.mcp.extra_servers[0].env_keys, vec!["TOKEN"]);
    }
}
