use anyhow::{Context, Result};
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
#[cfg(unix)]
use std::os::unix::fs::symlink;
use std::path::Path;

use crate::config::{AiboxConfig, ConfigLayout, TmuxStatusMode};
use crate::output;

/// Default vimrc content (embedded fallback).
const DEFAULT_VIMRC: &str = r#"" aibox default vimrc
set nocompatible
let mapleader=" "

set number relativenumber
set tabstop=4 shiftwidth=4 expandtab smartindent
set undofile undodir=~/.vim/undo
set noswapfile
set colorcolumn=88
set scrolloff=8
set signcolumn=yes
set cursorline
set t_u7=
set t_RV=
set wildmenu wildmode=longest:full,full
set incsearch hlsearch ignorecase smartcase
set backspace=indent,eol,start
set nowrap
set sidescroll=1
set sidescrolloff=4
set mouse=a
set laststatus=2
set ruler showcmd

" Filetype-specific indentation
autocmd FileType yaml,json,kdl,html,css,javascript setlocal tabstop=2 shiftwidth=2
autocmd FileType markdown setlocal wrap linebreak
autocmd VimEnter * redraw!

" Use ripgrep if available
if executable('rg')
  set grepprg=rg\ --vimgrep\ --smart-case
endif

" Netrw settings
let g:netrw_liststyle=3
let g:netrw_banner=0
let g:netrw_winsize=25

" ── VSCode-like cursor movement (v0.16.6+) ──────────────────────────────────
"
" Word and line jumps with the chords macOS / VSCode users expect, in
" insert mode AND in normal/visual modes so the same fingers work
" everywhere.
"
" Reliability:
"   <A-Left>/<A-Right>  → reliable across iTerm2, Ghostty, Alacritty,
"                          WezTerm, and tmux. Most terminals send
"                          ^[[1;3D / ^[[1;3C and vim recognises both.
"   <Home>/<End>        → universally reliable. Use these for line
"                          begin/end. To get macOS-native Cmd+Left /
"                          Cmd+Right behaviour, configure your
"                          terminal (iTerm2: Profiles → Keys; Ghostty:
"                          keybind config) to send Home/End on
"                          Cmd+Left / Cmd+Right.
"
" Insert-mode word movement uses <C-o> (one-shot normal command, then
" back to insert). The <Right> after `e` puts the cursor AFTER the
" word's last character, matching VSCode's "select to next word end"
" semantics; without it the cursor lands ON the last character.
inoremap <A-Left>  <C-o>b
inoremap <A-Right> <C-o>e<Right>
nnoremap <A-Left>  b
nnoremap <A-Right> e
vnoremap <A-Left>  b
vnoremap <A-Right> e

" Smart Home/End. Insert-mode <Home> goes to first non-whitespace
" (matches IDE 'smart home'); a second press goes to column 0.
" Vim's default insert-mode <End> already does the right thing
" (jumps past the last character) so we don't override it, but we
" also map normal/visual mode for consistency.
inoremap <expr> <Home> col('.') == match(getline('.'), '\S') + 1 ? "\<C-o>0" : "\<C-o>^"
nnoremap <Home> ^
nnoremap <End>  $
vnoremap <Home> ^
vnoremap <End>  $

set background=AIBOX_VIM_BG
set termguicolors
colorscheme AIBOX_VIM_COLORSCHEME
syntax on
filetype plugin indent on
"#;

/// Default gitconfig content.
const DEFAULT_GITCONFIG: &str = r#"[core]
    editor = vim
[init]
    defaultBranch = main
[pull]
    rebase = true
"#;

/// Default tmux config. Theme, prefix, status mode, and session name are
/// substituted from `aibox.toml` at seed time.
const DEFAULT_TMUX_CONF: &str = r##"# aibox tmux configuration
set -g default-shell /bin/bash
set -g default-command /bin/bash
set -g base-index 1
setw -g pane-base-index 1
set -g renumber-windows on
set -g mouse on
set -g history-limit 50000
set -g escape-time 10
set -g focus-events on
set -g allow-passthrough on
set -g default-terminal "tmux-256color"
set -ga terminal-features ",xterm-256color:RGB,tmux-256color:RGB"
set -ga terminal-overrides ",xterm-256color:Tc,tmux-256color:Tc"
set -g status-interval 5
set -g prefix AIBOX_TMUX_PREFIX
unbind C-b
bind AIBOX_TMUX_PREFIX send-prefix

# Pane navigation mirrors the old aibox leader muscle memory.
bind-key -N "Show aibox/tmux key bindings" ? display-popup -w 80% -h 75% -E "tmux list-keys -N | less -R"
bind-key -N "Select pane left" h select-pane -L
bind-key -N "Select pane down" j select-pane -D
bind-key -N "Select pane up" k select-pane -U
bind-key -N "Select pane right" l select-pane -R
bind-key -N "Split pane right" r split-window -h -c "#{pane_current_path}"
bind-key -N "Split pane down" d split-window -v -c "#{pane_current_path}"
bind-key -N "Kill pane" x kill-pane
bind-key -N "Toggle pane zoom" f resize-pane -Z
bind-key -N "Kill tmux session" q confirm-before -p "kill tmux session AIBOX_TMUX_SESSION? (y/n)" kill-session
bind-key -N "Reload tmux config" R source-file ~/.config/tmux/tmux.conf \; display-message "aibox tmux config reloaded"

set -g status AIBOX_TMUX_STATUS
set -g status-style "bg=AIBOX_TMUX_BG,fg=AIBOX_TMUX_FG"
set -g window-status-current-style "bg=AIBOX_TMUX_ACCENT,fg=AIBOX_TMUX_BG,bold"
set -g window-status-format " #I:#W "
set -g window-status-current-format " #I:#W "
set -g status-left " #S "
set -g status-right " AIBOX_TMUX_STATUS_RIGHT "

AIBOX_TMUX_POWERKIT_BLOCK

# aibox-managed plugins are installed and pinned by the runtime image. TPM is
# only a user convenience layer for additional personal plugins.
if-shell '[ -f /usr/local/share/aibox/tmux/plugins/tmux-sensible/sensible.tmux ]' 'run-shell /usr/local/share/aibox/tmux/plugins/tmux-sensible/sensible.tmux'
AIBOX_TMUX_POWERKIT_PLUGIN
if-shell '[ -f /usr/local/share/aibox/tmux/plugins/vim-tmux-navigator/vim-tmux-navigator.tmux ]' 'run-shell /usr/local/share/aibox/tmux/plugins/vim-tmux-navigator/vim-tmux-navigator.tmux'
if-shell '[ -f /usr/local/share/aibox/tmux/plugins/tmux-yank/yank.tmux ]' 'run-shell /usr/local/share/aibox/tmux/plugins/tmux-yank/yank.tmux'

# Persistence plugins are installed for later policy work but disabled by
# default in v0.25.0. Do not enable continuum/resurrect implicitly.
set -g @continuum-restore 'off'
set -g @continuum-save-interval '0'
set -g @resurrect-capture-pane-contents 'off'

if-shell '[ -x ~/.tmux/plugins/tpm/tpm ]' 'run-shell ~/.tmux/plugins/tpm/tpm'
"##;

fn addon_tool_effective_enabled(config: &AiboxConfig, addon: &str, tool: &str) -> bool {
    let Some(addon_section) = config.addons.get_addon(addon) else {
        return false;
    };

    if let Some(entry) = addon_section.tools.get(tool) {
        return entry.enabled.unwrap_or(true);
    }

    crate::addon_loader::get_addon(addon)
        .and_then(|addon_def| {
            addon_def
                .tools
                .iter()
                .find(|tool_def| tool_def.name == tool)
                .map(|tool_def| tool_def.default_enabled)
        })
        .unwrap_or(true)
}

fn include_lazygit_tab(config: &AiboxConfig) -> bool {
    addon_tool_effective_enabled(config, "git-ui", "lazygit")
}

/// Default yazi config.
///
/// Note on `[mgr]` (formerly `[manager]`):
/// Yazi 25+ renamed the `[manager]` section to `[mgr]`. Files using the old
/// name are silently ignored — `ratio` and friends have no effect. The
/// `migrate_yazi_section` helper rewrites existing host-side files at sync
/// time. Do not change `[mgr]` back to `[manager]`.
const DEFAULT_YAZI_CONFIG: &str = r#"[mgr]
ratio = [3, 5, 18]
sort_by = "natural"
sort_sensitive = false
sort_dir_first = true
show_hidden = true
show_symlink = true

[preview]
max_width = 600
max_height = 900
image_delay = 30
image_filter = "nearest"

[plugin]
prepend_fetchers = [
    { id = "git", url = "*",  run = "git", group = "git" },
    { id = "git", url = "*/", run = "git", group = "git" },
    { id = "status-git", url = "*", run = "status-git", group = "status-git" },
]
prepend_previewers = [
    # Directory preview: columnar listing with git status, size, date, owner, permissions
    { url = "*/", run = "dir-preview" },
AIBOX_YAZI_EXTRA_PREVIEWERS
    { url = "*.svg",  run = "svg" },
    { url = "*.eps",  run = "eps" },
    { url = "*.jpg",  run = "image" },
    { url = "*.jpeg", run = "image" },
    { url = "*.png",  run = "image" },
    { url = "*.gif",  run = "image" },
    { url = "*.webp", run = "image" },
    { url = "*.bmp",  run = "image" },
    { url = "*.tiff", run = "image" },
    { url = "*.tif",  run = "image" },
    { url = "*.pdf",  run = "pdf" },
]

[opener]
edit = [
    { run = 'vim --cmd "set t_u7=" --cmd "set t_RV=" "$@"', desc = "Edit in-place", block = true },
]
edit-pane = [
    { run = 'open-in-editor "$1"', desc = "Open in vim pane", block = false },
]

[open]
rules = [
    { mime = "text/*", use = "edit" },
    { url = "*", use = "edit" },
]
"#;

/// EPS previewer plugin — converts EPS to PNG via ghostscript.
const DEFAULT_YAZI_PLUGIN_EPS: &str = r#"-- eps.yazi — EPS previewer for yazi
-- Converts EPS to PNG using ghostscript (gs).
-- Requires: ghostscript in PATH (install via preview-enhanced addon or apt).

return {
	entry = function(self, job)
		local cache = ya.file_cache(job)
		if not cache then
			return Err("No cache path")
		end

		if cache:exists() then
			return Image:new(job, cache):show()
		end

		local ok = Command("gs")
			:args({
				"-q",
				"-dNOPAUSE",
				"-dBATCH",
				"-dSAFER",
				"-sDEVICE=png16m",
				"-r150",
				"-dEPSCrop",
				"-sOutputFile=" .. tostring(cache),
				tostring(job.file.url),
			})
			:stdout(Command.NULL)
			:stderr(Command.NULL)
			:status()

		if ok then
			return Image:new(job, cache):show()
		end

		return Err("EPS preview requires ghostscript: aibox set addon preview-enhanced enabled --apply")
	end,
}
"#;

/// SVG previewer plugin — converts SVG to PNG via resvg or rsvg-convert.
const DEFAULT_YAZI_PLUGIN_SVG: &str = r#"-- svg.yazi — SVG previewer for yazi
-- Converts SVG to PNG using resvg (x86_64) or rsvg-convert (aarch64 fallback).

return {
	entry = function(self, job)
		local cache = ya.file_cache(job)
		if not cache then
			return Err("No cache path")
		end

		if cache:exists() then
			return Image:new(job, cache):show()
		end

		-- Try resvg first (high quality, static binary — available on x86_64)
		local ok = Command("resvg")
			:args({
				"--width",
				tostring(job.area.w * 4),
				"--height",
				tostring(job.area.h * 4),
				tostring(job.file.url),
				tostring(cache),
			})
			:stdout(Command.NULL)
			:stderr(Command.NULL)
			:status()

		if ok then
			return Image:new(job, cache):show()
		end

		-- Fallback: rsvg-convert (from librsvg2-bin, available on all architectures)
		ok = Command("rsvg-convert")
			:args({
				"--width", tostring(job.area.w * 4),
				"--height", tostring(job.area.h * 4),
				"--keep-aspect-ratio",
				"--output", tostring(cache),
				tostring(job.file.url),
			})
			:stdout(Command.NULL)
			:stderr(Command.NULL)
			:status()

		if ok then
			return Image:new(job, cache):show()
		end

		return Err("SVG preview failed: install resvg or librsvg2-bin")
	end,
}
"#;

/// dir-preview.yazi plugin — columnar directory preview with git status.
/// Columns: git | icon+name | size | date | owner | permissions
/// Uses yazi's Lua fs API for metadata (no shell), one git command for status.
const DEFAULT_YAZI_PLUGIN_DIR_PREVIEW: &str = r##"--- @since 26.1.22
-- dir-preview.yazi — columnar directory preview with git status
-- Columns: git | icon+name | size | date | owner | permissions

local M = {}

local function uid_map()
	local map = {}
	local f = io.open("/etc/passwd")
	if f then
		for line in f:lines() do
			local name, uid = line:match("^([^:]+):[^:]*:(%d+)")
			if uid then map[tonumber(uid)] = name end
		end
		f:close()
	end
	return map
end

local function git_status(dir)
	local map = {}
	-- Get the directory's path relative to the repo root
	local prefix_out = Command("git"):cwd(tostring(dir))
		:arg({ "rev-parse", "--show-prefix" }):output()
	local prefix = prefix_out and prefix_out.stdout:gsub("%s+$", "") or ""
	local out = Command("git"):cwd(tostring(dir))
		:arg({ "--no-optional-locks", "-c", "core.quotePath=",
		       "status", "--porcelain", "-unormal", "--no-renames", "--ignored=matching", "--", "." })
		:output()
	if not out then return map end
	for line in out.stdout:gmatch("[^\r\n]+") do
		local signs, path = line:sub(1, 2), line:sub(4):gsub('"', ""):gsub("/$", "")
		-- Strip the repo-relative prefix to get paths relative to this directory
		if prefix ~= "" and path:sub(1, #prefix) == prefix then
			path = path:sub(#prefix + 1)
		end
		local base = path:match("^([^/]+)")
		if base and not map[base] then map[base] = signs end
	end
	return map
end

-- Map porcelain two-char codes to single-char signs matching the main list
local GIT_SIGNS = {
	["!!"] = "I", ["??"] = "?",
	["A "] = "A", ["AM"] = "A",
	[" M"] = "M", ["M "] = "M", ["MM"] = "M",
	[" D"] = "D", ["D "] = "D",
	["UU"] = "U",
}
-- Read theme git colors (set by yazi theme.toml [git] section)
local t = th.git or {}
-- Direct status (files): theme-aware styles
local GIT_STYLES = {
	["?"] = t.untracked or ui.Style():fg("magenta"),
	["I"] = t.ignored or ui.Style():fg("darkgray"),
	["A"] = t.added or ui.Style():fg("green"),
	["M"] = t.modified or ui.Style():fg("yellow"),
	["D"] = t.deleted or ui.Style():fg("red"),
	["U"] = t.updated or ui.Style():fg("yellow"),
}
-- Inherited status (directories): same styles but dimmed
local GIT_STYLES_DIM = {
	["?"] = (t.untracked or ui.Style():fg("magenta")):dim(),
	["I"] = t.ignored or ui.Style():fg("darkgray"),
	["A"] = (t.added or ui.Style():fg("green")):dim(),
	["M"] = (t.modified or ui.Style():fg("yellow")):dim(),
	["D"] = (t.deleted or ui.Style():fg("red")):dim(),
	["U"] = (t.updated or ui.Style():fg("yellow")):dim(),
}

local function fmt_size(n)
	local s
	if not n or n < 0 then s = "-"
	elseif n < 1024 then s = string.format("%d", n)
	elseif n < 1048576 then s = string.format("%.0fK", n / 1024)
	elseif n < 1073741824 then s = string.format("%.1fM", n / 1048576)
	else s = string.format("%.1fG", n / 1073741824)
	end
	return string.format("%6s", s)
end

local function pad(s, w) return s .. string.rep(" ", math.max(0, w - #s)) end
local function trunc(s, w) return #s <= w and s or s:sub(1, w - 1) .. "~" end

function M:peek(job)
	local files, err = fs.read_dir(job.file.url, { resolve = true })
	if not files then
		ya.preview_widget(job, ui.Text { ui.Line(tostring(err or "empty")) }:area(job.area))
		return
	end

	table.sort(files, function(a, b)
		if a.cha.is_dir ~= b.cha.is_dir then return a.cha.is_dir end
		return a.name:lower() < b.name:lower()
	end)

	local git = git_status(job.file.url)
	local owners = uid_map()
	local total, limit = #files, job.area.h
	-- fixed: git(2) + spc(1) + icon(2) + spc(1) + size(6) + spc(1) + date(13) + spc(1) + owner(9) + spc(1) + perm(10) = 47
	local name_w = math.max(8, job.area.w - 47)
	local lines = {}

	for i = job.skip + 1, math.min(total, job.skip + limit) do
		local f = files[i]
		local c = f.cha
		local name = f.name .. (c.is_dir and "/" or "")
		local raw = git[f.name] or ""
		local gs = GIT_SIGNS[raw] or ""
		local is_inherited = c.is_dir and gs ~= "" and gs ~= "I"
		local gs_style = is_inherited and GIT_STYLES_DIM[gs] or GIT_STYLES[gs]
		local ignored = gs == "I"
		gs = is_inherited and (gs:lower() .. " ") or (gs ~= "" and (gs .. " ") or "  ")
		local icon = f:icon()
		local size
		if c.is_dir then
			local children = fs.read_dir(f.url, {})
			local n = children and #children or 0
			size = string.format("%6s", "[" .. n .. "]")
		else
			size = fmt_size(c.len)
		end
		local date = c.mtime and os.date("%b %d %H:%M", math.floor(c.mtime)) or "            "
		local owner = pad(trunc(owners[c.uid] or tostring(c.uid or "?"), 8), 8)
		local perm = c:perm() or "----------"
		local ign = ignored and GIT_STYLES["I"] or nil

		lines[#lines + 1] = ui.Line {
			gs_style and ui.Span(gs):style(gs_style) or ui.Span(gs), ui.Span(" "),
			icon and (ign and ui.Span(icon.text .. " "):style(ign) or ui.Span(icon.text .. " "):style(icon.style)) or ui.Span("  "),
			ign and ui.Span(pad(trunc(name, name_w), name_w)):style(ign) or ui.Span(pad(trunc(name, name_w), name_w)),
			ui.Span(" " .. size):style(ign or ui.Style():fg("green")),
			ui.Span(" " .. date):style(ign or ui.Style():fg("cyan")),
			ui.Span(" " .. owner):style(ign or ui.Style():fg("yellow")),
			ui.Span(" " .. perm):dim(),
		}
	end

	if job.skip > 0 and total <= job.skip then
		ya.emit("peek", { math.max(0, total - limit), only_if = job.file.url, upper_bound = true })
	else
		ya.preview_widget(job, ui.Text(lines):area(job.area))
	end
end

function M:seek(job) require("code"):seek(job) end

return M
"##;

/// Yazi init.lua — registers plugins that need setup on every startup.
const DEFAULT_YAZI_INIT: &str = r#"-- =============================================================================
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

-- status-git.yazi: git branch + summary (left) and disk free (right) in status bar.
-- Data refresh is triggered via the fetcher registered in yazi.toml.
require("status-git"):setup()
AIBOX_YAZI_EXTRA_SETUPS
"#;

fn generate_yazi_config(config: &AiboxConfig) -> String {
    let mut extra_previewers = String::new();

    if config.addons.has_addon("data-preview") {
        extra_previewers.push_str(
            r#"    # Data previews (requires data-preview addon)
    { url = "*.sqlite",  run = "sqlite-preview" },
    { url = "*.sqlite3", run = "sqlite-preview" },
    { url = "*.db",      run = "sqlite-preview" },
    { url = "*.csv",     run = "tabular-preview" },
    { url = "*.tsv",     run = "tabular-preview" },
    { url = "*.xls",     run = "tabular-preview" },
    { url = "*.xlsx",    run = "tabular-preview" },
"#,
        );
    }

    if config.addons.has_addon("preview-enhanced") {
        extra_previewers.push_str(
            r#"    # Rich terminal preview for docs/data files (requires python3-rich from preview-enhanced)
    { url = "*.md",       run = "rich-preview" },
    { url = "*.markdown", run = "rich-preview" },
    { url = "*.rst",      run = "rich-preview" },
    { url = "*.json",     run = "rich-preview" },
    { url = "*.ipynb",    run = "rich-preview" },
"#,
        );
    }

    DEFAULT_YAZI_CONFIG.replace("AIBOX_YAZI_EXTRA_PREVIEWERS\n", &extra_previewers)
}

fn generate_yazi_init(config: &AiboxConfig) -> String {
    let extra_setups = if config.addons.has_addon("yazi-omp") {
        r#"
-- omp.yazi: show an Oh My Posh prompt in Yazi's header when enabled.
require("omp"):setup({
    config = os.getenv("HOME") .. "/.config/yazi/yazi-prompt.omp.json",
})
"#
    } else {
        ""
    };

    DEFAULT_YAZI_INIT.replace("AIBOX_YAZI_EXTRA_SETUPS\n", extra_setups)
}

/// git.yazi plugin main — shows git status signs next to file names.
const DEFAULT_YAZI_PLUGIN_GIT_MAIN: &str =
    include_str!("../../images/base-debian/config/yazi/plugins/git.yazi/main.lua");

/// git.yazi plugin types — type annotations for the git plugin.
const DEFAULT_YAZI_PLUGIN_GIT_TYPES: &str =
    include_str!("../../images/base-debian/config/yazi/plugins/git.yazi/types.lua");

/// status-git.yazi plugin — git branch/summary + disk free in status bar.
const DEFAULT_YAZI_PLUGIN_STATUS_GIT: &str =
    include_str!("../../images/base-debian/config/yazi/plugins/status-git.yazi/main.lua");

/// toggle-pane.yazi plugin — hide/maximize Yazi panes while preserving the base ratio.
const DEFAULT_YAZI_PLUGIN_TOGGLE_PANE: &str =
    include_str!("../../images/base-debian/config/yazi/plugins/toggle-pane.yazi/main.lua");

/// rich-preview.yazi plugin — terminal-rich preview for markdown and data files.
const DEFAULT_YAZI_PLUGIN_RICH_PREVIEW: &str =
    include_str!("../../images/base-debian/config/yazi/plugins/rich-preview.yazi/main.lua");

/// sqlite-preview.yazi plugin — SQLite schema preview.
const DEFAULT_YAZI_PLUGIN_SQLITE_PREVIEW: &str =
    include_str!("../../images/base-debian/config/yazi/plugins/sqlite-preview.yazi/main.lua");

/// tabular-preview.yazi plugin — CSV/TSV/XLS/XLSX table preview.
const DEFAULT_YAZI_PLUGIN_TABULAR_PREVIEW: &str =
    include_str!("../../images/base-debian/config/yazi/plugins/tabular-preview.yazi/main.lua");

/// pdf-watch helper — re-render a PDF preview whenever the file changes.
const DEFAULT_PDF_WATCH_SH: &str = include_str!("../../images/base-debian/config/bin/pdf-watch.sh");
/// open-in-editor helper — open the hovered Yazi file in the Vim pane/tab.
const DEFAULT_OPEN_IN_EDITOR_SH: &str =
    include_str!("../../images/base-debian/config/bin/open-in-editor.sh");
/// aibox-preview helper — full-pane rich previews from Yazi.
const DEFAULT_AIBOX_PREVIEW_SH: &str =
    include_str!("../../images/base-debian/config/bin/aibox-preview.sh");
/// aibox-status-toggle helper — toggle the tmux runtime status line.
const DEFAULT_AIBOX_STATUS_TOGGLE_SH: &str =
    include_str!("../../images/base-debian/config/bin/aibox-status-toggle.sh");

/// omp.yazi plugin — render an Oh My Posh prompt in Yazi's header.
const DEFAULT_YAZI_PLUGIN_OMP: &str =
    include_str!("../../images/base-debian/config/yazi/plugins/omp.yazi/main.lua");

/// Default OMP config used by omp.yazi when the addon is enabled.
const DEFAULT_YAZI_OMP_CONFIG: &str =
    include_str!("../../images/base-debian/config/yazi/plugins/omp.yazi/yazi-prompt.omp.json");

/// Default yazi keymap.
const DEFAULT_YAZI_KEYMAP: &str = r#"[mgr]
prepend_keymap = [
    { on = "<Enter>", run = "open", desc = "Edit in-place" },
    { on = "e", run = "shell 'open-in-editor \"$1\"'", desc = "Open in vim pane" },
    { on = "O", run = "open --interactive", desc = "Open interactively" },
    { on = "p", run = "shell 'aibox-preview \"$1\"' --block", desc = "Full-pane preview" },
    { on = [ "z", "h" ], run = "plugin toggle-pane min-parent", desc = "Toggle parent pane" },
    { on = [ "z", "l" ], run = "plugin toggle-pane min-preview", desc = "Toggle preview pane" },
    { on = [ "z", "m" ], run = "plugin toggle-pane max-preview", desc = "Maximize preview pane" },
    { on = [ "z", "c" ], run = "plugin toggle-pane max-current", desc = "Maximize current pane" },
    { on = [ "z", "0" ], run = "plugin toggle-pane", desc = "Reset pane layout" },
    { on = [ "w", "s" ], run = "shell 'du -sch \"$@\" | ${PAGER:-less}' --block", desc = "Size selected files" },
    { on = [ "w", "h" ], run = "shell 'bat --color=always --style=plain --paging=never \"$1\" | less -R -S' --block", desc = "Preview with horizontal scroll" },
    { on = [ "w", "p" ], run = "shell 'if [ -f \"$HOME/.local/bin/pdf-watch\" ]; then bash \"$HOME/.local/bin/pdf-watch\" \"$1\"; else pdf-watch \"$1\"; fi' --block", desc = "Watch PDF preview" },
    { on = [ "c", "p" ], run = "copy path", desc = "Copy selected paths" },
    { on = [ "c", "d" ], run = "copy dirname", desc = "Copy selected directories" },
    { on = [ "c", "f" ], run = "copy filename", desc = "Copy selected filenames" },
    { on = [ "c", "n" ], run = "copy name_without_ext", desc = "Copy names without extension" },
    { on = [ "g", "s" ], run = "shell 'git -c color.status=always status --short --branch --ignored=matching --untracked-files=all | ${PAGER:-less} -R' --block", desc = "Git summary" },
    { on = [ "g", "c" ], run = "shell 'git -c color.status=always status --short --ignored=matching --untracked-files=all | ${PAGER:-less} -R' --block", desc = "Show git changes" },
    { on = [ "g", "r" ], run = "cd .", desc = "Refresh directory" },
]
"#;

/// Quick reference cheatsheet.
const DEFAULT_CHEATSHEET: &str = r#"  aibox Quick Reference
  -----------------------------------------------
  TMUX (prefix: Ctrl+g)      YAZI (file manager)
  Ctrl+g h/j/k/l Move pane  h/j/k/l  Navigate
  Ctrl+g r/d     Split pane Enter    Open in vim
  Ctrl+g x       Close pane g s      Git summary
  Ctrl+g f       Zoom pane  g c      Git changes
  Ctrl+g c       New win    w s      Size selection
  Ctrl+g n/p     Next/prev  w h      Horizontal preview
  Ctrl+g 1-5     Jump win   w p      Watch PDF
  Ctrl+g [       Copy mode  c p/d/f  Copy path/dir/name
  Ctrl+g ]       Paste      g r      Refresh git
  Ctrl+g R       Reload
  Ctrl+g q       QUIT

  LAYOUTS: aibox up --layout dev|focus|cowork|cowork-swap|browse|ai
  WINDOWS: 1 dev  2 git  3 shell
"#;

/// Default .asoundrc for PulseAudio over TCP.
const DEFAULT_ASOUNDRC: &str = r#"pcm.!default {
    type pulse
}
ctl.!default {
    type pulse
}
"#;

/// Claude Code keybindings — disables Ctrl+g (reserved for the tmux prefix).
const DEFAULT_CLAUDE_KEYBINDINGS: &str = r#"{
  "$schema": "https://www.schemastore.org/claude-code-keybindings.json",
  "$docs": "https://code.claude.com/docs/en/keybindings",
  "bindings": [
    {
      "context": "Chat",
      "bindings": {
        "ctrl+g": null
      }
    }
  ]
}
"#;

/// OpenCode TypeScript plugin enforcing the processkit compliance contract.
///
/// Seeded into `<root>/.opencode/plugins/processkit-gate.ts` whenever the
/// OpenCode harness is enabled. Mirrors Claude Code's PreToolUse compliance
/// hook so an OpenCode session in this project meets the same compliance
/// bar as Claude Code (acknowledge_contract before any write-side MCP call).
///
/// Implementation notes:
/// - On `session.created`, the session is marked not-yet-acknowledged.
/// - On `tool.execute.before`, calls to `acknowledge_contract` /
///   `check_contract_acknowledged` / `route_task` / `find_skill` /
///   `list_skills` flip the session to acknowledged. Calls to any
///   processkit MCP write tool prior to that throw an error.
///
/// Closes aibox#51. Required upstream fixes (both shipped before this
/// plugin could function): sst/opencode#2319 (MCP tool calls now trigger
/// plugin hooks) and sst/opencode#5894 (subagent tool calls now trigger
/// plugin hooks).
const DEFAULT_OPENCODE_PROCESSKIT_GATE_TS: &str = r#"// Generated by aibox — do not edit manually.
// To modify, edit aibox.toml and run: aibox apply.
//
// processkit compliance enforcement plugin for OpenCode.
//
// Blocks calls to processkit MCP write tools until the agent has
// acknowledged the compliance contract via skill-gate. Mirrors the
// PreToolUse hook behavior used by Claude Code so an OpenCode session
// in this project meets the same compliance bar.

import type { Plugin } from "@opencode-ai/plugin";

// Tool names whose invocation counts as "this session has acknowledged
// the contract." Calling skill-gate's acknowledge_contract is the
// canonical path; route_task / find_skill / list_skills also count
// because they are themselves the gates the contract requires before
// any write-side tool.
const ACK_TOOLS = new Set<string>([
  "processkit-skill-gate__acknowledge_contract",
  "processkit-skill-gate__check_contract_acknowledged",
  "processkit-task-router__route_task",
  "processkit-skill-finder__find_skill",
  "processkit-skill-finder__list_skills",
]);

// Returns true if `tool` is a processkit MCP write-side operation that
// must be gated behind acknowledge_contract.
function isProcesskitWriteTool(tool: string): boolean {
  if (!tool.startsWith("processkit-")) return false;
  if (ACK_TOOLS.has(tool)) return false;
  return /__(create|update|transition|link|record|open|delete|append|register|set|apply|mark|promote|publish|deploy)_/.test(
    tool,
  );
}

export const ProcesskitGate: Plugin = async ({ project: _project }) => {
  // session_id -> has the agent acknowledged the contract this session?
  // Plugin loads once per OpenCode process; sessions inside that process
  // are tracked here. New process -> empty map -> fresh gate.
  const acknowledged = new Map<string, boolean>();

  return {
    "session.created": async (session: { id: string }) => {
      acknowledged.set(session.id, false);
    },
    "tool.execute.before": async (input: {
      tool?: string;
      sessionID?: string;
      sessionId?: string;
    }) => {
      const tool = String(input.tool ?? "");
      const sessionId = String(input.sessionID ?? input.sessionId ?? "");
      if (ACK_TOOLS.has(tool)) {
        acknowledged.set(sessionId, true);
        return;
      }
      if (isProcesskitWriteTool(tool) && !acknowledged.get(sessionId)) {
        throw new Error(
          `processkit compliance: '${tool}' is a write-side tool and requires ` +
            `'acknowledge_contract' (skill-gate) or 'route_task' to be called ` +
            `first this session. See AGENTS.md for the contract.`,
        );
      }
    },
  };
};
"#;

/// Create the managed `.aibox-home/` directory structure without writing files.
/// Safe to call on every sync/start because it only scaffolds missing directories.
pub fn ensure_runtime_dirs(config: &AiboxConfig) -> Result<()> {
    let root = config.host_root_dir();
    let include_lazygit = include_lazygit_tab(config);

    let mut dirs = vec![
        root.join(".ssh"),
        root.join(".local").join("bin"),
        root.join(".local").join("state"),
        root.join(".vim").join("undo"),
        root.join(".config").join("state"),
        root.join(".config").join("tmux").join("layouts"),
        root.join(".tmux").join("plugins"),
        root.join(".cache").join("starship"),
        root.join(".cache").join("uv"),
        root.join(".config").join("yazi"),
        root.join(".config")
            .join("yazi")
            .join("plugins")
            .join("eps.yazi"),
        root.join(".config")
            .join("yazi")
            .join("plugins")
            .join("svg.yazi"),
        root.join(".config")
            .join("yazi")
            .join("plugins")
            .join("git.yazi"),
        root.join(".config")
            .join("yazi")
            .join("plugins")
            .join("dir-preview.yazi"),
        root.join(".config")
            .join("yazi")
            .join("plugins")
            .join("status-git.yazi"),
        root.join(".config").join("git"),
    ];
    if include_lazygit {
        dirs.push(root.join(".config").join("lazygit"));
        dirs.push(root.join(".local").join("state").join("lazygit"));
        dirs.push(root.join(".config").join("state").join("lazygit"));
    }

    for harness in &config.ai.harnesses {
        if let Some(dir) = harness.config_dir() {
            dirs.push(root.join(dir));
        }
    }

    // OpenCode: needs `.opencode/plugins/` for the processkit-gate plugin.
    if config
        .ai
        .harnesses
        .contains(&crate::config::AiProvider::OpenCode)
    {
        dirs.push(root.join(".opencode").join("plugins"));
    }

    for dir in &dirs {
        fs::create_dir_all(dir)
            .with_context(|| format!("Failed to create directory: {}", dir.display()))?;
    }

    Ok(())
}

/// Return the managed runtime files that aibox generates inside `.aibox-home/`.
pub fn managed_runtime_files(config: &AiboxConfig) -> Vec<(std::path::PathBuf, String)> {
    let theme = config.customization.resolved_theme();
    let providers = &config.ai.harnesses;
    let include_lazygit = include_lazygit_tab(config);
    let session_name = &config.customization.tmux.session_name;
    let mut files = vec![
        (
            std::path::PathBuf::from(".vim/vimrc"),
            DEFAULT_VIMRC
                .replace(
                    "AIBOX_VIM_COLORSCHEME",
                    crate::themes::vim_colorscheme(&theme),
                )
                .replace("AIBOX_VIM_BG", crate::themes::vim_background(&theme)),
        ),
        (
            std::path::PathBuf::from(".config/git/config"),
            DEFAULT_GITCONFIG.to_string(),
        ),
        (
            std::path::PathBuf::from(".config/tmux/tmux.conf"),
            tmux_conf(config),
        ),
        (
            std::path::PathBuf::from(".config/tmux/layouts/dev.sh"),
            tmux_layout_script(&ConfigLayout::Dev, providers, include_lazygit, session_name),
        ),
        (
            std::path::PathBuf::from(".config/tmux/layouts/focus.sh"),
            tmux_layout_script(
                &ConfigLayout::Focus,
                providers,
                include_lazygit,
                session_name,
            ),
        ),
        (
            std::path::PathBuf::from(".config/tmux/layouts/cowork.sh"),
            tmux_layout_script(
                &ConfigLayout::Cowork,
                providers,
                include_lazygit,
                session_name,
            ),
        ),
        (
            std::path::PathBuf::from(".config/tmux/layouts/browse.sh"),
            tmux_layout_script(
                &ConfigLayout::Browse,
                providers,
                include_lazygit,
                session_name,
            ),
        ),
        (
            std::path::PathBuf::from(".config/tmux/layouts/ai.sh"),
            tmux_layout_script(&ConfigLayout::Ai, providers, include_lazygit, session_name),
        ),
        (
            std::path::PathBuf::from(".config/tmux/layouts/cowork-swap.sh"),
            tmux_layout_script(
                &ConfigLayout::CoworkSwap,
                providers,
                include_lazygit,
                session_name,
            ),
        ),
        (
            std::path::PathBuf::from(".config/tmux/aibox-session.sh"),
            tmux_session_script(config),
        ),
        (
            std::path::PathBuf::from(".config/yazi/yazi.toml"),
            generate_yazi_config(config),
        ),
        (
            std::path::PathBuf::from(".config/yazi/keymap.toml"),
            DEFAULT_YAZI_KEYMAP.to_string(),
        ),
        (
            std::path::PathBuf::from(".config/yazi/theme.toml"),
            crate::themes::yazi_theme(&theme).to_string(),
        ),
        (
            std::path::PathBuf::from(".config/yazi/init.lua"),
            generate_yazi_init(config),
        ),
        (
            std::path::PathBuf::from(".config/yazi/plugins/eps.yazi/init.lua"),
            DEFAULT_YAZI_PLUGIN_EPS.to_string(),
        ),
        (
            std::path::PathBuf::from(".config/yazi/plugins/svg.yazi/init.lua"),
            DEFAULT_YAZI_PLUGIN_SVG.to_string(),
        ),
        (
            std::path::PathBuf::from(".config/yazi/plugins/git.yazi/main.lua"),
            DEFAULT_YAZI_PLUGIN_GIT_MAIN.to_string(),
        ),
        (
            std::path::PathBuf::from(".config/yazi/plugins/git.yazi/types.lua"),
            DEFAULT_YAZI_PLUGIN_GIT_TYPES.to_string(),
        ),
        (
            std::path::PathBuf::from(".config/yazi/plugins/dir-preview.yazi/main.lua"),
            DEFAULT_YAZI_PLUGIN_DIR_PREVIEW.to_string(),
        ),
        (
            std::path::PathBuf::from(".config/yazi/plugins/status-git.yazi/main.lua"),
            DEFAULT_YAZI_PLUGIN_STATUS_GIT.to_string(),
        ),
        (
            std::path::PathBuf::from(".config/yazi/plugins/toggle-pane.yazi/main.lua"),
            DEFAULT_YAZI_PLUGIN_TOGGLE_PANE.to_string(),
        ),
        (
            std::path::PathBuf::from(".config/cheatsheet.txt"),
            DEFAULT_CHEATSHEET.to_string(),
        ),
        (
            std::path::PathBuf::from(".config/starship.toml"),
            crate::themes::starship_config(&config.customization.prompt, &theme),
        ),
        (
            std::path::PathBuf::from(".local/bin/pdf-watch"),
            DEFAULT_PDF_WATCH_SH.to_string(),
        ),
        (
            std::path::PathBuf::from(".local/bin/open-in-editor"),
            DEFAULT_OPEN_IN_EDITOR_SH.to_string(),
        ),
        (
            std::path::PathBuf::from(".local/bin/aibox-preview"),
            DEFAULT_AIBOX_PREVIEW_SH.to_string(),
        ),
        (
            std::path::PathBuf::from(".local/bin/aibox-status-toggle"),
            DEFAULT_AIBOX_STATUS_TOGGLE_SH.to_string(),
        ),
    ];

    if config.audio.enabled {
        files.push((
            std::path::PathBuf::from(".asoundrc"),
            DEFAULT_ASOUNDRC.to_string(),
        ));
    }

    if include_lazygit {
        files.push((
            std::path::PathBuf::from(".config/lazygit/config.yml"),
            crate::themes::lazygit_theme(&theme).to_string(),
        ));
    }

    if providers.contains(&crate::config::AiProvider::Claude) {
        files.push((
            std::path::PathBuf::from(".claude/keybindings.json"),
            DEFAULT_CLAUDE_KEYBINDINGS.to_string(),
        ));
        // Pre-create an empty .claude.json so the bind mount succeeds on first
        // build. Claude Code rewrites it during auth; seed_file is write-if-missing,
        // so a real logged-in state is never clobbered.
        files.push((std::path::PathBuf::from(".claude.json"), "{}\n".to_string()));
    }

    // OpenCode: ship the processkit-gate plugin so an OpenCode session in this
    // project enforces the same compliance contract as Claude Code's PreToolUse
    // hook. Closes aibox#51.
    if providers.contains(&crate::config::AiProvider::OpenCode) {
        files.push((
            std::path::PathBuf::from(".opencode/plugins/processkit-gate.ts"),
            DEFAULT_OPENCODE_PROCESSKIT_GATE_TS.to_string(),
        ));
    }

    if config.addons.has_addon("preview-enhanced") {
        files.push((
            std::path::PathBuf::from(".config/yazi/plugins/rich-preview.yazi/main.lua"),
            DEFAULT_YAZI_PLUGIN_RICH_PREVIEW.to_string(),
        ));
    }

    if config.addons.has_addon("data-preview") {
        files.push((
            std::path::PathBuf::from(".config/yazi/plugins/sqlite-preview.yazi/main.lua"),
            DEFAULT_YAZI_PLUGIN_SQLITE_PREVIEW.to_string(),
        ));
        files.push((
            std::path::PathBuf::from(".config/yazi/plugins/tabular-preview.yazi/main.lua"),
            DEFAULT_YAZI_PLUGIN_TABULAR_PREVIEW.to_string(),
        ));
    }

    if config.addons.has_addon("yazi-omp") {
        files.push((
            std::path::PathBuf::from(".config/yazi/plugins/omp.yazi/main.lua"),
            DEFAULT_YAZI_PLUGIN_OMP.to_string(),
        ));
        files.push((
            std::path::PathBuf::from(".config/yazi/yazi-prompt.omp.json"),
            DEFAULT_YAZI_OMP_CONFIG.to_string(),
        ));
    }

    files
}

/// Remove managed runtime files that should not exist for the current config.
pub fn cleanup_disabled_runtime_files(config: &AiboxConfig) -> Result<Vec<String>> {
    let root = config.host_root_dir();
    let mut updated = Vec::new();

    if !include_lazygit_tab(config) {
        let lazygit_config = root.join(".config").join("lazygit").join("config.yml");
        if lazygit_config.exists() {
            fs::remove_file(&lazygit_config)
                .with_context(|| format!("Failed to remove {}", lazygit_config.display()))?;
            updated.push(".config/lazygit/config.yml (removed)".to_string());
        }
        let lazygit_dir = root.join(".config").join("lazygit");
        if lazygit_dir.exists() {
            let _ = fs::remove_dir(lazygit_dir);
        }
    }

    for rel_path in [
        ".cache/zellij/permissions.kdl",
        ".cache/org/Zellij Contributors/Zellij/permissions.kdl",
    ] {
        let permission_cache = root.join(rel_path);
        if permission_cache.exists() {
            fs::remove_file(&permission_cache)
                .with_context(|| format!("Failed to remove {}", permission_cache.display()))?;
            updated.push(format!("{rel_path} (removed legacy Zellij permissions)"));
        }
    }

    updated.extend(cleanup_legacy_managed_zellij_config(&root)?);

    let legacy_status = root.join(".local").join("bin").join("aibox-status");
    if legacy_managed_aibox_status_helper(&legacy_status) {
        fs::remove_file(&legacy_status)
            .with_context(|| format!("Failed to remove {}", legacy_status.display()))?;
        updated.push(".local/bin/aibox-status (removed legacy shell status helper)".to_string());
    }

    let stale_claude = root.join(".local").join("bin").join("claude");
    if stale_claude_home_symlink(&stale_claude) {
        fs::remove_file(&stale_claude)
            .with_context(|| format!("Failed to remove {}", stale_claude.display()))?;
        updated.push(".local/bin/claude (removed stale home-installer symlink)".to_string());
    }
    if ensure_claude_home_bin_shim(config, &stale_claude)? {
        updated.push(".local/bin/claude (linked to /usr/local/bin/claude)".to_string());
    }

    updated.extend(cleanup_zellij_runtime_cache(&root, &config.container.name)?);

    for rel_path in [
        ".cache/zellij",
        ".cache/org/Zellij Contributors/Zellij",
        ".cache/org/Zellij Contributors",
    ] {
        let cache_dir = root.join(rel_path);
        if cache_dir.exists() {
            fs::remove_dir_all(&cache_dir)
                .with_context(|| format!("Failed to remove {}", cache_dir.display()))?;
            updated.push(format!("{rel_path} (removed legacy Zellij cache)"));
        }
    }

    Ok(updated)
}

fn cleanup_zellij_runtime_cache(root: &Path, session_name: &str) -> Result<Vec<String>> {
    let zellij_cache = root.join(".cache").join("zellij");
    let mut updated = Vec::new();
    let Ok(entries) = fs::read_dir(&zellij_cache) else {
        return Ok(updated);
    };

    for entry in entries {
        let entry = entry.with_context(|| format!("Failed to read {}", zellij_cache.display()))?;
        let path = entry.path();
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();

        if name.starts_with("contract_version_") && path.is_dir() {
            let session_info = path.join("session_info").join(session_name);
            if session_info.exists() {
                fs::remove_dir_all(&session_info)
                    .with_context(|| format!("Failed to remove {}", session_info.display()))?;
                updated.push(format!(
                    ".cache/zellij/{}/session_info/{} (removed stale Zellij session)",
                    name, session_name
                ));
            }
            continue;
        }

        if path.is_dir() && is_uuid_like(&name) {
            fs::remove_dir_all(&path)
                .with_context(|| format!("Failed to remove {}", path.display()))?;
            updated.push(format!(
                ".cache/zellij/{} (removed stale Zellij plugin state)",
                name
            ));
        }
    }

    Ok(updated)
}

fn cleanup_legacy_managed_zellij_config(root: &Path) -> Result<Vec<String>> {
    let zellij_config = root.join(".config").join("zellij");
    let mut updated = Vec::new();

    let config = zellij_config.join("config.kdl");
    if file_contains(&config, "aibox zellij configuration") {
        fs::remove_file(&config)
            .with_context(|| format!("Failed to remove {}", config.display()))?;
        updated
            .push(".config/zellij/config.kdl (removed legacy managed Zellij config)".to_string());
    }

    let layouts = zellij_config.join("layouts");
    for name in [
        "ai.kdl",
        "aibox-status-hidden.kdl",
        "aibox-status-visible.kdl",
        "browse.kdl",
        "cowork-swap.kdl",
        "cowork.kdl",
        "dev.kdl",
        "focus.kdl",
    ] {
        let path = layouts.join(name);
        if file_contains(&path, "aibox-tab") || file_contains(&path, "aibox-status") {
            fs::remove_file(&path)
                .with_context(|| format!("Failed to remove {}", path.display()))?;
            updated.push(format!(
                ".config/zellij/layouts/{name} (removed legacy managed Zellij layout)"
            ));
        }
    }
    let _ = fs::remove_dir(&layouts);

    let themes = zellij_config.join("themes");
    for name in ["gruvbox-dark.kdl", "tokyo-night.kdl"] {
        let path = themes.join(name);
        if path.exists() {
            fs::remove_file(&path)
                .with_context(|| format!("Failed to remove {}", path.display()))?;
            updated.push(format!(
                ".config/zellij/themes/{name} (removed legacy managed Zellij theme)"
            ));
        }
    }
    let _ = fs::remove_dir(&themes);
    let _ = fs::remove_dir(&zellij_config);

    Ok(updated)
}

fn file_contains(path: &Path, needle: &str) -> bool {
    fs::read_to_string(path)
        .map(|body| body.contains(needle))
        .unwrap_or(false)
}

fn is_uuid_like(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 36 {
        return false;
    }
    for (idx, byte) in bytes.iter().enumerate() {
        if matches!(idx, 8 | 13 | 18 | 23) {
            if *byte != b'-' {
                return false;
            }
        } else if !byte.is_ascii_hexdigit() {
            return false;
        }
    }
    true
}

fn stale_claude_home_symlink(path: &Path) -> bool {
    let Ok(target) = fs::read_link(path) else {
        return false;
    };
    target
        .to_string_lossy()
        .contains(".local/share/claude/versions/")
}

#[cfg(unix)]
fn ensure_claude_home_bin_shim(config: &AiboxConfig, path: &Path) -> Result<bool> {
    if !config
        .ai
        .harnesses
        .contains(&crate::config::AiProvider::Claude)
        || path.symlink_metadata().is_ok()
    {
        return Ok(false);
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }
    symlink("/usr/local/bin/claude", path)
        .with_context(|| format!("Failed to link {}", path.display()))?;
    Ok(true)
}

#[cfg(not(unix))]
fn ensure_claude_home_bin_shim(_config: &AiboxConfig, _path: &Path) -> Result<bool> {
    Ok(false)
}

fn legacy_managed_aibox_status_helper(path: &Path) -> bool {
    let Ok(body) = fs::read_to_string(path) else {
        return false;
    };
    body.contains("read_proc_cmdline")
        && body.contains("count_ai_agents")
        && body.contains("--plugin-json")
        && body.contains("AIBOX_STATUS_INTERVAL")
}

fn tmux_conf(config: &AiboxConfig) -> String {
    let theme = config.customization.resolved_theme();
    let (bg, fg, accent) = crate::themes::tmux_status_colors(&theme);
    let status = match config.customization.tmux.status.mode {
        TmuxStatusMode::Extended | TmuxStatusMode::Plain => "on",
        TmuxStatusMode::Disabled => "off",
    };

    let status_right = match config.customization.tmux.status.mode {
        TmuxStatusMode::Extended => "#(aibox-status --once 2>/dev/null || true) %H:%M",
        TmuxStatusMode::Plain | TmuxStatusMode::Disabled => "%H:%M",
    };
    let (powerkit_block, powerkit_plugin) = tmux_powerkit_settings(config);

    let mut conf = DEFAULT_TMUX_CONF
        .replace("AIBOX_TMUX_PREFIX", &config.customization.tmux.prefix)
        .replace(
            "AIBOX_TMUX_SESSION",
            &config.customization.tmux.session_name,
        )
        .replace("AIBOX_TMUX_STATUS_RIGHT", status_right)
        .replace("AIBOX_TMUX_STATUS", status)
        .replace("AIBOX_TMUX_POWERKIT_BLOCK", &powerkit_block)
        .replace("AIBOX_TMUX_POWERKIT_PLUGIN", &powerkit_plugin)
        .replace("AIBOX_TMUX_BG", bg)
        .replace("AIBOX_TMUX_FG", fg)
        .replace("AIBOX_TMUX_ACCENT", accent);

    // Keep the status block deterministic and avoid shelling out when disabled.
    if config.customization.tmux.status.mode == TmuxStatusMode::Disabled {
        conf = conf.replace("set -g status-interval 5", "set -g status-interval 60");
    }
    conf
}

fn push_enabled(items: &mut Vec<&'static str>, enabled: bool, id: &'static str) {
    if enabled {
        items.push(id);
    }
}

fn tmux_powerkit_settings(config: &AiboxConfig) -> (String, String) {
    if config.customization.tmux.status.mode != TmuxStatusMode::Extended {
        return (String::new(), String::new());
    }

    let elements = &config.customization.tmux.status.elements;

    let mut line1_right = Vec::new();
    push_enabled(&mut line1_right, elements.hostname, "hostname");
    push_enabled(&mut line1_right, elements.external_ip, "external_ip");
    push_enabled(&mut line1_right, elements.ssh, "ssh");
    push_enabled(&mut line1_right, elements.uptime, "uptime");
    push_enabled(&mut line1_right, elements.weather, "weather");
    push_enabled(&mut line1_right, elements.datetime, "datetime");

    let mut line2_left = Vec::new();
    push_enabled(&mut line2_left, elements.git, "git");
    push_enabled(&mut line2_left, elements.github, "github");
    push_enabled(&mut line2_left, elements.kubernetes, "kubernetes");
    push_enabled(&mut line2_left, elements.terraform, "terraform");
    push_enabled(&mut line2_left, elements.cloud, "cloud");
    push_enabled(&mut line2_left, elements.cloudstatus, "cloudstatus");

    let mut line2_right = Vec::new();
    push_enabled(&mut line2_right, elements.cpu, "cpu");
    push_enabled(&mut line2_right, elements.loadavg, "loadavg");
    push_enabled(&mut line2_right, elements.mem, "memory");
    push_enabled(&mut line2_right, elements.swap, "swap");
    push_enabled(&mut line2_right, elements.disk, "disk");
    push_enabled(&mut line2_right, elements.gpu, "gpu");
    push_enabled(&mut line2_right, elements.netspeed, "netspeed");
    push_enabled(&mut line2_right, elements.ping, "ping");
    push_enabled(&mut line2_right, elements.aibox, "aibox");

    let mut plugin_order = Vec::new();
    plugin_order.extend(line1_right.iter().copied());
    plugin_order.extend(line2_left.iter().copied());
    plugin_order.extend(line2_right.iter().copied());

    let metrics = &elements.aibox_metrics;
    let aibox_metrics = [
        (metrics.log, "log"),
        (metrics.oom, "oom"),
        (metrics.proc, "proc"),
        (metrics.ai, "ai"),
        (metrics.mcp, "mcp"),
        (metrics.mig, "mig"),
    ]
    .iter()
    .filter_map(|(enabled, key)| enabled.then_some(*key))
    .collect::<Vec<_>>()
    .join(",");

    let (powerkit_theme, powerkit_variant) =
        crate::themes::tmux_powerkit_theme(&config.customization.resolved_theme());
    let powerkit_block = format!(
        r##"# Powerkit status.
set -g @powerkit_plugins "{}"
set -g @powerkit_bar_layout "double"
set -g @powerkit_status_order "session,plugins"
set -g @powerkit_theme "{}"
set -g @powerkit_theme_variant "{}"
set -g @powerkit_separator_style "rounded"
set -g @powerkit_elements_spacing "both"
set -g @powerkit_status_interval "5"
set -g @powerkit_transparent "false"
set -g @powerkit_pane_border_status "top"
set -g @powerkit_pane_border_format "#{{?client_prefix,PREFIX,NORMAL}} #{{pane_title}} #{{pane_current_command}}"
set -g @powerkit_line1_right "{}"
set -g @powerkit_line2_left "{}"
set -g @powerkit_line2_right "{}"
set -g @powerkit_plugin_aibox_metrics "{}""##,
        plugin_order.join(","),
        powerkit_theme,
        powerkit_variant,
        line1_right.join(","),
        line2_left.join(","),
        line2_right.join(","),
        aibox_metrics
    );
    let powerkit_plugin = "if-shell '[ -f /usr/local/share/aibox/tmux/plugins/tmux-powerkit/tmux-powerkit.tmux ]' 'run-shell /usr/local/share/aibox/tmux/plugins/tmux-powerkit/tmux-powerkit.tmux'".to_string();
    (powerkit_block, powerkit_plugin)
}

fn tmux_layout_script(
    layout: &ConfigLayout,
    providers: &[crate::config::AiProvider],
    include_lazygit: bool,
    session_name: &str,
) -> String {
    let provider = providers
        .iter()
        .find(|provider| provider.is_active())
        .map(|provider| provider.binary_name())
        .unwrap_or("bash");
    let git_window = if include_lazygit {
        r#"tmux -S "$socket" new-window -t "$session:" -n git -c "$workspace" "$(tool_or_shell lazygit)"
"#
    } else {
        ""
    };

    let primary_window = match layout {
        ConfigLayout::Dev => "dev",
        ConfigLayout::Focus => "focus",
        ConfigLayout::Cowork => "cowork",
        ConfigLayout::CoworkSwap => "cowork-swap",
        ConfigLayout::Browse => "browse",
        ConfigLayout::Ai => "ai",
    };

    let layout_body = match layout {
        ConfigLayout::Dev => format!(
            r#"tmux -S "$socket" -f "$config" new-session -d -s "$session" -n dev -c "$workspace" "$(tool_or_shell vim)"
editor_pane="$(tmux -S "$socket" display-message -p -t "$session:dev" '#{{pane_id}}')"
files_pane="$(tmux -S "$socket" split-window -t "$session:dev" -h -p 35 -P -F '#{{pane_id}}' -c "$workspace" "$(tool_or_shell yazi)")"
agent_pane="$(tmux -S "$socket" split-window -t "$session:dev" -v -p 40 -P -F '#{{pane_id}}' -c "$workspace" "$(tool_or_shell {provider})")"
tmux -S "$socket" select-pane -t "$editor_pane"
"#
        ),
        ConfigLayout::Focus => format!(
            r#"tmux -S "$socket" -f "$config" new-session -d -s "$session" -n focus -c "$workspace" "$(tool_or_shell {provider})"
tmux -S "$socket" new-window -t "$session:" -n editor -c "$workspace" "$(tool_or_shell vim)"
"#
        ),
        ConfigLayout::Cowork => format!(
            r#"tmux -S "$socket" -f "$config" new-session -d -s "$session" -n cowork -c "$workspace" "$(tool_or_shell vim)"
editor_pane="$(tmux -S "$socket" display-message -p -t "$session:cowork" '#{{pane_id}}')"
agent_pane="$(tmux -S "$socket" split-window -t "$session:cowork" -h -p 50 -P -F '#{{pane_id}}' -c "$workspace" "$(tool_or_shell {provider})")"
files_pane="$(tmux -S "$socket" split-window -t "$editor_pane" -v -p 35 -P -F '#{{pane_id}}' -c "$workspace" "$(tool_or_shell yazi)")"
tmux -S "$socket" select-pane -t "$editor_pane"
"#
        ),
        ConfigLayout::CoworkSwap => format!(
            r#"tmux -S "$socket" -f "$config" new-session -d -s "$session" -n cowork-swap -c "$workspace" "$(tool_or_shell yazi)"
files_pane="$(tmux -S "$socket" display-message -p -t "$session:cowork-swap" '#{{pane_id}}')"
agent_pane="$(tmux -S "$socket" split-window -t "$session:cowork-swap" -v -p 45 -P -F '#{{pane_id}}' -c "$workspace" "$(tool_or_shell {provider})")"
editor_pane="$(tmux -S "$socket" split-window -t "$session:cowork-swap" -h -p 60 -P -F '#{{pane_id}}' -c "$workspace" "$(tool_or_shell vim)")"
tmux -S "$socket" select-pane -t "$editor_pane"
"#
        ),
        ConfigLayout::Browse => format!(
            r#"tmux -S "$socket" -f "$config" new-session -d -s "$session" -n browse -c "$workspace" "$(tool_or_shell yazi)"
files_pane="$(tmux -S "$socket" display-message -p -t "$session:browse" '#{{pane_id}}')"
agent_pane="$(tmux -S "$socket" split-window -t "$session:browse" -v -p 35 -P -F '#{{pane_id}}' -c "$workspace" "$(tool_or_shell {provider})")"
tmux -S "$socket" new-window -t "$session:" -n editor -c "$workspace" "$(tool_or_shell vim)"
"#
        ),
        ConfigLayout::Ai => format!(
            r#"tmux -S "$socket" -f "$config" new-session -d -s "$session" -n ai -c "$workspace" "$(tool_or_shell yazi)"
files_pane="$(tmux -S "$socket" display-message -p -t "$session:ai" '#{{pane_id}}')"
agent_pane="$(tmux -S "$socket" split-window -t "$session:ai" -h -p 50 -P -F '#{{pane_id}}' -c "$workspace" "$(tool_or_shell {provider})")"
tmux -S "$socket" new-window -t "$session:" -n editor -c "$workspace" "$(tool_or_shell vim)"
tmux -S "$socket" new-window -t "$session:" -n shell -c "$workspace" "bash"
"#
        ),
    };

    format!(
        r#"#!/usr/bin/env bash
set -euo pipefail

session="${{AIBOX_TMUX_SESSION:-{session_name}}}"
workspace="${{AIBOX_WORKSPACE:-/workspace}}"
config="${{AIBOX_TMUX_CONFIG:-$HOME/.config/tmux/tmux.conf}}"
socket="${{AIBOX_TMUX_SOCKET:-$HOME/.tmux/aibox.sock}}"
mkdir -p "$(dirname "$socket")"

if tmux -S "$socket" -f "$config" has-session -t "$session" 2>/dev/null; then
  exec tmux -S "$socket" -f "$config" attach-session -t "$session"
fi

tool_or_shell() {{
  local tool="$1"
  if [[ "$tool" == "yazi" ]]; then
    printf "bash -lc 'for _ in {{1..50}}; do tmux -S %q list-clients -t %q >/dev/null 2>&1 && break; sleep 0.1; done; if command -v yazi >/dev/null 2>&1; then exec yazi; fi; exec bash'" "$socket" "$session"
    return
  fi
  printf "bash -lc 'if command -v %q >/dev/null 2>&1; then %q; fi; exec bash'" "$tool" "$tool"
}}

{layout_body}{git_window}tmux -S "$socket" select-window -t "$session:{primary_window}" 2>/dev/null || true
exec tmux -S "$socket" -f "$config" attach-session -t "$session"
"#,
        primary_window = primary_window,
    )
}

fn tmux_session_script(config: &AiboxConfig) -> String {
    format!(
        r#"#!/usr/bin/env bash
set -euo pipefail

layout="${{1:-${{AIBOX_TMUX_LAYOUT:-{layout}}}}}"
session="${{2:-${{AIBOX_TMUX_SESSION:-{session}}}}}"
socket="${{AIBOX_TMUX_SOCKET:-$HOME/.tmux/aibox.sock}}"
script="${{HOME}}/.config/tmux/layouts/${{layout}}.sh"

if [[ ! -x "${{script}}" ]]; then
  echo "aibox-tmux-session: unknown or unavailable managed layout: ${{layout}}" >&2
  exit 2
fi

exec env AIBOX_TMUX_SESSION="${{session}}" AIBOX_TMUX_SOCKET="${{socket}}" "${{script}}"
"#,
        layout = config.customization.tmux_layout(),
        session = &config.customization.tmux.session_name,
    )
}

/// Seed the .root/ directory structure and default config files.
/// Never overwrites existing files.
pub fn seed_root_dir(config: &AiboxConfig) -> Result<()> {
    let root = config.host_root_dir();

    let root_display = root.display();
    output::info(&format!("Seeding {} directory...", root_display));

    ensure_runtime_dirs(config)?;

    for (rel_path, content) in managed_runtime_files(config) {
        let path = root.join(&rel_path);
        seed_file(&path, &content)?;
        if rel_path == Path::new(".local/bin/pdf-watch")
            || rel_path == Path::new(".local/bin/open-in-editor")
            || rel_path == Path::new(".local/bin/aibox-preview")
            || rel_path == Path::new(".local/bin/aibox-status-toggle")
            || (rel_path.starts_with(".config/tmux/")
                && rel_path.extension().is_some_and(|ext| ext == "sh"))
        {
            ensure_executable(&path)?;
        }
    }
    let claude_shim = root.join(".local").join("bin").join("claude");
    ensure_claude_home_bin_shim(config, &claude_shim)?;

    // Warn if .ssh/ is empty
    let ssh_dir = root.join(".ssh");
    if ssh_dir.exists() {
        let entries = fs::read_dir(&ssh_dir)
            .with_context(|| format!("Failed to read .ssh directory: {}", ssh_dir.display()))?;
        if entries.count() == 0 {
            output::warn(&format!(
                "No SSH keys found in {}/.ssh/ — copy your keys manually if needed",
                root_display
            ));
        }
    }

    output::ok("Directory seeding complete");
    Ok(())
}

/// Restore managed runtime files that are missing, without overwriting edits.
pub fn restore_missing_managed_runtime_files(config: &AiboxConfig) -> Result<Vec<String>> {
    let root = config.host_root_dir();
    let mut restored = Vec::new();

    ensure_runtime_dirs(config)?;
    for (rel_path, content) in managed_runtime_files(config) {
        let path = root.join(&rel_path);
        if path.exists() {
            continue;
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }
        fs::write(&path, content)
            .with_context(|| format!("Failed to restore {}", path.display()))?;
        if rel_path == Path::new(".local/bin/pdf-watch")
            || rel_path == Path::new(".local/bin/open-in-editor")
            || rel_path == Path::new(".local/bin/aibox-preview")
            || rel_path == Path::new(".local/bin/aibox-status-toggle")
            || (rel_path.starts_with(".config/tmux/")
                && rel_path.extension().is_some_and(|ext| ext == "sh"))
        {
            ensure_executable(&path)?;
        }
        restored.push(rel_path.to_string_lossy().replace('\\', "/"));
    }

    Ok(restored)
}

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
    let session_name = &config.customization.tmux.session_name;
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
        let body = tmux_layout_script(&layout, providers, include_lazygit, session_name);
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

fn seed_file(path: &Path, content: &str) -> Result<()> {
    crate::context::write_if_missing(path, content)
}

#[cfg(unix)]
fn ensure_executable(path: &Path) -> Result<()> {
    let mut permissions = fs::metadata(path)
        .with_context(|| format!("Failed to read permissions for {}", path.display()))?
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)
        .with_context(|| format!("Failed to chmod +x {}", path.display()))
}

#[cfg(not(unix))]
fn ensure_executable(_path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(unix)]
fn ensure_executable_if_present(path: &Path) -> Result<bool> {
    if !path.exists() {
        return Ok(false);
    }

    let permissions = fs::metadata(path)
        .with_context(|| format!("Failed to read permissions for {}", path.display()))?
        .permissions();
    if permissions.mode() & 0o111 != 0 {
        return Ok(false);
    }

    ensure_executable(path)?;
    Ok(true)
}

#[cfg(not(unix))]
fn ensure_executable_if_present(_path: &Path) -> Result<bool> {
    Ok(false)
}

/// Write content to a file, overwriting if content differs.
/// Returns true if the file changed, false if content was already identical.
#[allow(dead_code)]
pub fn force_seed_file(path: &Path, content: &str) -> Result<bool> {
    crate::context::write_if_changed(path, content)
}

/// Migrate the deprecated yazi `[manager]` section name to `[mgr]`.
///
/// Yazi 25+ renamed the section, and uses of `[manager]` are silently
/// ignored. This helper edits an existing yazi config file in place,
/// rewriting any line that begins with `[manager]` to `[mgr]`. It is
/// idempotent — files already using `[mgr]` are left untouched.
///
/// Used for `yazi.toml`, `keymap.toml`, and `theme.toml` (which all
/// previously used `[manager]`). User customizations OUTSIDE the section
/// header are preserved.
///
/// Returns Ok(true) if the file was modified, Ok(false) otherwise (file
/// missing or no `[manager]` line found).
#[allow(dead_code)]
pub fn migrate_yazi_section(path: &Path) -> Result<bool> {
    if !path.is_file() {
        return Ok(false);
    }
    let content =
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    if !content.lines().any(|l| l.trim_end() == "[manager]") {
        return Ok(false);
    }
    let new_content: String = content
        .lines()
        .map(|line| {
            if line.trim_end() == "[manager]" {
                "[mgr]".to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        // Preserve trailing newline if the original had one
        + if content.ends_with('\n') { "\n" } else { "" };
    fs::write(path, new_content).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(true)
}

/// Force-seed all theme-dependent and AI-provider-dependent config files.
/// Overwrites existing files when content has changed. Used by `aibox apply`.
#[allow(dead_code)]
pub fn sync_theme_files(config: &AiboxConfig) -> Result<Vec<String>> {
    let root = config.host_root_dir();
    let theme = &config.customization.resolved_theme();
    let providers = &config.ai.harnesses;
    let include_lazygit = include_lazygit_tab(config);
    let session_name = &config.customization.tmux.session_name;
    let mut updated = Vec::new();

    // vimrc — colorscheme and background
    let vimrc = DEFAULT_VIMRC
        .replace(
            "AIBOX_VIM_COLORSCHEME",
            crate::themes::vim_colorscheme(theme),
        )
        .replace("AIBOX_VIM_BG", crate::themes::vim_background(theme));
    if force_seed_file(&root.join(".vim").join("vimrc"), &vimrc)? {
        updated.push(".vim/vimrc".to_string());
    }

    // tmux config and layouts — theme, layout, AI providers, and lazygit all
    // come from aibox.toml.
    if force_seed_file(
        &root.join(".config").join("tmux").join("tmux.conf"),
        &tmux_conf(config),
    )? {
        updated.push(".config/tmux/tmux.conf".to_string());
    }
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
        let body = tmux_layout_script(&layout, providers, include_lazygit, session_name);
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

    if force_seed_file(
        &root.join(".local").join("bin").join("open-in-editor"),
        DEFAULT_OPEN_IN_EDITOR_SH,
    )? {
        ensure_executable(&root.join(".local").join("bin").join("open-in-editor"))?;
        updated.push(".local/bin/open-in-editor".to_string());
    }
    if force_seed_file(
        &root.join(".local").join("bin").join("aibox-preview"),
        DEFAULT_AIBOX_PREVIEW_SH,
    )? {
        ensure_executable(&root.join(".local").join("bin").join("aibox-preview"))?;
        updated.push(".local/bin/aibox-preview".to_string());
    }

    if force_seed_file(
        &root.join(".local").join("bin").join("aibox-status-toggle"),
        DEFAULT_AIBOX_STATUS_TOGGLE_SH,
    )? {
        ensure_executable(&root.join(".local").join("bin").join("aibox-status-toggle"))?;
        updated.push(".local/bin/aibox-status-toggle".to_string());
    }

    if include_lazygit
        && force_seed_file(
            &root.join(".config").join("lazygit").join("config.yml"),
            &crate::themes::lazygit_theme(theme),
        )?
    {
        updated.push(".config/lazygit/config.yml".to_string());
    }
    updated.extend(cleanup_disabled_runtime_files(config)?);

    // Yazi managed config. This is version-sensitive: Yazi 26 rejects the
    // historical `name = ...` matcher schema, so apply must refresh stale
    // project-owned runtime config even when the selected theme did not change.
    if force_seed_file(
        &root.join(".config").join("yazi").join("yazi.toml"),
        &generate_yazi_config(config),
    )? {
        updated.push(".config/yazi/yazi.toml".to_string());
    }
    if force_seed_file(
        &root.join(".config").join("yazi").join("keymap.toml"),
        DEFAULT_YAZI_KEYMAP,
    )? {
        updated.push(".config/yazi/keymap.toml".to_string());
    }
    if force_seed_file(
        &root.join(".config").join("yazi").join("init.lua"),
        &generate_yazi_init(config),
    )? {
        updated.push(".config/yazi/init.lua".to_string());
    }

    // Yazi theme — force-update from the bundled theme for the selected theme
    if force_seed_file(
        &root.join(".config").join("yazi").join("theme.toml"),
        crate::themes::yazi_theme(theme),
    )? {
        updated.push(".config/yazi/theme.toml".to_string());
    }

    // Yazi config migration: rewrite [manager] → [mgr] in existing files.
    // Yazi 25+ silently ignores [manager], so any user customization that
    // still uses the old section name (from older aibox releases) needs to
    // be migrated. The migration is idempotent and preserves user content
    // outside the section header.
    let yazi_dir = root.join(".config").join("yazi");
    for filename in ["yazi.toml", "keymap.toml", "theme.toml"] {
        let path = yazi_dir.join(filename);
        if migrate_yazi_section(&path)? {
            updated.push(format!(
                ".config/yazi/{} (migrated [manager] → [mgr])",
                filename
            ));
        }
    }

    // Starship prompt
    let prompt = &config.customization.prompt;
    let starship_content = crate::themes::starship_config(prompt, theme);
    if force_seed_file(
        &root.join(".config").join("starship.toml"),
        &starship_content,
    )? {
        updated.push(".config/starship.toml".to_string());
    }

    // Claude Code keybindings — disable Ctrl+g (reserved for the tmux prefix).
    if providers.contains(&crate::config::AiProvider::Claude)
        && force_seed_file(
            &root.join(".claude").join("keybindings.json"),
            DEFAULT_CLAUDE_KEYBINDINGS,
        )?
    {
        updated.push(".claude/keybindings.json".to_string());
    }

    updated.extend(sync_managed_runtime_permissions(config)?);

    Ok(updated)
}

pub fn sync_managed_runtime_permissions(config: &AiboxConfig) -> Result<Vec<String>> {
    let root = config.host_root_dir();
    let mut updated = Vec::new();

    for rel_path in [
        ".local/bin/pdf-watch",
        ".local/bin/open-in-editor",
        ".local/bin/aibox-preview",
        ".local/bin/aibox-status-toggle",
    ] {
        if ensure_executable_if_present(&root.join(rel_path))? {
            updated.push(format!("{} (chmod +x)", rel_path));
        }
    }

    Ok(updated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::*;
    use serial_test::serial;

    fn make_config(audio_enabled: bool, root_dir: std::path::PathBuf) -> AiboxConfig {
        crate::config::set_test_host_root(Some(root_dir));
        let mut config = crate::config::test_config();
        config.container.name = "test".to_string();
        config.container.hostname = "test".to_string();
        config.audio = AudioSection {
            enabled: audio_enabled,
            pulse_server: "tcp:localhost:4714".to_string(),
            ..AudioSection::default()
        };
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
    fn seed_root_dir_creates_directories() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let config = make_config(false, root.clone());
        seed_root_dir(&config).unwrap();

        assert!(root.join(".ssh").is_dir());
        assert!(root.join(".local").join("state").is_dir());
        assert!(root.join(".config").join("state").is_dir());
        assert!(root.join(".vim").join("undo").is_dir());
        assert!(root.join(".config").join("tmux").join("layouts").is_dir());
        assert!(root.join(".tmux").join("plugins").is_dir());
        assert!(!root.join(".config").join("zellij").exists());
        assert!(root.join(".config").join("yazi").is_dir());
        assert!(root.join(".config").join("git").is_dir());
        assert!(root.join(".claude").is_dir());
        clear_test_host_root();
    }

    #[test]
    #[serial]
    fn seed_root_dir_creates_lazygit_state_directory_when_enabled() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let mut config = make_config(false, root.clone());
        let mut tools = std::collections::HashMap::new();
        tools.insert(
            "lazygit".to_string(),
            ToolEntry {
                version: None,
                enabled: Some(true),
            },
        );
        config
            .addons
            .addons
            .insert("git-ui".to_string(), AddonToolsSection { tools });

        seed_root_dir(&config).unwrap();

        assert!(root.join(".config").join("lazygit").is_dir());
        assert!(root.join(".local").join("state").join("lazygit").is_dir());
        assert!(root.join(".config").join("state").join("lazygit").is_dir());
        clear_test_host_root();
    }

    #[test]
    fn default_yazi_open_rules_use_current_url_schema() {
        assert!(
            DEFAULT_YAZI_CONFIG.contains(r#"{ url = "*", use = "edit" }"#),
            "Yazi 26 open rules require url or mime matchers"
        );
        assert!(
            DEFAULT_YAZI_CONFIG.contains(r#"run = "git", group = "git""#)
                && DEFAULT_YAZI_CONFIG.contains(r#"run = "status-git", group = "status-git""#),
            "Yazi 26 plugin fetchers require explicit groups"
        );
        assert!(
            !DEFAULT_YAZI_CONFIG.contains("name = \"*\""),
            "Yazi 26 rejects name-only [open] rules"
        );
        assert!(
            !DEFAULT_YAZI_INIT.contains("th.git =") && DEFAULT_YAZI_INIT.contains("signs = {"),
            "Yazi 26 exposes th.git as a custom theme section; init.lua must pass sign overrides through git.yazi setup options"
        );
    }

    #[test]
    fn bundled_yazi_themes_use_current_url_schema() {
        for theme in [
            Theme::GruvboxDark,
            Theme::CatppuccinMocha,
            Theme::CatppuccinLatte,
            Theme::Dracula,
            Theme::TokyoNight,
            Theme::Nord,
            Theme::Projectious,
        ] {
            let body = crate::themes::yazi_theme(&theme);
            assert!(
                body.contains("url = \"*/\""),
                "Yazi 26 filetype rules require url or mime matchers for {theme}"
            );
            assert!(
                !body.contains("{ name ="),
                "Yazi 26 rejects name-only filetype rules for {theme}"
            );
        }
    }

    #[test]
    #[serial]
    fn seed_root_dir_creates_codex_directory_when_openai_enabled() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let mut config = make_config(false, root.clone());
        config.ai.harnesses = vec![AiProvider::Codex];
        seed_root_dir(&config).unwrap();

        assert!(root.join(".codex").is_dir());
        assert!(!root.join(".claude").exists());
        clear_test_host_root();
    }

    #[test]
    #[serial]
    fn seed_root_dir_seeds_opencode_processkit_gate_plugin_when_opencode_enabled() {
        // aibox#51 (v0.18.7): with OpenCode enabled, sync must seed
        // .opencode/plugins/processkit-gate.ts so an OpenCode session enforces
        // the same compliance gate as Claude Code's PreToolUse hook.
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let mut config = make_config(false, root.clone());
        config.ai.harnesses = vec![AiProvider::OpenCode];
        seed_root_dir(&config).unwrap();

        let plugin_path = root
            .join(".opencode")
            .join("plugins")
            .join("processkit-gate.ts");
        assert!(
            plugin_path.is_file(),
            "processkit-gate.ts must be seeded when OpenCode is configured: {}",
            plugin_path.display()
        );
        let body = fs::read_to_string(&plugin_path).unwrap();
        assert!(
            body.contains("export const ProcesskitGate"),
            "plugin must export ProcesskitGate"
        );
        assert!(
            body.contains("session.created") && body.contains("tool.execute.before"),
            "plugin must wire both lifecycle hooks"
        );
        assert!(
            body.contains("acknowledge_contract"),
            "plugin must reference acknowledge_contract as the gate"
        );
        clear_test_host_root();
    }

    #[test]
    #[serial]
    fn seed_root_dir_does_not_seed_opencode_plugin_when_opencode_absent() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let mut config = make_config(false, root.clone());
        config.ai.harnesses = vec![AiProvider::Claude];
        seed_root_dir(&config).unwrap();

        assert!(
            !root.join(".opencode").exists(),
            "OpenCode dir must not be created when OpenCode is not in harnesses"
        );
        clear_test_host_root();
    }

    #[test]
    #[serial]
    fn seed_root_dir_creates_config_files() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let config = make_config(false, root.clone());
        seed_root_dir(&config).unwrap();

        assert!(root.join(".vim").join("vimrc").exists());
        assert!(root.join(".config").join("git").join("config").exists());
        assert!(root.join(".config").join("tmux").join("tmux.conf").exists());
        assert!(
            root.join(".config")
                .join("tmux")
                .join("aibox-session.sh")
                .exists()
        );
        for layout in ["dev", "focus", "cowork", "browse", "ai", "cowork-swap"] {
            assert!(
                root.join(".config")
                    .join("tmux")
                    .join("layouts")
                    .join(format!("{layout}.sh"))
                    .exists(),
                "missing tmux layout {layout}"
            );
        }
        assert!(root.join(".config").join("yazi").join("yazi.toml").exists());
        assert!(
            root.join(".config")
                .join("yazi")
                .join("keymap.toml")
                .exists()
        );
        assert!(root.join(".config").join("cheatsheet.txt").exists());
        assert!(root.join(".local").join("bin").join("pdf-watch").exists());
        assert!(
            root.join(".local")
                .join("bin")
                .join("open-in-editor")
                .exists()
        );
        assert!(
            root.join(".local")
                .join("bin")
                .join("aibox-preview")
                .exists()
        );
        assert!(
            !root
                .join(".local")
                .join("bin")
                .join("aibox-status")
                .exists(),
            "aibox-status is image-owned Rust binary, not a seeded shell helper"
        );
        assert!(
            root.join(".local")
                .join("bin")
                .join("aibox-status-toggle")
                .exists()
        );
        #[cfg(unix)]
        assert_ne!(
            fs::metadata(root.join(".local").join("bin").join("pdf-watch"))
                .unwrap()
                .permissions()
                .mode()
                & 0o111,
            0,
            "pdf-watch should be executable"
        );
        #[cfg(unix)]
        assert_ne!(
            fs::metadata(root.join(".local").join("bin").join("open-in-editor"))
                .unwrap()
                .permissions()
                .mode()
                & 0o111,
            0,
            "open-in-editor should be executable"
        );
        #[cfg(unix)]
        assert_ne!(
            fs::metadata(root.join(".local").join("bin").join("aibox-preview"))
                .unwrap()
                .permissions()
                .mode()
                & 0o111,
            0,
            "aibox-preview should be executable"
        );
        let open_in_editor =
            fs::read_to_string(root.join(".local").join("bin").join("open-in-editor")).unwrap();
        assert!(
            open_in_editor.contains("tmux select-window")
                || open_in_editor.contains("tmux new-window"),
            "open-in-editor should target or create a tmux editor window"
        );
        assert!(
            open_in_editor.contains("tmux select-pane")
                || open_in_editor.contains("tmux split-window"),
            "open-in-editor should support same-window tmux editor panes"
        );
        assert!(
            open_in_editor.contains("find_directional_pane")
                && open_in_editor.contains("select-pane -t \"$source_pane\" \"$select_flag\""),
            "open-in-editor should use explicit adjacent editor panes before creating a split"
        );
        assert!(
            open_in_editor.contains(":edit ${vim_file}"),
            "open-in-editor should open the selected file in the focused Vim pane"
        );
        assert!(
            !open_in_editor.contains("edit --in-place"),
            "open-in-editor must not replace the Yazi pane with an in-place editor"
        );
        assert!(
            open_in_editor.contains("list-windows"),
            "open-in-editor should auto-detect editor windows"
        );
        assert!(
            open_in_editor.contains("list-panes")
                && open_in_editor.contains("tmux send-keys -t \"$target\" C-c"),
            "open-in-editor should rehydrate non-vim editor panes before opening files"
        );
        let aibox_preview =
            fs::read_to_string(root.join(".local").join("bin").join("aibox-preview")).unwrap();
        assert!(
            aibox_preview.contains("glow -p") && aibox_preview.contains("bat --paging=always"),
            "aibox-preview should prefer glow for Markdown and fall back to bat"
        );
        assert!(
            aibox_preview.contains("pdf-watch"),
            "aibox-preview should dispatch PDF previews to pdf-watch"
        );
        assert!(
            !root
                .join(".local")
                .join("bin")
                .join("aibox-status")
                .exists(),
            "aibox-status is provided by the image Rust binary and must not be seeded as a shell helper"
        );
        #[cfg(unix)]
        assert_ne!(
            fs::metadata(root.join(".local").join("bin").join("aibox-status-toggle"))
                .unwrap()
                .permissions()
                .mode()
                & 0o111,
            0,
            "aibox-status-toggle should be executable"
        );
        clear_test_host_root();
    }

    #[test]
    #[serial]
    fn managed_runtime_files_omit_lazygit_when_explicitly_disabled() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let mut config = make_config(false, root);
        let mut tools = std::collections::HashMap::new();
        tools.insert(
            "gh".to_string(),
            ToolEntry {
                version: None,
                enabled: None,
            },
        );
        tools.insert(
            "lazygit".to_string(),
            ToolEntry {
                version: None,
                enabled: Some(false),
            },
        );
        config
            .addons
            .addons
            .insert("git-ui".to_string(), AddonToolsSection { tools });

        let files = managed_runtime_files(&config);
        let generated_layouts: Vec<_> = files
            .iter()
            .filter(|(path, _)| {
                path.starts_with(".config/tmux/layouts")
                    && path.extension().is_some_and(|ext| ext == "sh")
            })
            .collect();

        assert!(
            generated_layouts.len() >= 6,
            "expected all managed tmux layouts to be generated"
        );
        for (path, body) in generated_layouts {
            assert!(
                !body.contains("lazygit"),
                "disabled lazygit must not appear in generated layout {}",
                path.display()
            );
            assert!(
                !body.contains("new-window -t \"$session:\" -n git"),
                "disabled lazygit must omit the git window in generated layout {}",
                path.display()
            );
        }
        assert!(
            !files
                .iter()
                .any(|(path, _)| path == &std::path::PathBuf::from(".config/lazygit/config.yml")),
            "disabled lazygit must not generate managed lazygit config"
        );
        clear_test_host_root();
    }

    #[test]
    fn managed_runtime_files_omit_legacy_zellij_permission_cache() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let config = make_config(false, root);
        let files = managed_runtime_files(&config);
        assert!(
            !files.iter().any(|(path, _)| path
                == &std::path::PathBuf::from(".cache/zellij/permissions.kdl")),
            "tmux runtime must not seed legacy Zellij plugin permission cache"
        );
    }

    #[test]
    fn cleanup_removes_legacy_zellij_permission_cache() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        fs::create_dir_all(root.join(".cache/zellij")).unwrap();
        fs::create_dir_all(root.join(".cache/org/Zellij Contributors/Zellij")).unwrap();
        fs::write(root.join(".cache/zellij/permissions.kdl"), "RunCommands").unwrap();
        fs::write(
            root.join(".cache/org/Zellij Contributors/Zellij/permissions.kdl"),
            "RunCommands",
        )
        .unwrap();

        let config = make_config(false, root.clone());

        let updated = cleanup_disabled_runtime_files(&config).unwrap();
        assert!(updated.iter().any(
            |path| path == ".cache/zellij/permissions.kdl (removed legacy Zellij permissions)"
        ));
        assert!(!root.join(".cache/zellij/permissions.kdl").exists());
        assert!(
            !root
                .join(".cache/org/Zellij Contributors/Zellij/permissions.kdl")
                .exists()
        );
    }

    #[test]
    fn cleanup_removes_stale_zellij_session_and_plugin_state() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let config = make_config(false, root.clone());
        let session_name = config.container.name.clone();
        let session = root
            .join(".cache/zellij/contract_version_1/session_info")
            .join(&session_name);
        let plugin_state = root.join(".cache/zellij/04db9126-52cd-4a49-a7d4-90c07443c87a");
        let named_plugin_cache = root.join(".cache/zellij/zellij:link/plugin_cache");
        fs::create_dir_all(&session).unwrap();
        fs::create_dir_all(&plugin_state).unwrap();
        fs::create_dir_all(&named_plugin_cache).unwrap();
        fs::write(root.join(".cache/zellij/permissions.kdl"), "RunCommands").unwrap();
        fs::write(session.join("session-layout.kdl"), "layout {}").unwrap();
        fs::write(plugin_state.join("state.bin"), "state").unwrap();

        let updated = cleanup_disabled_runtime_files(&config).unwrap();

        assert!(updated.iter().any(|path| {
            path
            == ".cache/zellij/contract_version_1/session_info/test (removed stale Zellij session)"
        }));
        assert!(updated.iter().any(|path| path
            == ".cache/zellij/04db9126-52cd-4a49-a7d4-90c07443c87a (removed stale Zellij plugin state)"));
        assert!(!session.exists());
        assert!(!plugin_state.exists());
        assert!(
            !root.join(".cache/zellij/permissions.kdl").exists(),
            "legacy permission cache must be removed"
        );
        assert!(
            !named_plugin_cache.exists(),
            "v0.25+ removes the legacy Zellij cache tree entirely"
        );
    }

    #[test]
    fn cleanup_removes_legacy_managed_zellij_config_but_keeps_user_files() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let config = make_config(false, root.clone());
        fs::create_dir_all(root.join(".config/zellij/layouts")).unwrap();
        fs::write(
            root.join(".config/zellij/config.kdl"),
            "// aibox zellij configuration\n",
        )
        .unwrap();
        fs::write(
            root.join(".config/zellij/layouts/dev.kdl"),
            "layout { aibox-tab name=\"dev\" }\n",
        )
        .unwrap();
        fs::write(
            root.join(".config/zellij/layouts/personal.kdl"),
            "layout {}\n",
        )
        .unwrap();

        let updated = cleanup_disabled_runtime_files(&config).unwrap();

        assert!(
            updated
                .iter()
                .any(|path| path
                    == ".config/zellij/config.kdl (removed legacy managed Zellij config)")
        );
        assert!(
            updated.iter().any(|path| path
                == ".config/zellij/layouts/dev.kdl (removed legacy managed Zellij layout)")
        );
        assert!(!root.join(".config/zellij/config.kdl").exists());
        assert!(!root.join(".config/zellij/layouts/dev.kdl").exists());
        assert!(root.join(".config/zellij/layouts/personal.kdl").exists());
    }

    #[test]
    fn cleanup_removes_legacy_shell_aibox_status_helper() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        fs::create_dir_all(root.join(".local/bin")).unwrap();
        fs::write(
            root.join(".local/bin/aibox-status"),
            "#!/usr/bin/env bash\nread_proc_cmdline() { :; }\ncount_ai_agents() { :; }\ncase \"$1\" in --plugin-json) :;; esac\nAIBOX_STATUS_INTERVAL=5\n",
        )
        .unwrap();

        let config = make_config(false, root.clone());
        let updated = cleanup_disabled_runtime_files(&config).unwrap();

        assert!(
            updated
                .iter()
                .any(|path| path == ".local/bin/aibox-status (removed legacy shell status helper)")
        );
        assert!(!root.join(".local/bin/aibox-status").exists());
    }

    #[test]
    #[serial]
    fn seed_root_dir_uses_resolved_theme_mode() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let mut config = make_config(false, root.clone());
        config.customization.theme = Theme::Dracula;
        config.customization.mode = ThemeMode::Light;
        seed_root_dir(&config).unwrap();

        assert!(
            root.join(".config").join("tmux").join("tmux.conf").exists(),
            "tmux config should be seeded"
        );
        let tmux = fs::read_to_string(root.join(".config/tmux/tmux.conf")).unwrap();
        assert!(
            tmux.contains("#1E66F5"),
            "light theme should render into tmux"
        );
        let vimrc = fs::read_to_string(root.join(".vim").join("vimrc")).unwrap();
        assert!(vimrc.contains("colorscheme catppuccin_latte"));
        assert!(vimrc.contains("set background=light"));
        clear_test_host_root();
    }

    #[test]
    #[serial]
    #[cfg(unix)]
    fn sync_theme_files_restores_managed_helper_executability() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let config = make_config(false, root.clone());
        seed_root_dir(&config).unwrap();

        let toggle_path = root.join(".local").join("bin").join("aibox-status-toggle");
        let mut permissions = fs::metadata(&toggle_path).unwrap().permissions();
        permissions.set_mode(0o644);
        fs::set_permissions(&toggle_path, permissions).unwrap();

        let updated = sync_theme_files(&config).unwrap();

        assert!(
            updated
                .iter()
                .any(|path| path == ".local/bin/aibox-status-toggle (chmod +x)")
        );
        assert_ne!(
            fs::metadata(&toggle_path).unwrap().permissions().mode() & 0o111,
            0,
            "aibox-status-toggle should be executable after apply-time sync"
        );
        clear_test_host_root();
    }

    #[test]
    #[cfg(unix)]
    #[serial]
    fn cleanup_removes_stale_claude_home_installer_symlink() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let config = make_config(false, root.clone());
        let stale = root.join(".local").join("bin").join("claude");
        fs::create_dir_all(stale.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink("/home/aibox/.local/share/claude/versions/2.1.129", &stale)
            .unwrap();

        let updated = cleanup_disabled_runtime_files(&config).unwrap();

        assert!(stale.symlink_metadata().is_ok());
        assert_eq!(
            fs::read_link(&stale).unwrap(),
            std::path::PathBuf::from("/usr/local/bin/claude")
        );
        assert!(
            updated
                .iter()
                .any(|path| path == ".local/bin/claude (removed stale home-installer symlink)")
        );
        assert!(
            updated
                .iter()
                .any(|path| path == ".local/bin/claude (linked to /usr/local/bin/claude)")
        );
        clear_test_host_root();
    }

    #[test]
    #[cfg(unix)]
    #[serial]
    fn cleanup_seeds_claude_home_bin_shim_when_missing() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let config = make_config(false, root.clone());
        let shim = root.join(".local").join("bin").join("claude");

        let updated = cleanup_disabled_runtime_files(&config).unwrap();

        assert_eq!(
            fs::read_link(&shim).unwrap(),
            std::path::PathBuf::from("/usr/local/bin/claude")
        );
        assert!(
            updated
                .iter()
                .any(|path| path == ".local/bin/claude (linked to /usr/local/bin/claude)")
        );
        clear_test_host_root();
    }

    #[test]
    #[cfg(unix)]
    #[serial]
    fn cleanup_keeps_non_claude_home_symlink() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let config = make_config(false, root.clone());
        let custom_target = root.join("custom-claude");
        fs::create_dir_all(&root).unwrap();
        fs::write(&custom_target, "#!/bin/sh\n").unwrap();
        let custom = root.join(".local").join("bin").join("claude");
        fs::create_dir_all(custom.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(&custom_target, &custom).unwrap();

        let updated = cleanup_disabled_runtime_files(&config).unwrap();

        assert!(custom.exists());
        assert!(
            !updated
                .iter()
                .any(|path| path.contains(".local/bin/claude"))
        );
        clear_test_host_root();
    }

    #[test]
    #[serial]
    fn sync_theme_files_refreshes_stale_yazi_26_matchers() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let config = make_config(false, root.clone());
        seed_root_dir(&config).unwrap();

        let yazi_dir = root.join(".config").join("yazi");
        fs::write(
            yazi_dir.join("yazi.toml"),
            r#"[open]
rules = [
    { mime = "text/*", use = "edit" },
    { name = "*", use = "edit" },
]
"#,
        )
        .unwrap();
        fs::write(
            yazi_dir.join("theme.toml"),
            r##"[filetype]
rules = [
    { name = "*.rs", fg = "#ffffff" },
]
"##,
        )
        .unwrap();

        let updated = sync_theme_files(&config).unwrap();

        assert!(
            updated.contains(&".config/yazi/yazi.toml".to_string()),
            "stale yazi.toml should be force-refreshed: {updated:?}"
        );
        assert!(
            updated.contains(&".config/yazi/theme.toml".to_string()),
            "stale yazi theme should be force-refreshed: {updated:?}"
        );
        let yazi = fs::read_to_string(yazi_dir.join("yazi.toml")).unwrap();
        let theme = fs::read_to_string(yazi_dir.join("theme.toml")).unwrap();
        assert!(yazi.contains(r#"{ url = "*", use = "edit" }"#));
        assert!(theme.contains(r#"url = "*.rs""#));
        assert!(!yazi.contains("{ name ="));
        assert!(!theme.contains("{ name ="));
        clear_test_host_root();
    }

    #[test]
    #[serial]
    fn seed_root_dir_does_not_overwrite_existing() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        fs::create_dir_all(root.join(".vim")).unwrap();
        fs::write(root.join(".vim").join("vimrc"), "custom vimrc").unwrap();

        let config = make_config(false, root.clone());
        seed_root_dir(&config).unwrap();

        let content = fs::read_to_string(root.join(".vim").join("vimrc")).unwrap();
        assert_eq!(
            content, "custom vimrc",
            "should not overwrite existing file"
        );
        clear_test_host_root();
    }

    #[test]
    #[serial]
    fn seed_root_dir_creates_asoundrc_when_audio_enabled() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let config = make_config(true, root.clone());
        seed_root_dir(&config).unwrap();

        assert!(
            root.join(".asoundrc").exists(),
            ".asoundrc should be created when audio enabled"
        );
        clear_test_host_root();
    }

    #[test]
    #[serial]
    fn seed_root_dir_no_asoundrc_when_audio_disabled() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let config = make_config(false, root.clone());
        seed_root_dir(&config).unwrap();

        assert!(
            !root.join(".asoundrc").exists(),
            ".asoundrc should not exist when audio disabled"
        );
        clear_test_host_root();
    }

    #[test]
    fn seed_file_creates_with_content() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        seed_file(&path, "hello world").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "hello world");
    }

    #[test]
    fn seed_file_skips_existing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        fs::write(&path, "original").unwrap();
        seed_file(&path, "new content").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "original");
    }

    #[test]
    fn tmux_dev_layout_uses_selected_primary_provider() {
        let providers = vec![AiProvider::Codex, AiProvider::Claude];
        let layout = tmux_layout_script(&ConfigLayout::Dev, &providers, true, "aibox");

        assert!(
            layout.contains(
                "tmux -S \"$socket\" -f \"$config\" new-session -d -s \"$session\" -n dev"
            )
        );
        assert!(layout.contains("tool_or_shell codex"));
        assert!(!layout.contains("tool_or_shell claude"));
        assert!(layout.contains("tmux -S \"$socket\" new-window -t \"$session:\" -n git"));
        assert!(layout.contains("socket=\"${AIBOX_TMUX_SOCKET:-$HOME/.tmux/aibox.sock}\""));
        assert!(!layout.contains("zellij"));
    }

    #[test]
    fn tmux_layouts_start_expected_windows_and_panes() {
        let providers = [AiProvider::Claude];
        let layouts = [
            (ConfigLayout::Dev, "dev"),
            (ConfigLayout::Focus, "focus"),
            (ConfigLayout::Cowork, "cowork"),
            (ConfigLayout::CoworkSwap, "cowork-swap"),
            (ConfigLayout::Browse, "browse"),
            (ConfigLayout::Ai, "ai"),
        ];

        for (layout, name) in layouts {
            let body = tmux_layout_script(&layout, &providers, false, "aibox");
            assert!(
                body.contains(&format!("-n {name}")),
                "{name} layout should name its first tmux window:\n{body}"
            );
            assert!(
                body.contains(&format!(
                    "tmux -S \"$socket\" select-window -t \"$session:{name}\""
                )),
                "{name} layout should reselect the named primary window, not a numeric index:\n{body}"
            );
            assert!(
                !body.contains("tmux select-window -t \"$session:1\""),
                "layout script should not assume window index 1:\n{body}"
            );
            assert!(!body.contains("start_suspended"));
            assert!(!body.contains("zellij"));
        }
    }

    #[test]
    #[serial]
    fn sync_tmux_runtime_files_refreshes_status_and_layout_scripts() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let mut config = make_config(false, root.clone());
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
        let tmux_conf = fs::read_to_string(root.join(".config/tmux/tmux.conf")).unwrap();
        let ai_layout = fs::read_to_string(root.join(".config/tmux/layouts/ai.sh")).unwrap();

        assert!(updated.contains(&".config/tmux/tmux.conf".to_string()));
        assert!(updated.contains(&".config/tmux/layouts/ai.sh".to_string()));
        assert!(tmux_conf.contains("set -g status on"));
        assert!(tmux_conf.contains("tmux-powerkit.tmux"));
        assert!(!tmux_conf.contains("off_RIGHT"));
        assert!(ai_layout.contains("list-clients"));
        clear_test_host_root();
    }

    #[test]
    fn tmux_ai_layout_omits_unselected_claude() {
        let providers = vec![AiProvider::Codex];
        let layout = tmux_layout_script(&ConfigLayout::Ai, &providers, false, "aibox");

        assert!(layout.contains("tool_or_shell codex"));
        assert!(!layout.contains("tool_or_shell claude"));
        assert!(layout.contains("tmux -S \"$socket\" new-window -t \"$session:\" -n editor"));
        assert!(layout.contains("tmux -S \"$socket\" new-window -t \"$session:\" -n shell"));
    }

    #[test]
    fn tmux_session_helper_dispatches_to_provider_aware_layout_script() {
        let mut config = crate::config::test_config();
        config.customization.layout = ConfigLayout::Browse;
        config.customization.tmux.layout = Some(ConfigLayout::Ai);
        config.customization.tmux.session_name = "work".to_string();
        let script = tmux_session_script(&config);

        assert!(script.contains(r#"AIBOX_TMUX_LAYOUT:-ai"#));
        assert!(script.contains(r#"AIBOX_TMUX_SESSION:-work"#));
        assert!(script.contains(r#"AIBOX_TMUX_SOCKET:-$HOME/.tmux/aibox.sock"#));
        assert!(script.contains(r#".config/tmux/layouts/${layout}.sh"#));
        assert!(script.contains(
            r#"exec env AIBOX_TMUX_SESSION="${session}" AIBOX_TMUX_SOCKET="${socket}" "${script}""#
        ));
        assert!(!script.contains("tmux new-session"));
    }

    #[test]
    fn default_yazi_config_uses_mgr_section() {
        // Regression test for the [manager] → [mgr] rename in yazi 25+.
        // The seeded config must use [mgr] or yazi will silently ignore it.
        assert!(
            DEFAULT_YAZI_CONFIG.contains("[mgr]"),
            "default yazi config must use [mgr] section"
        );
        assert!(
            !DEFAULT_YAZI_CONFIG.contains("[manager]"),
            "default yazi config must not use deprecated [manager] section"
        );
    }

    #[test]
    fn yazi_toggle_pane_keybindings_seeded() {
        assert!(
            DEFAULT_YAZI_KEYMAP.contains("plugin toggle-pane max-preview"),
            "default yazi keymap should expose toggle-pane maximize binding"
        );
        assert!(
            DEFAULT_YAZI_KEYMAP.contains("plugin toggle-pane min-preview"),
            "default yazi keymap should expose toggle-pane preview toggle binding"
        );
    }

    #[test]
    fn yazi_navigation_keybindings_seeded() {
        assert!(
            DEFAULT_YAZI_KEYMAP
                .contains(r#"{ on = "p", run = "shell 'aibox-preview \"$1\"' --block""#),
            "default yazi keymap should expose full-pane preview"
        );
        assert!(
            DEFAULT_YAZI_KEYMAP.contains(
                r#"{ on = [ "w", "s" ], run = "shell 'du -sch \"$@\" | ${PAGER:-less}' --block""#
            ),
            "default yazi keymap should expose selected-size calculation"
        );
        assert!(
            DEFAULT_YAZI_KEYMAP.contains("less -R -S"),
            "default yazi keymap should expose horizontal-scroll pager"
        );
        assert!(
            DEFAULT_YAZI_KEYMAP.contains("pdf-watch"),
            "default yazi keymap should expose PDF watch helper"
        );
        assert!(
            DEFAULT_YAZI_KEYMAP.contains(r#"{ on = [ "c", "p" ], run = "copy path""#),
            "default yazi keymap should expose path copy"
        );
        assert!(
            DEFAULT_YAZI_KEYMAP.contains(r#"{ on = [ "c", "d" ], run = "copy dirname""#),
            "default yazi keymap should expose directory copy"
        );
        assert!(
            DEFAULT_YAZI_KEYMAP.contains(r#"{ on = [ "c", "f" ], run = "copy filename""#),
            "default yazi keymap should expose filename copy"
        );
        assert!(
            DEFAULT_YAZI_KEYMAP.contains(r#"{ on = [ "c", "n" ], run = "copy name_without_ext""#),
            "default yazi keymap should expose stem copy"
        );
    }

    #[test]
    fn yazi_vim_openers_harden_terminal_handoff() {
        assert!(
            DEFAULT_YAZI_CONFIG.contains(r#"vim --cmd "set t_u7=" --cmd "set t_RV=" "$@""#),
            "in-place Yazi opener should use Vim terminal-query hardening"
        );
        assert!(
            DEFAULT_VIMRC.contains("set t_u7=")
                && DEFAULT_VIMRC.contains("set t_RV=")
                && DEFAULT_VIMRC.contains("autocmd VimEnter * redraw!")
                && DEFAULT_VIMRC.contains("set nowrap")
                && DEFAULT_VIMRC.contains("set sidescroll=1"),
            "default vimrc should harden redraw during Yazi/Vim handoff"
        );
    }

    #[test]
    fn claude_keybindings_use_bindings_object() {
        let value: serde_json::Value = serde_json::from_str(DEFAULT_CLAUDE_KEYBINDINGS).unwrap();
        let bindings = value["bindings"].as_array().unwrap();
        assert_eq!(bindings[0]["context"], "Chat");
        assert!(bindings[0]["bindings"]["ctrl+g"].is_null());
    }

    #[test]
    fn rich_preview_entries_only_enabled_with_preview_enhanced_addon() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let mut config = make_config(false, root);

        let without = generate_yazi_config(&config);
        assert!(
            !without.contains("rich-preview"),
            "rich previewers should not be enabled without preview-enhanced"
        );

        config
            .addons
            .addons
            .insert("preview-enhanced".to_string(), AddonToolsSection::default());
        let with = generate_yazi_config(&config);
        assert!(
            with.contains(r#"{ url = "*.md",       run = "rich-preview" }"#),
            "rich previewers should be enabled when preview-enhanced is configured"
        );
    }

    #[test]
    fn data_preview_entries_only_enabled_with_data_preview_addon() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let mut config = make_config(false, root);

        let without = generate_yazi_config(&config);
        assert!(
            !without.contains("sqlite-preview"),
            "SQLite previewers should not be enabled without data-preview"
        );
        assert!(
            !without.contains("tabular-preview"),
            "tabular previewers should not be enabled without data-preview"
        );

        config
            .addons
            .addons
            .insert("data-preview".to_string(), AddonToolsSection::default());
        let with = generate_yazi_config(&config);
        assert!(
            with.contains(r#"{ url = "*.sqlite",  run = "sqlite-preview" }"#),
            "SQLite previewers should be enabled when data-preview is configured"
        );
        assert!(
            with.contains(r#"{ url = "*.xlsx",    run = "tabular-preview" }"#),
            "spreadsheet previewers should be enabled when data-preview is configured"
        );
    }

    #[test]
    fn omp_setup_only_enabled_with_yazi_omp_addon() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let mut config = make_config(false, root);

        let without = generate_yazi_init(&config);
        assert!(
            !without.contains(r#"require("omp"):setup"#),
            "omp setup should be absent by default"
        );

        config
            .addons
            .addons
            .insert("yazi-omp".to_string(), AddonToolsSection::default());
        let with = generate_yazi_init(&config);
        assert!(
            with.contains(r#"require("omp"):setup"#),
            "omp setup should be present when the addon is configured"
        );
    }

    #[test]
    fn migrate_yazi_section_rewrites_manager_to_mgr() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("yazi.toml");
        fs::write(
            &path,
            "[manager]\nratio = [1, 3, 4]\nsort_by = \"natural\"\n",
        )
        .unwrap();

        let changed = migrate_yazi_section(&path).unwrap();
        assert!(changed, "should report change");

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.starts_with("[mgr]\n"), "should rename to [mgr]");
        assert!(
            content.contains("ratio = [1, 3, 4]"),
            "should preserve user values"
        );
        assert!(
            content.contains("sort_by = \"natural\""),
            "should preserve other lines"
        );
        assert!(
            !content.contains("[manager]"),
            "should not contain old section name"
        );
    }

    #[test]
    fn migrate_yazi_section_idempotent_on_mgr() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("yazi.toml");
        let original = "[mgr]\nratio = [1, 3, 4]\n";
        fs::write(&path, original).unwrap();

        let changed = migrate_yazi_section(&path).unwrap();
        assert!(
            !changed,
            "no change should be reported for already-migrated file"
        );

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, original, "file content must be unchanged");
    }

    #[test]
    fn migrate_yazi_section_missing_file_is_noop() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("missing.toml");
        let changed = migrate_yazi_section(&path).unwrap();
        assert!(!changed);
    }

    #[test]
    fn migrate_yazi_section_preserves_user_customization() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("yazi.toml");
        // User has changed ratio and added a comment — both must survive.
        let custom = "# my customizations\n[manager]\nratio = [2, 4, 1]\n# end\n";
        fs::write(&path, custom).unwrap();

        let changed = migrate_yazi_section(&path).unwrap();
        assert!(changed);

        let content = fs::read_to_string(&path).unwrap();
        assert!(
            content.contains("ratio = [2, 4, 1]"),
            "user ratio must be preserved"
        );
        assert!(
            content.contains("# my customizations"),
            "user comments must be preserved"
        );
        assert!(
            content.contains("# end"),
            "trailing comment must be preserved"
        );
        assert!(content.contains("[mgr]"), "section must be renamed");
        assert!(!content.contains("[manager]"));
    }

    #[test]
    fn migrate_yazi_section_does_not_touch_substring_matches() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("yazi.toml");
        // A line that contains "manager" as part of a value should NOT be touched.
        let content = "[mgr]\ndescription = \"the manager pane\"\n";
        fs::write(&path, content).unwrap();
        let changed = migrate_yazi_section(&path).unwrap();
        assert!(!changed);
        assert_eq!(fs::read_to_string(&path).unwrap(), content);
    }

    #[test]
    fn tmux_config_uses_pinned_managed_plugins_without_default_persistence() {
        let config = crate::config::test_config();
        let conf = tmux_conf(&config);

        assert!(
            conf.contains("set -g allow-passthrough on")
                && conf.contains("set -g default-terminal \"tmux-256color\""),
            "generated tmux config should enable passthrough and tmux-256color defaults for terminal app compatibility:\n{conf}"
        );
        assert!(
            conf.contains("/usr/local/share/aibox/tmux/plugins/tmux-sensible/sensible.tmux")
                && conf.contains(
                    "/usr/local/share/aibox/tmux/plugins/tmux-powerkit/tmux-powerkit.tmux"
                )
                && conf.contains(
                    "/usr/local/share/aibox/tmux/plugins/vim-tmux-navigator/vim-tmux-navigator.tmux"
                )
                && conf.contains("/usr/local/share/aibox/tmux/plugins/tmux-yank/yank.tmux"),
            "aibox-managed tmux plugins should load from preinstalled pinned runtime paths:\n{conf}"
        );
        assert!(
            conf.contains(
                r#"@powerkit_plugins "hostname,external_ip,ssh,uptime,weather,datetime,git,github,kubernetes,terraform,cloud,cloudstatus,cpu,loadavg,memory,swap,disk,gpu,netspeed,ping,aibox""#
            )
                && conf.contains(r#"@powerkit_bar_layout "double""#)
                && conf.contains(r#"@powerkit_status_order "session,plugins""#)
                && conf.contains(r#"@powerkit_status_interval "5""#)
                && conf.contains(r#"@powerkit_transparent "false""#)
                && conf.contains(r#"@powerkit_pane_border_status "top""#)
                && conf.contains(r##"@powerkit_pane_border_format "#{?client_prefix,PREFIX,NORMAL} #{pane_title} #{pane_current_command}""##)
                && conf.contains(r#"@powerkit_line1_right "hostname,external_ip,ssh,uptime,weather,datetime""#)
                && conf.contains(r#"@powerkit_line2_left "git,github,kubernetes,terraform,cloud,cloudstatus""#)
                && conf.contains(
                    r#"@powerkit_line2_right "cpu,loadavg,memory,swap,disk,gpu,netspeed,ping,aibox""#
                )
                && conf.contains(r#"@powerkit_plugin_aibox_metrics "log,oom,proc,ai,mcp,mig""#),
            "generated persistent tmux config should carry bounded powerkit defaults:\n{conf}"
        );
        assert!(
            conf.contains(
                r#"bind-key -N "Show aibox/tmux key bindings" ? display-popup -w 80% -h 75% -E "tmux list-keys -N | less -R""#
            ) && conf.contains(r#"bind-key -N "Select pane left" h select-pane -L"#),
            "generated persistent tmux config should expose native tmux keybinding help:\n{conf}"
        );
        assert!(
            !conf.contains("set -g @plugin 'tmux-plugins/tmux-continuum'")
                && !conf.contains("set -g @plugin 'tmux-plugins/tmux-resurrect'")
                && conf.contains("@continuum-restore 'off'")
                && conf.contains("@continuum-save-interval '0'")
                && conf.contains("@resurrect-capture-pane-contents 'off'"),
            "resurrect/continuum should stay disabled by default until persistence policy is decided:\n{conf}"
        );
        assert!(
            conf.contains("TPM is\n# only a user convenience layer"),
            "TPM should be documented as user convenience, not the managed plugin source:\n{conf}"
        );
    }

    #[test]
    fn tmux_status_layout_uses_image_status_binary() {
        let config = crate::config::test_config();
        let conf = tmux_conf(&config);

        assert!(conf.contains(r#"@powerkit_plugins "hostname,external_ip"#));
        assert!(conf.contains("tmux-powerkit.tmux"));
        assert!(
            !conf.contains("$HOME/.local/bin/aibox-status"),
            "tmux status must use the image-owned Rust status binary"
        );
    }

    #[test]
    fn tmux_plain_status_omits_runtime_segment() {
        let mut config = crate::config::test_config();
        config.customization.tmux.status.mode = TmuxStatusMode::Plain;
        let conf = tmux_conf(&config);

        assert!(conf.contains("set -g status on"));
        assert!(!conf.contains("aibox-status --once"));
        assert!(!conf.contains("@powerkit_plugins"));
        assert!(!conf.contains("tmux-powerkit.tmux"));
    }

    #[test]
    fn tmux_status_right_placeholder_is_replaced_before_status() {
        let config = crate::config::test_config();
        let conf = tmux_conf(&config);

        assert!(
            conf.contains(
                r#"set -g status-right " #(aibox-status --once 2>/dev/null || true) %H:%M ""#
            ),
            "status-right should retain runtime segment, not be truncated by status replacement:\n{conf}"
        );
        assert!(
            !conf.contains("on_RIGHT"),
            "status placeholder replacement order must not produce on_RIGHT artifacts:\n{conf}"
        );
    }

    #[test]
    fn tmux_disabled_status_turns_status_line_off() {
        let mut config = crate::config::test_config();
        config.customization.tmux.status.mode = TmuxStatusMode::Disabled;
        let conf = tmux_conf(&config);

        assert!(conf.contains("set -g status off"));
        assert!(!conf.contains("@powerkit_plugins"));
        assert!(!conf.contains("tmux-powerkit.tmux"));
    }

    #[test]
    #[serial]
    fn seed_root_dir_creates_aider_dir_when_configured() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let mut config = make_config(false, root.clone());
        config.ai.harnesses = vec![AiProvider::Aider];
        seed_root_dir(&config).unwrap();

        assert!(
            root.join(".aider").is_dir(),
            ".aider directory should be created"
        );
        assert!(!root.join(".claude").exists(), ".claude should not exist");
        clear_test_host_root();
    }

    #[test]
    #[serial]
    fn seed_root_dir_creates_gemini_dir_when_configured() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let mut config = make_config(false, root.clone());
        config.ai.harnesses = vec![AiProvider::Gemini];
        seed_root_dir(&config).unwrap();

        assert!(
            root.join(".gemini").is_dir(),
            ".gemini directory should be created"
        );
        clear_test_host_root();
    }
}
