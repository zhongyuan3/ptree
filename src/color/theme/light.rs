use phf::Map;
use phf_macros::phf_map;

use crate::color::style::{ColorStyle, RgbValue};
use crate::color::theme::ColorTheme;
use crate::color::theme::dark;

const BLUE_DIR: RgbValue = RgbValue::new(31, 97, 171);
const CYAN_LINK: RgbValue = RgbValue::new(0, 116, 131);
const GREEN_EXEC: RgbValue = RgbValue::from_hex(0x2a7f1f);
const RED_RO: RgbValue = RgbValue::from_hex(0xb91c1c);

const S_DIR: ColorStyle = ColorStyle::bold(BLUE_DIR);
const S_LINK: ColorStyle = ColorStyle::italic(CYAN_LINK);
const S_EXEC: ColorStyle = ColorStyle::plain(GREEN_EXEC);
const S_RO: ColorStyle = ColorStyle::bold(RED_RO);

static KIND_COLORS: Map<&str, ColorStyle> = phf_map! {
    "directory" => S_DIR,
    "symlink" => S_LINK,
    "executable" => S_EXEC,
    "readonly" => S_RO,
    "file" => ColorStyle::DEFAULT,
};

pub const THEME: ColorTheme = ColorTheme {
    name: "light",
    kinds: &KIND_COLORS,
    extensions: &dark::EXTENSION_COLORS,
    special: &dark::SPECIAL_COLORS,
    fallback: ColorStyle::DEFAULT,
};
