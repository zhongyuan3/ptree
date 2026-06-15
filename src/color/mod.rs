pub mod style;
pub mod theme;

use std::path::Path;

use clap::ValueEnum;
use colored::ColoredString;
use serde::Deserialize;
use supports_color::{Stream, on_cached};

pub use style::ColorStyle;
use theme::{COLOR_THEME_REGISTRY, ColorTheme};

use crate::icon::EntryKind;

pub fn stdout_supports_color() -> bool {
    on_cached(Stream::Stdout).is_some()
}

pub fn resolve_use_color(no_color_flag: bool) -> bool {
    !no_color_flag && stdout_supports_color()
}

pub fn configure_color_output(use_color: bool) {
    colored::control::set_override(use_color);
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BuiltinColorTheme {
    #[default]
    #[value(name = "dark")]
    Dark,
    #[value(name = "light")]
    Light,
    #[value(name = "auto")]
    Auto,
}

impl BuiltinColorTheme {
    fn registry_key(self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
            Self::Auto => "auto",
        }
    }
}

pub fn resolve_color_theme(theme: BuiltinColorTheme) -> BuiltinColorTheme {
    match theme {
        BuiltinColorTheme::Auto => detect_terminal_theme(),
        other => other,
    }
}

fn detect_terminal_theme() -> BuiltinColorTheme {
    if let Ok(fgbg) = std::env::var("COLORFGBG") {
        if let Some(bg) = fgbg
            .split(';')
            .nth(1)
            .and_then(|value| value.parse::<u8>().ok())
        {
            return if matches!(bg, 7 | 15) || bg >= 250 {
                BuiltinColorTheme::Light
            } else {
                BuiltinColorTheme::Dark
            };
        }
    }

    BuiltinColorTheme::Dark
}

fn lookup_special<'a>(
    map: &'a phf::Map<&'static str, ColorStyle>,
    name: &str,
) -> Option<&'a ColorStyle> {
    if let Some(style) = map.get(name) {
        return Some(style);
    }

    if name.bytes().any(|byte| byte.is_ascii_uppercase()) {
        return map.get(name.to_ascii_lowercase().as_str());
    }

    None
}

fn lookup_color<'a>(theme: &'a ColorTheme, kind: EntryKind, path: &Path) -> &'a ColorStyle {
    if let Some(name) = path.file_name().and_then(|name| name.to_str())
        && let Some(style) = lookup_special(theme.special, name)
    {
        return style;
    }

    if kind != EntryKind::Directory
        && let Some(ext) = path.extension().and_then(|ext| ext.to_str())
        && let Some(style) = theme.extensions.get(ext)
    {
        return style;
    }

    if kind == EntryKind::File {
        return &ColorStyle::DEFAULT;
    }

    theme
        .kinds
        .get(kind.registry_key())
        .unwrap_or(&theme.fallback)
}

fn adjust_style_for_light_theme(theme_name: &str, style: ColorStyle) -> ColorStyle {
    if theme_name != "light" {
        return style;
    }

    let Some(fg) = style.fg else {
        return style;
    };

    let luminance = fg.r as u16 + fg.g as u16 + fg.b as u16;
    if luminance <= 500 {
        return style;
    }

    ColorStyle {
        fg: Some(style::RgbValue::new(
            (fg.r as u16 * 2 / 5) as u8,
            (fg.g as u16 * 2 / 5) as u8,
            (fg.b as u16 * 2 / 5) as u8,
        )),
        attrs: style.attrs,
    }
}

pub struct Colors {
    theme: &'static ColorTheme,
}

impl Colors {
    pub fn new() -> Self {
        Self::with_theme(BuiltinColorTheme::default())
    }

    pub fn with_theme(theme: BuiltinColorTheme) -> Self {
        let resolved = resolve_color_theme(theme);
        let registry_key = match resolved {
            BuiltinColorTheme::Auto => "dark",
            other => other.registry_key(),
        };

        Self {
            theme: COLOR_THEME_REGISTRY
                .get(registry_key)
                .expect("every resolved BuiltinColorTheme must be registered"),
        }
    }

    #[allow(dead_code)]
    pub fn theme_name(&self) -> &'static str {
        self.theme.name
    }

    pub fn style(&self, kind: EntryKind, path: &Path) -> ColorStyle {
        let style = *lookup_color(self.theme, kind, path);
        adjust_style_for_light_theme(self.theme.name, style)
    }

    pub fn paint(&self, kind: EntryKind, path: &Path, text: &str) -> ColoredString {
        self.style(kind, path).apply(text)
    }
}

impl Default for Colors {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::style::RgbValue;

    const RUST: RgbValue = RgbValue::from_hex(0xdea584);
    const PYTHON: RgbValue = RgbValue::from_hex(0xffbc03);
    const README: RgbValue = RgbValue::from_hex(0xededed);
    const BLUE_DIR: RgbValue = RgbValue::new(97, 175, 239);

    #[test]
    fn dark_theme_colors_by_kind_and_extension() {
        let colors = Colors::with_theme(BuiltinColorTheme::Dark);
        assert_eq!(
            colors.style(EntryKind::Directory, Path::new("src")),
            ColorStyle::bold(BLUE_DIR)
        );
        assert_eq!(
            colors.style(EntryKind::File, Path::new("main.rs")),
            ColorStyle::plain(RUST)
        );
        assert_eq!(
            colors.style(EntryKind::File, Path::new("app.py")),
            ColorStyle::plain(PYTHON)
        );
    }

    #[test]
    fn plain_files_are_not_colored() {
        let colors = Colors::with_theme(BuiltinColorTheme::Dark);
        assert_eq!(
            colors.style(EntryKind::File, Path::new("notes")),
            ColorStyle::DEFAULT
        );
        assert_eq!(
            colors.style(EntryKind::File, Path::new("data.unknownext")),
            ColorStyle::DEFAULT
        );

        let light = Colors::with_theme(BuiltinColorTheme::Light);
        assert_eq!(
            light.style(EntryKind::File, Path::new("notes")),
            ColorStyle::DEFAULT
        );
    }

    #[test]
    fn special_file_lookup_is_case_insensitive() {
        let colors = Colors::new();
        assert_eq!(
            colors.style(EntryKind::File, Path::new("Cargo.toml")),
            ColorStyle::bold(RUST)
        );
        assert_eq!(
            colors.style(EntryKind::File, Path::new("README.md")),
            ColorStyle::bold(README)
        );
    }

    #[test]
    fn directories_ignore_extension_colors() {
        let colors = Colors::new();
        assert_eq!(
            colors.style(EntryKind::Directory, Path::new("src.rs")),
            ColorStyle::bold(BLUE_DIR)
        );
    }

    #[test]
    fn no_color_flag_always_disables_color() {
        assert!(!resolve_use_color(true));
    }

    #[test]
    fn auto_theme_selects_light_for_light_background() {
        assert_eq!(
            detect_terminal_theme_from_fgbg("0;15"),
            BuiltinColorTheme::Light
        );
        assert_eq!(
            detect_terminal_theme_from_fgbg("0;0"),
            BuiltinColorTheme::Dark
        );
    }

    #[test]
    fn light_theme_darkens_high_luminance_extension_colors() {
        let colors = Colors::with_theme(BuiltinColorTheme::Light);
        let style = colors.style(EntryKind::File, Path::new("README.md"));
        let fg = style.fg.expect("readme should be colored");
        let luminance = fg.r as u16 + fg.g as u16 + fg.b as u16;
        assert!(luminance <= 500);
    }

    fn detect_terminal_theme_from_fgbg(fgbg: &str) -> BuiltinColorTheme {
        let bg = fgbg
            .split(';')
            .nth(1)
            .and_then(|value| value.parse::<u8>().ok());
        if let Some(bg) = bg {
            if matches!(bg, 7 | 15) || bg >= 250 {
                return BuiltinColorTheme::Light;
            }
        }
        BuiltinColorTheme::Dark
    }
}
