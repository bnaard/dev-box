-- =============================================================================
-- Yazi init.lua — aibox defaults
-- Runs on every Yazi startup. Register plugins that need setup here.
-- =============================================================================

-- git.yazi: show git status in the file list with explicit, visible signs.
-- Fetcher registration is in yazi.toml [plugin.prepend_fetchers].
require("git"):setup {
	signs = {
		modified = "M",
		added = "A",
		deleted = "D",
		updated = "U",
		untracked = "?",
		ignored = "I",
	},
}
