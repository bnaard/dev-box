//! E2E test suite for aibox CLI.
//!
//! - Tier 1 (always run): appearance tests, config coverage tests
//! - Real container lifecycle and image-build validation runs only through the
//!   owner-controlled macOS release host gate. The development container is
//!   intentionally not given a container-runtime bridge.
//! - Tier 3 (--features e2e-render): cell-level rendered-color assertions
//!   replayed through vt100 in the development container.

pub mod local_runner;
pub mod mock_runtime;
#[cfg(feature = "e2e-render")]
pub mod vt_render;

// Tier 1 tests (fast, no container needed). Release validation runs the
// default target before the feature-specific tiers, so do not compile and run
// these modules again in either feature-enabled invocation.
#[cfg(not(any(feature = "e2e", feature = "e2e-render")))]
mod addon_disablement;
#[cfg(not(any(feature = "e2e", feature = "e2e-render")))]
mod appearance;
#[cfg(not(any(feature = "e2e", feature = "e2e-render")))]
mod config_coverage;
#[cfg(not(any(feature = "e2e", feature = "e2e-render")))]
mod local_cli_contracts;
#[cfg(not(any(feature = "e2e", feature = "e2e-render")))]
mod local_lifecycle;
#[cfg(not(any(feature = "e2e", feature = "e2e-render")))]
mod no_container_harness;
#[cfg(not(any(feature = "e2e", feature = "e2e-render")))]
mod preauth_merge;
#[cfg(not(any(feature = "e2e", feature = "e2e-render")))]
mod preview;
#[cfg(not(any(feature = "e2e", feature = "e2e-render")))]
mod runtime_generated;
#[cfg(not(any(feature = "e2e", feature = "e2e-render")))]
mod runtime_recovery;
#[cfg(not(any(feature = "e2e", feature = "e2e-render")))]
mod update;
#[cfg(not(any(feature = "e2e", feature = "e2e-render")))]
mod version_upgrade;
#[cfg(not(any(feature = "e2e", feature = "e2e-render")))]
mod visual;
#[cfg(not(any(feature = "e2e", feature = "e2e-render")))]
mod visual_keybindings;
#[cfg(not(any(feature = "e2e", feature = "e2e-render")))]
mod visual_matrix;

// Tier 3 — vt100 cell-level rendered-color assertions.
// visual_rendered_tmux + visual_rendered_yazi were removed in v0.26.2: they
// used `tmux capture-pane` to assert tmux status-bar colors, but capture-pane
// only captures pane contents, never the status bar — so the suite was
// structurally unable to detect the regressions it claimed to catch. The
// asciinema-based visual.rs + visual_matrix.rs tests cover the snapshot
// assertions correctly. Live layout-switch and theme-switch coverage needs
// an asciinema-driven rewrite — tracked separately as a follow-up.
#[cfg(feature = "e2e-render")]
mod visual_rendered_starship;
