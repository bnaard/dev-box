mod model;

#[cfg(feature = "zellij")]
mod zellij_plugin;

#[cfg(feature = "zellij")]
use zellij_plugin::AiboxStatusPlugin;
#[cfg(feature = "zellij")]
use zellij_tile::prelude::*;

#[cfg(feature = "zellij")]
zellij_tile::register_plugin!(AiboxStatusPlugin);

#[cfg(not(feature = "zellij"))]
fn main() {}
