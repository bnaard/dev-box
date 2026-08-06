//! E2E test suite for aibox CLI.
//!
//! - Tier 1 (always run): appearance tests, config coverage tests
//! - Tier 2 (--features e2e): lifecycle, reset, migration, addon, doctor, smoke,
//!   generated runtime, and visual tests. The two resource-heavy image builds
//!   are intentionally ignored here and are run as isolated shards by
//!   `scripts/run-e2e-shards.sh`; this prevents them from starving unrelated
//!   companion work and lets a failed shard be retried independently.
//! - Tier 3 (--features e2e-render): cell-level rendered-color assertions
//!   replayed through vt100. Starship runs locally; tmux/yazi need the
//!   companion (so combine with `--features e2e`).

pub mod mock_runtime;
pub mod runner;
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
mod no_container_harness;
#[cfg(not(any(feature = "e2e", feature = "e2e-render")))]
mod preauth_merge;
#[cfg(not(any(feature = "e2e", feature = "e2e-render")))]
mod preview;
#[cfg(not(any(feature = "e2e", feature = "e2e-render")))]
mod runtime_recovery;
#[cfg(not(any(feature = "e2e", feature = "e2e-render")))]
mod version_upgrade;

// Tier 2 tests (require e2e-runner companion container)
#[cfg(feature = "e2e")]
mod addon;
#[cfg(feature = "e2e")]
mod doctor;
#[cfg(feature = "e2e")]
mod latex_preview;
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
// visual_rendered_tmux + visual_rendered_yazi were removed in v0.26.2: they
// used `tmux capture-pane` to assert tmux status-bar colors, but capture-pane
// only captures pane contents, never the status bar — so the suite was
// structurally unable to detect the regressions it claimed to catch. The
// asciinema-based visual.rs + visual_matrix.rs tests cover the snapshot
// assertions correctly. Live layout-switch and theme-switch coverage needs
// an asciinema-driven rewrite — tracked separately as a follow-up.
#[cfg(feature = "e2e-render")]
mod visual_rendered_starship;
