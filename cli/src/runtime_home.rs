use std::path::PathBuf;

use serde::Serialize;

use crate::config::AiboxConfig;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct RuntimeHomeMount {
    pub source_rel: String,
    pub source: String,
    pub destination: String,
    pub read_only: bool,
    pub comment: &'static str,
}

impl RuntimeHomeMount {
    fn new(
        source_rel: impl Into<String>,
        destination: impl Into<String>,
        read_only: bool,
        comment: &'static str,
    ) -> Self {
        let source_rel = source_rel.into();
        Self {
            source: String::new(),
            source_rel,
            destination: destination.into(),
            read_only,
            comment,
        }
    }

    fn with_source(mut self, host_root: &str) -> Self {
        self.source = format!(
            "{}/{}",
            host_root.trim_end_matches('/'),
            self.source_rel.trim_start_matches('/')
        );
        self
    }
}

pub(crate) fn runtime_home_mounts(config: &AiboxConfig) -> Vec<RuntimeHomeMount> {
    let home = config.container_home();
    let mut mounts = vec![
        RuntimeHomeMount::new(".ssh", format!("{home}/.ssh"), true, "SSH keys"),
        RuntimeHomeMount::new(
            ".vim",
            format!("{home}/.vim"),
            false,
            "Vim config and undo history",
        ),
        RuntimeHomeMount::new(
            ".config",
            format!("{home}/.config"),
            false,
            "XDG config home",
        ),
        RuntimeHomeMount::new(".cache", format!("{home}/.cache"), false, "XDG cache home"),
        RuntimeHomeMount::new(
            ".inputrc",
            format!("{home}/.inputrc"),
            false,
            "readline key bindings",
        ),
        RuntimeHomeMount::new(
            ".local",
            format!("{home}/.local"),
            false,
            "XDG local data, state, and helper scripts",
        ),
        RuntimeHomeMount::new(
            ".tmux",
            format!("{home}/.tmux"),
            false,
            "tmux plugin and socket state",
        ),
    ];

    for provider in &config.ai.harnesses {
        if let Some(dir_name) = provider.config_dir() {
            mounts.push(RuntimeHomeMount::new(
                dir_name,
                format!("{home}/{dir_name}"),
                false,
                "AI harness config and login state",
            ));
        }
    }

    if config
        .ai
        .harnesses
        .contains(&crate::config::AiProvider::Claude)
    {
        mounts.push(RuntimeHomeMount::new(
            ".claude.json",
            format!("{home}/.claude.json"),
            false,
            "Claude Code account state",
        ));
    }

    if config.audio.enabled {
        mounts.push(RuntimeHomeMount::new(
            ".asoundrc",
            format!("{home}/.asoundrc"),
            false,
            "ALSA to PulseAudio bridge config",
        ));
    }

    if config.addons.has_rust() {
        mounts.push(RuntimeHomeMount::new(
            ".cargo/registry",
            format!("{home}/.cargo/registry"),
            false,
            "Cargo registry cache",
        ));
        mounts.push(RuntimeHomeMount::new(
            ".cargo/git",
            format!("{home}/.cargo/git"),
            false,
            "Cargo git cache",
        ));
    }

    mounts
}

pub(crate) fn compose_runtime_home_mounts(
    config: &AiboxConfig,
    host_root: &str,
) -> Vec<RuntimeHomeMount> {
    runtime_home_mounts(config)
        .into_iter()
        .map(|mount| mount.with_source(host_root))
        .collect()
}

pub(crate) fn writable_runtime_home_destinations(config: &AiboxConfig) -> Vec<String> {
    runtime_home_mounts(config)
        .into_iter()
        .filter(|mount| !mount.read_only)
        .map(|mount| mount.destination)
        .collect()
}

pub(crate) fn legacy_runtime_home_destinations(config: &AiboxConfig) -> Vec<String> {
    let home = config.container_home();
    let mut destinations = vec![
        format!("{home}/.config/git"),
        format!("{home}/.config/starship.toml"),
        format!("{home}/.config/state"),
        format!("{home}/.config/tmux"),
        format!("{home}/.config/yazi"),
        format!("{home}/.local/bin"),
    ];
    destinations.sort();
    destinations.dedup();
    destinations
}

pub(crate) fn runtime_home_scaffold_dirs(config: &AiboxConfig) -> Vec<PathBuf> {
    let mut dirs = vec![
        PathBuf::from(".ssh"),
        PathBuf::from(".local/bin"),
        PathBuf::from(".local/share"),
        PathBuf::from(".local/state"),
        PathBuf::from(".vim/undo"),
        PathBuf::from(".vim/colors"),
        PathBuf::from(".config"),
        PathBuf::from(".config/state"),
        PathBuf::from(".config/tmux/layouts"),
        PathBuf::from(".tmux/plugins"),
        PathBuf::from(".cache"),
        PathBuf::from(".cache/starship"),
        PathBuf::from(".cache/tmux-powerkit"),
        PathBuf::from(".cache/uv"),
        PathBuf::from(".config/yazi"),
        PathBuf::from(".config/yazi/plugins/eps.yazi"),
        PathBuf::from(".config/yazi/plugins/svg.yazi"),
        PathBuf::from(".config/yazi/plugins/git.yazi"),
        PathBuf::from(".config/yazi/plugins/dir-preview.yazi"),
        PathBuf::from(".config/yazi/plugins/status-git.yazi"),
        PathBuf::from(".config/git"),
        PathBuf::from(".config/aibox"),
        PathBuf::from(".config/lnav"),
    ];

    if crate::seed::include_lazygit_tab(config) {
        dirs.push(PathBuf::from(".config/lazygit"));
        dirs.push(PathBuf::from(".local/state/lazygit"));
        dirs.push(PathBuf::from(".config/state/lazygit"));
    }

    for harness in &config.ai.harnesses {
        if let Some(dir) = harness.config_dir() {
            dirs.push(PathBuf::from(dir));
        }
    }

    if config
        .ai
        .harnesses
        .contains(&crate::config::AiProvider::OpenCode)
    {
        dirs.push(PathBuf::from(".opencode/plugins"));
    }

    if config
        .ai
        .harnesses
        .contains(&crate::config::AiProvider::Copilot)
    {
        dirs.push(PathBuf::from(".copilot/hooks"));
    }

    if config
        .ai
        .harnesses
        .contains(&crate::config::AiProvider::Claude)
    {
        dirs.push(PathBuf::from(".cache/claude"));
        dirs.push(PathBuf::from(".cache/claude-cli-nodejs"));
        dirs.push(PathBuf::from(".config/claude"));
        dirs.push(PathBuf::from(".local/share/claude"));
        dirs.push(PathBuf::from(".local/state/claude"));
    }

    if config.addons.has_rust() {
        dirs.push(PathBuf::from(".cargo/registry"));
        dirs.push(PathBuf::from(".cargo/git"));
    }

    dirs.sort();
    dirs.dedup();
    dirs
}

pub(crate) fn extra_volume_conflict(config: &AiboxConfig, target: &str) -> Option<String> {
    let target = normalize_container_path(target);
    for broad in broad_runtime_home_mount_paths(config) {
        if target == broad || is_path_ancestor(&target, &broad) {
            return Some(broad);
        }
    }
    protected_runtime_home_paths(config)
        .into_iter()
        .find(|managed| {
            target == *managed
                || is_path_ancestor(managed, &target)
                || is_path_ancestor(&target, managed)
        })
}

fn broad_runtime_home_mount_paths(config: &AiboxConfig) -> Vec<String> {
    let home = config.container_home();
    vec![
        format!("{home}/.cache"),
        format!("{home}/.config"),
        format!("{home}/.local"),
        format!("{home}/.tmux"),
        format!("{home}/.vim"),
    ]
    .into_iter()
    .collect()
}

fn protected_runtime_home_paths(config: &AiboxConfig) -> Vec<String> {
    let home = config.container_home();
    let mut paths = vec![
        format!("{home}/.cache/starship"),
        format!("{home}/.cache/tmux-powerkit"),
        format!("{home}/.cache/uv"),
        format!("{home}/.config/git"),
        format!("{home}/.config/starship.toml"),
        format!("{home}/.config/state"),
        format!("{home}/.config/tmux"),
        format!("{home}/.config/yazi"),
        format!("{home}/.local/bin"),
        format!("{home}/.local/state"),
    ];
    let broad = broad_runtime_home_mount_paths(config);
    for mount in runtime_home_mounts(config) {
        if !broad.contains(&mount.destination) {
            paths.push(mount.destination);
        }
    }
    paths.sort();
    paths.dedup();
    paths
}

fn normalize_container_path(path: &str) -> String {
    let trimmed = path.trim_end_matches('/');
    if trimmed.is_empty() {
        "/".to_string()
    } else {
        trimmed.to_string()
    }
}

fn is_path_ancestor(parent: &str, child: &str) -> bool {
    if parent == "/" {
        return true;
    }
    child
        .strip_prefix(parent)
        .is_some_and(|rest| rest.starts_with('/'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_home_mounts_use_broad_writable_xdg_parents() {
        let config = crate::config::test_config();
        let mounts = runtime_home_mounts(&config);
        let destinations: Vec<_> = mounts
            .iter()
            .map(|mount| mount.destination.as_str())
            .collect();

        assert!(destinations.contains(&"/root/.config"));
        assert!(destinations.contains(&"/root/.cache"));
        assert!(destinations.contains(&"/root/.inputrc"));
        assert!(destinations.contains(&"/root/.local"));
        assert!(!destinations.contains(&"/root/.config/yazi"));
        assert!(!destinations.contains(&"/root/.local/bin"));
        assert!(!destinations.contains(&"/root/.cargo"));

        for destination in ["/root/.config", "/root/.cache", "/root/.local"] {
            let mount = mounts
                .iter()
                .find(|mount| mount.destination == destination)
                .expect("expected managed runtime-home mount");
            assert!(!mount.read_only, "{destination} must be writable");
        }
    }

    #[test]
    fn extra_volumes_cannot_shadow_managed_runtime_paths() {
        let config = crate::config::test_config();
        assert_eq!(
            extra_volume_conflict(&config, "/root/.config/yazi/plugins/git.yazi").as_deref(),
            Some("/root/.config/yazi")
        );
        assert_eq!(
            extra_volume_conflict(&config, "/root/.cache/uv").as_deref(),
            Some("/root/.cache/uv")
        );
        assert!(extra_volume_conflict(&config, "/root/.config/gh").is_none());
        assert!(extra_volume_conflict(&config, "/workspace/.cache").is_none());
    }

    #[test]
    fn rust_runtime_mounts_preserve_image_toolchain_shims() {
        let mut config = crate::config::test_config();
        config.addons.addons.insert(
            "rust".to_string(),
            crate::config::AddonToolsSection::default(),
        );
        let mounts = runtime_home_mounts(&config);
        let destinations: Vec<_> = mounts
            .iter()
            .map(|mount| mount.destination.as_str())
            .collect();

        assert!(destinations.contains(&"/root/.cargo/registry"));
        assert!(destinations.contains(&"/root/.cargo/git"));
        assert!(
            !destinations.contains(&"/root/.cargo"),
            "mounting the whole Cargo home shadows image-provided cargo/rustc shims"
        );
    }

    #[test]
    fn claude_runtime_home_preserves_login_state_locations() {
        let config = crate::config::test_config();
        let mounts = runtime_home_mounts(&config);
        let destinations: Vec<_> = mounts
            .iter()
            .map(|mount| mount.destination.as_str())
            .collect();
        assert!(destinations.contains(&"/root/.claude"));
        assert!(destinations.contains(&"/root/.claude.json"));
        assert!(destinations.contains(&"/root/.cache"));
        assert!(destinations.contains(&"/root/.config"));
        assert!(destinations.contains(&"/root/.local"));

        let dirs = runtime_home_scaffold_dirs(&config);
        for rel in [
            ".claude",
            ".cache/claude",
            ".cache/claude-cli-nodejs",
            ".config/claude",
            ".local/share/claude",
            ".local/state/claude",
        ] {
            assert!(
                dirs.iter().any(|dir| dir == std::path::Path::new(rel)),
                "runtime home should scaffold {rel} for Claude Code login persistence: {dirs:?}"
            );
        }
    }
}
