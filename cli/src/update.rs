use anyhow::{Context, Result};
use serde_json::Value;
use std::path::{Path, PathBuf};

use crate::config::AiboxConfig;
use crate::{generate, output, seed};

// --- Response structs for JSON deserialization ---

#[derive(serde::Deserialize)]
struct TagsList {
    tags: Vec<String>,
}

#[derive(serde::Deserialize)]
struct TokenResponse {
    token: String,
}

#[derive(serde::Deserialize)]
struct GhRelease {
    tag_name: String,
}

// --- Fetch latest image version from GHCR tags ---

/// Maximum tag-list pages we'll walk before giving up. The page size
/// (`?n=1000`) below combined with this bound permits up to 50k tags before
/// the resolver capitulates — orders of magnitude past current usage and
/// past Docker Registry's typical per-page cap.
const GHCR_MAX_PAGES: usize = 50;
const GHCR_MANIFEST_ACCEPT: &str = concat!(
    "application/vnd.oci.image.index.v1+json, ",
    "application/vnd.docker.distribution.manifest.list.v2+json, ",
    "application/vnd.oci.image.manifest.v1+json, ",
    "application/vnd.docker.distribution.manifest.v2+json"
);
const IMAGE_TAG_PREFIX_RUNTIME: &str = "base-{}-runtime-v";
const IMAGE_TAG_PREFIX_LEGACY: &str = "base-{}-v";

#[derive(Debug)]
struct ImageTagCandidate {
    is_runtime: bool,
    tag: String,
    version: semver::Version,
}

/// Query the GHCR tags list for the given image flavor and return the highest
/// semver version found.
///
/// Walks Docker Registry v2 pagination via the `Link: <…>; rel="next"`
/// header. Pre-v0.26.3 this function only read the first response page, so
/// any tag pushed after GHCR's default page-1 cutoff was invisible — a
/// fresh `aibox apply` would keep resolving `latest` to a long-stale image
/// (BACK-20260514_1902-ShinyLake).
pub(crate) fn fetch_latest_image_version(flavor: &str) -> Result<semver::Version> {
    let running_version = semver::Version::parse(env!("CARGO_PKG_VERSION"))
        .context("running aibox CLI has an invalid package version")?;
    fetch_latest_image_version_for_major(flavor, running_version.major)
}

/// Resolve the newest published image within one maintained major release line.
///
/// The registry contains independent v0.x and v1.x images. Treating all tags as
/// one SemVer stream allowed a v0 CLI configured with `latest` to select a v1
/// prerelease image. The running CLI's major version is the release-line
/// boundary; prereleases remain eligible inside that boundary.
fn fetch_latest_image_version_for_major(
    flavor: &str,
    release_major: u64,
) -> Result<semver::Version> {
    let all_tags = fetch_all_ghcr_tags()?;

    let mut versions = image_tag_candidates_for_major(&all_tags, flavor, release_major);

    if versions.is_empty() {
        anyhow::bail!(
            "No published v{}.x tags found for flavor '{}'",
            release_major,
            flavor
        );
    }

    versions.sort_by(|a, b| {
        b.version
            .cmp(&a.version)
            .then_with(|| b.is_runtime.cmp(&a.is_runtime))
    });

    for candidate in versions {
        let tag = candidate.tag;
        match ghcr_image_manifest_is_complete(&tag) {
            Ok(true) => return Ok(candidate.version),
            Ok(false) => continue,
            Err(err) => {
                return Err(err).with_context(|| format!("Failed to verify GHCR tag '{tag}'"));
            }
        }
    }

    anyhow::bail!(
        "No usable published v{}.x tags found for flavor '{}' — all matching GHCR tags have incomplete manifests",
        release_major,
        flavor,
    )
}

fn image_tag_candidates_for_major(
    all_tags: &[String],
    flavor: &str,
    release_major: u64,
) -> Vec<ImageTagCandidate> {
    all_tags
        .iter()
        .filter_map(|tag| parse_image_tag_version(tag, flavor))
        .filter(|candidate| candidate.version.major == release_major)
        .collect()
}

fn parse_image_tag_version(tag: &str, flavor: &str) -> Option<ImageTagCandidate> {
    let runtime_prefix = IMAGE_TAG_PREFIX_RUNTIME.replace("{}", flavor);
    let legacy_prefix = IMAGE_TAG_PREFIX_LEGACY.replace("{}", flavor);

    if let Some(raw_version) = tag.strip_prefix(&runtime_prefix) {
        return semver::Version::parse(raw_version)
            .ok()
            .map(|version| ImageTagCandidate {
                is_runtime: true,
                tag: tag.to_string(),
                version,
            });
    }

    if let Some(raw_version) = tag.strip_prefix(&legacy_prefix) {
        return semver::Version::parse(raw_version)
            .ok()
            .map(|version| ImageTagCandidate {
                is_runtime: false,
                tag: tag.to_string(),
                version,
            });
    }

    None
}

/// Walk every page of the GHCR tag listing for the aibox repository,
/// concatenating the `tags` arrays. Starts with `?n=1000` for a generous
/// first page; honors the `Link: <…>; rel="next"` header for any remaining
/// pages the registry advertises.
fn fetch_all_ghcr_tags() -> Result<Vec<String>> {
    let mut all: Vec<String> = Vec::new();
    let mut next_url: Option<String> =
        Some("https://ghcr.io/v2/projectious-work/aibox/tags/list?n=1000".to_string());

    let mut iters = 0;
    while let Some(url) = next_url.take() {
        iters += 1;
        if iters > GHCR_MAX_PAGES {
            anyhow::bail!(
                "GHCR pagination did not terminate within {} pages — refusing to loop",
                GHCR_MAX_PAGES
            );
        }
        let (tags, next) = ghcr_get_tags_page(&url)?;
        all.extend(tags);
        next_url = next;
    }
    Ok(all)
}

/// Perform a GET against GHCR (with anonymous-Bearer token exchange on
/// 401/403), parse the body as a `TagsList`, and return both the tags and
/// the parsed `rel="next"` URL from the response's `Link` header (or
/// `None` if absent).
fn ghcr_get_tags_page(url: &str) -> Result<(Vec<String>, Option<String>)> {
    let response = match ureq::get(url).header("User-Agent", "aibox-cli").call() {
        Ok(r) => r,
        Err(ureq::Error::StatusCode(401)) | Err(ureq::Error::StatusCode(403)) => {
            let token_url = "https://ghcr.io/token?service=ghcr.io&scope=repository:projectious-work/aibox:pull";
            let token_resp = ureq::get(token_url)
                .header("User-Agent", "aibox-cli")
                .call()?;
            let token_body = token_resp.into_body().read_to_string()?;
            let token_data: TokenResponse = serde_json::from_str(&token_body)?;
            ureq::get(url)
                .header("User-Agent", "aibox-cli")
                .header("Authorization", &format!("Bearer {}", token_data.token))
                .call()?
        }
        Err(e) => return Err(e.into()),
    };
    let next = response
        .headers()
        .get("Link")
        .and_then(|h| h.to_str().ok())
        .and_then(parse_next_link_url);
    let body = response.into_body().read_to_string()?;
    let tags_list: TagsList = serde_json::from_str(&body)?;
    Ok((tags_list.tags, next))
}

fn ghcr_image_manifest_is_complete(tag: &str) -> Result<bool> {
    let Some(manifest) = ghcr_get_manifest_json(tag)? else {
        return Ok(false);
    };

    let Some(media_type) = manifest.get("mediaType").and_then(Value::as_str) else {
        return Ok(false);
    };
    if matches!(
        media_type,
        "application/vnd.oci.image.manifest.v1+json"
            | "application/vnd.docker.distribution.manifest.v2+json"
    ) {
        return Ok(true);
    }

    if matches!(
        media_type,
        "application/vnd.oci.image.index.v1+json"
            | "application/vnd.docker.distribution.manifest.list.v2+json"
    ) {
        return Ok(!runnable_manifest_child_digests(&manifest).is_empty());
    }

    Ok(false)
}

fn ghcr_get_manifest_json(reference: &str) -> Result<Option<Value>> {
    let url = format!("https://ghcr.io/v2/projectious-work/aibox/manifests/{reference}");
    let response = match ureq::get(&url)
        .header("User-Agent", "aibox-cli")
        .header("Accept", GHCR_MANIFEST_ACCEPT)
        .call()
    {
        Ok(r) => r,
        Err(ureq::Error::StatusCode(401)) | Err(ureq::Error::StatusCode(403)) => {
            let token_url = "https://ghcr.io/token?service=ghcr.io&scope=repository:projectious-work/aibox:pull";
            let token_resp = ureq::get(token_url)
                .header("User-Agent", "aibox-cli")
                .call()?;
            let token_body = token_resp.into_body().read_to_string()?;
            let token_data: TokenResponse = serde_json::from_str(&token_body)?;
            match ureq::get(&url)
                .header("User-Agent", "aibox-cli")
                .header("Accept", GHCR_MANIFEST_ACCEPT)
                .header("Authorization", &format!("Bearer {}", token_data.token))
                .call()
            {
                Ok(r) => r,
                Err(ureq::Error::StatusCode(404)) => return Ok(None),
                Err(e) => return Err(e.into()),
            }
        }
        Err(ureq::Error::StatusCode(404)) => return Ok(None),
        Err(e) => return Err(e.into()),
    };

    let body = response.into_body().read_to_string()?;
    Ok(Some(serde_json::from_str(&body)?))
}

fn runnable_manifest_child_digests(manifest: &Value) -> Vec<String> {
    let Some(media_type) = manifest.get("mediaType").and_then(Value::as_str) else {
        return Vec::new();
    };
    if !matches!(
        media_type,
        "application/vnd.oci.image.index.v1+json"
            | "application/vnd.docker.distribution.manifest.list.v2+json"
    ) {
        return Vec::new();
    }

    manifest
        .get("manifests")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|entry| manifest_child_is_runnable_platform(entry))
        .filter_map(|entry| entry.get("digest").and_then(Value::as_str))
        .map(ToOwned::to_owned)
        .collect()
}

fn manifest_child_is_runnable_platform(entry: &Value) -> bool {
    let platform = entry.get("platform").and_then(Value::as_object);
    let os = platform
        .and_then(|p| p.get("os"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    let architecture = platform
        .and_then(|p| p.get("architecture"))
        .and_then(Value::as_str)
        .unwrap_or_default();

    !os.is_empty() && !architecture.is_empty() && os != "unknown" && architecture != "unknown"
}

/// Parse a Docker Registry v2 / RFC 5988 `Link` header and extract the URL
/// of the `rel="next"` relation, if present. Tolerates absolute and root-
/// relative URLs; resolves the latter against `https://ghcr.io`.
///
/// Examples:
///   `</v2/foo/tags/list?n=1000&last=v1>; rel="next"`
///   `<https://ghcr.io/v2/...?last=v1>; rel="next"`
///   `</v2/foo>; rel="prev", </v2/foo?last=v1>; rel="next"`
fn parse_next_link_url(link_header: &str) -> Option<String> {
    for entry in link_header.split(',') {
        let entry = entry.trim();
        if !entry.contains("rel=\"next\"") && !entry.contains("rel=next") {
            continue;
        }
        let start = entry.find('<')?;
        let end_offset = entry[start..].find('>')?;
        let end = start + end_offset;
        if end <= start + 1 {
            continue;
        }
        let target = &entry[start + 1..end];
        if target.starts_with("https://") || target.starts_with("http://") {
            return Some(target.to_string());
        }
        let glue = if target.starts_with('/') { "" } else { "/" };
        return Some(format!("https://ghcr.io{}{}", glue, target));
    }
    None
}

// --- Fetch latest CLI version from GitHub releases ---

/// Query the GitHub releases API for the latest release tag and parse it as
/// a semver version.
fn fetch_latest_cli_version() -> Result<semver::Version> {
    let url = "https://api.github.com/repos/projectious-work/aibox/releases/latest";
    let response = ureq::get(url)
        .header("User-Agent", "aibox-cli")
        .header("Accept", "application/vnd.github+json")
        .call()?;
    let body = response.into_body().read_to_string()?;
    let release: GhRelease = serde_json::from_str(&body)?;

    let version_str = release
        .tag_name
        .strip_prefix('v')
        .unwrap_or(&release.tag_name);
    let version = semver::Version::parse(version_str)?;
    Ok(version)
}

/// Check for available updates (CLI + image versions).
fn check_updates(config: &AiboxConfig) -> Result<()> {
    output::info("Checking for updates...");

    // --- CLI version ---
    let current_cli = env!("CARGO_PKG_VERSION");
    output::ok(&format!("Current CLI version: {}", current_cli));

    match fetch_latest_cli_version() {
        Ok(latest) => {
            let current = semver::Version::parse(current_cli)
                .unwrap_or_else(|_| semver::Version::new(0, 0, 0));
            if latest > current {
                output::warn(&format!(
                    "New CLI version available: {} -> {} \
                     (https://github.com/projectious-work/aibox/releases/latest)",
                    current, latest
                ));
            } else {
                output::ok("CLI is up to date");
            }
        }
        Err(e) => {
            output::warn(&format!("Could not check latest CLI version: {}", e));
        }
    }

    // --- Image version ---
    let flavor = config.container.image.base.to_string();
    output::ok(&format!(
        "Current config image version: {} ({})",
        config.container.image.version, flavor
    ));

    match fetch_latest_image_version(&flavor) {
        Ok(latest) => {
            let current = semver::Version::parse(&config.container.image.version)
                .unwrap_or_else(|_| semver::Version::new(0, 0, 0));
            if latest > current {
                output::warn(&format!(
                    "New image version available for '{}': {} -> {} \
                     (run 'aibox self update' to upgrade)",
                    flavor, current, latest
                ));
            } else {
                output::ok(&format!("Image '{}' is up to date", flavor));
            }
        }
        Err(e) => {
            output::warn(&format!(
                "Could not check latest image version for '{}': {}",
                flavor, e
            ));
        }
    }

    // --- Schema version (informational) ---
    output::ok(&format!(
        "Schema version: {}",
        config.context.schema_version
    ));

    Ok(())
}

/// Resolve the config file path, preferring the CLI option, then default.
fn resolve_config_path(config_path: &Option<String>) -> PathBuf {
    match config_path {
        Some(p) => PathBuf::from(p),
        None => PathBuf::from("aibox.toml"),
    }
}

/// Update the version field in aibox.toml using string replacement to preserve comments.
fn update_toml_version(toml_path: &Path, old_version: &str, new_version: &str) -> Result<()> {
    let content =
        std::fs::read_to_string(toml_path).context("Failed to read aibox.toml for upgrade")?;

    // Prefer the canonical [container.image] key, then fall back to the legacy
    // [aibox].version spelling for old projects.
    let candidates = [
        (
            format!("release_version = \"{}\"", old_version),
            format!("release_version = \"{}\"", new_version),
        ),
        (
            format!("version = \"{}\"", old_version),
            format!("version = \"{}\"", new_version),
        ),
    ];
    for (old_pattern, new_pattern) in candidates {
        if content.contains(&old_pattern) {
            let updated = content.replacen(&old_pattern, &new_pattern, 1);
            std::fs::write(toml_path, updated).context("Failed to write updated aibox.toml")?;
            return Ok(());
        }
    }

    anyhow::bail!(
        "Could not find image version {} in aibox.toml — manual edit may be needed",
        old_version
    )
}

/// Shared helper: seed + generate.
///
/// Skills, AGENTS.md, and the universal baseline are owned by processkit
/// since v0.16.0 and refreshed via the content-source diff in
/// `cmd_sync`. This helper now only handles the slice that is intrinsic
/// to aibox.
fn sync_config_files(config: &AiboxConfig) -> Result<()> {
    seed::seed_root_dir(config)?;
    generate::generate_all(config)?;
    Ok(())
}

/// Prompt the user with a yes/no question. Returns true if they answer yes.
/// If `auto_yes` is set, returns true without prompting.
fn ask_yes_no(question: &str, auto_yes: bool) -> bool {
    if auto_yes {
        return true;
    }
    eprint!("{} [y/N] ", question);
    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_err() {
        return false;
    }
    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

/// Perform the upgrade: fetch latest image version, update aibox.toml, regenerate files.
fn do_upgrade(config_path: &Option<String>, dry_run: bool, global_yes: bool) -> Result<()> {
    let config = AiboxConfig::from_cli_option(config_path)?;
    let flavor = config.container.image.base.to_string();
    let current_version = &config.container.image.version;

    output::info(&format!(
        "Current image version: {} ({})",
        current_version, flavor
    ));

    // Fetch latest image version from GHCR
    output::info("Fetching latest image version from registry...");
    let latest = match fetch_latest_image_version(&flavor) {
        Ok(v) => v,
        Err(e) => {
            output::warn(&format!(
                "Could not fetch latest image version from registry: {}\n\
                 If the registry requires authentication, try: docker login ghcr.io",
                e
            ));
            return Ok(());
        }
    };
    let current =
        semver::Version::parse(current_version).unwrap_or_else(|_| semver::Version::new(0, 0, 0));

    if latest <= current {
        output::ok(&format!(
            "Image '{}' is already at the latest version ({})",
            flavor, current
        ));
        return Ok(());
    }

    let latest_str = latest.to_string();
    output::ok(&format!(
        "New version available: {} -> {}",
        current, latest_str
    ));

    if dry_run {
        output::info("[dry-run] Would update version in aibox.toml");
        output::info("[dry-run] Would regenerate .devcontainer/ files");
        return Ok(());
    }

    // Confirm before applying
    if !ask_yes_no(
        &format!("Upgrade image from {} to {}?", current, latest_str),
        global_yes,
    ) {
        output::info("Upgrade cancelled");
        return Ok(());
    }

    // 1. Update version in aibox.toml
    let toml_path = resolve_config_path(config_path);
    update_toml_version(&toml_path, current_version, &latest_str)?;
    output::ok(&format!(
        "Updated version in {} ({} -> {})",
        toml_path.display(),
        current_version,
        latest_str
    ));

    // 2. Reload config with updated version and sync all config files
    let updated_config = AiboxConfig::from_cli_option(config_path)?;
    sync_config_files(&updated_config)?;

    output::ok(&format!("Upgrade complete: {} -> {}", current, latest_str));

    // 3. Offer to rebuild the container image
    if ask_yes_no("Rebuild container image now?", global_yes) {
        match crate::runtime::Runtime::detect() {
            Ok(runtime) => {
                output::info("Building container image...");
                runtime.compose_build(
                    crate::config::COMPOSE_FILE,
                    &updated_config.container.name,
                    false,
                )?;
                output::ok("Container image rebuilt");
            }
            Err(e) => {
                output::warn(&format!(
                    "No container runtime available: {}. Run `aibox apply` to rebuild later.",
                    e
                ));
            }
        }
    } else {
        output::info("Run `aibox apply` to rebuild the container image when ready.");
    }

    Ok(())
}

/// Update command implementation.
pub fn cmd_update(
    config_path: &Option<String>,
    check: bool,
    dry_run: bool,
    global_yes: bool,
) -> Result<()> {
    if check {
        let config = AiboxConfig::from_cli_option(config_path)?;
        check_updates(&config)
    } else {
        do_upgrade(config_path, dry_run, global_yes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_next_link_url ────────────────────────────────────────────────
    //
    // Docker Registry v2 paginates tag listings via RFC 5988 Link headers.
    // The regression we're guarding against is "v0.26.x tag is in GHCR but
    // aibox apply keeps reporting v0.25.12 as latest" — caused by reading
    // only the first page (BACK-20260514_1902-ShinyLake).

    #[test]
    fn parse_next_link_url_extracts_relative_next() {
        let h = r#"</v2/projectious-work/aibox/tags/list?n=1000&last=base-debian-v0.25.12>; rel="next""#;
        assert_eq!(
            parse_next_link_url(h),
            Some(
                "https://ghcr.io/v2/projectious-work/aibox/tags/list?n=1000&last=base-debian-v0.25.12"
                    .to_string()
            )
        );
    }

    #[test]
    fn parse_next_link_url_extracts_absolute_next() {
        let h = r#"<https://ghcr.io/v2/x?last=y>; rel="next""#;
        assert_eq!(
            parse_next_link_url(h),
            Some("https://ghcr.io/v2/x?last=y".to_string())
        );
    }

    #[test]
    fn parse_next_link_url_picks_next_from_multiple_relations() {
        // Multi-relation Link headers list each relation comma-separated.
        let h = r#"</v2/foo>; rel="prev", </v2/foo?last=v1>; rel="next""#;
        assert_eq!(
            parse_next_link_url(h),
            Some("https://ghcr.io/v2/foo?last=v1".to_string())
        );
    }

    #[test]
    fn parse_next_link_url_tolerates_unquoted_rel() {
        // RFC 5988 allows `rel=next` without quotes — guard against the
        // strict-quoted path so a registry quirk doesn't silently drop the
        // header.
        let h = "</v2/foo?last=v1>; rel=next";
        assert_eq!(
            parse_next_link_url(h),
            Some("https://ghcr.io/v2/foo?last=v1".to_string())
        );
    }

    #[test]
    fn parse_next_link_url_returns_none_when_no_next_relation() {
        let h = r#"</v2/foo>; rel="prev""#;
        assert_eq!(parse_next_link_url(h), None);
    }

    #[test]
    fn parse_next_link_url_returns_none_on_malformed_header() {
        assert_eq!(parse_next_link_url(""), None);
        assert_eq!(parse_next_link_url(r#"rel="next""#), None);
        assert_eq!(parse_next_link_url("<>; rel=\"next\""), None);
    }

    #[test]
    fn parse_image_tag_version_supports_runtime_and_legacy_prefixes() {
        let runtime = parse_image_tag_version("base-debian-runtime-v0.27.0", "debian")
            .expect("runtime tag should parse");
        assert_eq!(runtime.version.to_string(), "0.27.0");
        assert!(runtime.is_runtime);

        let legacy = parse_image_tag_version("base-debian-v0.26.0", "debian")
            .expect("legacy tag should parse");
        assert_eq!(legacy.version.to_string(), "0.26.0");
        assert!(!legacy.is_runtime);
    }

    #[test]
    fn parse_image_tag_version_ignores_non_semver_suffixes() {
        assert!(parse_image_tag_version("base-debian-runtime-latest", "debian").is_none());
        assert!(parse_image_tag_version("base-debian-latest", "debian").is_none());
        assert!(parse_image_tag_version("base-ubuntu-v1.2.3", "debian").is_none());
    }

    #[test]
    fn image_latest_candidates_stay_within_running_cli_major_line() {
        let tags = vec![
            "base-debian-runtime-v0.32.3".to_string(),
            "base-debian-runtime-v1.0.0-alpha.1".to_string(),
        ];
        let versions: Vec<_> = image_tag_candidates_for_major(&tags, "debian", 0)
            .into_iter()
            .map(|candidate| candidate.version)
            .collect();

        assert_eq!(versions, vec![semver::Version::parse("0.32.3").unwrap()]);
    }

    #[test]
    fn runnable_manifest_child_digests_extracts_platform_children() {
        let manifest = serde_json::json!({
            "mediaType": "application/vnd.oci.image.index.v1+json",
            "manifests": [
                {
                    "digest": "sha256:111",
                    "platform": { "os": "linux", "architecture": "amd64" }
                },
                {
                    "digest": "sha256:222",
                    "platform": { "os": "linux", "architecture": "arm64" }
                },
                { "mediaType": "application/vnd.oci.image.manifest.v1+json" }
            ]
        });

        assert_eq!(
            runnable_manifest_child_digests(&manifest),
            vec!["sha256:111".to_string(), "sha256:222".to_string()]
        );
    }

    #[test]
    fn runnable_manifest_child_digests_ignores_attestation_children() {
        let manifest = serde_json::json!({
            "mediaType": "application/vnd.oci.image.index.v1+json",
            "manifests": [
                {
                    "digest": "sha256:111",
                    "platform": { "os": "linux", "architecture": "arm64" }
                },
                {
                    "digest": "sha256:attestation",
                    "platform": { "os": "unknown", "architecture": "unknown" },
                    "annotations": { "vnd.docker.reference.type": "attestation-manifest" }
                }
            ]
        });

        assert_eq!(
            runnable_manifest_child_digests(&manifest),
            vec!["sha256:111".to_string()]
        );
    }

    #[test]
    fn runnable_manifest_child_digests_ignores_single_platform_manifest() {
        let manifest = serde_json::json!({
            "mediaType": "application/vnd.oci.image.manifest.v1+json",
            "layers": []
        });

        assert!(runnable_manifest_child_digests(&manifest).is_empty());
    }

    #[test]
    fn update_toml_version_replaces_version() {
        let dir = tempfile::tempdir().unwrap();
        let toml_path = dir.path().join("aibox.toml");
        let content = r#"[aibox]
version = "0.3.5"
image = "python"
process = "research"

[container]
name = "my-project"
"#;
        std::fs::write(&toml_path, content).unwrap();

        update_toml_version(&toml_path, "0.3.5", "0.3.7").unwrap();

        let updated = std::fs::read_to_string(&toml_path).unwrap();
        assert!(updated.contains("version = \"0.3.7\""));
        assert!(!updated.contains("version = \"0.3.5\""));
        // Ensure the rest is preserved
        assert!(updated.contains("image = \"python\""));
        assert!(updated.contains("name = \"my-project\""));
    }

    #[test]
    fn update_toml_version_preserves_comments() {
        let dir = tempfile::tempdir().unwrap();
        let toml_path = dir.path().join("aibox.toml");
        let content = r#"# My project config
[aibox]
# Image version from GHCR
version = "0.2.1"
image = "base"
process = "minimal"
"#;
        std::fs::write(&toml_path, content).unwrap();

        update_toml_version(&toml_path, "0.2.1", "0.3.0").unwrap();

        let updated = std::fs::read_to_string(&toml_path).unwrap();
        assert!(updated.contains("# My project config"));
        assert!(updated.contains("# Image version from GHCR"));
        assert!(updated.contains("version = \"0.3.0\""));
    }

    #[test]
    fn update_toml_version_fails_on_missing_version() {
        let dir = tempfile::tempdir().unwrap();
        let toml_path = dir.path().join("aibox.toml");
        std::fs::write(&toml_path, "[aibox]\nimage = \"base\"\n").unwrap();

        let result = update_toml_version(&toml_path, "0.3.5", "0.3.7");
        assert!(result.is_err());
    }

    #[test]
    fn update_toml_version_only_replaces_first_occurrence() {
        let dir = tempfile::tempdir().unwrap();
        let toml_path = dir.path().join("aibox.toml");
        // Hypothetical: version appears in a comment too
        let content = r#"[aibox]
version = "0.3.5"
image = "base"
process = "minimal"

# Note: schema version = "0.3.5" was also used elsewhere
"#;
        std::fs::write(&toml_path, content).unwrap();

        update_toml_version(&toml_path, "0.3.5", "0.4.0").unwrap();

        let updated = std::fs::read_to_string(&toml_path).unwrap();
        assert!(updated.contains("version = \"0.4.0\""));
        // The comment text should still contain the old version string
        assert!(updated.contains("0.3.5"));
    }
}
