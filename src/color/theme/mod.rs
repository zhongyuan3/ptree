use phf::Map;
use phf_macros::phf_map;

use crate::color::style::ColorStyle;

pub mod dark;

pub struct ColorTheme {
    pub name: &'static str,
    pub kinds: &'static Map<&'static str, ColorStyle>,
    pub extensions: &'static Map<&'static str, ColorStyle>,
    pub special: &'static Map<&'static str, ColorStyle>,
    pub fallback: ColorStyle,
}

pub static COLOR_THEME_REGISTRY: Map<&str, ColorTheme> = phf_map! {
    "dark" => dark::THEME,
};
