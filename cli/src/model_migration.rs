use anyhow::{Context, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const MARKER: &str = "legacy-model-to-artifact-model-spec";

#[derive(Debug, Clone)]
struct LegacyModel {
    id: String,
    path: PathBuf,
    artifact_id: String,
    artifact_path: PathBuf,
    provider: Option<String>,
    family: Option<String>,
}

#[derive(Debug, Clone)]
struct LegacyBinding {
    path: PathBuf,
    target: String,
    artifact_id: String,
}

pub fn write_legacy_model_spec_migration(project_root: &Path) -> Result<Option<PathBuf>> {
    let models = collect_legacy_models(project_root)?;
    let bindings = collect_legacy_model_bindings(project_root, &models)?;
    let legacy_schema = project_root.join("context/schemas/model.yaml");

    if models.is_empty() && bindings.is_empty() && !legacy_schema.is_file() {
        return Ok(None);
    }
    if migration_already_exists(project_root)? {
        return Ok(None);
    }

    let pending_dir = project_root.join("context/migrations/pending");
    fs::create_dir_all(&pending_dir)
        .with_context(|| format!("failed to create {}", pending_dir.display()))?;

    let now = chrono::Local::now();
    let id = format!(
        "MIG-{}-legacy-model-to-artifact-model-spec",
        now.format("%Y%m%d_%H%M%S")
    );
    let out_path = pending_dir.join(format!("{}.md", id));

    let mut body = String::new();
    body.push_str("---\n");
    body.push_str("apiVersion: processkit.projectious.work/v1\n");
    body.push_str("kind: Migration\n");
    body.push_str("metadata:\n");
    body.push_str(&format!("  id: {}\n", id));
    body.push_str(&format!("  created: {}\n", now.to_rfc3339()));
    body.push_str("spec:\n");
    body.push_str("  source: aibox\n");
    body.push_str("  state: pending\n");
    body.push_str("  generated_by: aibox apply\n");
    body.push_str(&format!("  generated_at: {}\n", now.to_rfc3339()));
    body.push_str(&format!(
        "  summary: {}\n",
        yaml_scalar("Legacy Model entities must migrate to Artifact model-spec records")
    ));
    body.push_str("  affected_groups:\n");
    body.push_str("    - models\n");
    body.push_str("    - bindings\n");
    body.push_str(&format!("  marker: {}\n", MARKER));
    body.push_str("---\n\n");

    body.push_str(&format!("# Migration {}\n\n", id));
    body.push_str("processkit no longer treats `Model` as a live primitive. Convert legacy model descriptions into `Artifact` entities with `spec.kind: model-spec`, then point `model-assignment` bindings at those artifacts.\n\n");

    if !models.is_empty() {
        body.push_str("## Model entity conversions\n\n");
        for model in &models {
            body.push_str(&format!(
                "### `{}` -> `{}`\n\n",
                rel(project_root, &model.path),
                model.artifact_path.display()
            ));
            body.push_str("Create this artifact:\n\n");
            body.push_str("```yaml\n");
            body.push_str("---\n");
            body.push_str("apiVersion: processkit.projectious.work/v1\n");
            body.push_str("kind: Artifact\n");
            body.push_str("metadata:\n");
            body.push_str(&format!("  id: {}\n", model.artifact_id));
            body.push_str("spec:\n");
            body.push_str(&format!(
                "  name: {}\n",
                yaml_scalar(&format!("Model spec: {}", model_display_name(model)))
            ));
            body.push_str("  kind: model-spec\n");
            body.push_str(&format!("  legacy_model_id: {}\n", yaml_scalar(&model.id)));
            body.push_str(&format!(
                "  source_model_file: {}\n",
                yaml_scalar(&rel(project_root, &model.path))
            ));
            if let Some(provider) = &model.provider {
                body.push_str(&format!("  provider: {}\n", yaml_scalar(provider)));
            }
            if let Some(family) = &model.family {
                body.push_str(&format!("  family: {}\n", yaml_scalar(family)));
            }
            body.push_str("---\n");
            body.push_str("```\n\n");
            body.push_str("Move the old `spec` fields into the artifact spec or Markdown body so provider, version, pricing, governance, lifecycle, and score metadata remain available for processkit projections such as `model_scores.json`.\n\n");
        }
    }

    if !bindings.is_empty() {
        body.push_str("## Binding rewrites\n\n");
        body.push_str("Rewrite these `model-assignment` bindings so the target is the model-spec Artifact and `target_kind` is `Artifact`:\n\n");
        for binding in &bindings {
            body.push_str(&format!(
                "- `{}`: `target: {}` -> `target: {}`; set `target_kind: Artifact`\n",
                rel(project_root, &binding.path),
                binding.target,
                binding.artifact_id
            ));
        }
        body.push('\n');
    }

    if legacy_schema.is_file() {
        body.push_str("## Legacy schema cleanup\n\n");
        body.push_str("- Archive or remove `context/schemas/model.yaml` after all model files and bindings have been migrated.\n");
    }
    if project_root.join("context/models").is_dir() {
        body.push_str("- Archive or remove `context/models/` after the new `context/artifacts/ART-*-model-spec.md` files validate.\n");
    }

    body.push_str("\n## Validation\n\n");
    body.push_str("- Run `pk-doctor` and resolve any artifact or binding schema errors.\n");
    body.push_str("- Rebuild or refresh model-recommender projections so `model_scores.json` is derived from model-spec artifacts, not hidden `MODEL-*` records.\n");

    fs::write(&out_path, body)
        .with_context(|| format!("failed to write {}", out_path.display()))?;
    Ok(Some(out_path))
}

fn collect_legacy_models(project_root: &Path) -> Result<Vec<LegacyModel>> {
    let models_dir = project_root.join("context/models");
    if !models_dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut models = Vec::new();
    for path in walk_files(&models_dir)? {
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let body = fs::read_to_string(&path)?;
        let id = yaml_get_str(&body, &["metadata", "id"])
            .or_else(|| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .map(str::to_string)
            })
            .unwrap_or_else(|| "MODEL-unknown".to_string());
        if !id.starts_with("MODEL-") {
            continue;
        }
        let provider = yaml_get_str(&body, &["spec", "provider"]);
        let family = yaml_get_str(&body, &["spec", "family"]);
        let artifact_id = model_artifact_id(&id);
        let artifact_path = PathBuf::from("context/artifacts").join(format!("{artifact_id}.md"));
        models.push(LegacyModel {
            id,
            path,
            artifact_id,
            artifact_path,
            provider,
            family,
        });
    }
    models.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(models)
}

fn collect_legacy_model_bindings(
    project_root: &Path,
    models: &[LegacyModel],
) -> Result<Vec<LegacyBinding>> {
    let bindings_dir = project_root.join("context/bindings");
    if !bindings_dir.is_dir() {
        return Ok(Vec::new());
    }

    let known: BTreeMap<String, String> = models
        .iter()
        .map(|model| (model.id.clone(), model.artifact_id.clone()))
        .collect();
    let mut bindings = Vec::new();
    let mut seen = BTreeSet::new();
    for path in walk_files(&bindings_dir)? {
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let body = fs::read_to_string(&path)?;
        if yaml_get_str(&body, &["spec", "type"]).as_deref() != Some("model-assignment") {
            continue;
        }
        let Some(target) = yaml_get_str(&body, &["spec", "target"]) else {
            continue;
        };
        if !target.starts_with("MODEL-") {
            continue;
        }
        if seen.insert((path.clone(), target.to_string())) {
            let artifact_id = known
                .get(&target)
                .cloned()
                .unwrap_or_else(|| model_artifact_id(&target));
            bindings.push(LegacyBinding {
                path,
                target,
                artifact_id,
            });
        }
    }
    bindings.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(bindings)
}

fn migration_already_exists(project_root: &Path) -> Result<bool> {
    for dir in ["pending", "in-progress", "applied"] {
        let path = project_root.join("context/migrations").join(dir);
        if !path.is_dir() {
            continue;
        }
        for file in walk_files(&path)? {
            if file.extension().and_then(|ext| ext.to_str()) == Some("md")
                && fs::read_to_string(&file)
                    .map(|body| body.contains(MARKER))
                    .unwrap_or(false)
            {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn walk_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    if !root.exists() {
        return Ok(out);
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            out.extend(walk_files(&path)?);
        } else if path.is_file() {
            out.push(path);
        }
    }
    Ok(out)
}

fn yaml_get_str(body: &str, path: &[&str]) -> Option<String> {
    let frontmatter = body.strip_prefix("---\n")?;
    let end = frontmatter.find("\n---")?;
    let yaml = &frontmatter[..end];
    let value: serde_yaml::Value = serde_yaml::from_str(yaml).ok()?;
    let mut current = &value;
    for key in path {
        current = current.get(serde_yaml::Value::String((*key).to_string()))?;
    }
    current.as_str().map(str::to_string)
}

fn model_artifact_id(model_id: &str) -> String {
    format!(
        "ART-{}-model-spec",
        model_id
            .trim_start_matches("MODEL-")
            .chars()
            .map(|ch| if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            })
            .collect::<String>()
            .split('-')
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    )
}

fn model_display_name(model: &LegacyModel) -> String {
    match (&model.provider, &model.family) {
        (Some(provider), Some(family)) => format!("{provider} {family}"),
        _ => model.id.clone(),
    }
}

fn rel(project_root: &Path, path: &Path) -> String {
    path.strip_prefix(project_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn yaml_scalar(value: &str) -> String {
    serde_yaml::to_string(value)
        .unwrap_or_else(|_| format!("{value:?}"))
        .trim()
        .trim_start_matches("---")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_legacy_model_spec_migration() {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("context/models")).unwrap();
        fs::create_dir_all(tmp.path().join("context/bindings")).unwrap();
        fs::create_dir_all(tmp.path().join("context/schemas")).unwrap();
        fs::write(
            tmp.path()
                .join("context/models/MODEL-anthropic-claude-haiku.md"),
            "---\nmetadata:\n  id: MODEL-anthropic-claude-haiku\nspec:\n  provider: anthropic\n  family: claude-haiku\n---\n",
        )
        .unwrap();
        fs::write(
            tmp.path().join("context/bindings/BIND-model.md"),
            "---\nspec:\n  type: model-assignment\n  target: MODEL-anthropic-claude-haiku\n---\n",
        )
        .unwrap();
        fs::write(
            tmp.path().join("context/schemas/model.yaml"),
            "kind: Schema\n",
        )
        .unwrap();

        let path = write_legacy_model_spec_migration(tmp.path())
            .unwrap()
            .expect("migration should be written");
        let body = fs::read_to_string(path).unwrap();
        assert!(body.contains(MARKER));
        assert!(body.contains("ART-anthropic-claude-haiku-model-spec"));
        assert!(body.contains("target_kind: Artifact"));
        assert!(body.contains("context/schemas/model.yaml"));

        assert!(
            write_legacy_model_spec_migration(tmp.path())
                .unwrap()
                .is_none(),
            "migration should be idempotent"
        );
    }
}
