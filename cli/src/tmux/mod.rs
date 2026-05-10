/// Tmux domain: status rendering, layout generation, and runtime sync.
///
/// Public surface is intentionally flat — callers use `crate::tmux::X`
/// rather than spelunking into sub-modules.
pub mod layouts;
pub mod status;
pub mod sync;

pub use layouts::{tmux_layout_script, tmux_session_script};
pub use status::{
    POWERKIT_RENDER_LIST_SH, POWERKIT_RENDER_SESSION_SH, cleanup_stale_tmux_plugins,
    cleanup_tmux_powerkit_cache, tmux_conf,
};
pub use sync::sync_tmux_runtime_files;
