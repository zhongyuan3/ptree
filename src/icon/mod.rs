pub mod theme;

use std::path::Path;

use clap::ValueEnum;
use serde::Deserialize;

use is_executable::IsExecutable;
use theme::{ICON_THEME_REGISTRY, IconTheme};

use crate::error::Error;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum, Deserialize)]
#[serde(rename_all = "kebab-case")]
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
        return Ok(
            if path
                .read_link()
                .map_err(|err| Error::ReadSymlink(path.display().to_string(), err))?
                .is_dir()
            {
                EntryKind::Directory
            } else {
                EntryKind::Symlink
            },
        );
    }

    if path.is_executable() {
        return Ok(EntryKind::Executable);
    }

    let metadata = path
        .symlink_metadata()
        .map_err(|e| Error::GetMetadata(path.display().to_string(), e))?;

    if metadata.permissions().readonly() {
        return Ok(EntryKind::ReadOnly);
    }

    Ok(EntryKind::File)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ptree_icon_{name}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn create_symlink(original: &Path, link: &Path, target_is_dir: bool) -> std::io::Result<()> {
        #[cfg(unix)]
        {
            let _ = target_is_dir;
            std::os::unix::fs::symlink(original, link)
        }
        #[cfg(windows)]
        {
            if target_is_dir {
                std::os::windows::fs::symlink_dir(original, link)
            } else {
                std::os::windows::fs::symlink_file(original, link)
            }
        }
    }

    fn symlink_privilege_denied(err: &std::io::Error) -> bool {
        err.kind() == std::io::ErrorKind::PermissionDenied || err.raw_os_error() == Some(1314)
    }

    /// Returns `false` when symlink creation is unavailable (e.g. Windows without Developer Mode).
    fn create_symlink_or_skip(original: &Path, link: &Path, target_is_dir: bool) -> bool {
        match create_symlink(original, link, target_is_dir) {
            Ok(()) => true,
            Err(err) if symlink_privilege_denied(&err) => {
                eprintln!("skipping: symlink creation requires privileges ({err})");
                false
            }
            Err(err) => panic!("failed to create symlink: {err}"),
        }
    }

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
        let dir = temp_dir("directory");
        assert_eq!(detect_entry_kind(&dir).unwrap(), EntryKind::Directory);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn detect_entry_kind_for_directory_symlink() {
        let dir = temp_dir("dir_symlink");
        let target = dir.join("target_dir");
        fs::create_dir_all(&target).unwrap();
        let link = dir.join("dir_link");
        if !create_symlink_or_skip(&target, &link, true) {
            let _ = fs::remove_dir_all(&dir);
            return;
        }

        assert_eq!(detect_entry_kind(&link).unwrap(), EntryKind::Directory);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn detect_entry_kind_for_file_symlink() {
        let dir = temp_dir("file_symlink");
        let target = dir.join("target.txt");
        fs::write(&target, b"hello").unwrap();
        let link = dir.join("file_link");
        if !create_symlink_or_skip(&target, &link, false) {
            let _ = fs::remove_dir_all(&dir);
            return;
        }

        assert_eq!(detect_entry_kind(&link).unwrap(), EntryKind::Symlink);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn detect_entry_kind_for_broken_symlink() {
        let dir = temp_dir("broken_symlink");
        let link = dir.join("broken_link");
        if !create_symlink_or_skip(&dir.join("missing.txt"), &link, false) {
            let _ = fs::remove_dir_all(&dir);
            return;
        }

        assert_eq!(detect_entry_kind(&link).unwrap(), EntryKind::Symlink);
        let _ = fs::remove_dir_all(&dir);
    }
}
