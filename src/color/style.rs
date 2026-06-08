use colored::{ColoredString, Colorize};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RgbValue {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbValue {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub const fn from_hex(hex: u32) -> Self {
        Self {
            r: ((hex >> 16) & 0xff) as u8,
            g: ((hex >> 8) & 0xff) as u8,
            b: (hex & 0xff) as u8,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextAttr {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl TextAttr {
    pub const NONE: Self = Self {
        bold: false,
        italic: false,
        underline: false,
    };

    pub const BOLD: Self = Self {
        bold: true,
        italic: false,
        underline: false,
    };

    pub const ITALIC: Self = Self {
        bold: false,
        italic: true,
        underline: false,
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ColorStyle {
    pub fg: Option<RgbValue>,
    pub attrs: TextAttr,
}

impl ColorStyle {
    pub const DEFAULT: Self = Self {
        fg: None,
        attrs: TextAttr::NONE,
    };

    pub const fn plain(rgb: RgbValue) -> Self {
        Self {
            fg: Some(rgb),
            attrs: TextAttr::NONE,
        }
    }

    pub const fn bold(rgb: RgbValue) -> Self {
        Self {
            fg: Some(rgb),
            attrs: TextAttr::BOLD,
        }
    }

    pub const fn italic(rgb: RgbValue) -> Self {
        Self {
            fg: Some(rgb),
            attrs: TextAttr::ITALIC,
        }
    }

    pub fn apply(&self, text: &str) -> ColoredString {
        let mut output = match self.fg {
            None => ColoredString::from(text),
            Some(rgb) => text.truecolor(rgb.r, rgb.g, rgb.b),
        };

        if self.attrs.bold {
            output = output.bold();
        }
        if self.attrs.italic {
            output = output.italic();
        }
        if self.attrs.underline {
            output = output.underline();
        }

        output
    }
}
