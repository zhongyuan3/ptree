pub mod style;
pub mod theme;

use std::path::Path;

use clap::ValueEnum;
use colored::ColoredString;
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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub enum BuiltinColorTheme {
    #[default]
    #[value(name = "dark")]
    Dark,
}

impl BuiltinColorTheme {
    fn registry_key(self) -> &'static str {
        match self {
            Self::Dark => "dark",
        }
    }
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

    theme
        .kinds
        .get(kind.registry_key())
        .unwrap_or(&theme.fallback)
}

pub struct Colors {
    theme: &'static ColorTheme,
}

impl Colors {
    pub fn new() -> Self {
        Self::with_theme(BuiltinColorTheme::default())
    }

    pub fn with_theme(theme: BuiltinColorTheme) -> Self {
        Self {
            theme: COLOR_THEME_REGISTRY
                .get(theme.registry_key())
                .expect("every BuiltinColorTheme must be registered in COLOR_THEME_REGISTRY"),
        }
    }

    #[allow(dead_code)]
    pub fn theme_name(&self) -> &'static str {
        self.theme.name
    }

    pub fn style(&self, kind: EntryKind, path: &Path) -> ColorStyle {
        *lookup_color(self.theme, kind, path)
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
        let colors = Colors::new();
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
}
