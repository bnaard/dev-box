use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

use crate::config::{AiboxConfig, Theme, ThemeMode};
use crate::output;
use crate::runtime::{ContainerState, Runtime};

fn toml_path(config_path: &Option<String>) -> PathBuf {
    match config_path {
        Some(p) => PathBuf::from(p),
        None => PathBuf::from("aibox.toml"),
    }
}

fn update_theme_toml(path: &Path, mode: &ThemeMode, theme: Option<&Theme>) -> Result<bool> {
    if !path.exists() {
        bail!("No aibox.toml found. Run 'aibox init' first.");
    }

    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let mut doc = content
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("Failed to parse {}", path.display()))?;

    let table_name = if doc.get("customization").is_some() {
        "customization"
    } else if doc.get("appearance").is_some() {
        "appearance"
    } else {
        doc.insert(
            "customization",
            toml_edit::Item::Table(toml_edit::Table::new()),
        );
        "customization"
    };

    if !doc[table_name].is_table() {
        bail!("[{}] in aibox.toml must be a TOML table", table_name);
    }

    let before = doc.to_string();
    doc[table_name]["mode"] = toml_edit::value(mode.to_string());
    if let Some(theme) = theme {
        doc[table_name]["theme"] = toml_edit::value(theme.to_string());
    }

    let after = doc.to_string();
    if before == after {
        return Ok(false);
    }

    std::fs::write(path, after).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(true)
}

fn restart_tmux_session(config: &AiboxConfig) -> Result<()> {
    let runtime = Runtime::detect()?;
    let name = &config.container.name;
    let session_name = config.tmux_session_name();

    match runtime.container_status(name)? {
        ContainerState::Running => {
            let _ = runtime.exec_status(
                name,
                &config.container.user,
                &[
                    "sh",
                    "-lc",
                    r#"socket="${AIBOX_TMUX_SOCKET:-$HOME/.tmux/aibox.sock}"; tmux -S "$socket" kill-session -t "$1" >/dev/null 2>&1 || true"#,
                    "aibox-tmux-kill",
                    &session_name,
                ],
            )?;
            let layout = config.customization.tmux_layout().to_string();
            output::info(&format!("Attaching via tmux (layout: {})...", layout));
            runtime.exec_interactive(
                name,
                &config.container.user,
                &["aibox-tmux-session", &layout, &session_name],
            )?;
        }
        ContainerState::Stopped => {
            output::warn("Container is stopped; theme will apply on next `aibox up`");
        }
        ContainerState::Missing => {
            output::warn("Container is missing; theme will apply after the container is created");
        }
    }

    Ok(())
}

/// Switch the global light/dark theme mode.
pub fn cmd_theme(
    config_path: &Option<String>,
    mode: ThemeMode,
    theme: Option<Theme>,
    restart_session: bool,
) -> Result<()> {
    let path = toml_path(config_path);
    let changed = update_theme_toml(&path, &mode, theme.as_ref())?;
    if changed {
        output::ok("Updated theme settings in aibox.toml");
    } else {
        output::info("Theme settings already match the request");
    }

    let config = AiboxConfig::from_cli_option(config_path)?;
    let updated = crate::seed::sync_theme_files(&config)?;
    if updated.is_empty() {
        output::info("Runtime theme files already up to date");
    } else {
        output::ok(&format!("Updated {} runtime theme file(s)", updated.len()));
        for rel_path in updated {
            output::info(&format!("  {}", rel_path));
        }
    }

    output::info(&format!(
        "Resolved theme: {} (mode: {})",
        config.customization.resolved_theme(),
        config.customization.mode
    ));

    if restart_session {
        restart_tmux_session(&config)?;
    } else {
        output::info(
            "Running TUI processes may need to be restarted. Use `--restart-session` to refresh tmux.",
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_theme_toml_adds_mode_without_touching_theme() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("aibox.toml");
        std::fs::write(
            &path,
            r#"[aibox]
version = "0.21.2"

[container]
name = "test"

[customization]
# keep this comment
theme = "dracula"
"#,
        )
        .unwrap();

        let changed = update_theme_toml(&path, &ThemeMode::Light, None).unwrap();
        assert!(changed);

        let updated = std::fs::read_to_string(&path).unwrap();
        assert!(updated.contains("# keep this comment"));
        assert!(updated.contains(r#"theme = "dracula""#));
        assert!(updated.contains(r#"mode = "light""#));

        let config: AiboxConfig = toml::from_str(&updated).unwrap();
        assert_eq!(config.customization.theme, Theme::Dracula);
        assert_eq!(config.customization.mode, ThemeMode::Light);
        assert_eq!(
            config.customization.resolved_theme(),
            Theme::CatppuccinLatte
        );
    }

    #[test]
    fn update_theme_toml_can_set_concrete_theme() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("aibox.toml");
        std::fs::write(
            &path,
            r#"[aibox]
version = "0.21.2"

[container]
name = "test"
"#,
        )
        .unwrap();

        let changed = update_theme_toml(&path, &ThemeMode::Dark, Some(&Theme::TokyoNight)).unwrap();
        assert!(changed);

        let updated = std::fs::read_to_string(&path).unwrap();
        assert!(updated.contains("[customization]"));
        assert!(updated.contains(r#"theme = "tokyo-night""#));
        assert!(updated.contains(r#"mode = "dark""#));
    }
}
