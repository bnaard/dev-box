use anyhow::{Context, Result};
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::config::{AiboxConfig, ZellijStatusMode};
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
set wildmenu wildmode=longest:full,full
set incsearch hlsearch ignorecase smartcase
set backspace=indent,eol,start
set laststatus=2
set ruler showcmd

" Filetype-specific indentation
autocmd FileType yaml,json,kdl,html,css,javascript setlocal tabstop=2 shiftwidth=2
autocmd FileType markdown setlocal wrap linebreak

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
"                          WezTerm, and zellij. Most terminals send
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

/// Default zellij config.kdl content. Theme name and layout are replaced at seed time.
const DEFAULT_ZELLIJ_CONFIG: &str = r#"// aibox zellij configuration
theme "AIBOX_THEME"
default_layout "AIBOX_LAYOUT"
default_shell "bash"
mouse_mode true
copy_on_select true
scroll_buffer_size 10000
rounded_corners true
simplified_ui false
pane_frames true

// Leader: Ctrl+g (press Ctrl+g, release, then press the action key)
// Quick reference:
//   Alt+h/j/k/l          Navigate panes (no leader needed; shown in status bar)
//   Alt+p               Toggle floating panes
//   Alt+[/]             Previous/next tab
//   Alt+1-5             Jump to tab (see top tab bar for names per layout)
//   Ctrl+g → h/j/k/l    Navigate panes (leader variant)
//   Ctrl+g → n/d/r       New pane / split down / split right
//   Ctrl+g → x           Close pane
//   Ctrl+g → f           Toggle fullscreen
//   Ctrl+g → z           Toggle pane frames
//   Ctrl+g → v           Toggle aibox runtime status line
//   Ctrl+g → b           Toggle aibox key-hint line
//   Ctrl+g → p           Toggle floating panes
//   Ctrl+g → t/w         New tab / close tab
//   Ctrl+g → [/]         Previous/next tab
//   Ctrl+g → ,/.         Previous/next stacked pane
//   Ctrl+g → 1-5         Jump to tab
//   Ctrl+g → s           Strider file picker
//   Ctrl+g → u           Scroll mode
//   Ctrl+g → /           Search scrollback
//   Ctrl+g → q           Quit zellij (entire session)
keybinds clear-defaults=true {
    normal {
        bind "Ctrl g" { SwitchToMode "Tmux"; }
    }
    tmux {
        bind "Ctrl g" { SwitchToMode "Normal"; }
        bind "Esc" { SwitchToMode "Normal"; }
        bind "h" "Left"  { MoveFocus "Left"; SwitchToMode "Normal"; }
        bind "j" "Down"  { MoveFocus "Down"; SwitchToMode "Normal"; }
        bind "k" "Up"    { MoveFocus "Up"; SwitchToMode "Normal"; }
        bind "l" "Right" { MoveFocus "Right"; SwitchToMode "Normal"; }
        bind "n"     { NewPane; SwitchToMode "Normal"; }
        bind "d"     { NewPane "Down"; SwitchToMode "Normal"; }
        bind "r"     { NewPane "Right"; SwitchToMode "Normal"; }
        bind "x"     { CloseFocus; SwitchToMode "Normal"; }
        bind "f"     { ToggleFocusFullscreen; SwitchToMode "Normal"; }
        bind "z"     { TogglePaneFrames; SwitchToMode "Normal"; }
        bind "v" {
            MessagePlugin {
                name "aibox_toggle_runtime"
                payload "toggle"
            }
            SwitchToMode "Normal"
        }
        bind "b" {
            MessagePlugin {
                name "aibox_toggle_keys"
                payload "toggle"
            }
            SwitchToMode "Normal"
        }
        bind "e"     { TogglePaneEmbedOrFloating; SwitchToMode "Normal"; }
        bind "p"     { ToggleFloatingPanes; SwitchToMode "Normal"; }
        bind "=" { Resize "Increase"; }
        bind "-" { Resize "Decrease"; }
        bind "R" { SwitchToMode "Resize"; }
        bind "t"     { NewTab; SwitchToMode "Normal"; }
        bind "w"     { CloseTab; SwitchToMode "Normal"; }
        bind "["     { GoToPreviousTab; SwitchToMode "Normal"; }
        bind "]"     { GoToNextTab; SwitchToMode "Normal"; }
        bind ","     { PreviousSwapLayout; SwitchToMode "Normal"; }
        bind "."     { NextSwapLayout; SwitchToMode "Normal"; }
        bind "1"     { GoToTab 1; SwitchToMode "Normal"; }
        bind "2"     { GoToTab 2; SwitchToMode "Normal"; }
        bind "3"     { GoToTab 3; SwitchToMode "Normal"; }
        bind "4"     { GoToTab 4; SwitchToMode "Normal"; }
        bind "5"     { GoToTab 5; SwitchToMode "Normal"; }
        bind "i"     { MoveTab "Left"; SwitchToMode "Normal"; }
        bind "o"     { MoveTab "Right"; SwitchToMode "Normal"; }
        bind "s" {
            LaunchOrFocusPlugin "zellij:strider" {
                floating true
                move_to_focused_tab true
            }
            SwitchToMode "Normal"
        }
        bind "m" {
            LaunchOrFocusPlugin "zellij:session-manager" {
                floating true
                move_to_focused_tab true
            }
            SwitchToMode "Normal"
        }
        bind "u" { SwitchToMode "Scroll"; }
        bind "/" { SwitchToMode "EnterSearch"; SearchInput 0; }
        bind "q" { Quit; }
    }
    scroll {
        bind "Ctrl g" { SwitchToMode "Normal"; }
        bind "Ctrl c" "Esc" "q" { SwitchToMode "Normal"; }
        bind "j" "Down"  { ScrollDown; }
        bind "k" "Up"    { ScrollUp; }
        bind "d"         { HalfPageScrollDown; }
        bind "u"         { HalfPageScrollUp; }
        bind "f" "PageDown" { PageScrollDown; }
        bind "b" "PageUp"   { PageScrollUp; }
        bind "g"         { ScrollToTop; }
        bind "G"         { ScrollToBottom; }
        bind "/"         { SwitchToMode "EnterSearch"; SearchInput 0; }
    }
    search {
        bind "Ctrl g" { SwitchToMode "Normal"; }
        bind "Ctrl c" "Esc" { SwitchToMode "Normal"; }
        bind "n"     { Search "down"; }
        bind "N"     { Search "up"; }
        bind "c"     { SearchToggleOption "CaseSensitivity"; }
        bind "w"     { SearchToggleOption "Wrap"; }
        bind "o"     { SearchToggleOption "WholeWord"; }
    }
    entersearch {
        bind "Ctrl c" "Esc" { SwitchToMode "Normal"; }
        bind "Enter" { SwitchToMode "Search"; }
    }
    resize {
        bind "Ctrl g" { SwitchToMode "Normal"; }
        bind "Ctrl c" "Esc" { SwitchToMode "Normal"; }
        bind "h" "Left"  { Resize "Increase Left"; }
        bind "j" "Down"  { Resize "Increase Down"; }
        bind "k" "Up"    { Resize "Increase Up"; }
        bind "l" "Right" { Resize "Increase Right"; }
        bind "H"         { Resize "Decrease Left"; }
        bind "J"         { Resize "Decrease Down"; }
        bind "K"         { Resize "Decrease Up"; }
        bind "L"         { Resize "Decrease Right"; }
        bind "=" "+"     { Resize "Increase"; }
        bind "-"         { Resize "Decrease"; }
    }
    // Alt key bindings available in all modes (except locked).
    shared_except "locked" {
        bind "Alt h" { MoveFocus "Left"; }
        bind "Alt j" { MoveFocus "Down"; }
        bind "Alt k" { MoveFocus "Up"; }
        bind "Alt l" { MoveFocus "Right"; }
        bind "Alt p" { ToggleFloatingPanes; }
        bind "Alt [" { GoToPreviousTab; }
        bind "Alt ]" { GoToNextTab; }
        bind "Alt 1" { GoToTab 1; }
        bind "Alt 2" { GoToTab 2; }
        bind "Alt 3" { GoToTab 3; }
        bind "Alt 4" { GoToTab 4; }
        bind "Alt 5" { GoToTab 5; }
    }
}
"#;

/// Generate the KDL snippet for the primary AI provider pane in a tab.
///
/// Interactive agent TUIs need a real terminal surface. Do not stack multiple
/// agent CLIs in one pane: inactive zellij stack children shrink to a title
/// line, which is hostile to full-screen TUIs such as Claude Code.
/// Returns empty string if no providers are configured.
fn ai_pane_kdl(providers: &[crate::config::AiProvider]) -> String {
    providers.first().map_or_else(String::new, |p| {
        let name = p.to_string();
        let cmd = p.binary_name();
        format!(
            "        pane name=\"{name}\" {{\n\
             \x20           command \"{cmd}\"\n\
             \x20           cwd \"/workspace\"\n\
             \x20       }}"
        )
    })
}

fn ai_extra_tabs_kdl(providers: &[crate::config::AiProvider]) -> String {
    if providers.len() <= 1 {
        String::new()
    } else {
        ai_tabs_kdl(&providers[1..])
    }
}

fn ai_tabs_kdl(providers: &[crate::config::AiProvider]) -> String {
    if providers.is_empty() {
        return String::new();
    }

    providers
        .iter()
        .map(|p| {
            let name = p.to_string();
            let cmd = p.binary_name();
            format!(
                "    aibox-tab name=\"{name}\" {{\n\
                 \x20       pane name=\"{name}\" {{\n\
                 \x20           command \"{cmd}\"\n\
                 \x20           cwd \"/workspace\"\n\
                 \x20           start_suspended true\n\
                 \x20       }}\n\
                 \x20   }}"
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

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

fn git_tab_kdl(include_lazygit: bool) -> &'static str {
    if include_lazygit {
        r#"
    aibox-tab name="git" {
        pane name="lazygit" {
            command "lazygit"
            cwd "/workspace"
            start_suspended true
        }
    }"#
    } else {
        ""
    }
}

fn zellij_status_template_kdl(mode: &ZellijStatusMode) -> String {
    match mode {
        ZellijStatusMode::Native => r#"    tab_template name="aibox-tab" {
        children
        pane size=1 borderless=true {
            plugin location="file:/usr/local/share/aibox/zellij/aibox-status.wasm" {
                role "keys"
            }
        }
        pane size=1 borderless=true {
            plugin location="file:/usr/local/share/aibox/zellij/aibox-status.wasm" {
                role "status"
            }
        }
    }"#
        .to_string(),
        ZellijStatusMode::Shell => r#"    tab_template name="aibox-tab" {
        children
        pane size=1 borderless=true {
            plugin location="zellij:status-bar"
        }
        pane size=1 borderless=true {
            command "bash"
            args "-lc" "if [ -x \"$HOME/.local/bin/aibox-status\" ]; then exec \"$HOME/.local/bin/aibox-status\" --watch; else exec aibox-status --watch; fi"
        }
    }"#
        .to_string(),
        ZellijStatusMode::Hidden => zellij_status_hidden_template_kdl(mode),
    }
}

fn zellij_status_hidden_template_kdl(mode: &ZellijStatusMode) -> String {
    match mode {
        ZellijStatusMode::Native => r#"    tab_template name="aibox-tab" {
        children
    }"#
        .to_string(),
        ZellijStatusMode::Shell => r#"    tab_template name="aibox-tab" {
        children
        pane size=1 borderless=true {
            plugin location="zellij:status-bar"
        }
    }"#
        .to_string(),
        ZellijStatusMode::Hidden => r#"    tab_template name="aibox-tab" {
        children
    }
"#
        .to_string(),
    }
}

fn zellij_status_visible_layout(mode: &ZellijStatusMode) -> String {
    format!(
        "layout {{\n{}\n    aibox-tab\n}}\n",
        zellij_status_template_kdl(mode)
    )
}

fn zellij_status_hidden_layout(mode: &ZellijStatusMode) -> String {
    format!(
        "layout {{\n{}\n    aibox-tab\n}}\n",
        zellij_status_hidden_template_kdl(mode)
    )
}

/// Generate the zellij dev layout dynamically based on configured AI providers.
#[cfg(test)]
fn generate_dev_layout(providers: &[crate::config::AiProvider]) -> String {
    generate_dev_layout_with_options(providers, true, &ZellijStatusMode::default())
}

fn generate_dev_layout_with_options(
    providers: &[crate::config::AiProvider],
    include_lazygit: bool,
    status_mode: &ZellijStatusMode,
) -> String {
    let ai_tabs = ai_tabs_kdl(providers);
    let ai_section = if ai_tabs.is_empty() {
        String::new()
    } else {
        format!("\n{}", ai_tabs)
    };
    let git_section = git_tab_kdl(include_lazygit);
    let status_template = zellij_status_template_kdl(status_mode);

    format!(
        r##"layout {{
{status_template}
    aibox-tab name="dev" focus=true {{
        pane split_direction="vertical" {{
            pane size="40%" name="files" focus=true {{
                command "yazi"
                cwd "/workspace"
            }}
            pane size="60%" name="editor" {{
                command "vim-loop"
                cwd "/workspace"
                start_suspended true
            }}
        }}
    }}{ai_section}{git_section}
    aibox-tab name="shell" {{
        pane name="bash" {{
            command "bash"
            cwd "/workspace"
            start_suspended true
        }}
    }}
}}
"##
    )
}

/// Generate the zellij focus layout dynamically based on configured AI providers.
#[cfg(test)]
fn generate_focus_layout(providers: &[crate::config::AiProvider]) -> String {
    generate_focus_layout_with_options(providers, true, &ZellijStatusMode::default())
}

fn generate_focus_layout_with_options(
    providers: &[crate::config::AiProvider],
    include_lazygit: bool,
    status_mode: &ZellijStatusMode,
) -> String {
    let ai_tabs = ai_tabs_kdl(providers);
    let ai_section = if ai_tabs.is_empty() {
        String::new()
    } else {
        format!("\n{}", ai_tabs)
    };
    let git_section = git_tab_kdl(include_lazygit);
    let status_template = zellij_status_template_kdl(status_mode);

    format!(
        r##"layout {{
{status_template}
    aibox-tab name="files" focus=true {{
        pane name="yazi" {{
            command "bash"
            args "-c" "AIBOX_EDITOR_DIR=tab exec yazi"
            cwd "/workspace"
        }}
    }}
    aibox-tab name="editor" {{
        pane name="vim" {{
            command "bash"
            args "-c" "AIBOX_EDITOR_DIR=tab exec vim-loop"
            cwd "/workspace"
            start_suspended true
        }}
    }}{ai_section}{git_section}
    aibox-tab name="shell" {{
        pane name="bash" {{
            command "bash"
            cwd "/workspace"
            start_suspended true
        }}
    }}
}}
"##
    )
}

/// Generate the zellij cowork layout dynamically based on configured AI providers.
#[cfg(test)]
fn generate_cowork_layout(providers: &[crate::config::AiProvider]) -> String {
    generate_cowork_layout_with_options(providers, true, &ZellijStatusMode::default())
}

fn generate_cowork_layout_with_options(
    providers: &[crate::config::AiProvider],
    include_lazygit: bool,
    status_mode: &ZellijStatusMode,
) -> String {
    let ai_pane = ai_pane_kdl(providers);
    let ai_extra_tabs = ai_extra_tabs_kdl(providers);
    let ai_extra_section = if ai_extra_tabs.is_empty() {
        String::new()
    } else {
        format!("\n{}", ai_extra_tabs)
    };
    let git_section = git_tab_kdl(include_lazygit);
    let status_template = zellij_status_template_kdl(status_mode);

    if ai_pane.is_empty() {
        // No AI providers — full-width editor layout
        return r##"layout {
{status_template}
    aibox-tab name="cowork" focus=true {
        pane split_direction="vertical" {
            pane size="40%" name="files" focus=true {
                command "bash"
                args "-c" "AIBOX_EDITOR_DIR=down exec yazi"
                cwd "/workspace"
            }
            pane size="60%" name="editor" {
                command "bash"
                args "-c" "AIBOX_EDITOR_DIR=down exec vim-loop"
                cwd "/workspace"
                start_suspended true
            }
        }
    }
{git_section}
    aibox-tab name="shell" {
        pane name="bash" {
            command "bash"
            cwd "/workspace"
            start_suspended true
        }
    }
}
"##
        .replace("{status_template}", &status_template)
        .replace("{git_section}", git_section);
    }

    format!(
        r##"layout {{
{status_template}
    aibox-tab name="cowork" focus=true {{
        pane split_direction="vertical" {{
            pane size="50%" split_direction="horizontal" {{
                pane size="40%" name="files" focus=true {{
                    command "bash"
                    args "-c" "AIBOX_EDITOR_DIR=down exec yazi"
                    cwd "/workspace"
                }}
                pane size="60%" name="editor" {{
                    command "bash"
                    args "-c" "AIBOX_EDITOR_DIR=down exec vim-loop"
                    cwd "/workspace"
                    start_suspended true
                }}
            }}
            pane size="50%" {{
{ai_pane}
            }}
        }}
    }}
{ai_extra_section}
{git_section}
    aibox-tab name="shell" {{
        pane name="bash" {{
            command "bash"
            cwd "/workspace"
            start_suspended true
        }}
    }}
}}
"##
    )
}

/// Generate the zellij cowork-swap layout dynamically based on configured AI providers.
///
/// cowork-swap is a re-arrangement of `cowork` for users who prefer the editor on
/// the right (the bigger pane). The outer split is 40/60 instead of cowork's
/// 50/50, and the editor and AI panes swap roles:
///
///   Tab 1 ("cowork-swap"):
///     ┌──────────────────┬────────────────────────────────────────┐
///     │  yazi (top, 40%) │                                        │
///     │                  │                                        │
///     ├──────────────────┤  vim editor (60%)                      │
///     │  AI agent (60%)  │                                        │
///     │                  │                                        │
///     └──────────────────┴────────────────────────────────────────┘
///   Tab 2 ("git"):    fullscreen lazygit
///   Tab 3 ("shell"):  fullscreen bash
///
/// When no AI providers are configured, the cowork-swap tab degenerates to
/// the same yazi-left + vim-right shape as `dev` (with the cowork-swap tab
/// name preserved).
///
/// AIBOX_EDITOR_DIR is "right" (the default) on yazi/vim because vim is
/// to the right of yazi geometrically — opening a file from yazi via `e`
/// moves focus right.
#[cfg(test)]
fn generate_cowork_swap_layout(providers: &[crate::config::AiProvider]) -> String {
    generate_cowork_swap_layout_with_options(providers, true, &ZellijStatusMode::default())
}

fn generate_cowork_swap_layout_with_options(
    providers: &[crate::config::AiProvider],
    include_lazygit: bool,
    status_mode: &ZellijStatusMode,
) -> String {
    let ai_pane = ai_pane_kdl(providers);
    let ai_extra_tabs = ai_extra_tabs_kdl(providers);
    let ai_extra_section = if ai_extra_tabs.is_empty() {
        String::new()
    } else {
        format!("\n{}", ai_extra_tabs)
    };
    let git_section = git_tab_kdl(include_lazygit);
    let status_template = zellij_status_template_kdl(status_mode);

    if ai_pane.is_empty() {
        // No AI providers — fall back to a simple yazi-left + vim-right shape
        // (same as dev, with the cowork-swap tab name preserved).
        return r##"layout {
{status_template}
    aibox-tab name="cowork-swap" focus=true {
        pane split_direction="vertical" {
            pane size="40%" name="files" focus=true {
                command "yazi"
                cwd "/workspace"
            }
            pane size="60%" name="editor" {
                command "vim-loop"
                cwd "/workspace"
                start_suspended true
            }
        }
    }
{git_section}
    aibox-tab name="shell" {
        pane name="bash" {
            command "bash"
            cwd "/workspace"
            start_suspended true
        }
    }
}
"##
        .replace("{status_template}", &status_template)
        .replace("{git_section}", git_section);
    }

    format!(
        r##"layout {{
{status_template}
    aibox-tab name="cowork-swap" focus=true {{
        pane split_direction="vertical" {{
            pane size="40%" split_direction="horizontal" {{
                pane size="40%" name="files" focus=true {{
                    command "yazi"
                    cwd "/workspace"
                }}
                pane size="60%" {{
{ai_pane}
                }}
            }}
            pane size="60%" name="editor" {{
                command "vim-loop"
                cwd "/workspace"
                start_suspended true
            }}
        }}
    }}
{ai_extra_section}
{git_section}
    aibox-tab name="shell" {{
        pane name="bash" {{
            command "bash"
            cwd "/workspace"
            start_suspended true
        }}
    }}
}}
"##
    )
}

/// Generate the zellij ai layout dynamically based on configured AI providers.
///
/// AI layout: yazi-first, AI-first.
///   Tab 1 ("ai"):     left 50% yazi, right 50% AI agent pane (vertical split, no editor)
///   Tab 2 ("editor"): fullscreen vim
///   Tab 3 ("git"):    fullscreen lazygit
///   Tab 4 ("shell"):  fullscreen bash
///
/// When no AI providers are configured, the ai tab is fullscreen yazi (the
/// editor still lives in tab 2; opening files via `e` from yazi works as
/// usual).
#[cfg(test)]
fn generate_ai_layout(providers: &[crate::config::AiProvider]) -> String {
    generate_ai_layout_with_options(providers, true, &ZellijStatusMode::default())
}

fn generate_ai_layout_with_options(
    providers: &[crate::config::AiProvider],
    include_lazygit: bool,
    status_mode: &ZellijStatusMode,
) -> String {
    let ai_pane = ai_pane_kdl(providers);
    let ai_extra_tabs = ai_extra_tabs_kdl(providers);
    let ai_extra_section = if ai_extra_tabs.is_empty() {
        String::new()
    } else {
        format!("\n{}", ai_extra_tabs)
    };
    let git_section = git_tab_kdl(include_lazygit);
    let status_template = zellij_status_template_kdl(status_mode);

    if ai_pane.is_empty() {
        return r##"layout {
{status_template}
    aibox-tab name="ai" focus=true {
        pane name="files" {
            command "bash"
            args "-c" "AIBOX_EDITOR_DIR=tab exec yazi"
            cwd "/workspace"
        }
    }
    aibox-tab name="editor" {
        pane name="vim" {
            command "bash"
            args "-c" "AIBOX_EDITOR_DIR=tab exec vim-loop"
            cwd "/workspace"
            start_suspended true
        }
    }
{git_section}
    aibox-tab name="shell" {
        pane name="bash" {
            command "bash"
            cwd "/workspace"
            start_suspended true
        }
    }
}
"##
        .replace("{status_template}", &status_template)
        .replace("{git_section}", git_section);
    }

    format!(
        r##"layout {{
{status_template}
    aibox-tab name="ai" focus=true {{
        pane split_direction="vertical" {{
            pane size="50%" name="files" focus=true {{
                command "bash"
                args "-c" "AIBOX_EDITOR_DIR=tab exec yazi"
                cwd "/workspace"
            }}
            pane size="50%" {{
{ai_pane}
            }}
        }}
    }}
{ai_extra_section}
    aibox-tab name="editor" {{
        pane name="vim" {{
            command "bash"
            args "-c" "AIBOX_EDITOR_DIR=tab exec vim-loop"
            cwd "/workspace"
            start_suspended true
        }}
    }}{git_section}
    aibox-tab name="shell" {{
        pane name="bash" {{
            command "bash"
            cwd "/workspace"
            start_suspended true
        }}
    }}
}}
"##
    )
}

/// Generate the zellij browse layout dynamically based on configured AI providers.
///
/// Browse layout: yazi-focused with large preview.
///   Tab 1 ("browse"): top 60% yazi, bottom 40% AI agent pane
///   Tab 2 ("editor"): fullscreen vim
///   Tab 3 ("git"):    fullscreen lazygit
///   Tab 4 ("shell"):  fullscreen bash
///
/// When no AI providers are configured, the browse tab is fullscreen yazi.
#[cfg(test)]
fn generate_browse_layout(providers: &[crate::config::AiProvider]) -> String {
    generate_browse_layout_with_options(providers, true, &ZellijStatusMode::default())
}

fn generate_browse_layout_with_options(
    providers: &[crate::config::AiProvider],
    include_lazygit: bool,
    status_mode: &ZellijStatusMode,
) -> String {
    let ai_pane = ai_pane_kdl(providers);
    let ai_extra_tabs = ai_extra_tabs_kdl(providers);
    let ai_extra_section = if ai_extra_tabs.is_empty() {
        String::new()
    } else {
        format!("\n{}", ai_extra_tabs)
    };
    let git_section = git_tab_kdl(include_lazygit);
    let status_template = zellij_status_template_kdl(status_mode);

    if ai_pane.is_empty() {
        return r##"layout {
{status_template}
    aibox-tab name="browse" focus=true {
        pane name="files" focus=true {
            command "bash"
            args "-c" "AIBOX_EDITOR_DIR=tab exec yazi"
            cwd "/workspace"
        }
    }
    aibox-tab name="editor" {
        pane name="vim" {
            command "bash"
            args "-c" "AIBOX_EDITOR_DIR=tab exec vim-loop"
            cwd "/workspace"
            start_suspended true
        }
    }
{git_section}
    aibox-tab name="shell" {
        pane name="bash" {
            command "bash"
            cwd "/workspace"
            start_suspended true
        }
    }
}
"##
        .replace("{status_template}", &status_template)
        .replace("{git_section}", git_section);
    }

    format!(
        r##"layout {{
{status_template}
    aibox-tab name="browse" focus=true {{
        pane split_direction="horizontal" {{
            pane size="60%" name="files" focus=true {{
                command "bash"
                args "-c" "AIBOX_EDITOR_DIR=tab exec yazi"
                cwd "/workspace"
            }}
            pane size="40%" {{
{ai_pane}
            }}
        }}
    }}
{ai_extra_section}
    aibox-tab name="editor" {{
        pane name="vim" {{
            command "bash"
            args "-c" "AIBOX_EDITOR_DIR=tab exec vim-loop"
            cwd "/workspace"
            start_suspended true
        }}
    }}{git_section}
    aibox-tab name="shell" {{
        pane name="bash" {{
            command "bash"
            cwd "/workspace"
            start_suspended true
        }}
    }}
}}
"##
    )
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
    { id = "git", url = "*",  run = "git" },
    { id = "git", url = "*/", run = "git" },
    { id = "status-git", url = "*", run = "status-git" },
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
    { run = '${EDITOR:-vim} "$@"', desc = "Edit in-place", block = true },
]
edit-pane = [
    { run = 'open-in-editor "$1"', desc = "Open in vim pane", block = false },
]

[open]
rules = [
    { mime = "text/*", use = "edit" },
    { name = "*", use = "edit" },
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
th.git = th.git or {}
th.git.modified_sign = "M"
th.git.added_sign = "A"
th.git.deleted_sign = "D"
th.git.updated_sign = "U"
th.git.untracked_sign = "?"
th.git.ignored_sign = "I"

require("git"):setup {}

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
/// aibox-status helper — compact cgroup/procfs status line for Zellij layouts.
const DEFAULT_AIBOX_STATUS_SH: &str =
    include_str!("../../images/base-debian/config/bin/aibox-status.sh");
/// aibox-status-toggle helper — toggle the runtime status pane in Zellij.
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
  ZELLIJ (leader: Ctrl+g)    YAZI (file manager)
  Alt+h/j/k/l     Move pane  h/j/k/l  Navigate
  Alt+p            Float pane Enter    Open in vim
  Alt+[/]          Prev/next  g s      Git summary
  Alt+1-5          Jump tab   g c      Git changes
                             w s      Size selection
                             w h      Horizontal preview
                             w p      Watch PDF
                             c p/d/f  Copy path/dir/name
  Ctrl+g h/j/k/l  Move pane  g r      Refresh git
  Ctrl+g f         Fullscreen Space    Select
  Ctrl+g x         Close pane
  Ctrl+g n/d/r     New pane
  Ctrl+g t/w       Tab +/-
  Ctrl+g p         Toggle float panes
  Ctrl+g v         Status line
  Ctrl+g R         Resize mode (h/j/k/l)
  Ctrl+g s         Strider
  Ctrl+g u         Scroll
  Ctrl+g /         Search
  Ctrl+g q         QUIT

  LAYOUTS: aibox up --layout dev|focus|cowork|cowork-swap|browse|ai
  TABS: Alt+1 dev  Alt+2 git  Alt+3 shell
"#;

/// Default .asoundrc for PulseAudio over TCP.
const DEFAULT_ASOUNDRC: &str = r#"pcm.!default {
    type pulse
}
ctl.!default {
    type pulse
}
"#;

/// Claude Code keybindings — disables Ctrl+g (reserved for zellij leader key).
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
        root.join(".vim").join("undo"),
        root.join(".config").join("zellij").join("themes"),
        root.join(".config").join("zellij").join("layouts"),
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
    let theme = &config.customization.resolved_theme();
    let providers = &config.ai.harnesses;
    let include_lazygit = include_lazygit_tab(config);
    let status_mode = &config.customization.zellij_status.mode;
    let mut files = vec![
        (
            std::path::PathBuf::from(".vim/vimrc"),
            DEFAULT_VIMRC
                .replace(
                    "AIBOX_VIM_COLORSCHEME",
                    crate::themes::vim_colorscheme(theme),
                )
                .replace("AIBOX_VIM_BG", crate::themes::vim_background(theme)),
        ),
        (
            std::path::PathBuf::from(".config/git/config"),
            DEFAULT_GITCONFIG.to_string(),
        ),
        (
            std::path::PathBuf::from(".config/zellij/config.kdl"),
            DEFAULT_ZELLIJ_CONFIG
                .replace("AIBOX_THEME", &theme.to_string())
                .replace("AIBOX_LAYOUT", &config.customization.layout.to_string()),
        ),
        (
            std::path::PathBuf::from(format!(".config/zellij/themes/{}.kdl", theme)),
            crate::themes::zellij_theme(theme).to_string(),
        ),
        (
            std::path::PathBuf::from(".config/zellij/layouts/dev.kdl"),
            generate_dev_layout_with_options(providers, include_lazygit, status_mode),
        ),
        (
            std::path::PathBuf::from(".config/zellij/layouts/focus.kdl"),
            generate_focus_layout_with_options(providers, include_lazygit, status_mode),
        ),
        (
            std::path::PathBuf::from(".config/zellij/layouts/cowork.kdl"),
            generate_cowork_layout_with_options(providers, include_lazygit, status_mode),
        ),
        (
            std::path::PathBuf::from(".config/zellij/layouts/browse.kdl"),
            generate_browse_layout_with_options(providers, include_lazygit, status_mode),
        ),
        (
            std::path::PathBuf::from(".config/zellij/layouts/ai.kdl"),
            generate_ai_layout_with_options(providers, include_lazygit, status_mode),
        ),
        (
            std::path::PathBuf::from(".config/zellij/layouts/cowork-swap.kdl"),
            generate_cowork_swap_layout_with_options(providers, include_lazygit, status_mode),
        ),
        (
            std::path::PathBuf::from(".config/zellij/layouts/aibox-status-visible.kdl"),
            zellij_status_visible_layout(status_mode),
        ),
        (
            std::path::PathBuf::from(".config/zellij/layouts/aibox-status-hidden.kdl"),
            zellij_status_hidden_layout(status_mode),
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
            crate::themes::yazi_theme(theme).to_string(),
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
            crate::themes::starship_config(&config.customization.prompt, theme),
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
            std::path::PathBuf::from(".local/bin/aibox-status"),
            DEFAULT_AIBOX_STATUS_SH.to_string(),
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
            crate::themes::lazygit_theme(theme).to_string(),
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

    Ok(updated)
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
            || rel_path == Path::new(".local/bin/aibox-status")
            || rel_path == Path::new(".local/bin/aibox-status-toggle")
        {
            ensure_executable(&path)?;
        }
    }

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
    let status_mode = &config.customization.zellij_status.mode;
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

    // Zellij config — theme name and default layout
    let zellij_config = DEFAULT_ZELLIJ_CONFIG
        .replace("AIBOX_THEME", &theme.to_string())
        .replace("AIBOX_LAYOUT", &config.customization.layout.to_string());
    if force_seed_file(
        &root.join(".config").join("zellij").join("config.kdl"),
        &zellij_config,
    )? {
        updated.push(".config/zellij/config.kdl".to_string());
    }

    // Zellij theme file
    let theme_filename = format!("{}.kdl", &theme.to_string());
    if force_seed_file(
        &root
            .join(".config")
            .join("zellij")
            .join("themes")
            .join(&theme_filename),
        crate::themes::zellij_theme(theme),
    )? {
        updated.push(format!(".config/zellij/themes/{}", theme_filename));
    }

    // Zellij layouts — depend on AI providers
    if force_seed_file(
        &root
            .join(".config")
            .join("zellij")
            .join("layouts")
            .join("dev.kdl"),
        &generate_dev_layout_with_options(providers, include_lazygit, status_mode),
    )? {
        updated.push(".config/zellij/layouts/dev.kdl".to_string());
    }
    if force_seed_file(
        &root
            .join(".config")
            .join("zellij")
            .join("layouts")
            .join("focus.kdl"),
        &generate_focus_layout_with_options(providers, include_lazygit, status_mode),
    )? {
        updated.push(".config/zellij/layouts/focus.kdl".to_string());
    }
    if force_seed_file(
        &root
            .join(".config")
            .join("zellij")
            .join("layouts")
            .join("cowork.kdl"),
        &generate_cowork_layout_with_options(providers, include_lazygit, status_mode),
    )? {
        updated.push(".config/zellij/layouts/cowork.kdl".to_string());
    }
    if force_seed_file(
        &root
            .join(".config")
            .join("zellij")
            .join("layouts")
            .join("browse.kdl"),
        &generate_browse_layout_with_options(providers, include_lazygit, status_mode),
    )? {
        updated.push(".config/zellij/layouts/browse.kdl".to_string());
    }
    if force_seed_file(
        &root
            .join(".config")
            .join("zellij")
            .join("layouts")
            .join("ai.kdl"),
        &generate_ai_layout_with_options(providers, include_lazygit, status_mode),
    )? {
        updated.push(".config/zellij/layouts/ai.kdl".to_string());
    }
    if force_seed_file(
        &root
            .join(".config")
            .join("zellij")
            .join("layouts")
            .join("cowork-swap.kdl"),
        &generate_cowork_swap_layout_with_options(providers, include_lazygit, status_mode),
    )? {
        updated.push(".config/zellij/layouts/cowork-swap.kdl".to_string());
    }
    if force_seed_file(
        &root
            .join(".config")
            .join("zellij")
            .join("layouts")
            .join("aibox-status-visible.kdl"),
        &zellij_status_visible_layout(status_mode),
    )? {
        updated.push(".config/zellij/layouts/aibox-status-visible.kdl".to_string());
    }
    if force_seed_file(
        &root
            .join(".config")
            .join("zellij")
            .join("layouts")
            .join("aibox-status-hidden.kdl"),
        &zellij_status_hidden_layout(status_mode),
    )? {
        updated.push(".config/zellij/layouts/aibox-status-hidden.kdl".to_string());
    }

    if force_seed_file(
        &root.join(".local").join("bin").join("open-in-editor"),
        DEFAULT_OPEN_IN_EDITOR_SH,
    )? {
        ensure_executable(&root.join(".local").join("bin").join("open-in-editor"))?;
        updated.push(".local/bin/open-in-editor".to_string());
    }

    if force_seed_file(
        &root.join(".local").join("bin").join("aibox-status"),
        DEFAULT_AIBOX_STATUS_SH,
    )? {
        ensure_executable(&root.join(".local").join("bin").join("aibox-status"))?;
        updated.push(".local/bin/aibox-status".to_string());
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
            crate::themes::lazygit_theme(theme),
        )?
    {
        updated.push(".config/lazygit/config.yml".to_string());
    }
    updated.extend(cleanup_disabled_runtime_files(config)?);

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

    // Claude Code keybindings — disable Ctrl+g (reserved for zellij leader key).
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
        ".local/bin/aibox-status",
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
        unsafe {
            std::env::set_var("AIBOX_HOST_ROOT", root_dir.to_str().unwrap());
        }
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

    #[test]
    #[serial]
    fn seed_root_dir_creates_directories() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let config = make_config(false, root.clone());
        seed_root_dir(&config).unwrap();

        assert!(root.join(".ssh").is_dir());
        assert!(root.join(".vim").join("undo").is_dir());
        assert!(root.join(".config").join("zellij").join("themes").is_dir());
        assert!(root.join(".config").join("zellij").join("layouts").is_dir());
        assert!(root.join(".config").join("yazi").is_dir());
        assert!(root.join(".config").join("git").is_dir());
        assert!(root.join(".claude").is_dir());

        unsafe {
            std::env::remove_var("AIBOX_HOST_ROOT");
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

        unsafe {
            std::env::remove_var("AIBOX_HOST_ROOT");
        }
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

        unsafe {
            std::env::remove_var("AIBOX_HOST_ROOT");
        }
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

        unsafe {
            std::env::remove_var("AIBOX_HOST_ROOT");
        }
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
        assert!(
            root.join(".config")
                .join("zellij")
                .join("config.kdl")
                .exists()
        );
        assert!(
            root.join(".config")
                .join("zellij")
                .join("themes")
                .join("gruvbox-dark.kdl")
                .exists()
        );
        assert!(
            root.join(".config")
                .join("zellij")
                .join("layouts")
                .join("dev.kdl")
                .exists()
        );
        assert!(
            root.join(".config")
                .join("zellij")
                .join("layouts")
                .join("focus.kdl")
                .exists()
        );
        assert!(
            root.join(".config")
                .join("zellij")
                .join("layouts")
                .join("cowork.kdl")
                .exists()
        );
        assert!(
            root.join(".config")
                .join("zellij")
                .join("layouts")
                .join("browse.kdl")
                .exists()
        );
        assert!(
            root.join(".config")
                .join("zellij")
                .join("layouts")
                .join("ai.kdl")
                .exists()
        );
        assert!(
            root.join(".config")
                .join("zellij")
                .join("layouts")
                .join("cowork-swap.kdl")
                .exists()
        );
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
                .join("aibox-status")
                .exists()
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
        let open_in_editor =
            fs::read_to_string(root.join(".local").join("bin").join("open-in-editor")).unwrap();
        assert!(
            open_in_editor.contains("zellij action go-to-tab-name \"editor\""),
            "open-in-editor should target the dedicated editor tab when the layout declares AIBOX_EDITOR_DIR=tab"
        );
        assert!(
            open_in_editor.contains("zellij action move-focus right"),
            "open-in-editor should support same-tab editor panes"
        );
        assert!(
            open_in_editor.contains(":edit ${vim_file}"),
            "open-in-editor should open the selected file in the focused Vim pane"
        );
        assert!(
            !open_in_editor.contains("edit --in-place"),
            "open-in-editor must not replace the Yazi pane with an in-place editor"
        );
        #[cfg(unix)]
        assert_ne!(
            fs::metadata(root.join(".local").join("bin").join("aibox-status"))
                .unwrap()
                .permissions()
                .mode()
                & 0o111,
            0,
            "aibox-status should be executable"
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

        unsafe {
            std::env::remove_var("AIBOX_HOST_ROOT");
        }
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
                path.starts_with(".config/zellij/layouts")
                    && path.extension().is_some_and(|ext| ext == "kdl")
            })
            .collect();

        assert!(
            generated_layouts.len() >= 6,
            "expected all managed Zellij layouts to be generated"
        );
        for (path, body) in generated_layouts {
            assert!(
                !body.contains("lazygit"),
                "disabled lazygit must not appear in generated layout {}",
                path.display()
            );
            assert!(
                !body.contains("aibox-tab name=\"git\""),
                "disabled lazygit must omit the git tab in generated layout {}",
                path.display()
            );
        }
        assert!(
            !files
                .iter()
                .any(|(path, _)| path == &std::path::PathBuf::from(".config/lazygit/config.yml")),
            "disabled lazygit must not generate managed lazygit config"
        );

        unsafe {
            std::env::remove_var("AIBOX_HOST_ROOT");
        }
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
            root.join(".config")
                .join("zellij")
                .join("themes")
                .join("catppuccin-latte.kdl")
                .exists(),
            "light mode should render the resolved light theme"
        );
        let vimrc = fs::read_to_string(root.join(".vim").join("vimrc")).unwrap();
        assert!(vimrc.contains("colorscheme catppuccin_latte"));
        assert!(vimrc.contains("set background=light"));

        unsafe {
            std::env::remove_var("AIBOX_HOST_ROOT");
        }
    }

    #[test]
    #[serial]
    #[cfg(unix)]
    fn sync_theme_files_restores_managed_helper_executability() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let config = make_config(false, root.clone());
        seed_root_dir(&config).unwrap();

        let status_path = root.join(".local").join("bin").join("aibox-status");
        let mut permissions = fs::metadata(&status_path).unwrap().permissions();
        permissions.set_mode(0o644);
        fs::set_permissions(&status_path, permissions).unwrap();

        let updated = sync_theme_files(&config).unwrap();

        assert!(
            updated
                .iter()
                .any(|path| path == ".local/bin/aibox-status (chmod +x)")
        );
        assert_ne!(
            fs::metadata(&status_path).unwrap().permissions().mode() & 0o111,
            0,
            "aibox-status should be executable after apply-time sync"
        );

        unsafe {
            std::env::remove_var("AIBOX_HOST_ROOT");
        }
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

        unsafe {
            std::env::remove_var("AIBOX_HOST_ROOT");
        }
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

        unsafe {
            std::env::remove_var("AIBOX_HOST_ROOT");
        }
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

        unsafe {
            std::env::remove_var("AIBOX_HOST_ROOT");
        }
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

    fn occurrences(haystack: &str, needle: &str) -> usize {
        haystack.match_indices(needle).count()
    }

    #[test]
    fn dev_layout_claude_only() {
        let providers = vec![AiProvider::Claude];
        let layout = generate_dev_layout(&providers);
        assert!(
            layout.contains("aibox-tab name=\"claude\""),
            "should have claude tab"
        );
        assert!(
            layout.contains("command \"claude\""),
            "should have claude command"
        );
        assert!(!layout.contains("aider"), "should not have aider");
    }

    #[test]
    fn dev_layout_aider_only() {
        let providers = vec![AiProvider::Aider];
        let layout = generate_dev_layout(&providers);
        assert!(
            layout.contains("aibox-tab name=\"aider\""),
            "should have aider tab"
        );
        assert!(
            layout.contains("command \"aider\""),
            "should have aider command"
        );
        assert!(!layout.contains("claude"), "should not have claude");
    }

    #[test]
    fn dev_layout_multiple_providers() {
        let providers = vec![AiProvider::Claude, AiProvider::Aider];
        let layout = generate_dev_layout(&providers);
        assert!(
            layout.contains("aibox-tab name=\"claude\""),
            "should have claude tab"
        );
        assert!(
            layout.contains("aibox-tab name=\"aider\""),
            "should have aider tab"
        );
        assert!(
            !layout.contains("stacked"),
            "multiple providers should use separate tabs"
        );
        assert!(
            occurrences(&layout, "start_suspended true") >= 5,
            "editor, git, shell, and AI tabs should start suspended"
        );
    }

    #[test]
    fn dev_layout_no_providers() {
        let providers: Vec<AiProvider> = vec![];
        let layout = generate_dev_layout(&providers);
        assert!(!layout.contains("claude"), "should not have claude");
        assert!(!layout.contains("aider"), "should not have aider");
        assert!(!layout.contains("gemini"), "should not have gemini");
        assert!(
            layout.contains("aibox-tab name=\"dev\""),
            "should still have dev tab"
        );
        assert!(
            layout.contains("aibox-tab name=\"git\""),
            "should still have git tab"
        );
    }

    #[test]
    fn focus_layout_gemini() {
        let providers = vec![AiProvider::Gemini];
        let layout = generate_focus_layout(&providers);
        assert!(
            layout.contains("aibox-tab name=\"gemini\""),
            "should have gemini tab"
        );
        assert!(
            layout.contains("command \"gemini\""),
            "should have gemini command"
        );
    }

    #[test]
    fn cowork_layout_single_provider() {
        let providers = vec![AiProvider::Claude];
        let layout = generate_cowork_layout(&providers);
        assert!(
            layout.contains("command \"claude\""),
            "should have claude pane"
        );
        assert!(
            !layout.contains("stacked"),
            "single provider should not be stacked"
        );
    }

    #[test]
    fn cowork_layout_multiple_providers_use_tabs() {
        let providers = vec![AiProvider::Claude, AiProvider::Aider];
        let layout = generate_cowork_layout(&providers);
        assert!(
            !layout.contains("stacked"),
            "multiple providers should use separate tabs"
        );
        assert!(layout.contains("command \"claude\""), "should have claude");
        assert!(layout.contains("command \"aider\""), "should have aider");
        assert!(
            layout.contains("aibox-tab name=\"aider\""),
            "secondary provider should get its own tab"
        );
    }

    #[test]
    fn cowork_layout_no_providers() {
        let providers: Vec<AiProvider> = vec![];
        let layout = generate_cowork_layout(&providers);
        assert!(!layout.contains("claude"), "should not have claude");
        assert!(
            layout.contains("aibox-tab name=\"cowork\""),
            "should still have cowork tab"
        );
    }

    #[test]
    fn browse_layout_single_provider() {
        let providers = vec![AiProvider::Claude];
        let layout = generate_browse_layout(&providers);
        assert!(
            layout.contains("aibox-tab name=\"browse\""),
            "should have browse tab"
        );
        assert!(
            layout.contains("command \"claude\""),
            "should have claude pane"
        );
        assert!(
            layout.contains("aibox-tab name=\"editor\""),
            "should have editor tab"
        );
        assert!(
            layout.contains("aibox-tab name=\"git\""),
            "should have git tab"
        );
        assert!(
            layout.contains("AIBOX_EDITOR_DIR=tab"),
            "should use tab editor direction"
        );
        assert!(
            !layout.contains("stacked"),
            "single provider should not be stacked"
        );
    }

    #[test]
    fn browse_layout_multiple_providers_use_tabs() {
        let providers = vec![AiProvider::Claude, AiProvider::Aider];
        let layout = generate_browse_layout(&providers);
        assert!(
            !layout.contains("stacked"),
            "multiple providers should use separate tabs"
        );
        assert!(layout.contains("command \"claude\""), "should have claude");
        assert!(layout.contains("command \"aider\""), "should have aider");
        assert!(
            layout.contains("aibox-tab name=\"aider\""),
            "secondary provider should get its own tab"
        );
    }

    #[test]
    fn browse_layout_no_providers() {
        let providers: Vec<AiProvider> = vec![];
        let layout = generate_browse_layout(&providers);
        assert!(
            layout.contains("aibox-tab name=\"browse\""),
            "should still have browse tab"
        );
        assert!(!layout.contains("claude"), "should not have claude");
        assert!(
            layout.contains("aibox-tab name=\"editor\""),
            "should still have editor tab"
        );
    }

    #[test]
    fn browse_layout_yazi_above_ai() {
        let providers = vec![AiProvider::Claude];
        let layout = generate_browse_layout(&providers);
        let yazi_pos = layout.find("yazi").unwrap();
        let claude_pos = layout.find("command \"claude\"").unwrap();
        assert!(
            yazi_pos < claude_pos,
            "yazi should appear before AI pane (top position)"
        );
        assert!(layout.contains("size=\"60%\""), "yazi pane should be 60%");
    }

    #[test]
    fn ai_layout_single_provider() {
        let providers = vec![AiProvider::Claude];
        let layout = generate_ai_layout(&providers);
        assert!(
            layout.contains("aibox-tab name=\"ai\""),
            "should have ai tab"
        );
        assert!(
            layout.contains("command \"claude\""),
            "should have claude pane"
        );
        assert!(
            layout.contains("aibox-tab name=\"editor\""),
            "should have editor tab"
        );
        assert!(
            layout.contains("aibox-tab name=\"git\""),
            "should have git tab"
        );
        assert!(
            layout.contains("aibox-tab name=\"shell\""),
            "should have shell tab"
        );
        assert!(
            layout.contains("split_direction=\"vertical\""),
            "should split vertically"
        );
        // v0.16.5: yazi gets 50%, AI pane gets 50% (was 53/47 in v0.14.5+)
        assert!(
            layout.contains("size=\"50%\" name=\"files\""),
            "yazi pane should be 50%"
        );
        assert!(layout.contains("size=\"50%\""), "ai pane should be 50%");
        assert!(
            !layout.contains("stacked"),
            "single provider should not be stacked"
        );
    }

    #[test]
    fn ai_layout_codex_only_omits_unselected_claude() {
        let providers = vec![AiProvider::Codex];
        let layout = generate_ai_layout(&providers);

        assert!(
            layout.contains("command \"codex\""),
            "selected Codex provider should start"
        );
        assert!(
            !layout.contains("command \"claude\""),
            "unselected Claude provider must not start"
        );
        assert!(
            !layout.contains("aibox-tab name=\"claude\""),
            "unselected Claude provider must not get a tab"
        );
    }

    #[test]
    fn ai_layout_multiple_providers_use_tabs() {
        let providers = vec![AiProvider::Claude, AiProvider::Aider];
        let layout = generate_ai_layout(&providers);
        assert!(
            !layout.contains("stacked"),
            "multiple providers should use separate tabs"
        );
        assert!(layout.contains("command \"claude\""), "should have claude");
        assert!(layout.contains("command \"aider\""), "should have aider");
        assert!(
            layout.contains("aibox-tab name=\"aider\""),
            "secondary provider should get its own tab"
        );
    }

    #[test]
    fn ai_layout_no_providers() {
        let providers: Vec<AiProvider> = vec![];
        let layout = generate_ai_layout(&providers);
        assert!(
            layout.contains("aibox-tab name=\"ai\""),
            "should still have ai tab"
        );
        assert!(!layout.contains("claude"), "should not have claude");
        assert!(
            layout.contains("aibox-tab name=\"editor\""),
            "should still have editor tab"
        );
        assert!(layout.contains("yazi"), "should still have yazi pane");
    }

    #[test]
    fn ai_layout_yazi_left_of_ai() {
        let providers = vec![AiProvider::Claude];
        let layout = generate_ai_layout(&providers);
        let yazi_pos = layout.find("yazi").unwrap();
        let claude_pos = layout.find("command \"claude\"").unwrap();
        assert!(
            yazi_pos < claude_pos,
            "yazi should appear left of (before) AI pane"
        );
    }

    #[test]
    fn cowork_swap_layout_single_provider() {
        let providers = vec![AiProvider::Claude];
        let layout = generate_cowork_swap_layout(&providers);
        assert!(
            layout.contains("aibox-tab name=\"cowork-swap\""),
            "should have cowork-swap tab"
        );
        assert!(
            layout.contains("command \"claude\""),
            "should have claude pane"
        );
        assert!(
            layout.contains("name=\"editor\""),
            "should have editor pane"
        );
        assert!(layout.contains("vim-loop"), "should run vim-loop");
        assert!(layout.contains("yazi"), "should run yazi");
        assert!(
            layout.contains("aibox-tab name=\"git\""),
            "should have git tab"
        );
        assert!(
            layout.contains("aibox-tab name=\"shell\""),
            "should have shell tab"
        );
        // Outer split: left 40% / right 60% (editor on the right gets the bigger half)
        assert!(
            layout.contains("size=\"40%\" split_direction=\"horizontal\""),
            "left side should be 40% with horizontal sub-split"
        );
        assert!(
            layout.contains("size=\"60%\" name=\"editor\""),
            "right side (editor) should be 60%"
        );
        // Inner left split: yazi 40% top, AI 60% bottom
        assert!(
            layout.contains("size=\"40%\" name=\"files\""),
            "yazi pane should be 40% of left stack"
        );
        assert!(
            !layout.contains("stacked"),
            "single provider should not be stacked"
        );
    }

    #[test]
    fn cowork_swap_layout_multiple_providers_use_tabs() {
        let providers = vec![AiProvider::Claude, AiProvider::Aider];
        let layout = generate_cowork_swap_layout(&providers);
        assert!(
            !layout.contains("stacked"),
            "multiple providers should use separate tabs"
        );
        assert!(layout.contains("command \"claude\""), "should have claude");
        assert!(layout.contains("command \"aider\""), "should have aider");
        assert!(
            layout.contains("aibox-tab name=\"aider\""),
            "secondary provider should get its own tab"
        );
    }

    #[test]
    fn cowork_swap_layout_no_providers() {
        let providers: Vec<AiProvider> = vec![];
        let layout = generate_cowork_swap_layout(&providers);
        assert!(
            layout.contains("aibox-tab name=\"cowork-swap\""),
            "should still have cowork-swap tab"
        );
        assert!(!layout.contains("claude"), "should not have claude");
        assert!(layout.contains("yazi"), "should still have yazi pane");
        assert!(layout.contains("vim-loop"), "should still have vim editor");
        assert!(
            layout.contains("size=\"40%\" name=\"files\""),
            "yazi should be 40% (left)"
        );
        assert!(
            layout.contains("size=\"60%\" name=\"editor\""),
            "vim should be 60% (right)"
        );
    }

    #[test]
    fn cowork_swap_layout_editor_right_of_files() {
        let providers = vec![AiProvider::Claude];
        let layout = generate_cowork_swap_layout(&providers);
        let files_pos = layout.find("name=\"files\"").unwrap();
        let editor_pos = layout.find("name=\"editor\"").unwrap();
        assert!(
            files_pos < editor_pos,
            "yazi (files) should appear before editor in layout source — editor sits to the right"
        );
    }

    #[test]
    fn cowork_swap_layout_ai_below_files_in_left_stack() {
        let providers = vec![AiProvider::Claude];
        let layout = generate_cowork_swap_layout(&providers);
        let files_pos = layout.find("name=\"files\"").unwrap();
        let claude_pos = layout.find("command \"claude\"").unwrap();
        let editor_pos = layout.find("name=\"editor\"").unwrap();
        assert!(
            files_pos < claude_pos && claude_pos < editor_pos,
            "yazi → claude (left stack top→bottom) → editor (right) order in source"
        );
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
    fn zellij_config_substitutes_layout() {
        // Regression test for the layout-sync bug fixed in v0.14.2:
        // DEFAULT_ZELLIJ_CONFIG must contain the AIBOX_LAYOUT placeholder
        // that seed_root_dir / sync_theme_files replace with the configured layout.
        assert!(
            DEFAULT_ZELLIJ_CONFIG.contains("AIBOX_LAYOUT"),
            "config template must contain AIBOX_LAYOUT placeholder"
        );
        assert!(
            !DEFAULT_ZELLIJ_CONFIG.contains("default_layout \"dev\""),
            "config template must not hard-code dev as default_layout"
        );
    }

    #[test]
    fn zellij_layout_suspends_non_focused_dev_commands() {
        let layout = generate_dev_layout(&[]);
        assert!(
            layout.contains("plugin location=\"zellij:status-bar\"")
                && layout.contains("aibox-status")
                && !layout.contains("aibox-status.wasm"),
            "default layouts should use the shell-backed status rows until the native Zellij plugin is proven in visual E2E"
        );
        assert!(
            layout.contains(
                "pane size=\"40%\" name=\"files\" focus=true {\n                command \"yazi\"\n                cwd \"/workspace\"\n            }"
            ),
            "focused first-screen file pane should start active"
        );
        assert!(
            layout.contains(
                "pane size=\"60%\" name=\"editor\" {\n                command \"vim-loop\"\n                cwd \"/workspace\"\n                start_suspended true"
            ),
            "non-focused editor pane should start suspended"
        );
        assert!(
            layout.contains("command \"lazygit\"\n            cwd \"/workspace\"\n            start_suspended true"),
            "git tab should start suspended"
        );
        assert!(
            layout.contains(
                "command \"bash\"\n            cwd \"/workspace\"\n            start_suspended true"
            ),
            "shell tab should start suspended"
        );
    }

    #[test]
    fn zellij_status_mode_shell_uses_legacy_fallback() {
        let layout = generate_dev_layout_with_options(&[], true, &ZellijStatusMode::Shell);
        assert!(layout.contains("plugin location=\"zellij:status-bar\""));
        assert!(layout.contains("aibox-status"));
        assert!(!layout.contains("aibox-status.wasm"));
    }

    #[test]
    fn zellij_status_mode_native_uses_custom_keybar_above_runtime_status() {
        let layout = generate_dev_layout_with_options(&[], true, &ZellijStatusMode::Native);
        let keybar = layout.find("role \"keys\"").unwrap();
        let runtime = layout.find("role \"status\"").unwrap();
        assert!(
            keybar < runtime,
            "aibox key hints should render above the aibox runtime status row"
        );
        assert!(
            layout.contains("aibox-status.wasm"),
            "native mode should use the aibox WASM keybar and runtime status rows"
        );
        assert!(
            !layout.contains("zellij:status-bar"),
            "native mode should not depend on Zellij's built-in status bar"
        );
    }

    #[test]
    fn zellij_status_mode_hidden_omits_status_rows() {
        let layout = generate_dev_layout_with_options(&[], true, &ZellijStatusMode::Hidden);
        assert!(layout.contains("tab_template name=\"aibox-tab\""));
        assert!(!layout.contains("zellij:status-bar"));
        assert!(!layout.contains("aibox-status"));
        assert!(!layout.contains("role \"status\""));
    }

    #[test]
    fn aibox_status_watch_does_not_clear_line_on_refresh() {
        assert!(
            DEFAULT_AIBOX_STATUS_SH.contains("previous_width"),
            "watch mode should track prior line width for in-place redraw"
        );
        assert!(
            DEFAULT_AIBOX_STATUS_SH.contains("AIBOX")
                && DEFAULT_AIBOX_STATUS_SH.contains("\\033[7m"),
            "watch mode should render a zellij-like status segment"
        );
        assert!(
            DEFAULT_AIBOX_STATUS_SH.contains("--plugin-json")
                && DEFAULT_AIBOX_STATUS_SH.contains("print_status_json")
                && DEFAULT_AIBOX_STATUS_SH.contains("\"load_average\"")
                && DEFAULT_AIBOX_STATUS_SH.contains("\"git_branch\""),
            "status helper should expose structured metrics for the native plugin"
        );
        for group in ["MEM", "CPU", "load", "PROC", "FS", "UP", "PROJ"] {
            assert!(
                DEFAULT_AIBOX_STATUS_SH.contains(group),
                "status line should include the {group} metric group"
            );
        }
        assert!(
            !DEFAULT_AIBOX_STATUS_SH.contains("\\033[2K"),
            "watch mode should not clear the line on every refresh"
        );
    }

    #[test]
    fn zellij_config_exposes_status_toggle_keybinding() {
        assert!(
            DEFAULT_ZELLIJ_CONFIG.contains("MessagePlugin")
                && DEFAULT_ZELLIJ_CONFIG.contains("bind \"v\"")
                && DEFAULT_ZELLIJ_CONFIG.contains("bind \"b\""),
            "Ctrl+g then v/b should toggle the aibox status rows"
        );
    }

    #[test]
    fn ai_pane_kdl_empty() {
        let result = ai_pane_kdl(&[]);
        assert!(
            result.is_empty(),
            "empty providers should produce empty string"
        );
    }

    #[test]
    fn ai_pane_kdl_single() {
        let result = ai_pane_kdl(&[AiProvider::Claude]);
        assert!(result.contains("command \"claude\""));
        assert!(!result.contains("stacked"));
        assert!(!result.contains("start_suspended"));
    }

    #[test]
    fn ai_pane_kdl_multiple() {
        let result = ai_pane_kdl(&[AiProvider::Claude, AiProvider::Aider, AiProvider::Gemini]);
        assert!(result.contains("command \"claude\""));
        assert!(!result.contains("stacked"));
        assert!(!result.contains("start_suspended"));
        assert!(!result.contains("command \"aider\""));
        assert!(!result.contains("command \"gemini\""));
    }

    #[test]
    fn ai_extra_tabs_kdl_skips_primary_provider() {
        let result =
            ai_extra_tabs_kdl(&[AiProvider::Claude, AiProvider::Aider, AiProvider::Gemini]);
        assert!(!result.contains("command \"claude\""));
        assert!(result.contains("command \"aider\""));
        assert!(result.contains("command \"gemini\""));
        assert_eq!(
            occurrences(&result, "start_suspended true"),
            2,
            "secondary AI tabs should start suspended"
        );
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

        unsafe {
            std::env::remove_var("AIBOX_HOST_ROOT");
        }
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

        unsafe {
            std::env::remove_var("AIBOX_HOST_ROOT");
        }
    }
}
