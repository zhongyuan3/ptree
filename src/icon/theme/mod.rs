use phf::Map;
use phf_macros::phf_map;

pub mod ascii;
pub mod nerd_font;

pub struct IconTheme {
    #[allow(dead_code)]
    pub name: &'static str,
    pub kinds: &'static Map<&'static str, &'static str>,
    pub extensions: &'static Map<&'static str, &'static str>,
    pub special: &'static Map<&'static str, &'static str>,
    pub fallback: &'static str,
}

pub static ICON_THEME_REGISTRY: Map<&str, IconTheme> = phf_map! {
    "nerd-font" => nerd_font::THEME,
    "ascii" => ascii::THEME,
};
