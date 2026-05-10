/// Sync managed tmux runtime files from `AiboxConfig`.
///
/// `sync_tmux_runtime_files` is the narrowly-scoped counterpart of
/// `seed::sync_theme_files`: it refreshes only the tmux-related surfaces
/// (tmux.conf, layout scripts, aibox-session.sh) and is called by
/// `aibox up --forget-tmux-state`.
use anyhow::Result;

use super::layouts::{tmux_layout_script, tmux_session_script};
use super::status::tmux_conf;
use crate::config::{AiboxConfig, ConfigLayout};
use crate::seed::{
    ensure_executable, ensure_executable_if_present, ensure_runtime_dirs, force_seed_file,
    include_lazygit_tab, tool_windows_for_config,
};

/// Refresh managed tmux runtime files from aibox.toml.
///
/// This is intentionally narrower than `sync_theme_files`: `aibox up
/// --forget-tmux-state` needs the configured tmux status/layout to win when
/// recreating a session, but it should not rewrite unrelated runtime surfaces.
pub fn sync_tmux_runtime_files(config: &AiboxConfig) -> Result<Vec<String>> {
    let root = config.host_root_dir();
    let mut updated = Vec::new();

    ensure_runtime_dirs(config)?;
    if force_seed_file(
        &root.join(".config").join("tmux").join("tmux.conf"),
        &tmux_conf(config),
    )? {
        updated.push(".config/tmux/tmux.conf".to_string());
    }

    let providers = &config.ai.harnesses;
    let include_lazygit = include_lazygit_tab(config);
    let tool_windows = tool_windows_for_config(config);
    let session_name = config.tmux_session_name();
    for layout in [
        ConfigLayout::Dev,
        ConfigLayout::Focus,
        ConfigLayout::Cowork,
        ConfigLayout::Browse,
        ConfigLayout::Ai,
        ConfigLayout::CoworkSwap,
    ] {
        let rel = format!(".config/tmux/layouts/{layout}.sh");
        let path = root
            .join(".config")
            .join("tmux")
            .join("layouts")
            .join(format!("{layout}.sh"));
        let body = tmux_layout_script(
            &layout,
            providers,
            include_lazygit,
            &tool_windows,
            &session_name,
        );
        if force_seed_file(&path, &body)? {
            ensure_executable(&path)?;
            updated.push(rel);
        } else if ensure_executable_if_present(&path)? {
            updated.push(format!("{rel} (chmod +x)"));
        }
    }

    let session_path = root.join(".config").join("tmux").join("aibox-session.sh");
    if force_seed_file(&session_path, &tmux_session_script(config))? {
        ensure_executable(&session_path)?;
        updated.push(".config/tmux/aibox-session.sh".to_string());
    } else if ensure_executable_if_present(&session_path)? {
        updated.push(".config/tmux/aibox-session.sh (chmod +x)".to_string());
    }

    Ok(updated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TmuxStatusMode;
    use serial_test::serial;
    use std::fs;

    fn make_config(root_dir: std::path::PathBuf) -> AiboxConfig {
        crate::config::set_test_host_root(Some(root_dir));
        let mut config = crate::config::test_config();
        config.container.name = "test".to_string();
        config.container.hostname = "test".to_string();
        config
    }

    fn clear_test_host_root() {
        crate::config::set_test_host_root(None);
        unsafe {
            std::env::remove_var("AIBOX_HOST_ROOT");
        }
    }

    #[test]
    #[serial]
    fn sync_tmux_runtime_files_refreshes_status_and_layout_scripts() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let mut config = make_config(root.clone());
        config.customization.tmux.status.mode = TmuxStatusMode::Extended;
        fs::create_dir_all(root.join(".config/tmux/layouts")).unwrap();
        fs::write(
            root.join(".config/tmux/tmux.conf"),
            r#"set -g status off
set -g status-right " off_RIGHT "
"#,
        )
        .unwrap();
        fs::write(
            root.join(".config/tmux/layouts/ai.sh"),
            r#"tool_or_shell() {
  local tool="$1"
  printf "bash -lc 'if command -v %q >/dev/null 2>&1; then %q; fi; exec bash'" "$tool" "$tool"
}
"#,
        )
        .unwrap();

        let updated = sync_tmux_runtime_files(&config).unwrap();
        let tmux_conf_text = fs::read_to_string(root.join(".config/tmux/tmux.conf")).unwrap();
        let ai_layout = fs::read_to_string(root.join(".config/tmux/layouts/ai.sh")).unwrap();

        assert!(updated.contains(&".config/tmux/tmux.conf".to_string()));
        assert!(updated.contains(&".config/tmux/layouts/ai.sh".to_string()));
        assert!(tmux_conf_text.contains("set -g status on"));
        assert!(tmux_conf_text.contains("tmux-powerkit.tmux"));
        assert!(!tmux_conf_text.contains("off_RIGHT"));
        assert!(ai_layout.contains("list-clients"));
        clear_test_host_root();
    }
}
