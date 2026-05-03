//! Preauth merge — surface processkit's `preauth.json` (shipped with
//! processkit ≥ v0.22.0) into harness configuration so pre-approved
//! Bash patterns and MCP servers do not prompt the user on first
//! invocation.
//!
//! ## What this module does
//!
//! processkit ships a single source-of-truth permissions spec at
//! `context/skills/processkit/skill-gate/assets/preauth.json` (shape
//! pinned by `SUPPORTED_PREAUTH_VERSION`). On `aibox init` / `aibox
//! sync` we read that file and merge two arrays into the harness
//! config:
//!
//! - `permissions.allow[]` — the wildcards listed under
//!   `permissions.allow` in the spec.
//! - `enabledMcpjsonServers[]` — the MCP server names listed under
//!   `enabledMcpjsonServers` in the spec.
//! - `codex.mcp.allowed_tools[]` — Codex MCP allow patterns when the
//!   installed processkit spec provides them. Older specs fall back to
//!   `permissions.allow[]` because the processkit MCP tool names share
//!   Claude's `mcp__<server>__<tool>` shape.
//!
//! ## Sidecar manifest (`_processkit_managed_keys`)
//!
//! To support deletion propagation (when processkit drops a wildcard
//! from a future spec, we must drop it from the user's settings on the
//! next sync) we keep a tiny sidecar at the top level of
//! `.claude/settings.json`:
//!
//! ```json
//! "_processkit_managed_keys": {
//!   "allow": ["mcp__processkit-actor-profile__*", ...],
//!   "enabled_servers": ["processkit-actor-profile", ...]
//! }
//! ```
//!
//! This is the *previous-run snapshot* of what aibox managed. On a
//! re-merge:
//!
//! 1. `previous`        = what was in the sidecar
//! 2. `current`         = what's in the new spec
//! 3. `existing_user`   = (settings list) minus `previous` — preserves
//!    user-added entries
//! 4. `final`           = `existing_user ∪ current` (sorted, deduped)
//!
//! Then both the visible array and the sidecar are refreshed to mirror
//! `current`.
//!
//! ## What this module does NOT do
//!
//! - It does not write `.claude/settings.local.json` (that's WS-2's
//!   `[mcp.permissions]` machinery).
//! - It does not touch `hooks.SessionStart` /
//!   `hooks.UserPromptSubmit` / `hooks.PreToolUse` — those are owned by
//!   `hook_registration.rs` and live behind a different marker
//!   (`_processkit_managed: true` per array entry).
//! - Claude top-level keys outside `permissions.allow`,
//!   `enabledMcpjsonServers`, and `_processkit_managed_keys` are
//!   round-tripped byte-for-byte.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;

use crate::output;

/// The version field this CLI knows how to interpret. If an installed
/// processkit ships a `preauth.json` with a different version, we
/// soft-warn and skip the merge (rather than partially apply a spec we
/// don't fully understand).
pub(crate) const SUPPORTED_PREAUTH_VERSION: u32 = 1;

/// Forward-compat-tolerant deserialization target for
/// `context/skills/processkit/skill-gate/assets/preauth.json`. Unknown
/// top-level fields are silently accepted (`#[serde(default)]` + no
/// `deny_unknown_fields`). Version checking is done at runtime.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PreauthSpec {
    pub version: u32,
    #[serde(default)]
    #[allow(dead_code)]
    pub description: Option<String>,
    #[serde(default)]
    pub permissions: PreauthPermissions,
    #[serde(default, rename = "enabledMcpjsonServers")]
    pub enabled_mcp_json_servers: Vec<String>,
    #[serde(default)]
    pub codex: Option<CodexPreauth>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub(crate) struct PreauthPermissions {
    #[serde(default)]
    pub allow: Vec<String>,
    /// Reserved for future use; absent in v1 spec.
    #[serde(default)]
    #[allow(dead_code)]
    pub deny: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub(crate) struct CodexPreauth {
    #[serde(default)]
    pub mcp: CodexMcpPreauth,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub(crate) struct CodexMcpPreauth {
    #[serde(default)]
    pub allowed_tools: Vec<String>,
}

/// Read the processkit-shipped preauth spec from the project and merge
/// its contents into supported harness settings.
///
/// Best-effort contract:
///
/// - **Missing spec file** → soft-warn, return `Ok(())`. (Older
///   processkit versions don't ship preauth.json yet.)
/// - **Malformed JSON** → return `Err(...)`.
/// - **Unknown version** → soft-warn, return `Ok(())` without
///   modifying settings.
/// - **Happy path** → settings.json is created (if missing) or
///   merged-into (if present), with the `_processkit_managed_keys`
///   sidecar updated.
pub fn merge_processkit_preauth_into_harness_settings(project_root: &Path) -> Result<()> {
    let spec_path = project_root.join("context/skills/processkit/skill-gate/assets/preauth.json");

    if !spec_path.is_file() {
        output::warn(&format!(
            "preauth.json not found at {}; processkit must be >= v0.22.0 — skipping merge",
            spec_path.display()
        ));
        return Ok(());
    }

    let body = fs::read_to_string(&spec_path)
        .with_context(|| format!("failed to read {}", spec_path.display()))?;
    let spec: PreauthSpec = serde_json::from_str(&body)
        .with_context(|| format!("malformed preauth.json at {}", spec_path.display()))?;

    if spec.version != SUPPORTED_PREAUTH_VERSION {
        output::warn(&format!(
            "preauth spec at {} reports version {}, this aibox CLI supports up to version {}; \
             processkit MCP tools will continue to work but additional features in the v{} spec \
             are not applied. Upgrade aibox CLI to consume the new spec.",
            spec_path.display(),
            spec.version,
            SUPPORTED_PREAUTH_VERSION,
            spec.version,
        ));
        return Ok(());
    }

    let settings_path = project_root.join(".claude/settings.json");
    let mut top = read_or_empty_object(&settings_path)?;
    merge_claude_managed_lists(project_root, &mut top, &spec)?;
    write_atomic(&settings_path, &top)?;

    merge_codex_preauth(project_root, &spec)?;

    let (claude_allow, claude_servers) = claude_preauth_sets(project_root, &spec)?;
    let codex_tools = codex_allowed_tools_for_project(project_root, &spec)?;
    output::ok(&format!(
        "preauth merged: {} Claude allow patterns, {} enabled servers, {} Codex allowed tools",
        claude_allow.len(),
        claude_servers.len(),
        codex_tools.len()
    ));
    Ok(())
}

/// Backward-compatible wrapper kept for existing call sites/tests.
pub fn merge_processkit_preauth_into_claude_settings(project_root: &Path) -> Result<()> {
    merge_processkit_preauth_into_harness_settings(project_root)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Read `path` as a JSON object, returning an empty `Map` if the file
/// is absent or empty. Mirrors the load helper at the top of
/// `hook_registration::write_claude_settings_hooks`.
fn read_or_empty_object(path: &Path) -> Result<serde_json::Map<String, Value>> {
    if !path.is_file() {
        return Ok(serde_json::Map::new());
    }
    let body =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    if body.trim().is_empty() {
        return Ok(serde_json::Map::new());
    }
    serde_json::from_str(&body)
        .with_context(|| format!("failed to parse existing JSON at {}", path.display()))
}

/// In-place merge of the spec's allow + enabled-servers arrays into the
/// `top` settings map, refreshing the `_processkit_managed_keys`
/// sidecar.
fn merge_claude_managed_lists(
    project_root: &Path,
    top: &mut serde_json::Map<String, Value>,
    spec: &PreauthSpec,
) -> Result<()> {
    // ── Read previous-run snapshot from sidecar (default: empty). ──────
    let (previous_allow, previous_servers) = read_managed_snapshot(top);

    // ── Read user-visible existing arrays. ─────────────────────────────
    let existing_allow = read_string_array(top, "permissions", Some("allow"))?;
    let existing_servers = read_top_level_string_array(top, "enabledMcpjsonServers")?;

    // ── Compute the merged sets. ───────────────────────────────────────
    let (current_allow, current_servers) = claude_preauth_sets(project_root, spec)?;

    let final_allow: BTreeSet<String> = existing_allow
        .into_iter()
        .filter(|e| !previous_allow.contains(e))
        .chain(current_allow.iter().cloned())
        .collect();
    let final_servers: BTreeSet<String> = existing_servers
        .into_iter()
        .filter(|e| !previous_servers.contains(e))
        .chain(current_servers.iter().cloned())
        .collect();

    // ── Write back the visible arrays. ─────────────────────────────────
    write_permissions_allow(top, &final_allow)?;
    write_top_level_array(top, "enabledMcpjsonServers", &final_servers);

    // ── Refresh the sidecar to mirror the current spec. ────────────────
    write_managed_snapshot(top, &current_allow, &current_servers);

    Ok(())
}

fn codex_allowed_tools(spec: &PreauthSpec) -> BTreeSet<String> {
    let explicit = spec
        .codex
        .as_ref()
        .map(|c| c.mcp.allowed_tools.as_slice())
        .unwrap_or(&[]);
    if explicit.is_empty() {
        spec.permissions.allow.iter().cloned().collect()
    } else {
        explicit.iter().cloned().collect()
    }
}

fn claude_preauth_sets(
    project_root: &Path,
    spec: &PreauthSpec,
) -> Result<(BTreeSet<String>, BTreeSet<String>)> {
    let mut allow: BTreeSet<String> = spec.permissions.allow.iter().cloned().collect();
    let mut servers: BTreeSet<String> = spec.enabled_mcp_json_servers.iter().cloned().collect();

    if generated_dot_mcp_uses_processkit_gateway(project_root)? {
        servers.clear();
        servers.insert("processkit-gateway".to_string());
        allow.insert("mcp__processkit-gateway__*".to_string());
    }

    Ok((allow, servers))
}

fn generated_dot_mcp_uses_processkit_gateway(project_root: &Path) -> Result<bool> {
    let path = project_root.join(".mcp.json");
    if !path.is_file() {
        return Ok(false);
    }

    let body =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    if body.trim().is_empty() {
        return Ok(false);
    }
    let parsed: Value = serde_json::from_str(&body)
        .with_context(|| format!("failed to parse existing JSON at {}", path.display()))?;
    Ok(parsed
        .get("mcpServers")
        .and_then(|v| v.as_object())
        .map(|servers| servers.contains_key("processkit-gateway"))
        .unwrap_or(false))
}

fn codex_config_uses_processkit_gateway(project_root: &Path) -> Result<bool> {
    let path = project_root.join(".codex/config.toml");
    if !path.is_file() {
        return Ok(false);
    }

    let body =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    if body.trim().is_empty() {
        return Ok(false);
    }
    let document: toml_edit::DocumentMut = body
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(document
        .get("mcp_servers")
        .and_then(|item| item.as_table())
        .map(|servers| servers.contains_key("processkit-gateway"))
        .unwrap_or(false))
}

fn codex_allowed_tools_for_project(
    project_root: &Path,
    spec: &PreauthSpec,
) -> Result<BTreeSet<String>> {
    let mut current = codex_allowed_tools(spec);
    if codex_config_uses_processkit_gateway(project_root)? {
        current.insert("processkit-gateway".to_string());
    }
    Ok(current)
}

fn merge_codex_preauth(project_root: &Path, spec: &PreauthSpec) -> Result<()> {
    let current = codex_allowed_tools_for_project(project_root, spec)?;
    if current.is_empty() {
        return Ok(());
    }

    let path = project_root.join(".codex/config.toml");
    let mut document: toml_edit::DocumentMut = if path.is_file() {
        let body = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        body.parse::<toml_edit::DocumentMut>()
            .with_context(|| format!("failed to parse {}", path.display()))?
    } else {
        toml_edit::DocumentMut::new()
    };

    if !document.contains_key("mcp") {
        document["mcp"] = toml_edit::table();
    }

    let mcp = &mut document["mcp"];
    let existing = read_toml_string_array(mcp, "allowed_tools")?;
    let previous = read_toml_string_array(mcp, "_processkit_managed_allowed_tools")?;

    let previous_set: BTreeSet<String> = previous.into_iter().collect();
    let final_allowed: BTreeSet<String> = existing
        .into_iter()
        .filter(|item| !previous_set.contains(item))
        .chain(current.iter().cloned())
        .collect();

    write_toml_string_array(mcp, "allowed_tools", &final_allowed);
    write_toml_string_array(mcp, "_processkit_managed_allowed_tools", &current);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create directory {}", parent.display()))?;
    }
    fs::write(&path, document.to_string())
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

fn read_toml_string_array(item: &toml_edit::Item, key: &str) -> Result<Vec<String>> {
    let Some(array) = item.get(key).and_then(|v| v.as_array()) else {
        return Ok(Vec::new());
    };
    array
        .iter()
        .map(|v| {
            v.as_str()
                .map(String::from)
                .ok_or_else(|| anyhow::anyhow!("non-string entry in [mcp].{key}"))
        })
        .collect()
}

fn write_toml_string_array(item: &mut toml_edit::Item, key: &str, values: &BTreeSet<String>) {
    let array = toml_edit::Array::from_iter(values.iter().map(|s| s.as_str()));
    item[key] = toml_edit::value(array);
}

/// Pull `_processkit_managed_keys.{allow, enabled_servers}` from the
/// settings map, defaulting to empty sets when absent / wrong type.
fn read_managed_snapshot(
    top: &serde_json::Map<String, Value>,
) -> (BTreeSet<String>, BTreeSet<String>) {
    let mut prev_allow = BTreeSet::new();
    let mut prev_servers = BTreeSet::new();
    if let Some(Value::Object(snap)) = top.get("_processkit_managed_keys") {
        if let Some(Value::Array(arr)) = snap.get("allow") {
            for v in arr {
                if let Some(s) = v.as_str() {
                    prev_allow.insert(s.to_string());
                }
            }
        }
        if let Some(Value::Array(arr)) = snap.get("enabled_servers") {
            for v in arr {
                if let Some(s) = v.as_str() {
                    prev_servers.insert(s.to_string());
                }
            }
        }
    }
    (prev_allow, prev_servers)
}

/// Read a string array nested at `top.<outer>.<inner>` (when `inner` is
/// `Some`) or at `top.<outer>` (when `None`). Missing returns empty.
/// Wrong type is a hard error so we never silently corrupt user state.
fn read_string_array(
    top: &serde_json::Map<String, Value>,
    outer: &str,
    inner: Option<&str>,
) -> Result<Vec<String>> {
    let val = match inner {
        Some(inner) => top
            .get(outer)
            .and_then(|v| v.as_object())
            .and_then(|o| o.get(inner)),
        None => top.get(outer),
    };
    match val {
        None => Ok(Vec::new()),
        Some(Value::Array(arr)) => arr
            .iter()
            .map(|v| {
                v.as_str()
                    .map(|s| s.to_string())
                    .ok_or_else(|| anyhow::anyhow!("non-string entry in array"))
            })
            .collect(),
        Some(_) => {
            let path = match inner {
                Some(i) => format!("{}.{}", outer, i),
                None => outer.to_string(),
            };
            Err(anyhow::anyhow!("`{}` is not a JSON array", path))
        }
    }
}

fn read_top_level_string_array(
    top: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<Vec<String>> {
    read_string_array(top, key, None)
}

/// Write `final_allow` to `top.permissions.allow`, creating the
/// `permissions` object if it doesn't yet exist.
fn write_permissions_allow(
    top: &mut serde_json::Map<String, Value>,
    final_allow: &BTreeSet<String>,
) -> Result<()> {
    let perms_val = top
        .entry("permissions".to_string())
        .or_insert_with(|| Value::Object(serde_json::Map::new()));
    let perms = perms_val
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("`permissions` is not a JSON object"))?;
    perms.insert(
        "allow".to_string(),
        Value::Array(
            final_allow
                .iter()
                .map(|s| Value::String(s.clone()))
                .collect(),
        ),
    );
    Ok(())
}

fn write_top_level_array(
    top: &mut serde_json::Map<String, Value>,
    key: &str,
    items: &BTreeSet<String>,
) {
    top.insert(
        key.to_string(),
        Value::Array(items.iter().map(|s| Value::String(s.clone())).collect()),
    );
}

fn write_managed_snapshot(
    top: &mut serde_json::Map<String, Value>,
    current_allow: &BTreeSet<String>,
    current_servers: &BTreeSet<String>,
) {
    let mut snap = serde_json::Map::new();
    snap.insert(
        "allow".to_string(),
        Value::Array(
            current_allow
                .iter()
                .map(|s| Value::String(s.clone()))
                .collect(),
        ),
    );
    snap.insert(
        "enabled_servers".to_string(),
        Value::Array(
            current_servers
                .iter()
                .map(|s| Value::String(s.clone()))
                .collect(),
        ),
    );
    top.insert("_processkit_managed_keys".to_string(), Value::Object(snap));
}

/// Atomic write: serialize → tmp → rename. Trailing newline appended so
/// the file ends correctly.
fn write_atomic(path: &Path, top: &serde_json::Map<String, Value>) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create directory {}", parent.display()))?;
    }
    let mut formatted = serde_json::to_string_pretty(&Value::Object(top.clone()))
        .context("failed to serialize settings JSON")?;
    if !formatted.ends_with('\n') {
        formatted.push('\n');
    }
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, formatted).with_context(|| format!("failed to write {}", tmp.display()))?;
    fs::rename(&tmp, path)
        .with_context(|| format!("failed to rename {} → {}", tmp.display(), path.display()))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Synthetic v0.22.0-shaped preauth.json string with 18 wildcards
    /// and 18 enabled servers. The exact identifiers don't matter for
    /// these unit tests; we just need realistic shape and counts.
    fn v0_22_0_preauth_json() -> String {
        let allow: Vec<String> = (0..18)
            .map(|i| format!("\"mcp__processkit-skill-{i:02}__*\""))
            .collect();
        let servers: Vec<String> = (0..18)
            .map(|i| format!("\"processkit-skill-{i:02}\""))
            .collect();
        format!(
            r#"{{
              "version": 1,
              "description": "synthetic v0.22.0",
              "permissions": {{ "allow": [{}] }},
              "enabledMcpjsonServers": [{}]
            }}"#,
            allow.join(", "),
            servers.join(", ")
        )
    }

    fn write_preauth(project: &Path, body: &str) {
        let asset_dir = project.join("context/skills/processkit/skill-gate/assets");
        fs::create_dir_all(&asset_dir).unwrap();
        fs::write(asset_dir.join("preauth.json"), body).unwrap();
    }

    fn write_settings(project: &Path, body: &str) {
        let dir = project.join(".claude");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("settings.json"), body).unwrap();
    }

    fn read_settings(project: &Path) -> Value {
        let body = fs::read_to_string(project.join(".claude/settings.json")).unwrap();
        serde_json::from_str(&body).unwrap()
    }

    fn read_settings_bytes(project: &Path) -> Vec<u8> {
        fs::read(project.join(".claude/settings.json")).unwrap()
    }

    // ───── Test 1 ──────────────────────────────────────────────────────
    #[test]
    fn preauth_spec_deserializes_v0_22_0_verbatim() {
        let body = v0_22_0_preauth_json();
        let spec: PreauthSpec = serde_json::from_str(&body).unwrap();
        assert_eq!(spec.version, 1);
        assert_eq!(spec.permissions.allow.len(), 18);
        assert!(
            spec.permissions
                .allow
                .iter()
                .any(|s| s.contains("skill-00"))
        );
        assert_eq!(spec.enabled_mcp_json_servers.len(), 18);
    }

    // ───── Test 2 ──────────────────────────────────────────────────────
    #[test]
    fn unknown_version_is_warn_and_skip() {
        let tmp = TempDir::new().unwrap();
        let project = tmp.path();

        write_preauth(
            project,
            r#"{"version": 99, "permissions": {"allow": ["x"]}, "enabledMcpjsonServers": ["y"]}"#,
        );
        let pre_settings = r#"{"model":"claude-opus-4-5"}"#;
        write_settings(project, pre_settings);
        let before = read_settings_bytes(project);

        merge_processkit_preauth_into_claude_settings(project).unwrap();

        let after = read_settings_bytes(project);
        assert_eq!(
            before, after,
            "settings.json must be untouched on version mismatch"
        );
    }

    // ───── Test 3 ──────────────────────────────────────────────────────
    #[test]
    fn malformed_json_returns_err() {
        let tmp = TempDir::new().unwrap();
        let project = tmp.path();

        // Truncated body — missing closing braces.
        write_preauth(project, r#"{"version": 1, "permissions": {"#);

        let r = merge_processkit_preauth_into_claude_settings(project);
        assert!(r.is_err(), "malformed preauth.json must produce Err");
    }

    // ───── Test 4 ──────────────────────────────────────────────────────
    #[test]
    fn missing_spec_file_is_warn_and_skip() {
        let tmp = TempDir::new().unwrap();
        let project = tmp.path();
        // No preauth.json, no .claude/ either.

        merge_processkit_preauth_into_claude_settings(project).unwrap();

        assert!(
            !project.join(".claude/settings.json").exists(),
            "no settings.json should be created when preauth.json is absent"
        );
    }

    // ───── Test 5 ──────────────────────────────────────────────────────
    #[test]
    fn merge_replaces_managed_block_preserves_user_entries() {
        // Sub-case A: empty settings.json
        {
            let tmp = TempDir::new().unwrap();
            let project = tmp.path();
            write_preauth(project, &v0_22_0_preauth_json());

            merge_processkit_preauth_into_claude_settings(project).unwrap();

            let s = read_settings(project);
            let allow = s["permissions"]["allow"].as_array().unwrap();
            assert_eq!(allow.len(), 18);
            let snap_allow = s["_processkit_managed_keys"]["allow"].as_array().unwrap();
            assert_eq!(snap_allow.len(), 18);
        }

        // Sub-case B: settings with user permissions.allow entry
        {
            let tmp = TempDir::new().unwrap();
            let project = tmp.path();
            write_preauth(project, &v0_22_0_preauth_json());
            write_settings(project, r#"{"permissions":{"allow":["Bash(npm test:*)"]}}"#);

            merge_processkit_preauth_into_claude_settings(project).unwrap();

            let s = read_settings(project);
            let allow: Vec<String> = s["permissions"]["allow"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect();
            assert_eq!(allow.len(), 19, "user entry + 18 preauth = 19");
            assert!(allow.contains(&"Bash(npm test:*)".to_string()));
            // sorted (BTreeSet at serialize time)
            let mut sorted = allow.clone();
            sorted.sort();
            assert_eq!(allow, sorted, "permissions.allow must be sorted");
        }

        // Sub-case C: pre-existing sidecar with previous managed entries
        {
            let tmp = TempDir::new().unwrap();
            let project = tmp.path();
            write_preauth(project, &v0_22_0_preauth_json());
            // Settings include user entry + an old processkit-managed
            // entry that's NOT in the new spec.
            write_settings(
                project,
                r#"{
                  "permissions": {"allow": ["Bash(npm test:*)", "mcp__old-tool__*"]},
                  "_processkit_managed_keys": {"allow": ["mcp__old-tool__*"], "enabled_servers": []}
                }"#,
            );

            merge_processkit_preauth_into_claude_settings(project).unwrap();

            let s = read_settings(project);
            let allow: Vec<String> = s["permissions"]["allow"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect();
            assert!(
                allow.contains(&"Bash(npm test:*)".to_string()),
                "user entry preserved"
            );
            assert!(
                !allow.contains(&"mcp__old-tool__*".to_string()),
                "old managed entry dropped"
            );
            assert_eq!(allow.len(), 19, "user + 18 new = 19");
        }
    }

    // ───── Test 6 ──────────────────────────────────────────────────────
    #[test]
    fn idempotent_double_merge() {
        let tmp = TempDir::new().unwrap();
        let project = tmp.path();
        write_preauth(project, &v0_22_0_preauth_json());

        merge_processkit_preauth_into_claude_settings(project).unwrap();
        let after_first = read_settings_bytes(project);

        merge_processkit_preauth_into_claude_settings(project).unwrap();
        let after_second = read_settings_bytes(project);

        assert_eq!(
            after_first, after_second,
            "two consecutive merges on a stable spec must be byte-identical"
        );
    }

    // ───── Test 7 ──────────────────────────────────────────────────────
    #[test]
    fn removed_from_spec_is_removed_from_settings() {
        let tmp = TempDir::new().unwrap();
        let project = tmp.path();

        // First sync: 18 patterns.
        write_preauth(project, &v0_22_0_preauth_json());
        merge_processkit_preauth_into_claude_settings(project).unwrap();
        let s1 = read_settings(project);
        assert_eq!(s1["permissions"]["allow"].as_array().unwrap().len(), 18);

        // Second sync: trim spec to 17 patterns + 17 servers.
        let allow: Vec<String> = (0..17)
            .map(|i| format!("\"mcp__processkit-skill-{i:02}__*\""))
            .collect();
        let servers: Vec<String> = (0..17)
            .map(|i| format!("\"processkit-skill-{i:02}\""))
            .collect();
        let trimmed = format!(
            r#"{{"version":1,"permissions":{{"allow":[{}]}},"enabledMcpjsonServers":[{}]}}"#,
            allow.join(","),
            servers.join(",")
        );
        write_preauth(project, &trimmed);
        merge_processkit_preauth_into_claude_settings(project).unwrap();

        let s2 = read_settings(project);
        let allow_after: Vec<String> = s2["permissions"]["allow"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        assert_eq!(allow_after.len(), 17, "dropped pattern is removed");
        assert!(
            !allow_after.iter().any(|s| s.contains("skill-17")),
            "pattern 17 must no longer appear"
        );
        // Sidecar must mirror the new (smaller) spec.
        let snap = s2["_processkit_managed_keys"]["allow"].as_array().unwrap();
        assert_eq!(snap.len(), 17);
    }

    // ───── Test 8 ──────────────────────────────────────────────────────
    #[test]
    fn merge_unions_with_per_skill_enabled_servers() {
        let tmp = TempDir::new().unwrap();
        let project = tmp.path();
        write_preauth(project, &v0_22_0_preauth_json());
        write_settings(project, r#"{"enabledMcpjsonServers": ["my-other-mcp"]}"#);

        merge_processkit_preauth_into_claude_settings(project).unwrap();

        let s = read_settings(project);
        let servers: Vec<String> = s["enabledMcpjsonServers"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        assert!(servers.contains(&"my-other-mcp".to_string()));
        assert_eq!(servers.len(), 19, "user server + 18 preauth = 19");
        // sorted
        let mut sorted = servers.clone();
        sorted.sort();
        assert_eq!(servers, sorted);
        // dedup: re-running shouldn't grow the list
        merge_processkit_preauth_into_claude_settings(project).unwrap();
        let s2 = read_settings(project);
        assert_eq!(
            s2["enabledMcpjsonServers"].as_array().unwrap().len(),
            19,
            "dedup on re-merge"
        );
    }

    // ───── Test 9 ──────────────────────────────────────────────────────
    #[test]
    fn top_level_non_managed_keys_preserved() {
        let tmp = TempDir::new().unwrap();
        let project = tmp.path();
        write_preauth(project, &v0_22_0_preauth_json());
        write_settings(
            project,
            r#"{"model":"claude-opus-4-5","apiKeyHelper":"my-helper"}"#,
        );

        merge_processkit_preauth_into_claude_settings(project).unwrap();

        let s = read_settings(project);
        assert_eq!(s["model"].as_str().unwrap(), "claude-opus-4-5");
        assert_eq!(s["apiKeyHelper"].as_str().unwrap(), "my-helper");
    }

    // ───── Test 10 ─────────────────────────────────────────────────────
    #[test]
    fn creates_settings_json_when_missing() {
        let tmp = TempDir::new().unwrap();
        let project = tmp.path();
        write_preauth(project, &v0_22_0_preauth_json());
        // No .claude/ directory at all.
        assert!(!project.join(".claude").exists());

        merge_processkit_preauth_into_claude_settings(project).unwrap();

        assert!(project.join(".claude/settings.json").exists());
        let s = read_settings(project);
        assert_eq!(s["permissions"]["allow"].as_array().unwrap().len(), 18);
        assert_eq!(s["enabledMcpjsonServers"].as_array().unwrap().len(), 18);
    }

    #[test]
    fn codex_allowed_tools_merge_preserves_user_and_removes_old_managed() {
        let tmp = TempDir::new().unwrap();
        let project = tmp.path();
        write_preauth(
            project,
            r#"{
              "version": 1,
              "permissions": {"allow": ["mcp__processkit-fallback__*"]},
              "enabledMcpjsonServers": [],
              "codex": {"mcp": {"allowed_tools": ["mcp__processkit-new__*"]}}
            }"#,
        );
        let codex_dir = project.join(".codex");
        fs::create_dir_all(&codex_dir).unwrap();
        fs::write(
            codex_dir.join("config.toml"),
            "[mcp]\nallowed_tools = [\"user-tool\", \"mcp__processkit-old__*\"]\n_processkit_managed_allowed_tools = [\"mcp__processkit-old__*\"]\n",
        )
        .unwrap();

        merge_processkit_preauth_into_claude_settings(project).unwrap();

        let body = fs::read_to_string(codex_dir.join("config.toml")).unwrap();
        assert!(body.contains("\"user-tool\""));
        assert!(body.contains("\"mcp__processkit-new__*\""));
        assert!(!body.contains("\"mcp__processkit-old__*\""));
    }

    #[test]
    fn codex_allowed_tools_falls_back_to_claude_allow_for_older_spec() {
        let tmp = TempDir::new().unwrap();
        let project = tmp.path();
        write_preauth(project, &v0_22_0_preauth_json());

        merge_processkit_preauth_into_claude_settings(project).unwrap();

        let body = fs::read_to_string(project.join(".codex/config.toml")).unwrap();
        assert!(body.contains("mcp__processkit-skill-00__*"));
    }

    #[test]
    fn preauth_uses_gateway_server_when_generated_mcp_config_collapsed() {
        let tmp = TempDir::new().unwrap();
        let project = tmp.path();
        write_preauth(project, &v0_22_0_preauth_json());
        fs::write(
            project.join(".mcp.json"),
            r#"{"mcpServers":{"processkit-gateway":{"command":"uv","args":[]}}}"#,
        )
        .unwrap();
        let codex_dir = project.join(".codex");
        fs::create_dir_all(&codex_dir).unwrap();
        fs::write(
            codex_dir.join("config.toml"),
            "[mcp_servers.processkit-gateway]\ncommand = \"uv\"\nargs = []\n",
        )
        .unwrap();

        merge_processkit_preauth_into_claude_settings(project).unwrap();

        let settings = read_settings(project);
        let servers: Vec<String> = settings["enabledMcpjsonServers"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        assert_eq!(servers, vec!["processkit-gateway".to_string()]);
        let allow: Vec<String> = settings["permissions"]["allow"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        assert!(allow.contains(&"mcp__processkit-gateway__*".to_string()));

        let codex_body = fs::read_to_string(codex_dir.join("config.toml")).unwrap();
        assert!(codex_body.contains("\"processkit-gateway\""));
    }
}
