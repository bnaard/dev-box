//! Theme data for all supported color themes.
//!
//! Each theme provides config snippets for tmux, Vim, Yazi, and lazygit.

use crate::config::{StarshipPreset, Theme, ThemeEmphasis};
use std::collections::BTreeMap;
use std::sync::LazyLock;

static AUDITED_THEMES: LazyLock<toml::Value> = LazyLock::new(|| {
    toml::from_str(include_str!("../assets/aibox-theme-corrections.toml"))
        .expect("embedded audited theme data must be valid TOML")
});

fn audited_theme(theme: &Theme) -> &'static toml::Value {
    let key = format!("{theme:?}");
    AUDITED_THEMES["themes"]
        .get(&key)
        .unwrap_or_else(|| panic!("audited theme data missing {key}"))
}

fn audited_color<'a>(spec: &'a toml::Value, key: &str) -> &'a str {
    spec.get(key)
        .and_then(toml::Value::as_str)
        .unwrap_or_else(|| panic!("audited theme field {key} must be a color string"))
}

fn audited_chrome_color(theme: &Theme, key: &str) -> Option<&'static str> {
    audited_theme(theme)
        .get("chrome")
        .and_then(|chrome| chrome.get(key))
        .and_then(toml::Value::as_str)
}

fn audited_magenta(theme: &Theme) -> &'static str {
    audited_theme(theme)
        .get("magenta")
        .and_then(toml::Value::as_str)
        .or_else(|| audited_chrome_color(theme, "magenta"))
        .expect("audited theme must define magenta")
}

fn audited_accent_text(theme: &Theme) -> &'static str {
    audited_chrome_color(theme, "accent_text")
        .unwrap_or_else(|| audited_color(audited_theme(theme), "accent"))
}

fn audited_accent_fill(theme: &Theme) -> &'static str {
    audited_chrome_color(theme, "accent_fill")
        .unwrap_or_else(|| audited_color(audited_theme(theme), "accent"))
}

/// Resolve `auto` once while generating managed runtime files. An explicit
/// level is deterministic; `NO_COLOR` deliberately disables the secondary
/// decoration channel together with color-oriented affordances.
pub fn resolved_emphasis(requested: ThemeEmphasis) -> ThemeEmphasis {
    match requested {
        ThemeEmphasis::Auto if std::env::var_os("NO_COLOR").is_some() => ThemeEmphasis::None,
        ThemeEmphasis::Auto => {
            let terminfo = std::process::Command::new("infocmp").arg("-1").output();
            match terminfo {
                Ok(output) if output.status.success() => {
                    let body = String::from_utf8_lossy(&output.stdout);
                    let italic = body.contains("sitm=") && body.contains("ritm=");
                    let dim = body.contains("dim=");
                    if italic {
                        ThemeEmphasis::Standard
                    } else if dim {
                        ThemeEmphasis::Minimal
                    } else {
                        // Bold remains the universal fallback even when this
                        // terminfo entry lacks optional style capabilities.
                        ThemeEmphasis::Minimal
                    }
                }
                _ => ThemeEmphasis::Standard,
            }
        }
        explicit => explicit,
    }
}

fn fallback_role_attributes(level: ThemeEmphasis, role: &str) -> &'static str {
    let level = resolved_emphasis(level);
    match (level, role) {
        (ThemeEmphasis::None, _) => "",
        (ThemeEmphasis::Full, "code_invalid" | "search_current" | "git_conflicted") => {
            "bold underline"
        }
        (ThemeEmphasis::Full, "code_deprecated") => "strikethrough dim",
        (ThemeEmphasis::Standard | ThemeEmphasis::Minimal, "code_deprecated") => "dim",
        (
            ThemeEmphasis::Full | ThemeEmphasis::Standard,
            "code_comment" | "code_decorator" | "git_untracked",
        ) => "italic",
        (ThemeEmphasis::Minimal, "code_comment" | "code_decorator") => "dim",
        (
            _,
            "code_keyword"
            | "code_type"
            | "code_invalid"
            | "diff_emphasis"
            | "diff_header"
            | "status_error"
            | "status_warning"
            | "active_foreground"
            | "pane_active_foreground"
            | "search_current"
            | "git_modified"
            | "git_staged"
            | "git_conflicted",
        ) => "bold",
        (
            _,
            "diff_hunk"
            | "status_disabled"
            | "inactive_foreground"
            | "pane_inactive_foreground"
            | "git_ignored",
        ) => "dim",
        _ => "",
    }
}

fn clamp_attributes(level: ThemeEmphasis, attributes: &str) -> String {
    let level = resolved_emphasis(level);
    if level == ThemeEmphasis::None {
        return String::new();
    }
    let mut output = Vec::new();
    for attribute in attributes.split_whitespace() {
        let degraded = match (level, attribute) {
            (ThemeEmphasis::Full, value) => value,
            (ThemeEmphasis::Standard, "underline") => "bold",
            (ThemeEmphasis::Standard, "strikethrough") => "dim",
            (ThemeEmphasis::Standard, value @ ("bold" | "italic" | "dim")) => value,
            (ThemeEmphasis::Minimal, "italic" | "strikethrough") => "dim",
            (ThemeEmphasis::Minimal, "underline") => "bold",
            (ThemeEmphasis::Minimal, value @ ("bold" | "dim")) => value,
            // Validation rejects unknown values. Keeping this defensive arm
            // makes direct library callers degrade safely too.
            _ => continue,
        };
        if !output.contains(&degraded) {
            output.push(degraded);
        }
    }
    output.join(" ")
}

fn role_attributes(
    level: ThemeEmphasis,
    role: &str,
    overrides: Option<&BTreeMap<String, String>>,
) -> String {
    let requested = overrides
        .and_then(|values| values.get(role))
        .map(String::as_str)
        .unwrap_or_else(|| fallback_role_attributes(level, role));
    clamp_attributes(level, requested)
}

fn theme_role_attributes(
    theme: &Theme,
    level: ThemeEmphasis,
    role: &str,
    overrides: Option<&BTreeMap<String, String>>,
) -> String {
    let authored_slot = match role {
        "code_function" => Some("accent"),
        "code_string" => Some("green"),
        "code_invalid" => Some("red"),
        "code_type" => Some("yellow"),
        "code_number" => Some("orange"),
        "code_operator" => Some("cyan"),
        "code_comment" => Some("muted"),
        "code_keyword" => Some("magenta"),
        _ => None,
    };
    let authored = authored_slot.and_then(|slot| {
        audited_theme(theme)
            .get("attributes")
            .and_then(|attributes| attributes.get(slot))
            .and_then(toml::Value::as_str)
    });
    let requested = overrides
        .and_then(|values| values.get(role))
        .map(String::as_str)
        .or(authored)
        .unwrap_or_else(|| fallback_role_attributes(level, role));
    clamp_attributes(level, requested)
}

fn vim_attrs(
    theme: &Theme,
    level: ThemeEmphasis,
    role: &str,
    overrides: Option<&BTreeMap<String, String>>,
) -> String {
    // Vim does not accept `dim` as a :highlight gui attribute (E418). Color
    // still provides the secondary channel for muted/inactive roles, so drop
    // the unsupported attribute instead of emitting an invalid colorscheme.
    let attrs = theme_role_attributes(theme, level, role, overrides)
        .split_whitespace()
        .filter(|attr| *attr != "dim")
        .collect::<Vec<_>>()
        .join(",");
    if attrs.is_empty() {
        "NONE".to_string()
    } else {
        attrs
    }
}

pub fn tmux_role_attributes(
    level: ThemeEmphasis,
    role: &str,
    overrides: Option<&BTreeMap<String, String>>,
) -> String {
    role_attributes(level, role, overrides)
        .split_whitespace()
        .map(|attr| match attr {
            "italic" => "italics",
            "underline" => "underscore",
            "strikethrough" => "strikethrough",
            other => other,
        })
        .collect::<Vec<_>>()
        .join(",")
}

/// Returns terminal surface colors for tmux panes and app backgrounds.
pub fn terminal_surface_colors(theme: &Theme) -> (&str, &str, &str, &str, String, String) {
    let (bg, fg, accent, _green, _red, _yellow, _orange, _cyan, muted) = theme_palette(theme);
    (
        bg,
        fg,
        accent,
        muted,
        audited_chrome_color(theme, "pane_inactive_fg")
            .unwrap_or(muted)
            .to_string(),
        accent.to_string(),
    )
}

/// Dimmed (bg, fg) for *inactive* tmux panes — used by `window-style`
/// so the focused pane is immediately identifiable even when a full-screen
/// TUI such as Yazi paints every pane edge-to-edge.
///
/// Themes with audited pane colors keep those exact values. Other themes use
/// a stronger muted bias than the old subtle treatment: 22% for the surface
/// and 38% for text. Contrast tests keep the inactive content readable.
pub fn dim_inactive_pane_colors(theme: &Theme) -> (String, String) {
    if let (Some(bg), Some(fg)) = (
        audited_chrome_color(theme, "pane_inactive_bg"),
        audited_chrome_color(theme, "pane_inactive_fg"),
    ) {
        return (bg.to_string(), fg.to_string());
    }
    let (bg, fg, _accent, _green, _red, _yellow, _orange, _cyan, muted) = theme_palette(theme);
    let dim_bg = mix_hex_colors(bg, muted, 78);
    let dim_fg = mix_hex_colors(fg, muted, 62);
    (dim_bg, dim_fg)
}

pub fn terminal_border_colors(theme: &Theme) -> (&'static str, &'static str) {
    let (_, _, _accent, _, _, _, _, _, muted) = theme_palette(theme);
    (
        audited_chrome_color(theme, "border_active").unwrap_or_else(|| audited_accent_fill(theme)),
        audited_chrome_color(theme, "border_inactive").unwrap_or(muted),
    )
}

fn mix_hex_colors(primary: &str, secondary: &str, primary_percent: u8) -> String {
    let Some((pr, pg, pb)) = parse_hex_rgb(primary) else {
        return primary.to_string();
    };
    let Some((sr, sg, sb)) = parse_hex_rgb(secondary) else {
        return primary.to_string();
    };
    let p = u16::from(primary_percent.min(100));
    let s = 100 - p;
    let mix = |a: u8, b: u8| -> u8 { ((u16::from(a) * p + u16::from(b) * s) / 100) as u8 };
    format!("#{:02X}{:02X}{:02X}", mix(pr, sr), mix(pg, sg), mix(pb, sb))
}

fn parse_hex_rgb(color: &str) -> Option<(u8, u8, u8)> {
    let hex = color.strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some((r, g, b))
}

/// Returns the Vim colorscheme name for the given theme.
///
/// Every theme uses the generated `aibox` colorscheme (see
/// [`vim_aibox_colorscheme`]). Vim picks it up from
/// `~/.vim/colors/aibox.vim`, which `seed.rs` materializes from the active
/// palette. This guarantees Vim highlights match Yazi/tmux/Starship colors
/// across all 28 themes, including the ones that previously fell back to
/// `catppuccin_mocha`/`tokyonight`/`gruvbox`.
pub fn vim_colorscheme(_theme: &Theme) -> &'static str {
    "aibox"
}

/// Generate `~/.vim/colors/aibox.vim` content from the theme palette.
#[cfg(test)]
pub fn vim_aibox_colorscheme(theme: &Theme) -> String {
    vim_aibox_colorscheme_with_emphasis(theme, ThemeEmphasis::Auto)
}

#[cfg(test)]
pub fn vim_aibox_colorscheme_with_emphasis(theme: &Theme, emphasis: ThemeEmphasis) -> String {
    vim_aibox_colorscheme_with_style(theme, emphasis, &BTreeMap::new())
}

pub fn vim_aibox_colorscheme_with_style(
    theme: &Theme,
    emphasis: ThemeEmphasis,
    overrides: &BTreeMap<String, String>,
) -> String {
    let (bg, fg, accent, green, red, yellow, orange, cyan, muted) = theme_palette(theme);
    let accent_fill = audited_accent_fill(theme);
    let surface = yazi_surface_color(theme);
    let selection_bg = audited_chrome_color(theme, "selection_bg").unwrap_or(surface);
    let selection_fg = audited_chrome_color(theme, "selection_fg").unwrap_or(fg);
    let active_ink = audited_chrome_color(theme, "status_active_ink").unwrap_or(bg);
    let cursor = audited_chrome_color(theme, "cursor").unwrap_or(accent);
    let cursor_text = audited_chrome_color(theme, "cursor_text").unwrap_or(bg);
    let diff_add_bg = audited_chrome_color(theme, "diff_add_bg").unwrap_or(bg);
    let diff_del_bg = audited_chrome_color(theme, "diff_del_bg").unwrap_or(bg);
    let diff_change_bg = audited_chrome_color(theme, "diff_change_bg").unwrap_or(bg);
    let background = vim_background(theme);
    let magenta = audited_magenta(theme);
    let cursor_line = surface;
    let keyword_attr = vim_attrs(theme, emphasis, "code_keyword", Some(overrides));
    let type_attr = vim_attrs(theme, emphasis, "code_type", Some(overrides));
    let function_attr = vim_attrs(theme, emphasis, "code_function", Some(overrides));
    let string_attr = vim_attrs(theme, emphasis, "code_string", Some(overrides));
    let number_attr = vim_attrs(theme, emphasis, "code_number", Some(overrides));
    let operator_attr = vim_attrs(theme, emphasis, "code_operator", Some(overrides));
    let comment_attr = vim_attrs(theme, emphasis, "code_comment", Some(overrides));
    let decorator_attr = vim_attrs(theme, emphasis, "code_decorator", Some(overrides));
    let invalid_attr = vim_attrs(theme, emphasis, "code_invalid", Some(overrides));
    let active_attr = vim_attrs(theme, emphasis, "active_foreground", Some(overrides));
    let inactive_attr = vim_attrs(theme, emphasis, "inactive_foreground", Some(overrides));
    let error_attr = vim_attrs(theme, emphasis, "status_error", Some(overrides));
    let warning_attr = vim_attrs(theme, emphasis, "status_warning", Some(overrides));
    let search_current_attr = vim_attrs(theme, emphasis, "search_current", Some(overrides));
    format!(
        r#"" aibox.vim — generated by aibox CLI from the active theme palette.
" Do NOT edit; re-run `aibox theme` to regenerate.

hi clear
if exists("syntax_on")
  syntax reset
endif
let g:colors_name = "aibox"
set background={background}

" ── UI chrome ──────────────────────────────────────────────────────────────
hi Normal         guifg={fg}     guibg={bg}     ctermfg=NONE ctermbg=NONE
hi NormalNC       guifg={fg}     guibg={bg}
hi Cursor         guifg={cursor_text} guibg={cursor}
hi lCursor        guifg={cursor_text} guibg={cursor}
hi LineNr         guifg={muted}  guibg={bg}
hi CursorLineNr   guifg={accent} guibg={cursor_line} gui={active_attr}
hi CursorLine     guibg={cursor_line}
hi CursorColumn   guibg={cursor_line}
hi ColorColumn    guibg={cursor_line}
hi VertSplit      guifg={muted}  guibg={bg}
hi WinSeparator   guifg={muted}  guibg={bg}
hi StatusLine     guifg={active_ink} guibg={accent_fill} gui={active_attr}
hi StatusLineNC   guifg={muted}  guibg={surface} gui={inactive_attr}
hi TabLine        guifg={muted}  guibg={surface} gui={inactive_attr}
hi TabLineFill    guibg={bg}
hi TabLineSel     guifg={active_ink} guibg={accent_fill} gui={active_attr}
hi SignColumn     guifg={muted}  guibg={bg}
hi FoldColumn     guifg={muted}  guibg={bg}
hi Folded         guifg={muted}  guibg={surface}
hi NonText        guifg={muted}
hi EndOfBuffer    guifg={bg}     guibg={bg}
hi SpecialKey     guifg={muted}
hi MatchParen     guifg={accent} guibg={surface} gui={search_current_attr}
hi Conceal        guifg={muted}  guibg={bg}
hi Directory      guifg={accent} gui=bold

" ── Selection / search ────────────────────────────────────────────────────
hi Visual         guifg={selection_fg} guibg={selection_bg}
hi VisualNOS      guifg={selection_fg} guibg={selection_bg}
hi Search         guifg={bg}     guibg={yellow} gui=bold
hi IncSearch      guifg={bg}     guibg={orange} gui={search_current_attr}
hi CurSearch      guifg={active_ink} guibg={accent_fill} gui={search_current_attr}
hi QuickFixLine   guibg={surface} gui={active_attr}

" ── Popup menu ────────────────────────────────────────────────────────────
hi Pmenu          guifg={fg}     guibg={surface}
hi PmenuSel       guifg={active_ink} guibg={accent_fill} gui={active_attr}
hi PmenuSbar      guibg={surface}
hi PmenuThumb     guibg={muted}
hi WildMenu       guifg={active_ink} guibg={accent_fill} gui={active_attr}

" ── Messages ──────────────────────────────────────────────────────────────
hi ErrorMsg       guifg={red}    gui={error_attr}
hi WarningMsg     guifg={yellow} gui={warning_attr}
hi ModeMsg        guifg={accent} gui={active_attr}
hi MoreMsg        guifg={green}
hi Question       guifg={accent}
hi Title          guifg={accent} gui={active_attr}

" ── Syntax (linked groups) ────────────────────────────────────────────────
hi Comment        guifg={muted}    gui={comment_attr}
hi Constant       guifg={orange}
hi String         guifg={green}     gui={string_attr}
hi Character      guifg={green}     gui={string_attr}
hi Number         guifg={orange}    gui={number_attr}
hi Boolean        guifg={orange}    gui={number_attr}
hi Float          guifg={orange}    gui={number_attr}
hi Identifier     guifg={fg}
hi Function       guifg={cyan}      gui={function_attr}
hi Statement      guifg={magenta}  gui={keyword_attr}
hi Conditional    guifg={magenta}  gui={keyword_attr}
hi Repeat         guifg={magenta}  gui={keyword_attr}
hi Label          guifg={yellow}
hi Operator       guifg={cyan}      gui={operator_attr}
hi Keyword        guifg={magenta}  gui={keyword_attr}
hi Exception      guifg={magenta}  gui={keyword_attr}
hi PreProc        guifg={magenta}
hi Include        guifg={magenta}
hi Define         guifg={magenta}
hi Macro          guifg={accent}   gui={decorator_attr}
hi PreCondit      guifg={yellow}
hi Type           guifg={yellow}   gui={type_attr}
hi StorageClass   guifg={yellow}   gui={type_attr}
hi Structure      guifg={yellow}   gui={type_attr}
hi Typedef        guifg={yellow}   gui={type_attr}
hi Special        guifg={cyan}
hi SpecialChar    guifg={orange}
hi Tag            guifg={accent}
hi Delimiter      guifg={fg}
hi SpecialComment guifg={muted}    gui={comment_attr}
hi Debug          guifg={orange}
hi Underlined     guifg={accent}   gui=underline
hi Ignore         guifg={muted}
hi Error          guifg={red}      gui={invalid_attr}
hi Todo           guifg={yellow}   guibg={surface} gui=bold

" ── Diff ──────────────────────────────────────────────────────────────────
hi DiffAdd        guifg={green}    guibg={diff_add_bg}
hi DiffChange     guifg={yellow}   guibg={diff_change_bg}
hi DiffDelete     guifg={red}      guibg={diff_del_bg}
hi DiffText       guifg={accent}   guibg={diff_change_bg} gui=bold
hi diffAdded      guifg={green}
hi diffRemoved    guifg={red}
hi diffChanged    guifg={yellow}
hi diffFile       guifg={accent}   gui=bold
hi diffNewFile    guifg={green}    gui=bold
hi diffOldFile    guifg={red}      gui=bold
hi diffLine       guifg={muted}

" ── Spelling ──────────────────────────────────────────────────────────────
hi SpellBad       gui=undercurl guisp={red}
hi SpellCap       gui=undercurl guisp={yellow}
hi SpellLocal     gui=undercurl guisp={cyan}
hi SpellRare      gui=undercurl guisp={magenta}

" ── Git plugins / gitgutter / signify ────────────────────────────────────
hi GitGutterAdd          guifg={green}    guibg={bg}
hi GitGutterChange       guifg={yellow}   guibg={bg}
hi GitGutterDelete       guifg={red}      guibg={bg}
hi GitGutterChangeDelete guifg={orange}   guibg={bg}
hi SignifySignAdd        guifg={green}    guibg={bg}
hi SignifySignChange     guifg={yellow}   guibg={bg}
hi SignifySignDelete     guifg={red}      guibg={bg}

" ── Markdown ──────────────────────────────────────────────────────────────
hi link markdownH1        Title
hi link markdownH2        Title
hi link markdownH3        Title
hi link markdownH4        Title
hi link markdownCode      String
hi link markdownCodeBlock String
hi link markdownLinkText  Function
hi markdownUrl   guifg={cyan}  gui=underline

" ── Vim help ──────────────────────────────────────────────────────────────
hi link helpHyperTextEntry Underlined
hi link helpHyperTextJump  Underlined
hi link helpHeader         Title
hi link helpExample        String
hi link helpOption         Type
hi link helpVim            Identifier

" ── Misc plugins ──────────────────────────────────────────────────────────
hi link NvimTreeFolderName Directory
hi link NvimTreeRootFolder Title
hi link TelescopeBorder    FloatBorder
hi FloatBorder    guifg={muted}    guibg={bg}
hi NormalFloat    guifg={fg}       guibg={surface}
"#
    )
}

/// Returns the Vim background setting (dark/light).
pub fn vim_background(theme: &Theme) -> &'static str {
    match theme {
        Theme::GruvboxLight
        | Theme::CatppuccinLatte
        | Theme::TokyoNightDay
        | Theme::RosePineDawn
        | Theme::MaterialLighter
        | Theme::SolarizedLight
        | Theme::GithubLight
        | Theme::GithubLightHighContrast
        | Theme::AyuLight
        | Theme::NightOwlLight
        | Theme::OneLight
        | Theme::VitesseLight
        | Theme::MinLight
        | Theme::KanagawaLotus
        | Theme::EverforestLight
        | Theme::VsCodeLightPlus
        | Theme::SlackOchin
        | Theme::SnazzyLight
        | Theme::ProjectiousLight
        | Theme::ProjectiousHCLight
        | Theme::MonoLight
        | Theme::ContrastLight
        | Theme::ContrastLightMax
        | Theme::ContrastMonoLight
        | Theme::ContrastMonoLightMax => "light",
        _ => "dark",
    }
}

/// Returns the Yazi theme.toml content for the given theme.
///
/// Generated entirely from the 9-slot palette so every aibox theme renders
/// consistently in Yazi (status bar, mode, tabs, popup pickers, filetype
/// colors) without relying on bundled per-theme toml files.
/// Yazi tab separator open/close glyphs for a given tmux separator style.
/// Keeps the file manager's tab chevrons in lockstep with the status-bar style.
pub fn yazi_tab_separators(style: &str) -> (&'static str, &'static str) {
    match style {
        "normal" => ("", ""),
        "rounded" => ("", ""),
        "slant" => ("", ""),
        "slantup" => ("", ""),
        "flame" => ("", ""),
        "pixel" => ("▕", "▏"),
        "none" => ("", ""),
        // trapezoid / honeycomb / unknown — fall back to chevrons
        _ => ("", ""),
    }
}

/// Returns the Yazi theme.toml content for the given theme + tmux separator
/// style. The style argument lets Yazi's tab chevrons mirror the tmux status
/// bar's `@powerkit_separator_style`.
#[cfg(test)]
pub fn yazi_theme_with_separator(theme: &Theme, sep_style: &str) -> String {
    yazi_theme_with_emphasis(theme, sep_style, ThemeEmphasis::Auto)
}

fn yazi_flags(
    level: ThemeEmphasis,
    role: &str,
    overrides: Option<&BTreeMap<String, String>>,
) -> String {
    role_attributes(level, role, overrides)
        .split_whitespace()
        .map(|attr| match attr {
            "strikethrough" => ", crossed = true".to_string(),
            other => format!(", {other} = true"),
        })
        .collect()
}

#[cfg(test)]
pub fn yazi_theme_with_emphasis(theme: &Theme, sep_style: &str, emphasis: ThemeEmphasis) -> String {
    yazi_theme_with_style(theme, sep_style, emphasis, &BTreeMap::new())
}

pub fn yazi_theme_with_style(
    theme: &Theme,
    sep_style: &str,
    emphasis: ThemeEmphasis,
    overrides: &BTreeMap<String, String>,
) -> String {
    let (bg, fg, accent, green, red, yellow, orange, cyan, muted) = theme_palette(theme);
    let accent_fill = audited_accent_fill(theme);
    let surface = yazi_surface_color(theme);
    let selection_bg = audited_chrome_color(theme, "selection_bg").unwrap_or(surface);
    let selection_fg = audited_chrome_color(theme, "selection_fg").unwrap_or(fg);
    let active_ink = audited_chrome_color(theme, "status_active_ink").unwrap_or(bg);
    let (sep_open, sep_close) = yazi_tab_separators(sep_style);
    // Derive a magenta/pink slot for marker_copied / image-file hue without
    // adding a 10th palette entry — mix accent and red, biased toward red.
    let magenta = audited_magenta(theme);
    let active_attrs = yazi_flags(emphasis, "active_foreground", Some(overrides));
    let inactive_attrs = yazi_flags(emphasis, "inactive_foreground", Some(overrides));
    let modified_attrs = yazi_flags(emphasis, "git_modified", Some(overrides));
    let untracked_attrs = yazi_flags(emphasis, "git_untracked", Some(overrides));
    let ignored_attrs = yazi_flags(emphasis, "git_ignored", Some(overrides));
    let search_attrs = yazi_flags(emphasis, "search_current", Some(overrides));
    let error_attrs = yazi_flags(emphasis, "status_error", Some(overrides));
    format!(
        r#"# =============================================================================
# Yazi theme — generated by aibox CLI from the active theme palette
# Docs: https://yazi-rs.github.io/docs/configuration/theme
# =============================================================================

[mgr]
cwd = {{ fg = "{accent}" }}
hovered = {{ fg = "{selection_fg}", bg = "{selection_bg}"{active_attrs} }}
preview_hovered = {{ fg = "{selection_fg}", bg = "{selection_bg}"{search_attrs} }}
find_keyword = {{ fg = "{yellow}"{search_attrs} }}
find_position = {{ fg = "{magenta}" }}
marker_selected = {{ fg = "{green}", bg = "{green}" }}
marker_copied = {{ fg = "{magenta}", bg = "{magenta}" }}
marker_cut = {{ fg = "{red}", bg = "{red}" }}

[indicator]
parent = {{ fg = "{muted}" }}
current = {{ fg = "{accent_fill}", bg = "{accent_fill}"{active_attrs} }}
preview = {{ fg = "{cyan}" }}
padding = {{ open = "▐", close = "▌" }}

[tabs]
active = {{ fg = "{active_ink}", bg = "{accent_fill}"{active_attrs} }}
inactive = {{ fg = "{muted}", bg = "{surface}"{inactive_attrs} }}
sep_inner = {{ open = "{sep_open}", close = "{sep_close}" }}
sep_outer = {{ open = "{sep_open}", close = "{sep_close}" }}

[mode]
normal_main = {{ fg = "{active_ink}", bg = "{green}"{active_attrs} }}
normal_alt = {{ fg = "{green}", bg = "{surface}"{active_attrs} }}
select_main = {{ fg = "{active_ink}", bg = "{accent_fill}"{active_attrs} }}
select_alt = {{ fg = "{accent}", bg = "{surface}"{active_attrs} }}
unset_main = {{ fg = "{active_ink}", bg = "{magenta}"{active_attrs} }}
unset_alt = {{ fg = "{magenta}", bg = "{surface}"{active_attrs} }}

[status]
overall = {{ fg = "{fg}", bg = "{surface}" }}
sep_left = {{ open = "{sep_open}", close = "{sep_close}" }}
sep_right = {{ open = "{sep_open}", close = "{sep_close}" }}
perm_type = {{ fg = "{accent}" }}
perm_read = {{ fg = "{yellow}" }}
perm_write = {{ fg = "{red}" }}
perm_exec = {{ fg = "{green}" }}
perm_sep = {{ fg = "{muted}" }}
progress_label = {{ fg = "{fg}"{active_attrs} }}
progress_normal = {{ fg = "{accent}", bg = "{surface}" }}
progress_error = {{ fg = "{red}", bg = "{surface}"{error_attrs} }}

[input]
border = {{ fg = "{accent}" }}
title = {{ fg = "{accent}", bg = "{bg}"{active_attrs} }}
value = {{ fg = "{fg}", bg = "{bg}" }}
selected = {{ fg = "{selection_fg}", bg = "{selection_bg}" }}

[pick]
border = {{ fg = "{accent}" }}
active = {{ fg = "{magenta}" }}
inactive = {{ fg = "{muted}", bg = "{bg}" }}

[cmp]
border = {{ fg = "{accent}" }}
active = {{ fg = "{magenta}" }}
inactive = {{ fg = "{muted}", bg = "{bg}" }}

[tasks]
border = {{ fg = "{accent}" }}
title = {{ fg = "{accent}", bg = "{bg}"{active_attrs} }}
hovered = {{ fg = "{selection_fg}", bg = "{selection_bg}"{search_attrs} }}

[which]
mask = {{ bg = "{surface}" }}
cand = {{ fg = "{cyan}" }}
rest = {{ fg = "{muted}" }}
desc = {{ fg = "{magenta}" }}
separator = "  "
separator_style = {{ fg = "{muted}" }}

[help]
on = {{ fg = "{cyan}" }}
run = {{ fg = "{magenta}" }}
desc = {{ fg = "{fg}", bg = "{bg}" }}
hovered = {{ fg = "{selection_fg}", bg = "{selection_bg}"{active_attrs} }}
footer = {{ fg = "{bg}", bg = "{fg}" }}

[git]
modified = {{ fg = "{yellow}"{modified_attrs} }}
untracked = {{ fg = "{magenta}"{untracked_attrs} }}
added = {{ fg = "{green}" }}
deleted = {{ fg = "{red}" }}
updated = {{ fg = "{accent}" }}
ignored = {{ fg = "{muted}"{ignored_attrs} }}

[filetype]
rules = [
    {{ url = "*/", fg = "{accent}"{active_attrs} }},
    {{ url = "*.rs", fg = "{orange}" }},
    {{ url = "*.py", fg = "{yellow}" }},
    {{ url = "*.js", fg = "{yellow}" }},
    {{ url = "*.ts", fg = "{accent}" }},
    {{ url = "*.sh", fg = "{green}" }},
    {{ url = "*.toml", fg = "{cyan}" }},
    {{ url = "*.yaml", fg = "{cyan}" }},
    {{ url = "*.yml", fg = "{cyan}" }},
    {{ url = "*.json", fg = "{cyan}" }},
    {{ url = "*.kdl", fg = "{cyan}" }},
    {{ url = "*.html", fg = "{orange}" }},
    {{ url = "*.css", fg = "{accent}" }},
    {{ url = "*.md", fg = "{fg}" }},
    {{ url = "*.tex", fg = "{green}" }},
    {{ url = "*.typ", fg = "{green}" }},
    {{ url = "*.pdf", fg = "{red}" }},
    {{ url = "*.png", fg = "{magenta}" }},
    {{ url = "*.jpg", fg = "{magenta}" }},
    {{ url = "*.jpeg", fg = "{magenta}" }},
    {{ url = "*.gif", fg = "{magenta}" }},
    {{ url = "*.svg", fg = "{magenta}" }},
    {{ url = "*.zip", fg = "{orange}" }},
    {{ url = "*.tar*", fg = "{orange}" }},
    {{ url = "*.gz", fg = "{orange}" }},
    {{ url = "*.gitignore", fg = "{muted}" }},
    {{ url = ".env*", fg = "{red}" }},
    {{ url = "Dockerfile*", fg = "{accent}" }},
    {{ url = "Makefile", fg = "{green}" }},
    {{ url = "Cargo.toml", fg = "{orange}"{active_attrs} }},
    {{ url = "Cargo.lock", fg = "{muted}" }},
]
"#
    )
}

fn yazi_surface_color(theme: &Theme) -> &'static str {
    audited_chrome_color(theme, "surface").expect("audited theme must define chrome.surface")
}

/// Returns the lazygit theme YAML snippet (gui.theme section).
#[cfg(test)]
pub fn lazygit_theme(theme: &Theme) -> String {
    lazygit_theme_with_emphasis(theme, ThemeEmphasis::Auto)
}

#[cfg(test)]
pub fn lazygit_theme_with_emphasis(theme: &Theme, emphasis: ThemeEmphasis) -> String {
    lazygit_theme_with_style(theme, emphasis, &BTreeMap::new())
}

pub fn lazygit_theme_with_style(
    theme: &Theme,
    emphasis: ThemeEmphasis,
    overrides: &BTreeMap<String, String>,
) -> String {
    let (bg, fg, accent, _green, red, yellow, _orange, cyan, muted) = theme_palette(theme);
    let selection_bg = audited_chrome_color(theme, "selection_bg").unwrap_or(bg);
    let active_bold =
        if role_attributes(emphasis, "active_foreground", Some(overrides)).contains("bold") {
            "\n      - bold"
        } else {
            ""
        };
    let warning_bold =
        if role_attributes(emphasis, "status_warning", Some(overrides)).contains("bold") {
            "\n      - bold"
        } else {
            ""
        };
    format!(
        r#"gui:
  theme:
    activeBorderColor:
      - '{accent}'{active_bold}
    inactiveBorderColor:
      - '{muted}'
    optionsTextColor:
      - '{cyan}'
    selectedLineBgColor:
      - '{selection_bg}'
    cherryPickedCommitBgColor:
      - '{muted}'
    cherryPickedCommitFgColor:
      - '{accent}'
    unstagedChangesColor:
      - '{red}'
    defaultFgColor:
      - '{fg}'
    searchingActiveBorderColor:
      - '{yellow}'{warning_bold}
"#
    )
}

/// Color palette values for Starship prompt theming.
fn theme_palette(
    theme: &Theme,
) -> (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
) {
    let spec = audited_theme(theme);
    (
        audited_color(spec, "bg"),
        audited_color(spec, "fg"),
        audited_accent_text(theme),
        audited_color(spec, "green"),
        audited_color(spec, "red"),
        audited_color(spec, "yellow"),
        audited_color(spec, "orange"),
        audited_color(spec, "cyan"),
        audited_color(spec, "muted"),
    )
}
/// Generate starship.toml content for the given preset and theme.
pub fn starship_config(preset: &StarshipPreset, theme: &Theme) -> String {
    let (bg, fg, accent, green, red, yellow, orange, cyan, muted) = theme_palette(theme);
    let accent_fill = audited_accent_fill(theme);

    match preset {
        StarshipPreset::Default => format!(
            r#"# aibox starship config — default preset
palette = "aibox"

format = "$directory$git_branch$git_status$python$rust$nodejs$golang$cmd_duration$line_break$character"

[directory]
style = "bold fg:{accent}"
truncation_length = 3

[git_branch]
style = "fg:{green}"

[git_status]
style = "fg:{accent}"

[python]
style = "fg:{yellow}"
format = "[$symbol$version]($style) "

[rust]
style = "fg:{orange}"
format = "[$symbol$version]($style) "

[nodejs]
style = "fg:{green}"
format = "[$symbol$version]($style) "

[golang]
style = "fg:{cyan}"
format = "[$symbol$version]($style) "

[cmd_duration]
style = "fg:{muted}"
min_time = 2_000

[character]
success_symbol = "[❯](bold fg:{green})"
error_symbol = "[❯](bold fg:{red})"

[palettes.aibox]
bg = "{bg}"
fg = "{fg}"
accent = "{accent}"
accent_fill = "{accent_fill}"
"#
        ),

        StarshipPreset::Plain => format!(
            r#"# aibox starship config — plain preset (no Nerd Font needed)
format = "$directory$git_branch$git_status$cmd_duration$line_break$character"

[directory]
style = "bold fg:{accent}"

[git_branch]
symbol = ""
style = "fg:{green}"

[git_status]
style = "fg:{accent}"

[character]
success_symbol = "[>](bold fg:{green})"
error_symbol = "[>](bold fg:{red})"

[python]
symbol = "py "
[rust]
symbol = "rs "
[nodejs]
symbol = "js "
[golang]
symbol = "go "
"#
        ),

        StarshipPreset::Minimal => format!(
            r#"# aibox starship config — minimal preset
format = "$directory$git_branch$line_break$character"

[directory]
style = "bold fg:{accent}"
truncation_length = 2

[git_branch]
style = "fg:{green}"
format = " [$branch]($style)"

[character]
success_symbol = "[❯](fg:{accent})"
error_symbol = "[❯](bold fg:{red})"
"#
        ),

        StarshipPreset::NerdFont => format!(
            r#"# aibox starship config — nerd-font preset
palette = "aibox"

format = "$os$directory$git_branch$git_status$python$rust$nodejs$golang$docker_context$cmd_duration$line_break$character"

[os]
disabled = false
style = "fg:{fg}"

[directory]
style = "bold fg:{accent}"
read_only = " 󰌾"

[git_branch]
symbol = " "
style = "fg:{green}"

[git_status]
style = "fg:{accent}"

[python]
symbol = " "
[rust]
symbol = " "
[nodejs]
symbol = " "
[golang]
symbol = " "
[docker_context]
symbol = " "

[cmd_duration]
style = "fg:{muted}"

[character]
success_symbol = "[❯](bold fg:{green})"
error_symbol = "[❯](bold fg:{red})"

[palettes.aibox]
bg = "{bg}"
fg = "{fg}"
accent = "{accent}"
accent_fill = "{accent_fill}"
"#
        ),

        StarshipPreset::Pastel | StarshipPreset::PastelPowerline => format!(
            r#"# aibox starship config — pastel powerline preset
# One-line prompt inspired by https://starship.rs/presets/#pastel-powerline.
palette = "aibox"

format = """
[](fg:{accent_fill})\
$directory\
[](fg:{accent} bg:{green})\
$git_branch\
$git_status\
[](fg:{green} bg:{orange})\
$python\
$rust\
$nodejs\
$golang\
[](fg:{orange} bg:{bg})\
$cmd_duration\
$character"""

[directory]
style = "bold bg:{accent_fill} fg:{bg}"
format = "[ $path ]($style)"
truncation_length = 3
truncate_to_repo = true

[git_branch]
style = "bg:{green} fg:{bg}"
symbol = " "
format = "[ $symbol$branch ]($style)"

[git_status]
style = "bg:{green} fg:{bg}"
format = "[$all_status$ahead_behind ]($style)"
ahead = "⇡$count"
behind = "⇣$count"
diverged = "⇕⇡$ahead_count⇣$behind_count"
modified = "!$count"
staged = "+$count"
untracked = "?$count"

[python]
style = "bg:{orange} fg:{bg}"
format = "[ $symbol$version ]($style)"
[rust]
style = "bg:{orange} fg:{bg}"
format = "[ $symbol$version ]($style)"
[nodejs]
style = "bg:{orange} fg:{bg}"
format = "[ $symbol$version ]($style)"
[golang]
style = "bg:{orange} fg:{bg}"
format = "[ $symbol$version ]($style)"

[cmd_duration]
style = "fg:{muted}"
min_time = 2_000
format = "[ $duration ]($style)"

[character]
success_symbol = "[❯](bold fg:{accent}) "
error_symbol = "[❯](bold fg:{red}) "

[palettes.aibox]
bg = "{bg}"
fg = "{fg}"
accent = "{accent}"
accent_fill = "{accent_fill}"
"#
        ),

        StarshipPreset::Bracketed => format!(
            r#"# aibox starship config — bracketed segments preset
format = "$directory$git_branch$git_status$python$rust$nodejs$golang$cmd_duration$line_break$character"

[directory]
style = "fg:{accent}"
format = "[$path]($style)[$read_only]($read_only_style) "

[git_branch]
style = "fg:{green}"
format = "[\\[$branch\\]]($style) "

[git_status]
style = "fg:{accent}"
format = "[\\[$all_status$ahead_behind\\]]($style) "

[python]
format = "[\\[$symbol$version\\]](fg:{yellow}) "
[rust]
format = "[\\[$symbol$version\\]](fg:{orange}) "
[nodejs]
format = "[\\[$symbol$version\\]](fg:{green}) "
[golang]
format = "[\\[$symbol$version\\]](fg:{cyan}) "

[cmd_duration]
format = "[\\[$duration\\]](fg:{muted}) "

[character]
success_symbol = "[❯](bold fg:{green})"
error_symbol = "[❯](bold fg:{red})"
"#
        ),

        StarshipPreset::Arrow => format!(
            r#"# aibox starship config — arrow preset (powerline chevron/airline style)
# Requires a Nerd Font or Powerline-patched font for the arrow separators (e0b0/e0b2).
palette = "aibox"

format = """
[](fg:{accent_fill})\
$directory\
[](fg:{accent} bg:{green})\
$git_branch\
$git_status\
[](fg:{green} bg:{bg})\
 $cmd_duration\
$line_break\
$character"""

[directory]
style = "bold bg:{accent_fill} fg:{bg}"
format = "[ $path ]($style)"
truncation_length = 3
truncate_to_repo = true

[git_branch]
style = "bg:{green} fg:{bg}"
symbol = " "
format = "[ $symbol$branch ]($style)"

[git_status]
style = "bg:{green} fg:{bg}"
format = "[$all_status$ahead_behind]($style)"
ahead = "⇡$count"
behind = "⇣$count"
diverged = "⇕⇡$ahead_count⇣$behind_count"
modified = "!$count"
staged = "+$count"
untracked = "?$count"

[cmd_duration]
style = "fg:{muted}"
min_time = 2_000
format = "[ $duration]($style)"

[character]
success_symbol = "[❯](bold fg:{accent})"
error_symbol = "[❯](bold fg:{red})"

[python]
style = "fg:{yellow}"
format = "[$symbol$version]($style) "
[rust]
style = "fg:{orange}"
format = "[$symbol$version]($style) "
[nodejs]
style = "fg:{green}"
format = "[$symbol$version]($style) "
[golang]
style = "fg:{cyan}"
format = "[$symbol$version]($style) "

[palettes.aibox]
bg = "{bg}"
fg = "{fg}"
accent = "{accent}"
accent_fill = "{accent_fill}"
"#
        ),
    }
}

/// Map an aibox theme to a `bat` built-in syntax theme.
pub fn starship_config_with_style(
    preset: &StarshipPreset,
    theme: &Theme,
    emphasis: ThemeEmphasis,
    overrides: &BTreeMap<String, String>,
) -> String {
    let mut body = starship_config(preset, theme);
    if !role_attributes(emphasis, "active_foreground", Some(overrides)).contains("bold") {
        for attribute in [
            "bold ",
            "italic ",
            "underline ",
            "dimmed ",
            "strikethrough ",
        ] {
            body = body.replace(attribute, "");
        }
    }
    body
}
/// Delta consumes the generated custom bat theme.
/// `delta` `syntax-theme` value — matches `bat_theme` since delta consumes bat themes.
pub fn delta_syntax_theme(theme: &Theme) -> &'static str {
    let _ = theme;
    "aibox"
}

/// Generated TextMate theme consumed by bat, delta, and Codex. This removes
/// nearest-built-in fallbacks and gives each consumer the same audited syntax
/// roles and emphasis policy as Vim.
pub fn bat_tmtheme_with_style(
    theme: &Theme,
    emphasis: ThemeEmphasis,
    overrides: &BTreeMap<String, String>,
) -> String {
    let (bg, fg, accent, green, red, yellow, orange, cyan, muted) = theme_palette(theme);
    let magenta = audited_magenta(theme);
    let cursor = audited_chrome_color(theme, "cursor").unwrap_or(accent);
    let selection_fg = audited_chrome_color(theme, "selection_fg").unwrap_or(fg);
    let comment = theme_role_attributes(theme, emphasis, "code_comment", Some(overrides));
    let keyword = theme_role_attributes(theme, emphasis, "code_keyword", Some(overrides));
    let ty = theme_role_attributes(theme, emphasis, "code_type", Some(overrides));
    let function = theme_role_attributes(theme, emphasis, "code_function", Some(overrides));
    let string = theme_role_attributes(theme, emphasis, "code_string", Some(overrides));
    let number = theme_role_attributes(theme, emphasis, "code_number", Some(overrides));
    let operator = theme_role_attributes(theme, emphasis, "code_operator", Some(overrides));
    let decorator = theme_role_attributes(theme, emphasis, "code_decorator", Some(overrides));
    let invalid = theme_role_attributes(theme, emphasis, "code_invalid", Some(overrides));
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>name</key><string>aibox</string>
<key>settings</key><array>
<dict><key>settings</key><dict><key>background</key><string>{bg}</string><key>foreground</key><string>{fg}</string><key>caret</key><string>{cursor}</string><key>selection</key><string>{surface}</string><key>selectionForeground</key><string>{selection_fg}</string></dict></dict>
<dict><key>scope</key><string>comment</string><key>settings</key><dict><key>foreground</key><string>{muted}</string><key>fontStyle</key><string>{comment}</string></dict></dict>
<dict><key>scope</key><string>keyword, storage</string><key>settings</key><dict><key>foreground</key><string>{magenta}</string><key>fontStyle</key><string>{keyword}</string></dict></dict>
<dict><key>scope</key><string>entity.name.type, support.type, storage.type</string><key>settings</key><dict><key>foreground</key><string>{yellow}</string><key>fontStyle</key><string>{ty}</string></dict></dict>
<dict><key>scope</key><string>entity.name.function, support.function</string><key>settings</key><dict><key>foreground</key><string>{cyan}</string><key>fontStyle</key><string>{function}</string></dict></dict>
<dict><key>scope</key><string>string</string><key>settings</key><dict><key>foreground</key><string>{green}</string><key>fontStyle</key><string>{string}</string></dict></dict>
<dict><key>scope</key><string>constant.numeric, constant.language</string><key>settings</key><dict><key>foreground</key><string>{orange}</string><key>fontStyle</key><string>{number}</string></dict></dict>
<dict><key>scope</key><string>keyword.operator, punctuation</string><key>settings</key><dict><key>foreground</key><string>{cyan}</string><key>fontStyle</key><string>{operator}</string></dict></dict>
<dict><key>scope</key><string>meta.annotation, entity.name.tag</string><key>settings</key><dict><key>foreground</key><string>{accent}</string><key>fontStyle</key><string>{decorator}</string></dict></dict>
<dict><key>scope</key><string>invalid</string><key>settings</key><dict><key>foreground</key><string>{red}</string><key>fontStyle</key><string>{invalid}</string></dict></dict>
</array></dict></plist>
"#,
        surface = audited_chrome_color(theme, "selection_bg").unwrap_or(bg)
    )
}
/// fzf `--color` clause string (without the leading `--color=`).
#[cfg(test)]
pub fn fzf_color_spec(theme: &Theme) -> String {
    fzf_color_spec_with_emphasis(theme, ThemeEmphasis::Auto)
}

#[cfg(test)]
pub fn fzf_color_spec_with_emphasis(theme: &Theme, emphasis: ThemeEmphasis) -> String {
    fzf_color_spec_with_style(theme, emphasis, &BTreeMap::new())
}

pub fn fzf_color_spec_with_style(
    theme: &Theme,
    emphasis: ThemeEmphasis,
    overrides: &BTreeMap<String, String>,
) -> String {
    let (bg, fg, accent, green, _red, _yellow, _orange, _cyan, muted) = theme_palette(theme);
    let selection_bg =
        audited_chrome_color(theme, "selection_bg").unwrap_or_else(|| yazi_surface_color(theme));
    let selection_fg = audited_chrome_color(theme, "selection_fg").unwrap_or(fg);
    let active = if role_attributes(emphasis, "active_foreground", Some(overrides)).contains("bold")
    {
        ":bold"
    } else {
        ""
    };
    let inactive =
        if role_attributes(emphasis, "inactive_foreground", Some(overrides)).contains("dim") {
            ":dim"
        } else {
            ""
        };
    format!(
        "bg+:{selection_bg},bg:{bg},fg:{fg},fg+:{selection_fg}{active},hl:{accent},hl+:{accent}{active},\
pointer:{accent},marker:{green},spinner:{accent},info:{muted},header:{muted},\
border:{muted},prompt:{accent}{active},query:{fg},disabled:{muted}{inactive},gutter:{bg},\
preview-bg:{bg},preview-fg:{fg},separator:{muted},label:{accent}"
    )
}

/// EZA_COLORS env value. The format is a colon-separated list of `key=spec`
/// items where `spec` is an ANSI SGR sequence. We map a small but visible set
/// of file categories to palette slots so `ls` output coheres with the rest.
pub fn eza_colors_spec_with_style(
    theme: &Theme,
    emphasis: ThemeEmphasis,
    overrides: &BTreeMap<String, String>,
) -> String {
    let (_bg, _fg, accent, green, red, yellow, orange, cyan, muted) = theme_palette(theme);
    let a = hex_to_sgr_fg(accent);
    let g = hex_to_sgr_fg(green);
    let r = hex_to_sgr_fg(red);
    let y = hex_to_sgr_fg(yellow);
    let o = hex_to_sgr_fg(orange);
    let c = hex_to_sgr_fg(cyan);
    let m = hex_to_sgr_fg(muted);
    let bold = if !role_attributes(emphasis, "git_modified", Some(overrides)).contains("bold") {
        ""
    } else {
        ";1"
    };
    let italic = if role_attributes(emphasis, "git_untracked", Some(overrides)).contains("italic") {
        ";3"
    } else {
        ""
    };
    let dim = if !role_attributes(emphasis, "git_ignored", Some(overrides)).contains("dim") {
        ""
    } else {
        ";2"
    };
    // di=directory, ex=executable, ln=symlink, fi=regular file, *.rs/*.py etc.
    // git/perm columns and size units use accent/muted to match the prompt.
    format!(
        "di={a}{bold}:ex={g}:ln={c}:fi=0:or={r}{bold}:mi={r}{bold}:\
da={m}{dim}:sn={a}:sb={a}:uu={m}:un={m}:gu={m}:gn={m}:\
ga={g}{bold}:gm={y}{bold}:gd={r}{bold}:gv={y}:gt={o}{italic}:\
xx={m}{dim}:da={m}{dim}"
    )
}

fn hex_to_sgr_fg(hex: &str) -> String {
    if let Some((r, g, b)) = parse_hex_rgb(hex) {
        format!("38;2;{r};{g};{b}")
    } else {
        "0".to_string()
    }
}

/// Generate `~/.config/aibox/theme-env.sh` content: exports BAT_THEME,
/// FZF_DEFAULT_OPTS, EZA_COLORS, LESS_TERMCAP_*, and a few helper LS_COLORS
/// hints. Sourced from `~/.bashrc` if present.
#[cfg(test)]
pub fn theme_env_script(theme: &Theme) -> String {
    theme_env_script_with_emphasis(theme, ThemeEmphasis::Auto)
}

#[cfg(test)]
pub fn theme_env_script_with_emphasis(theme: &Theme, emphasis: ThemeEmphasis) -> String {
    theme_env_script_with_style(theme, emphasis, &BTreeMap::new())
}

pub fn theme_env_script_with_style(
    theme: &Theme,
    emphasis: ThemeEmphasis,
    overrides: &BTreeMap<String, String>,
) -> String {
    let bat = "aibox";
    let fzf = fzf_color_spec_with_style(theme, emphasis, overrides);
    let eza = eza_colors_spec_with_style(theme, emphasis, overrides);
    let (_bg, _fg, accent, _green, red, yellow, _orange, _cyan, _muted) = theme_palette(theme);
    let acc = hex_to_sgr_fg(accent);
    let yel = hex_to_sgr_fg(yellow);
    let red = hex_to_sgr_fg(red);
    let (bold, underline, reverse) = if resolved_emphasis(emphasis) == ThemeEmphasis::None {
        ("", "", "")
    } else {
        (";1", ";4", ";7")
    };
    format!(
        r#"# Generated by `aibox` — do NOT edit. Re-run `aibox theme` to refresh.
# Exports keep ANSI-colored tools (bat, delta, fzf, eza, less) aligned with
# the active aibox theme.

export BAT_THEME="{bat}"
# Bat indexes custom TextMate themes into its cache. Rebuild only when the
# generated aibox theme is not present (normally once after apply/theme switch).
if command -v bat >/dev/null 2>&1 && ! bat --list-themes 2>/dev/null | grep -qx 'aibox'; then
    bat cache --build >/dev/null 2>&1 || true
fi
export FZF_DEFAULT_OPTS="${{FZF_DEFAULT_OPTS:-}} --color={fzf}"
export EZA_COLORS="{eza}"

# `less` headings / search hits — match prompt accent / warnings.
export LESS_TERMCAP_md=$'\e[{acc}{bold}m'    # section headers, command names
export LESS_TERMCAP_us=$'\e[{yel}{underline}m'    # options, args
export LESS_TERMCAP_so=$'\e[{acc}{reverse}m'  # search / status line
export LESS_TERMCAP_me=$'\e[0m'
export LESS_TERMCAP_ue=$'\e[0m'
export LESS_TERMCAP_se=$'\e[0m'
export GROFF_NO_SGR=1                  # tell groff to emit ANSI, not SGR-bypass
unset LESS_TERMCAP_mb || true
# Keep an obvious error color available for tools that look for $AIBOX_ERROR_COLOR.
export AIBOX_ERROR_SGR='{red}'
"#
    )
}

/// Generate `~/.config/lnav/config.json` content: pins lnav to the closest
/// matching built-in theme.
#[cfg(test)]
pub fn lnav_config(theme: &Theme) -> String {
    lnav_config_with_emphasis(theme, ThemeEmphasis::Auto)
}

#[cfg(test)]
pub fn lnav_config_with_emphasis(theme: &Theme, emphasis: ThemeEmphasis) -> String {
    lnav_config_with_style(theme, emphasis, &BTreeMap::new())
}

pub fn lnav_config_with_style(
    theme: &Theme,
    emphasis: ThemeEmphasis,
    overrides: &BTreeMap<String, String>,
) -> String {
    let (bg, fg, accent, green, red, yellow, orange, cyan, muted) = theme_palette(theme);
    let accent_fill = audited_accent_fill(theme);
    let active_ink = audited_chrome_color(theme, "status_active_ink").unwrap_or(bg);
    let surface = yazi_surface_color(theme);
    let selection_bg = audited_chrome_color(theme, "selection_bg").unwrap_or(surface);
    let selection_fg = audited_chrome_color(theme, "selection_fg").unwrap_or(fg);
    let has =
        |role: &str, attr: &str| role_attributes(emphasis, role, Some(overrides)).contains(attr);
    let doc = serde_json::json!({
        "$schema": "https://lnav.org/schemas/config-v1.schema.json",
        "ui": {
            "theme": "aibox",
            "theme-defs": {
                "aibox": {
                    "vars": { "black": bg, "red": red, "green": green, "yellow": yellow, "blue": accent, "magenta": orange, "cyan": cyan, "white": fg },
                    "styles": {
                        "text": { "color": fg, "background-color": bg },
                        "selected-text": { "color": selection_fg, "background-color": selection_bg },
                        "identifier": { "color": accent },
                        "alt-text": { "color": muted, "italic": has("code_comment", "italic") },
                        "ok": { "color": green },
                        "info": { "color": cyan },
                        "warning": { "color": yellow, "bold": has("status_warning", "bold") },
                        "error": { "color": red, "bold": has("status_error", "bold") },
                        "invalid-msg": { "color": red, "bold": has("code_invalid", "bold"), "underline": has("code_invalid", "underline") },
                        "popup": { "color": fg, "background-color": surface },
                        "table-border": { "color": accent },
                        "focused": { "color": selection_fg, "background-color": selection_bg },
                        "disabled-focused": { "color": muted, "background-color": surface }
                    },
                    "status-styles": {
                        "title": { "color": active_ink, "background-color": accent_fill, "bold": has("active_foreground", "bold") },
                        "text": { "color": fg, "background-color": surface },
                        "warn": { "color": yellow, "background-color": surface, "bold": has("status_warning", "bold") },
                        "alert": { "color": red, "background-color": surface, "bold": has("status_error", "bold") },
                        "active": { "color": green, "background-color": surface },
                        "info": { "color": cyan, "background-color": surface },
                        "inactive": { "color": muted, "background-color": surface }
                    }
                }
            }
        }
    });
    format!(
        "{}\n",
        serde_json::to_string_pretty(&doc).expect("lnav theme JSON serializes")
    )
}

/// Generate a tmux-powerkit custom theme file from the aibox palette.
///
/// Loaded by powerkit via `@powerkit_theme "custom"` +
/// `@powerkit_custom_theme_path`. Producing this file ourselves means the
/// status-bar chevron separators always end on the exact background color
/// aibox already uses — no more "strange colored separators" when the upstream
/// powerkit theme variant doesn't quite match our palette (Projectious in
/// particular previously fell back to tokyo-night and rendered chevrons in
/// tokyo-night's bg).
#[cfg(test)]
pub fn tmux_powerkit_custom_theme(theme: &Theme) -> String {
    tmux_powerkit_custom_theme_with_emphasis(theme, ThemeEmphasis::Auto)
}

#[cfg(test)]
pub fn tmux_powerkit_custom_theme_with_emphasis(theme: &Theme, emphasis: ThemeEmphasis) -> String {
    tmux_powerkit_custom_theme_with_style(theme, emphasis, &BTreeMap::new())
}

pub fn tmux_powerkit_custom_theme_with_style(
    theme: &Theme,
    emphasis: ThemeEmphasis,
    overrides: &BTreeMap<String, String>,
) -> String {
    let (bg, fg, accent, green, red, yellow, orange, cyan, muted) = theme_palette(theme);
    let accent_fill = audited_accent_fill(theme);
    let surface = yazi_surface_color(theme);
    let magenta = audited_magenta(theme);
    let border_active = audited_chrome_color(theme, "border_active").unwrap_or(accent_fill);
    let border_inactive = audited_chrome_color(theme, "border_inactive").unwrap_or(muted);
    let active_ink = audited_chrome_color(theme, "status_active_ink").unwrap_or(bg);
    let active_style = tmux_role_attributes(emphasis, "active_foreground", Some(overrides));
    let inactive_style = tmux_role_attributes(emphasis, "inactive_foreground", Some(overrides));
    let error_style = tmux_role_attributes(emphasis, "status_error", Some(overrides));
    // A "neutral OK" tone: muted accent — keeps non-warning segments calm
    // without making them disappear against the surface.
    let ok = mix_hex_colors(accent, muted, 35);
    format!(
        r#"#!/usr/bin/env bash
# tmux-powerkit theme — generated by aibox CLI from the active theme palette.
# Loaded via @powerkit_theme "custom" + @powerkit_custom_theme_path.
# Do NOT edit; re-run `aibox theme` to regenerate.

declare -gA THEME_COLORS=(
    # Core
    [background]="{bg}"

    # Status bar
    [statusbar-bg]="{surface}"
    [statusbar-fg]="{fg}"

    # Session (status-left)
    [session-bg]="{accent_fill}"
    [session-fg]="{active_ink}"
    [session-prefix-bg]="{red}"
    [session-copy-bg]="{cyan}"
    [session-search-bg]="{yellow}"
    [session-command-bg]="{magenta}"

    # Windows
    [window-active-base]="{accent_fill}"
    [window-active-style]="{active_style}"
    [window-inactive-base]="{muted}"
    [window-inactive-style]="{inactive_style}"
    [window-activity-style]="{inactive_style}"
    [window-bell-style]="{error_style}"
    [window-zoomed-bg]="{cyan}"

    # Panes
    [pane-border-active]="{border_active}"
    [pane-border-inactive]="{border_inactive}"

    # Health / state segments (chevron color rotation source)
    [ok-base]="{ok}"
    [good-base]="{green}"
    [info-base]="{cyan}"
    [warning-base]="{yellow}"
    [error-base]="{red}"
    [disabled-base]="{muted}"

    # Messages
    [message-bg]="{surface}"
    [message-fg]="{fg}"

    # Popup & menu
    [popup-bg]="{surface}"
    [popup-fg]="{fg}"
    [popup-border]="{accent}"
    [menu-bg]="{surface}"
    [menu-fg]="{fg}"
    [menu-selected-bg]="{accent_fill}"
    [menu-selected-fg]="{active_ink}"
    [menu-border]="{accent}"
)

# Extra slots some plugins read directly — also derived from the palette so
# every chevron lands on a known-good bg.
declare -gA THEME_EXTRA=(
    [orange]="{orange}"
    [magenta]="{magenta}"
    [surface]="{surface}"
)
"#
    )
}

/// Whether the theme is best rendered with a dark or light TUI background.
pub fn is_light_theme(theme: &Theme) -> bool {
    matches!(vim_background(theme), "light")
}

/// Claude Code `~/.claude/settings.json` `theme` field.
/// Claude supports: "dark", "light", "dark-daltonized", "light-daltonized",
/// "dark-ansi", "light-ansi", "system". We map to light/dark only.
pub fn claude_theme(theme: &Theme) -> &'static str {
    match theme {
        Theme::MonoLight
        | Theme::ContrastLight
        | Theme::ContrastLightMax
        | Theme::ContrastMonoLight
        | Theme::ContrastMonoLightMax => "light-ansi",
        Theme::MonoDark
        | Theme::ContrastDark
        | Theme::ContrastDarkMax
        | Theme::ContrastMonoDark
        | Theme::ContrastMonoDarkMax => "dark-ansi",
        _ if is_light_theme(theme) => "light",
        _ => "dark",
    }
}

/// Aider `code-theme` (a Pygments style). Aider does NOT understand most named
/// theme palettes; it picks a code-highlight Pygments theme + a dark/light
/// background flag. We map to the closest Pygments style.
pub fn aider_code_theme(theme: &Theme) -> &'static str {
    match theme {
        Theme::GruvboxDark => "gruvbox-dark",
        Theme::SolarizedDark => "solarized-dark",
        Theme::SolarizedLight => "solarized-light",
        Theme::Dracula => "dracula",
        Theme::GithubDark => "github-dark",
        Theme::GithubLight => "default",
        Theme::Nord => "nord",
        Theme::CatppuccinLatte
        | Theme::TokyoNightDay
        | Theme::RosePineDawn
        | Theme::MaterialLighter
        | Theme::AyuLight
        | Theme::NightOwlLight
        | Theme::OneLight
        | Theme::VitesseLight
        | Theme::MinLight
        | Theme::KanagawaLotus
        | Theme::EverforestLight
        | Theme::VsCodeLightPlus
        | Theme::GithubLightHighContrast
        | Theme::SlackOchin
        | Theme::SnazzyLight
        | Theme::GruvboxLight
        | Theme::ProjectiousLight
        | Theme::ProjectiousHCLight
        | Theme::MonoLight
        | Theme::ContrastLight
        | Theme::ContrastLightMax
        | Theme::ContrastMonoLight
        | Theme::ContrastMonoLightMax => "default",
        Theme::DraculaSoft => "dracula",
        Theme::Monokai | Theme::OneDarkPro | Theme::Plastic => "monokai",
        Theme::MonoDark
        | Theme::ContrastDark
        | Theme::ContrastDarkMax
        | Theme::ContrastMonoDark
        | Theme::ContrastMonoDarkMax
        | Theme::ProjectiousHCDark => "github-dark",
        _ => "monokai",
    }
}

/// Gemini CLI `~/.gemini/settings.json` `theme` value. Gemini ships a fixed
/// set of named themes. We map each aibox theme to its closest match.
pub fn gemini_theme(theme: &Theme) -> &'static str {
    match theme {
        Theme::Dracula | Theme::DraculaSoft => "Dracula",
        Theme::GithubDark | Theme::GithubDarkDimmed | Theme::GithubDarkHighContrast => "GitHub",
        Theme::GithubLight | Theme::GithubLightHighContrast => "GitHub Light",
        Theme::AyuDark | Theme::AyuMirage => "Ayu",
        Theme::AyuLight => "Ayu Light",
        Theme::Monokai => "Monokai",
        Theme::CatppuccinLatte
        | Theme::GruvboxLight
        | Theme::TokyoNightDay
        | Theme::RosePineDawn
        | Theme::MaterialLighter
        | Theme::SolarizedLight
        | Theme::NightOwlLight
        | Theme::OneLight
        | Theme::VitesseLight
        | Theme::MinLight
        | Theme::KanagawaLotus
        | Theme::EverforestLight
        | Theme::VsCodeLightPlus
        | Theme::SlackOchin
        | Theme::SnazzyLight => "Default Light",
        Theme::NightOwl => "Atom One Dark",
        _ => "Default",
    }
}
/// Native OpenCode custom theme. OpenCode's schema is color-only, so semantic
/// emphasis remains encoded through the audited high-contrast palette.
pub fn opencode_custom_theme(theme: &Theme) -> String {
    let (bg, fg, accent, green, red, yellow, orange, cyan, muted) = theme_palette(theme);
    let magenta = audited_magenta(theme);
    let surface = yazi_surface_color(theme);
    let border = audited_chrome_color(theme, "border_inactive").unwrap_or(muted);
    let border_active = audited_chrome_color(theme, "border_active").unwrap_or(accent);
    let value = |color: &str| serde_json::json!({ "dark": color, "light": color });
    let doc = serde_json::json!({
        "$schema": "https://opencode.ai/theme.json",
        "defs": { "bg": bg, "fg": fg, "accent": accent, "green": green, "red": red, "yellow": yellow, "orange": orange, "cyan": cyan, "muted": muted, "magenta": magenta, "surface": surface, "border": border, "borderActive": border_active },
        "theme": {
            "primary": value(accent), "secondary": value(cyan), "accent": value(magenta),
            "error": value(red), "warning": value(yellow), "success": value(green), "info": value(cyan),
            "text": value(fg), "textMuted": value(muted), "background": value(bg),
            "backgroundPanel": value(surface), "backgroundElement": value(surface),
            "border": value(border), "borderActive": value(border_active), "borderSubtle": value(border),
            "diffAdded": value(green), "diffRemoved": value(red),
            "markdownEmph": value(cyan), "markdownStrong": value(yellow),
            "markdownHorizontalRule": value(muted), "markdownListItem": value(accent),
            "markdownListEnumeration": value(magenta), "markdownImage": value(cyan),
            "markdownImageText": value(accent), "markdownCodeBlock": value(fg),
            "syntaxComment": value(muted), "syntaxKeyword": value(magenta),
            "syntaxFunction": value(cyan), "syntaxVariable": value(fg),
            "syntaxString": value(green), "syntaxNumber": value(orange),
            "syntaxType": value(yellow), "syntaxOperator": value(cyan), "syntaxPunctuation": value(fg)
        }
    });
    format!(
        "{}\n",
        serde_json::to_string_pretty(&doc).expect("OpenCode theme JSON serializes")
    )
}

/// Git config snippet that wires up delta as the diff/log/show pager. Emitted
/// in place of the previous minimal gitconfig so `git diff`, `git log -p` and
/// lazygit hunks all render with theme-matched syntax highlighting.
#[cfg(test)]
pub fn gitconfig_with_delta_and_emphasis(theme: &Theme, emphasis: ThemeEmphasis) -> String {
    gitconfig_with_delta_and_style(theme, emphasis, &BTreeMap::new())
}

pub fn gitconfig_with_delta_and_style(
    theme: &Theme,
    emphasis: ThemeEmphasis,
    overrides: &BTreeMap<String, String>,
) -> String {
    let (_bg, _fg, accent, _green, _red, _yellow, _orange, _cyan, muted) = theme_palette(theme);
    let syntax = delta_syntax_theme(theme);
    let dark = matches!(vim_background(theme), "dark");
    let light_flag = if dark { "false" } else { "true" };
    let diff_add_bg = audited_chrome_color(theme, "diff_add_bg").unwrap_or("#1F3B25");
    let diff_del_bg = audited_chrome_color(theme, "diff_del_bg").unwrap_or("#3B1F22");
    let diff_change_bg = audited_chrome_color(theme, "diff_change_bg").unwrap_or("#313244");
    let diff_bold = if role_attributes(emphasis, "diff_emphasis", Some(overrides)).contains("bold")
    {
        " bold"
    } else {
        ""
    };
    let header_bold = if role_attributes(emphasis, "diff_header", Some(overrides)).contains("bold")
    {
        " bold"
    } else {
        ""
    };
    format!(
        r##"[core]
    editor = vim
    pager = delta
[init]
    defaultBranch = main
[pull]
    rebase = true
[interactive]
    diffFilter = delta --color-only
[delta]
    syntax-theme = {syntax}
    light = {light_flag}
    navigate = true
    line-numbers = true
    side-by-side = false
    hunk-header-decoration-style = "{muted}" box{header_bold}
    file-decoration-style = "{accent}" ul{header_bold}
    minus-style = syntax "{diff_del_bg}"
    minus-emph-style = syntax "{diff_del_bg}"{diff_bold}
    plus-style = syntax "{diff_add_bg}"
    plus-emph-style = syntax "{diff_add_bg}"{diff_bold}
    zero-style = syntax "{diff_change_bg}"
[merge]
    conflictStyle = zdiff3
"##
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::StarshipPreset;

    /// Every theme this crate exposes, used by the matrix tests below to
    /// guarantee no variant silently regresses.
    const ALL_THEMES: &[Theme] = &[
        Theme::GruvboxDark,
        Theme::GruvboxLight,
        Theme::CatppuccinMocha,
        Theme::CatppuccinMacchiato,
        Theme::CatppuccinFrappe,
        Theme::CatppuccinLatte,
        Theme::Dracula,
        Theme::DraculaSoft,
        Theme::TokyoNight,
        Theme::TokyoNightStorm,
        Theme::TokyoNightDay,
        Theme::Nord,
        Theme::RosePine,
        Theme::RosePineMoon,
        Theme::RosePineDawn,
        Theme::Material,
        Theme::MaterialOcean,
        Theme::MaterialPalenight,
        Theme::MaterialLighter,
        Theme::MaterialDarker,
        Theme::SolarizedDark,
        Theme::SolarizedLight,
        Theme::GithubDark,
        Theme::GithubLight,
        Theme::GithubDarkDimmed,
        Theme::GithubDarkHighContrast,
        Theme::GithubLightHighContrast,
        Theme::AyuDark,
        Theme::AyuMirage,
        Theme::AyuLight,
        Theme::NightOwl,
        Theme::NightOwlLight,
        Theme::Moonlight,
        Theme::Projectious,
        // New themes
        Theme::Andromeeda,
        Theme::AuroraX,
        Theme::EverforestDark,
        Theme::EverforestLight,
        Theme::Houston,
        Theme::KanagawaWave,
        Theme::KanagawaDragon,
        Theme::KanagawaLotus,
        Theme::Laserwave,
        Theme::MinDark,
        Theme::MinLight,
        Theme::Monokai,
        Theme::OneDarkPro,
        Theme::OneLight,
        Theme::Plastic,
        Theme::Poimandres,
        Theme::Red,
        Theme::SlackDark,
        Theme::SlackOchin,
        Theme::SnazzyLight,
        Theme::Synthwave84,
        Theme::Vesper,
        Theme::VitesseDark,
        Theme::VitesseLight,
        Theme::VitesseBlack,
        Theme::VsCodeDarkPlus,
        Theme::VsCodeLightPlus,
        Theme::ProjectiousNavy,
        Theme::ProjectiousDeep,
        Theme::ProjectiousLight,
        Theme::ProjectiousHCDark,
        Theme::ProjectiousHCLight,
        Theme::MonoDark,
        Theme::MonoLight,
        Theme::ContrastDark,
        Theme::ContrastDarkMax,
        Theme::ContrastLight,
        Theme::ContrastLightMax,
        Theme::ContrastMonoDark,
        Theme::ContrastMonoDarkMax,
        Theme::ContrastMonoLight,
        Theme::ContrastMonoLightMax,
    ];

    fn relative_luminance(color: &str) -> f64 {
        let (r, g, b) = parse_hex_rgb(color).expect("test colors must be #RRGGBB");
        let linear = |channel: u8| {
            let value = f64::from(channel) / 255.0;
            if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(r) + 0.7152 * linear(g) + 0.0722 * linear(b)
    }

    fn contrast_ratio(first: &str, second: &str) -> f64 {
        let first = relative_luminance(first);
        let second = relative_luminance(second);
        (first.max(second) + 0.05) / (first.min(second) + 0.05)
    }

    fn assert_contrast(theme: &Theme, role: &str, fg: &str, bg: &str, floor: f64) {
        let ratio = contrast_ratio(fg, bg);
        assert!(
            ratio + 0.005 >= floor,
            "theme {theme} role {role}: {fg} on {bg} is {ratio:.2}:1, below {floor:.2}:1"
        );
    }

    #[test]
    fn every_audited_role_meets_its_contrast_floor() {
        for theme in ALL_THEMES {
            let spec = audited_theme(theme);
            let bg = audited_color(spec, "bg");
            for (role, floor) in [
                ("fg", 7.0),
                ("green", 4.5),
                ("red", 4.5),
                ("yellow", 4.5),
                ("orange", 4.5),
                ("cyan", 4.5),
                ("muted", 4.5),
            ] {
                assert_contrast(theme, role, audited_color(spec, role), bg, floor);
            }
            assert_contrast(theme, "accent text", audited_accent_text(theme), bg, 4.5);
            assert_contrast(theme, "magenta", audited_magenta(theme), bg, 4.5);

            let chrome = &spec["chrome"];
            let surface = audited_color(chrome, "surface");
            // Projectious surfaces are authored brand ramp steps rather than
            // generated status surfaces. Keep those exact v2.1.1 tokens; the
            // imported and accessibility families must satisfy the 1.2 floor.
            if spec["family"].as_str() != Some("projectious") {
                assert_contrast(theme, "surface", surface, bg, 1.2);
            }

            let selection_bg = audited_color(chrome, "selection_bg");
            let selection_fg = audited_color(chrome, "selection_fg");
            let selection_surface_floor = if spec["mode"].as_str() == Some("light") {
                1.4
            } else {
                // The authored Mono selection is 1.72:1; imported dark
                // palettes target 1.8 and all remain above this hard floor.
                1.7
            };
            assert_contrast(
                theme,
                "selection surface",
                selection_bg,
                bg,
                selection_surface_floor,
            );
            assert_contrast(theme, "selection ink", selection_fg, selection_bg, 4.5);

            if let (Some(cursor), Some(cursor_text)) = (
                chrome.get("cursor").and_then(toml::Value::as_str),
                chrome.get("cursor_text").and_then(toml::Value::as_str),
            ) {
                assert_contrast(theme, "cursor ink", cursor_text, cursor, 4.5);
            }
            for role in ["border_active", "border_inactive"] {
                if let Some(color) = chrome.get(role).and_then(toml::Value::as_str) {
                    assert_contrast(theme, role, color, bg, 3.0);
                }
            }
            if let (Some(pane_bg), Some(pane_fg)) = (
                chrome.get("pane_inactive_bg").and_then(toml::Value::as_str),
                chrome.get("pane_inactive_fg").and_then(toml::Value::as_str),
            ) {
                assert_contrast(theme, "pane inactive", pane_fg, pane_bg, 3.0);
            }

            let active_ink = audited_color(chrome, "status_active_ink");
            assert_contrast(
                theme,
                "active status ink",
                active_ink,
                audited_accent_fill(theme),
                4.5,
            );
            let (pane_bg, pane_fg) = dim_inactive_pane_colors(theme);
            assert_contrast(theme, "generated inactive pane", &pane_fg, &pane_bg, 3.0);
        }
    }

    #[test]
    fn generated_tools_share_the_audited_active_ink_and_function_role() {
        for theme in ALL_THEMES {
            let spec = audited_theme(theme);
            let chrome = &spec["chrome"];
            let active_ink = audited_color(chrome, "status_active_ink");
            let accent_fill = audited_accent_fill(theme);
            let cyan = audited_color(spec, "cyan");

            let vim = vim_aibox_colorscheme(theme);
            assert!(
                vim.contains(&format!(
                    "hi StatusLine     guifg={active_ink} guibg={accent_fill}"
                )),
                "Vim active status differs from the audited source for {theme}"
            );
            assert!(
                vim.contains(&format!("hi Function       guifg={cyan}")),
                "Vim function role differs from the reference sample for {theme}"
            );

            let yazi = yazi_theme_with_separator(theme, "powerline");
            assert!(
                yazi.contains(&format!(
                    "active = {{ fg = \"{active_ink}\", bg = \"{accent_fill}\""
                )),
                "Yazi active tab differs from the audited source for {theme}"
            );
            assert!(yazi.contains("[indicator]"));
            assert!(yazi.contains(&format!(
                "current = {{ fg = \"{accent_fill}\", bg = \"{accent_fill}\""
            )));

            let bat = bat_tmtheme_with_style(theme, ThemeEmphasis::Auto, &BTreeMap::new());
            assert!(
                bat.contains(&format!(
                    "<string>entity.name.function, support.function</string><key>settings</key><dict><key>foreground</key><string>{cyan}</string>"
                )),
                "bat/delta function role differs from the reference sample for {theme}"
            );

            let opencode = opencode_custom_theme(theme);
            let opencode: serde_json::Value =
                serde_json::from_str(&opencode).expect("generated OpenCode theme is valid JSON");
            assert_eq!(
                opencode["theme"]["syntaxFunction"]["dark"].as_str(),
                Some(cyan),
                "OpenCode function role differs from the reference sample for {theme}"
            );

            let tmux = tmux_powerkit_custom_theme(theme);
            assert!(tmux.contains(&format!("[session-fg]=\"{active_ink}\"")));
            assert!(tmux.contains(&format!("[session-bg]=\"{accent_fill}\"")));
        }
    }

    #[test]
    fn decoration_overrides_are_clamped_and_reach_generated_tools() {
        let mut overrides = BTreeMap::new();
        overrides.insert("code_comment".to_string(), "bold underline".to_string());
        overrides.insert("status_error".to_string(), "underline".to_string());

        let vim =
            vim_aibox_colorscheme_with_style(&Theme::GruvboxDark, ThemeEmphasis::Full, &overrides);
        assert!(vim.contains("hi Comment        guifg=#A89984    gui=bold,underline"));
        assert!(vim.contains("hi ErrorMsg       guifg=#FB5440    gui=underline"));

        let bat = bat_tmtheme_with_style(&Theme::GruvboxDark, ThemeEmphasis::Minimal, &overrides);
        assert!(bat.contains("<key>fontStyle</key><string>bold</string>"));
        assert!(!bat.contains("bold underline"));

        assert_eq!(
            role_attributes(ThemeEmphasis::Standard, "status_error", Some(&overrides)),
            "bold",
            "underline must degrade to bold at standard"
        );
        assert_eq!(
            role_attributes(ThemeEmphasis::None, "code_comment", Some(&overrides)),
            "",
            "none must clamp even explicit overrides"
        );
    }

    #[test]
    fn vim_colorscheme_omits_unsupported_dim_attribute() {
        for emphasis in [ThemeEmphasis::Standard, ThemeEmphasis::Minimal] {
            let vim =
                vim_aibox_colorscheme_with_style(&Theme::GruvboxDark, emphasis, &BTreeMap::new());
            assert!(!vim.lines().any(|line| {
                line.split_whitespace().any(|field| {
                    field
                        .strip_prefix("gui=")
                        .is_some_and(|attrs| attrs.split(',').any(|attr| attr == "dim"))
                })
            }));
        }
    }

    #[test]
    fn accessibility_theme_authored_attributes_reach_syntax_renderers() {
        let vim = vim_aibox_colorscheme_with_style(
            &Theme::MonoDark,
            ThemeEmphasis::Full,
            &BTreeMap::new(),
        );
        assert!(vim.contains("hi Operator       guifg=#BDBDBD      gui=italic"));
        assert!(vim.contains("hi Keyword        guifg=#A3A3A3  gui=bold"));

        let bat = bat_tmtheme_with_style(
            &Theme::ContrastMonoDarkMax,
            ThemeEmphasis::Full,
            &BTreeMap::new(),
        );
        assert!(bat.contains("<string>keyword.operator, punctuation</string>"));
        assert!(bat.contains("<key>fontStyle</key><string>italic</string>"));
    }

    #[test]
    fn every_exposed_theme_has_complete_audited_semantic_data() {
        use clap::ValueEnum as _;
        assert_eq!(ALL_THEMES.len(), Theme::value_variants().len());
        for theme in ALL_THEMES {
            let spec = audited_theme(theme);
            for key in [
                "bg", "fg", "accent", "green", "red", "yellow", "orange", "cyan", "muted",
            ] {
                assert!(
                    audited_color(spec, key).starts_with('#'),
                    "{theme}: missing {key}"
                );
            }
            assert!(
                audited_magenta(theme).starts_with('#'),
                "{theme}: missing magenta"
            );
            assert!(
                audited_chrome_color(theme, "surface").is_some(),
                "{theme}: missing chrome.surface"
            );
            assert!(
                audited_chrome_color(theme, "selection_bg").is_some(),
                "{theme}: missing selection background"
            );
            assert!(
                audited_chrome_color(theme, "selection_fg").is_some(),
                "{theme}: missing selection foreground"
            );
            assert!(
                audited_chrome_color(theme, "cursor").is_some(),
                "{theme}: missing cursor"
            );
            assert!(
                audited_chrome_color(theme, "cursor_text").is_some(),
                "{theme}: missing cursor text"
            );
        }
    }

    const ALL_PROMPTS: &[StarshipPreset] = &[
        StarshipPreset::Default,
        StarshipPreset::Plain,
        StarshipPreset::Minimal,
        StarshipPreset::NerdFont,
        StarshipPreset::Pastel,
        StarshipPreset::PastelPowerline,
        StarshipPreset::Bracketed,
        StarshipPreset::Arrow,
    ];

    /// No theme leaves a palette placeholder string ("#" plus all caps) in any
    /// generated artifact. Catches forgotten substitutions in starship presets,
    /// yazi themes, vim colorschemes, gitconfig, and lazygit.
    #[test]
    fn every_theme_renders_without_placeholders() {
        // Tokens we use across format!() strings — any leak indicates a bug.
        let placeholders = [
            "{bg}",
            "{fg}",
            "{accent}",
            "{green}",
            "{red}",
            "{yellow}",
            "{orange}",
            "{cyan}",
            "{muted}",
            "{surface}",
            "{magenta}",
        ];
        for theme in ALL_THEMES {
            let artifacts: Vec<(&str, String)> = vec![
                ("vim_aibox_colorscheme", vim_aibox_colorscheme(theme)),
                ("yazi_theme", yazi_theme_with_separator(theme, "rounded")),
                ("lazygit_theme", lazygit_theme(theme)),
                (
                    "gitconfig_with_delta",
                    gitconfig_with_delta_and_emphasis(theme, ThemeEmphasis::Auto),
                ),
                ("theme_env_script", theme_env_script(theme)),
                ("lnav_config", lnav_config(theme)),
                (
                    "tmux_powerkit_custom_theme",
                    tmux_powerkit_custom_theme(theme),
                ),
            ];
            for (name, body) in artifacts {
                for placeholder in &placeholders {
                    assert!(
                        !body.contains(placeholder),
                        "theme {theme}: {name} leaked placeholder {placeholder}:\n{body}"
                    );
                }
            }
            for preset in ALL_PROMPTS {
                let body = starship_config(preset, theme);
                for placeholder in &placeholders {
                    assert!(
                        !body.contains(placeholder),
                        "theme {theme} preset {preset}: starship_config leaked {placeholder}:\n{body}"
                    );
                }
                assert!(
                    !body.is_empty(),
                    "theme {theme} preset {preset}: starship_config returned empty body"
                );
            }
        }
    }

    /// Every theme's generated artifacts contain that theme's accent color
    /// somewhere it is actually used. Catches accidental cross-theme palette
    /// drift (e.g. a fallback that always emits gruvbox colors).
    #[test]
    fn every_theme_aligns_tools_to_its_accent() {
        for theme in ALL_THEMES {
            let (_bg, _fg, accent, _green, _red, _yellow, _orange, _cyan, _muted) =
                theme_palette(theme);
            let vim = vim_aibox_colorscheme(theme);
            assert!(
                vim.contains(accent),
                "theme {theme}: aibox.vim should embed accent {accent}, got:\n{vim}"
            );
            let yazi = yazi_theme_with_separator(theme, "rounded");
            assert!(
                yazi.contains(accent),
                "theme {theme}: yazi theme.toml should embed accent {accent}"
            );
            let starship = starship_config(&StarshipPreset::Default, theme);
            assert!(
                starship.contains(accent),
                "theme {theme}: default starship config should embed accent {accent}"
            );
            let powerkit = tmux_powerkit_custom_theme(theme);
            assert!(
                powerkit.contains(accent),
                "theme {theme}: powerkit theme should embed accent {accent}"
            );
        }
    }

    #[test]
    fn fzf_color_spec_avoids_nth_color() {
        let spec = fzf_color_spec(&Theme::Nord);
        assert!(
            !spec.contains("nth:"),
            "fzf 0.72 accepts only ANSI attributes for nth, not colors:\n{spec}"
        );
    }

    /// Preset-specific format expectations. Catches regressions where a preset
    /// rewrite drops its core module (`[directory]`, line-break shape, …).
    #[test]
    fn starship_presets_emit_their_expected_shape() {
        let theme = Theme::GruvboxDark;
        let default_body = starship_config(&StarshipPreset::Default, &theme);
        assert!(
            default_body.contains("directory") && default_body.contains("git_branch"),
            "default preset must include directory and git_branch modules:\n{default_body}"
        );

        let plain_body = starship_config(&StarshipPreset::Plain, &theme);
        // Plain preset must not use powerline glyphs (E0B0 left chevron).
        assert!(
            !plain_body.contains('\u{e0b0}'),
            "plain preset must be ASCII-only:\n{plain_body}"
        );

        for preset in [StarshipPreset::Pastel, StarshipPreset::PastelPowerline] {
            let body = starship_config(&preset, &theme);
            assert!(
                body.contains("pastel powerline preset"),
                "{preset} preset banner missing:\n{body}"
            );
            assert!(
                body.contains('\u{e0b0}'),
                "{preset} preset should use a powerline chevron:\n{body}"
            );
            assert!(
                !body.contains("$line_break"),
                "{preset} preset should render as a one-line prompt:\n{body}"
            );
        }
    }

    /// Every theme produces a Yazi theme that uses the current schema (Yazi 26
    /// requires url-based filetype rules and the [tabs]/[mode]/[status]/[git]
    /// sections; legacy keys must not survive).
    #[test]
    fn every_theme_yazi_uses_current_schema() {
        for theme in ALL_THEMES {
            let yazi = yazi_theme_with_separator(theme, "rounded");
            for required in [
                "[tabs]",
                "[mode]",
                "[status]",
                "[git]",
                "normal_main",
                "overall =",
                "url = \"*/\"",
            ] {
                assert!(
                    yazi.contains(required),
                    "theme {theme}: yazi theme should contain '{required}'"
                );
            }
            for legacy in [
                "tab_active",
                "mode_normal",
                "separator_open",
                "permissions_t",
                "[select]",
                "[completion]",
            ] {
                assert!(
                    !yazi.contains(legacy),
                    "theme {theme}: yazi theme must not contain legacy key '{legacy}'"
                );
            }
            assert!(
                !yazi.contains("{ name ="),
                "theme {theme}: Yazi 26 rejects name-only filetype rules"
            );
        }
    }

    #[test]
    fn powerkit_custom_theme_renders_palette_bg_for_every_theme() {
        // All 28 themes must emit `[background]=<their bg>` so chevrons land
        // on the exact aibox surface. Spot-check a representative roster.
        for (theme, expected_bg) in [
            (Theme::GruvboxDark, "#282828"),
            (Theme::GruvboxLight, "#FBF1C7"),
            (Theme::Projectious, "#0E1720"),
            (Theme::CatppuccinMocha, "#1E1E2E"),
            (Theme::TokyoNightDay, "#E1E2E7"),
            (Theme::Nord, "#2E3440"),
            (Theme::GithubLight, "#FFFFFF"),
        ] {
            let body = tmux_powerkit_custom_theme(&theme);
            assert!(
                body.contains(&format!("[background]=\"{expected_bg}\"")),
                "powerkit custom theme for {theme} must declare bg {expected_bg}, got:\n{body}"
            );
        }
    }
}
