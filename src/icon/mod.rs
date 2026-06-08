pub mod theme;

use std::path::Path;

use clap::ValueEnum;

use theme::{ICON_THEME_REGISTRY, IconTheme};

use crate::error::Error;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub enum BuiltinIconTheme {
    #[default]
    NerdFont,
    Ascii,
}

impl BuiltinIconTheme {
    fn registry_key(self) -> &'static str {
        match self {
            Self::NerdFont => "nerd-font",
            Self::Ascii => "ascii",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntryKind {
    Directory,
    Symlink,
    Executable,
    ReadOnly,
    File,
}

impl EntryKind {
    pub(crate) fn registry_key(self) -> &'static str {
        match self {
            Self::Directory => "directory",
            Self::Symlink => "symlink",
            Self::Executable => "executable",
            Self::ReadOnly => "readonly",
            Self::File => "file",
        }
    }
}

fn lookup_special(map: &phf::Map<&'static str, &'static str>, name: &str) -> Option<&'static str> {
    if let Some(icon) = map.get(name) {
        return Some(*icon);
    }

    if name.bytes().any(|byte| byte.is_ascii_uppercase()) {
        return map.get(name.to_ascii_lowercase().as_str()).copied();
    }

    None
}

fn lookup_icon<'a>(theme: &'a IconTheme, kind: EntryKind, path: &Path) -> &'a str {
    if let Some(name) = path.file_name().and_then(|name| name.to_str())
        && let Some(icon) = lookup_special(theme.special, name)
    {
        return icon;
    }

    if kind != EntryKind::Directory
        && let Some(ext) = path.extension().and_then(|ext| ext.to_str())
        && let Some(icon) = theme.extensions.get(ext)
    {
        return icon;
    }

    theme
        .kinds
        .get(kind.registry_key())
        .copied()
        .unwrap_or(theme.fallback)
}

pub struct Icons {
    theme: &'static IconTheme,
}

impl Icons {
    pub fn new(icon_theme: BuiltinIconTheme) -> Self {
        Self::with_theme(icon_theme)
    }

    pub fn with_theme(icon_theme: BuiltinIconTheme) -> Self {
        Self {
            theme: ICON_THEME_REGISTRY
                .get(icon_theme.registry_key())
                .expect("every BuiltinIconTheme must be registered in ICON_THEME_REGISTRY"),
        }
    }

    pub fn icon(&self, kind: EntryKind, path: &Path) -> &str {
        lookup_icon(self.theme, kind, path)
    }
}

impl Default for Icons {
    fn default() -> Self {
        Self::new(BuiltinIconTheme::default())
    }
}

/// Classify a filesystem entry for icon and color lookup.
///
/// Directory symlinks are followed: a symlink pointing at a directory is reported as
/// [`EntryKind::Directory`], not [`EntryKind::Symlink`]. Only non-directory symlinks
/// (including broken links) are reported as [`EntryKind::Symlink`].
pub fn detect_entry_kind(path: &Path) -> Result<EntryKind, Error> {
    if path.is_dir() {
        return Ok(EntryKind::Directory);
    }

    if path.is_symlink() {
        return Ok(EntryKind::Symlink);
    }

    let metadata = path
        .symlink_metadata()
        .map_err(|e| Error::GetMetadata(path.display().to_string(), e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o111 != 0 {
            return Ok(EntryKind::Executable);
        }
    }

    if metadata.permissions().readonly() {
        return Ok(EntryKind::ReadOnly);
    }

    Ok(EntryKind::File)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn nerd_font_icons_use_extension_and_special_maps() {
        let icons = Icons::new(BuiltinIconTheme::NerdFont);
        assert_eq!(icons.icon(EntryKind::File, Path::new("main.rs")), " ");
        assert_eq!(icons.icon(EntryKind::Directory, Path::new("src")), "󰉋 ");
        assert_eq!(icons.icon(EntryKind::File, Path::new("Makefile")), " ");
        assert_eq!(icons.icon(EntryKind::File, Path::new("Cargo.toml")), "󰒓 ");
        assert_eq!(icons.icon(EntryKind::File, Path::new("main.go")), " ");
        assert_eq!(icons.icon(EntryKind::File, Path::new("schema.sql")), "󰆼 ");
    }

    #[test]
    fn ascii_icons_use_fallback_markers() {
        let icons = Icons::new(BuiltinIconTheme::Ascii);
        assert_eq!(icons.icon(EntryKind::File, Path::new("main.txt")), "[F] ");
        assert_eq!(icons.icon(EntryKind::Symlink, Path::new("link")), "[L] ");
    }

    #[test]
    fn directories_ignore_extension_icons() {
        let icons = Icons::new(BuiltinIconTheme::NerdFont);
        assert_eq!(icons.icon(EntryKind::Directory, Path::new("src.rs")), "󰉋 ");
    }

    #[test]
    fn detect_entry_kind_for_directory() {
        let dir = std::env::temp_dir().join(format!("ptree_icon_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        assert_eq!(detect_entry_kind(&dir).unwrap(), EntryKind::Directory);
        let _ = fs::remove_dir_all(&dir);
    }
}
