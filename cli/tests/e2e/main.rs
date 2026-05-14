//! E2E test suite for aibox CLI.
//!
//! - Tier 1 (always run): appearance tests, config coverage tests
//! - Tier 2 (--features e2e): lifecycle, reset, migration, addon, doctor, smoke,
//!   generated runtime, and visual tests
//! - Tier 3 (--features e2e-render): cell-level rendered-color assertions
//!   replayed through vt100. Starship runs locally; tmux/yazi need the
//!   companion (so combine with `--features e2e`).

pub mod mock_runtime;
pub mod runner;
#[cfg(feature = "e2e-render")]
pub mod vt_render;

// Tier 1 tests (fast, no container needed)
mod addon_disablement;
mod appearance;
mod config_coverage;
mod no_container_harness;
mod preauth_merge;
mod preview;
mod runtime_recovery;
mod version_upgrade;

// Tier 2 tests (require e2e-runner companion container)
#[cfg(feature = "e2e")]
mod addon;
#[cfg(feature = "e2e")]
mod doctor;
#[cfg(feature = "e2e")]
mod lifecycle;
#[cfg(feature = "e2e")]
mod migration;
#[cfg(feature = "e2e")]
mod reset;
#[cfg(feature = "e2e")]
mod runtime_generated;
#[cfg(feature = "e2e")]
mod smoke;
#[cfg(feature = "e2e")]
mod update;
#[cfg(feature = "e2e")]
mod visual;
#[cfg(feature = "e2e")]
mod visual_keybindings;
#[cfg(feature = "e2e")]
mod visual_matrix;

// Tier 3 — vt100 cell-level rendered-color assertions.
#[cfg(feature = "e2e-render")]
mod visual_rendered_starship;
#[cfg(all(feature = "e2e", feature = "e2e-render"))]
mod visual_rendered_tmux;
#[cfg(all(feature = "e2e", feature = "e2e-render"))]
mod visual_rendered_yazi;
