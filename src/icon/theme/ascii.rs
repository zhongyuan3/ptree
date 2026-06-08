use phf::Map;
use phf_macros::phf_map;

use super::IconTheme;

static KIND_ICONS: Map<&str, &str> = phf_map! {
    "directory" => "[D] ",
    "symlink" => "[L] ",
    "executable" => "[X] ",
    "readonly" => "[R] ",
    "file" => "[F] ",
};

static EXTENSION_ICONS: Map<&str, &str> = phf_map! {};

static SPECIAL_ICONS: Map<&str, &str> = phf_map! {};

pub const THEME: IconTheme = IconTheme {
    name: "ascii",
    kinds: &KIND_ICONS,
    extensions: &EXTENSION_ICONS,
    special: &SPECIAL_ICONS,
    fallback: "[F] ",
};
