//! Theme data for all supported color themes.
//!
//! Each theme provides config snippets for tmux, Vim, Yazi, and lazygit.

use crate::config::{StarshipPreset, Theme};

/// Returns terminal surface colors for tmux panes and app backgrounds.
pub fn terminal_surface_colors(theme: &Theme) -> (&str, &str, &str, &str, String, String) {
    let (bg, fg, accent, _green, _red, _yellow, _orange, _cyan, muted) = theme_palette(theme);
    (
        bg,
        fg,
        accent,
        muted,
        mix_hex_colors(fg, muted, 50),
        mix_hex_colors(accent, fg, 85),
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
/// These are single-file .vim colorschemes bundled in the image.
pub fn vim_colorscheme(theme: &Theme) -> &'static str {
    match theme {
        Theme::GruvboxDark | Theme::GruvboxLight => "gruvbox",
        Theme::CatppuccinMocha => "catppuccin_mocha",
        Theme::CatppuccinMacchiato | Theme::CatppuccinFrappe => "catppuccin_mocha",
        Theme::CatppuccinLatte => "catppuccin_latte",
        Theme::Dracula => "dracula",
        Theme::TokyoNight | Theme::TokyoNightStorm | Theme::TokyoNightDay => "tokyonight",
        Theme::Nord => "nord",
        Theme::RosePine | Theme::RosePineMoon | Theme::RosePineDawn => "catppuccin_mocha",
        Theme::Material | Theme::MaterialOcean | Theme::MaterialPalenight => "catppuccin_mocha",
        Theme::MaterialLighter => "catppuccin_latte",
        Theme::SolarizedDark => "gruvbox",
        Theme::SolarizedLight => "catppuccin_latte",
        Theme::GithubDark => "catppuccin_mocha",
        Theme::GithubLight => "catppuccin_latte",
        Theme::AyuDark | Theme::AyuMirage => "catppuccin_mocha",
        Theme::AyuLight => "catppuccin_latte",
        Theme::NightOwl => "tokyonight",
        Theme::NightOwlLight => "catppuccin_latte",
        Theme::Moonlight => "tokyonight",
        Theme::Projectious => "projectious",
    }
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
        | Theme::AyuLight
        | Theme::NightOwlLight => "light",
        _ => "dark",
    }
}

/// Returns the Yazi theme.toml content for the given theme.
/// Gruvbox uses the default theme.toml; others are bundled from images/base-debian/config/yazi/themes/.
pub fn yazi_theme(theme: &Theme) -> String {
    let source = match theme {
        Theme::GruvboxDark | Theme::GruvboxLight => {
            include_str!("../../images/base-debian/config/yazi/theme.toml")
        }
        Theme::CatppuccinMocha => {
            include_str!("../../images/base-debian/config/yazi/themes/catppuccin-mocha.toml")
        }
        Theme::CatppuccinMacchiato | Theme::CatppuccinFrappe => {
            include_str!("../../images/base-debian/config/yazi/themes/catppuccin-mocha.toml")
        }
        Theme::CatppuccinLatte => {
            include_str!("../../images/base-debian/config/yazi/themes/catppuccin-latte.toml")
        }
        Theme::Dracula => include_str!("../../images/base-debian/config/yazi/themes/dracula.toml"),
        Theme::TokyoNight | Theme::TokyoNightStorm | Theme::TokyoNightDay => {
            include_str!("../../images/base-debian/config/yazi/themes/tokyo-night.toml")
        }
        Theme::Nord => include_str!("../../images/base-debian/config/yazi/themes/nord.toml"),
        Theme::RosePine | Theme::RosePineMoon | Theme::Moonlight => {
            include_str!("../../images/base-debian/config/yazi/themes/tokyo-night.toml")
        }
        Theme::RosePineDawn
        | Theme::MaterialLighter
        | Theme::SolarizedLight
        | Theme::GithubLight
        | Theme::AyuLight
        | Theme::NightOwlLight => {
            include_str!("../../images/base-debian/config/yazi/themes/catppuccin-latte.toml")
        }
        Theme::Material
        | Theme::MaterialOcean
        | Theme::MaterialPalenight
        | Theme::SolarizedDark
        | Theme::GithubDark
        | Theme::AyuDark
        | Theme::AyuMirage
        | Theme::NightOwl => {
            include_str!("../../images/base-debian/config/yazi/themes/catppuccin-mocha.toml")
        }
        Theme::Projectious => {
            include_str!("../../images/base-debian/config/yazi/themes/projectious.toml")
        }
    };
    yazi_theme_with_explicit_surfaces(theme, source)
}

fn yazi_theme_with_explicit_surfaces(theme: &Theme, source: &str) -> String {
    let (bg, fg, accent, _green, _red, _yellow, _orange, _cyan, muted) = theme_palette(theme);
    let surface = yazi_surface_color(theme);
    source
        .replace(
            "preview_hovered = { underline = true }",
            &format!("preview_hovered = {{ bg = \"{surface}\", underline = true }}"),
        )
        .replace(
            "title = {}",
            &format!("title = {{ fg = \"{accent}\", bg = \"{bg}\", bold = true }}"),
        )
        .replace(
            "value = {}",
            &format!("value = {{ fg = \"{fg}\", bg = \"{bg}\" }}"),
        )
        .replace(
            "inactive = {}",
            &format!("inactive = {{ fg = \"{muted}\", bg = \"{bg}\" }}"),
        )
        .replace(
            "hovered = { underline = true }",
            &format!("hovered = {{ bg = \"{surface}\", underline = true }}"),
        )
        .replace(
            "desc = {}",
            &format!("desc = {{ fg = \"{fg}\", bg = \"{bg}\" }}"),
        )
}

fn yazi_surface_color(theme: &Theme) -> &'static str {
    match theme {
        Theme::GruvboxLight => "#EBDBB2",
        Theme::CatppuccinLatte
        | Theme::TokyoNightDay
        | Theme::RosePineDawn
        | Theme::MaterialLighter
        | Theme::SolarizedLight
        | Theme::GithubLight
        | Theme::AyuLight
        | Theme::NightOwlLight => "#CCD0DA",
        Theme::GruvboxDark => "#3C3836",
        Theme::Nord => "#3B4252",
        Theme::Dracula => "#44475A",
        Theme::TokyoNight | Theme::TokyoNightStorm | Theme::Moonlight => "#283457",
        Theme::Projectious => "#131E2B",
        _ => "#313244",
    }
}

/// Returns the lazygit theme YAML snippet (gui.theme section).
pub fn lazygit_theme(theme: &Theme) -> String {
    let (bg, fg, accent, _green, red, yellow, _orange, cyan, muted) = theme_palette(theme);
    format!(
        r#"gui:
  theme:
    activeBorderColor:
      - '{accent}'
      - bold
    inactiveBorderColor:
      - '{muted}'
    optionsTextColor:
      - '{cyan}'
    selectedLineBgColor:
      - '{bg}'
    cherryPickedCommitBgColor:
      - '{muted}'
    cherryPickedCommitFgColor:
      - '{accent}'
    unstagedChangesColor:
      - '{red}'
    defaultFgColor:
      - '{fg}'
    searchingActiveBorderColor:
      - '{yellow}'
"#
    )
}

/// Maps aibox themes to tmux-powerkit base theme + variant.
pub fn tmux_powerkit_theme(theme: &Theme) -> (&'static str, &'static str) {
    match theme {
        Theme::GruvboxDark => ("gruvbox", "dark"),
        Theme::GruvboxLight => ("gruvbox", "light"),
        Theme::CatppuccinMocha => ("catppuccin", "mocha"),
        Theme::CatppuccinMacchiato => ("catppuccin", "macchiato"),
        Theme::CatppuccinFrappe => ("catppuccin", "frappe"),
        Theme::CatppuccinLatte => ("catppuccin", "latte"),
        Theme::Dracula => ("dracula", "dark"),
        Theme::TokyoNight => ("tokyo-night", "night"),
        Theme::TokyoNightStorm => ("tokyo-night", "storm"),
        Theme::TokyoNightDay => ("tokyo-night", "day"),
        Theme::Nord => ("nord", "dark"),
        Theme::RosePine => ("rose-pine", "main"),
        Theme::RosePineMoon => ("rose-pine", "moon"),
        Theme::RosePineDawn => ("rose-pine", "dawn"),
        Theme::Material => ("material", "default"),
        Theme::MaterialOcean => ("material", "ocean"),
        Theme::MaterialPalenight => ("material", "palenight"),
        Theme::MaterialLighter => ("material", "lighter"),
        Theme::SolarizedDark => ("solarized", "dark"),
        Theme::SolarizedLight => ("solarized", "light"),
        Theme::GithubDark => ("github", "dark"),
        Theme::GithubLight => ("github", "light"),
        Theme::AyuDark => ("ayu", "dark"),
        Theme::AyuMirage => ("ayu", "mirage"),
        Theme::AyuLight => ("ayu", "light"),
        Theme::NightOwl => ("night-owl", "default"),
        Theme::NightOwlLight => ("night-owl", "light"),
        Theme::Moonlight => ("moonlight", "default"),
        Theme::Projectious => ("tokyo-night", "night"),
    }
}

/// Color palette values for Starship prompt theming.
fn theme_palette(theme: &Theme) -> (&str, &str, &str, &str, &str, &str, &str, &str, &str) {
    // Returns (bg, fg, accent, green, red, yellow, orange, cyan, muted)
    match theme {
        Theme::GruvboxDark => (
            "#282828", "#D5C4A1", "#D79921", "#98971A", "#CC241D", "#D79921", "#D65D0E", "#689D6A",
            "#928374",
        ),
        Theme::GruvboxLight => (
            "#FBF1C7", "#3C3836", "#D65D0E", "#79740E", "#CC241D", "#B57614", "#D65D0E", "#076678",
            "#928374",
        ),
        Theme::CatppuccinMocha => (
            "#1E1E2E", "#CDD6F4", "#89B4FA", "#A6E3A1", "#F38BA8", "#F9E2AF", "#FAB387", "#94E2D5",
            "#6C7086",
        ),
        Theme::CatppuccinMacchiato => (
            "#24273A", "#CAD3F5", "#8AADF4", "#A6DA95", "#ED8796", "#EED49F", "#F5A97F", "#8BD5CA",
            "#6E738D",
        ),
        Theme::CatppuccinFrappe => (
            "#303446", "#C6D0F5", "#8CAAEE", "#A6D189", "#E78284", "#E5C890", "#EF9F76", "#81C8BE",
            "#737994",
        ),
        Theme::CatppuccinLatte => (
            "#EFF1F5", "#4C4F69", "#1E66F5", "#40A02B", "#D20F39", "#DF8E1D", "#FE640B", "#179299",
            "#9CA0B0",
        ),
        Theme::Dracula => (
            "#282A36", "#F8F8F2", "#BD93F9", "#50FA7B", "#FF5555", "#F1FA8C", "#FFB86C", "#8BE9FD",
            "#6272A4",
        ),
        Theme::TokyoNight => (
            "#1A1B26", "#C0CAF5", "#7AA2F7", "#9ECE6A", "#F7768E", "#E0AF68", "#FF9E64", "#7DCFFF",
            "#565F89",
        ),
        Theme::TokyoNightStorm => (
            "#24283B", "#C0CAF5", "#7AA2F7", "#9ECE6A", "#F7768E", "#E0AF68", "#FF9E64", "#7DCFFF",
            "#565F89",
        ),
        Theme::TokyoNightDay => (
            "#E1E2E7", "#3760BF", "#2E7DE9", "#587539", "#F52A65", "#8C6C3E", "#B15C00", "#007197",
            "#7B8496",
        ),
        Theme::Nord => (
            "#2E3440", "#D8DEE9", "#88C0D0", "#A3BE8C", "#BF616A", "#EBCB8B", "#D08770", "#81A1C1",
            "#4C566A",
        ),
        Theme::RosePine => (
            "#191724", "#E0DEF4", "#C4A7E7", "#31748F", "#EB6F92", "#F6C177", "#EA9A97", "#9CCFD8",
            "#6E6A86",
        ),
        Theme::RosePineMoon => (
            "#232136", "#E0DEF4", "#C4A7E7", "#3E8FB0", "#EB6F92", "#EA9A97", "#F6C177", "#9CCFD8",
            "#6E6A86",
        ),
        Theme::RosePineDawn => (
            "#FAF4ED", "#575279", "#907AA9", "#56949F", "#B4637A", "#EA9D34", "#D7827E", "#286983",
            "#9893A5",
        ),
        Theme::Material => (
            "#263238", "#EEFFFF", "#82AAFF", "#C3E88D", "#F07178", "#FFCB6B", "#F78C6C", "#89DDFF",
            "#546E7A",
        ),
        Theme::MaterialOcean => (
            "#0F111A", "#A6ACCD", "#82AAFF", "#C3E88D", "#F07178", "#FFCB6B", "#F78C6C", "#89DDFF",
            "#464B5D",
        ),
        Theme::MaterialPalenight => (
            "#292D3E", "#A6ACCD", "#82AAFF", "#C3E88D", "#F07178", "#FFCB6B", "#F78C6C", "#89DDFF",
            "#676E95",
        ),
        Theme::MaterialLighter => (
            "#FAFAFA", "#546E7A", "#6182B8", "#91B859", "#E53935", "#F6A434", "#F76D47", "#39ADB5",
            "#90A4AE",
        ),
        Theme::SolarizedDark => (
            "#002B36", "#93A1A1", "#268BD2", "#859900", "#DC322F", "#B58900", "#CB4B16", "#2AA198",
            "#657B83",
        ),
        Theme::SolarizedLight => (
            "#FDF6E3", "#586E75", "#268BD2", "#859900", "#DC322F", "#B58900", "#CB4B16", "#2AA198",
            "#93A1A1",
        ),
        Theme::GithubDark => (
            "#0D1117", "#C9D1D9", "#58A6FF", "#3FB950", "#F85149", "#D29922", "#DB6D28", "#79C0FF",
            "#8B949E",
        ),
        Theme::GithubLight => (
            "#FFFFFF", "#24292F", "#0969DA", "#1A7F37", "#CF222E", "#9A6700", "#BC4C00", "#0969DA",
            "#6E7781",
        ),
        Theme::AyuDark => (
            "#0A0E14", "#B3B1AD", "#39BAE6", "#AAD94C", "#F07178", "#FFB454", "#FF8F40", "#95E6CB",
            "#626A73",
        ),
        Theme::AyuMirage => (
            "#1F2430", "#CCCAC2", "#5CCFE6", "#BAE67E", "#F28779", "#FFD173", "#FFAD66", "#95E6CB",
            "#707A8C",
        ),
        Theme::AyuLight => (
            "#FAFAFA", "#5C6773", "#55B4D4", "#86B300", "#F51818", "#FA8D3E", "#F07171", "#4CBF99",
            "#ABB0B6",
        ),
        Theme::NightOwl => (
            "#011627", "#D6DEEB", "#82AAFF", "#22DA6E", "#EF5350", "#C5E478", "#F78C6C", "#21C7A8",
            "#637777",
        ),
        Theme::NightOwlLight => (
            "#FBFBFB", "#403F53", "#4876D6", "#2AA298", "#D3423E", "#DAA520", "#DD6A58", "#08916A",
            "#989FB1",
        ),
        Theme::Moonlight => (
            "#212337", "#C8D3F5", "#82AAFF", "#C3E88D", "#FF757F", "#FFC777", "#F78C6C", "#86E1FC",
            "#7A88CF",
        ),
        Theme::Projectious => (
            "#0E1720", "#C5DAF0", "#E05232", "#2D6A4F", "#A32D2D", "#8B6508", "#E05232", "#8AACC8",
            "#546A82",
        ),
    }
}

/// Generate starship.toml content for the given preset and theme.
pub fn starship_config(preset: &StarshipPreset, theme: &Theme) -> String {
    let (bg, fg, accent, green, red, yellow, orange, cyan, muted) = theme_palette(theme);

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
"#
        ),

        StarshipPreset::Pastel => format!(
            r#"# aibox starship config — pastel powerline preset
palette = "aibox"

format = """
[](fg:{accent})\
$directory\
[](fg:{accent} bg:{green})\
$git_branch\
$git_status\
[](fg:{green} bg:{bg})\
$python$rust$nodejs$golang\
$cmd_duration\
$line_break$character"""

[directory]
style = "bold bg:{accent} fg:{bg}"
truncation_length = 3

[git_branch]
style = "bg:{green} fg:{bg}"
symbol = " "

[git_status]
style = "bg:{green} fg:{bg}"

[character]
success_symbol = "[❯](bold fg:{accent})"
error_symbol = "[❯](bold fg:{red})"

[palettes.aibox]
bg = "{bg}"
fg = "{fg}"
accent = "{accent}"
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
[](fg:{accent})\
$directory\
[](fg:{accent} bg:{green})\
$git_branch\
$git_status\
[](fg:{green} bg:{bg})\
 $cmd_duration\
$line_break\
$character"""

[directory]
style = "bold bg:{accent} fg:{bg}"
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
"#
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tmux_powerkit_popular_theme_roster_is_mapped() {
        let cases = [
            (Theme::TokyoNight, ("tokyo-night", "night")),
            (Theme::TokyoNightStorm, ("tokyo-night", "storm")),
            (Theme::TokyoNightDay, ("tokyo-night", "day")),
            (Theme::CatppuccinMocha, ("catppuccin", "mocha")),
            (Theme::CatppuccinMacchiato, ("catppuccin", "macchiato")),
            (Theme::CatppuccinFrappe, ("catppuccin", "frappe")),
            (Theme::CatppuccinLatte, ("catppuccin", "latte")),
            (Theme::Dracula, ("dracula", "dark")),
            (Theme::Nord, ("nord", "dark")),
            (Theme::GruvboxDark, ("gruvbox", "dark")),
            (Theme::GruvboxLight, ("gruvbox", "light")),
            (Theme::RosePine, ("rose-pine", "main")),
            (Theme::RosePineMoon, ("rose-pine", "moon")),
            (Theme::RosePineDawn, ("rose-pine", "dawn")),
            (Theme::Material, ("material", "default")),
            (Theme::MaterialOcean, ("material", "ocean")),
            (Theme::MaterialPalenight, ("material", "palenight")),
            (Theme::MaterialLighter, ("material", "lighter")),
            (Theme::SolarizedDark, ("solarized", "dark")),
            (Theme::SolarizedLight, ("solarized", "light")),
            (Theme::GithubDark, ("github", "dark")),
            (Theme::GithubLight, ("github", "light")),
            (Theme::AyuDark, ("ayu", "dark")),
            (Theme::AyuMirage, ("ayu", "mirage")),
            (Theme::AyuLight, ("ayu", "light")),
            (Theme::NightOwl, ("night-owl", "default")),
            (Theme::NightOwlLight, ("night-owl", "light")),
            (Theme::Moonlight, ("moonlight", "default")),
        ];

        for (theme, expected) in cases {
            assert_eq!(tmux_powerkit_theme(&theme), expected, "{theme}");
        }
    }
}
