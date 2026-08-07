//! Resolve `version = "latest"` to concrete upstream versions.
//!
//! For key tools (rustc, node, python, Codex), queries upstream APIs to find the
//! actual latest stable version. Other tools fall back to the addon's
//! `default_version`. Resolvers fail gracefully — on network error, they
//! return `None` and the caller falls back to the addon default.

use anyhow::Result;

use crate::output;

/// Attempt to resolve the latest stable version for the given tool.
/// Returns `None` if the tool has no upstream resolver or the query fails.
pub fn resolve_latest(tool_name: &str) -> Option<String> {
    match tool_name {
        "rustc" => resolve_rustc().ok().flatten(),
        "node" => resolve_node().ok().flatten(),
        "python" => resolve_python().ok().flatten(),
        "codex" => resolve_npm_latest("@openai/codex", "Codex").ok().flatten(),
        _ => None,
    }
}

/// Resolve an npm package's `latest` dist-tag to a concrete version.
fn resolve_npm_latest(package: &str, display_name: &str) -> Result<Option<String>> {
    let encoded_package = package.replace('/', "%2f");
    let url = format!("https://registry.npmjs.org/{encoded_package}/latest");
    output::info(&format!(
        "Resolving latest stable {display_name} version..."
    ));
    let body = ureq::get(&url).call()?.body_mut().read_to_string()?;
    let version = parse_npm_latest_version(&body)?;
    if let Some(version) = &version {
        output::ok(&format!("Resolved {} latest -> {}", display_name, version));
    }
    Ok(version)
}

fn parse_npm_latest_version(body: &str) -> Result<Option<String>> {
    let metadata: serde_json::Value = serde_json::from_str(body)?;
    Ok(metadata
        .get("version")
        .and_then(|value| value.as_str())
        .filter(|version| !version.is_empty())
        .map(str::to_string))
}

/// Resolve the latest stable Rust version from the release channel manifest.
fn resolve_rustc() -> Result<Option<String>> {
    let url = "https://static.rust-lang.org/dist/channel-rust-stable.toml";
    output::info("Resolving latest stable Rust version...");
    let body = ureq::get(url).call()?.body_mut().read_to_string()?;
    for line in body.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("version = \"")
            && let Some(ver) = rest.split_whitespace().next()
        {
            let ver = ver.trim_matches('"');
            if ver.split('.').count() >= 2 {
                output::ok(&format!("Resolved rustc latest -> {}", ver));
                return Ok(Some(ver.to_string()));
            }
        }
    }
    Ok(None)
}

/// Resolve the latest LTS Node.js version from the distribution index.
fn resolve_node() -> Result<Option<String>> {
    let url = "https://nodejs.org/dist/index.json";
    output::info("Resolving latest LTS Node.js version...");
    let body = ureq::get(url).call()?.body_mut().read_to_string()?;
    let entries: Vec<serde_json::Value> = serde_json::from_str(&body)?;
    for entry in &entries {
        if entry.get("lts").and_then(|v| v.as_str()).is_some()
            && let Some(version) = entry.get("version").and_then(|v| v.as_str())
        {
            let major = version
                .trim_start_matches('v')
                .split('.')
                .next()
                .unwrap_or(version);
            output::ok(&format!("Resolved node latest -> {}", major));
            return Ok(Some(major.to_string()));
        }
    }
    Ok(None)
}

/// Resolve the latest stable Python version from the endoflife.date API.
fn resolve_python() -> Result<Option<String>> {
    let url = "https://endoflife.date/api/python.json";
    output::info("Resolving latest stable Python version...");
    let body = ureq::get(url).call()?.body_mut().read_to_string()?;
    let entries: Vec<serde_json::Value> = serde_json::from_str(&body)?;
    if let Some(latest) = entries
        .first()
        .and_then(|e| e.get("latest"))
        .and_then(|v| v.as_str())
    {
        let parts: Vec<&str> = latest.split('.').collect();
        if parts.len() >= 2 {
            let short = format!("{}.{}", parts[0], parts[1]);
            output::ok(&format!("Resolved python latest -> {}", short));
            return Ok(Some(short));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_tool_returns_none() {
        assert!(resolve_latest("unknown-tool-xyz").is_none());
    }

    #[test]
    fn parses_npm_latest_version() {
        assert_eq!(
            parse_npm_latest_version(r#"{"name":"@openai/codex","version":"0.147.0"}"#).unwrap(),
            Some("0.147.0".to_string())
        );
        assert_eq!(
            parse_npm_latest_version(r#"{"name":"@openai/codex"}"#).unwrap(),
            None
        );
    }
}
