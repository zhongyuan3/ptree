use std::fs::DirEntry;
use std::path::{Path, PathBuf};

use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::gitignore::{Gitignore, GitignoreBuilder};

use crate::config::Config;
use crate::error::Error;

pub struct GlobFilter {
    include: Option<GlobSet>,
    exclude: GlobSet,
}

impl GlobFilter {
    pub fn from_pattern_args(
        include_patterns: &[String],
        exclude_patterns: &[String],
    ) -> Result<Option<Self>, Error> {
        let include = build_glob_set(include_patterns)?;
        let exclude = build_glob_set(exclude_patterns)?.unwrap_or_else(GlobSet::empty);

        if include.is_none() && exclude.is_empty() {
            return Ok(None);
        }

        Ok(Some(Self { include, exclude }))
    }

    pub fn has_include_patterns(&self) -> bool {
        self.include.is_some()
    }

    pub fn is_excluded(&self, relative_path: &Path, file_name: &str) -> bool {
        if self.exclude.is_empty() {
            return false;
        }

        self.exclude.is_match(path_match_string(relative_path)) || self.exclude.is_match(file_name)
    }

    pub fn matches_include(&self, relative_path: &Path, file_name: &str) -> bool {
        let Some(include) = &self.include else {
            return true;
        };

        include.is_match(path_match_string(relative_path)) || include.is_match(file_name)
    }
}

fn expand_patterns(patterns: &[String]) -> Vec<String> {
    patterns
        .iter()
        .flat_map(|pattern| pattern.split('|'))
        .map(str::trim)
        .filter(|pattern| !pattern.is_empty())
        .map(str::to_owned)
        .collect()
}

fn build_glob_set(patterns: &[String]) -> Result<Option<GlobSet>, Error> {
    let patterns = expand_patterns(patterns);
    if patterns.is_empty() {
        return Ok(None);
    }

    let mut builder = GlobSetBuilder::new();
    for pattern in &patterns {
        let glob = Glob::new(pattern).map_err(|err| Error::GlobParse {
            pattern: pattern.clone(),
            detail: err.to_string(),
        })?;
        builder.add(glob);
    }

    builder.build().map(Some).map_err(|err| Error::GlobParse {
        pattern: patterns.join("|"),
        detail: err.to_string(),
    })
}

fn path_match_string(relative_path: &Path) -> String {
    relative_path.to_string_lossy().replace('\\', "/")
}

struct GitignoreLayer {
    root: PathBuf,
    matcher: Gitignore,
}

pub struct GitignoreFilter {
    parents: Vec<GitignoreLayer>,
    stack: Vec<GitignoreLayer>,
}

impl GitignoreFilter {
    pub fn new(tree_root: &Path) -> Result<Self, Error> {
        let tree_root = tree_root
            .canonicalize()
            .map_err(|e| Error::GetMetadata(tree_root.display().to_string(), e))?;

        Ok(Self {
            parents: load_parent_gitignores(&tree_root)?,
            stack: Vec::new(),
        })
    }

    /// `canonical_dir` must already be the canonical path of the directory being entered.
    pub fn enter_dir(&mut self, canonical_dir: &Path) -> Result<(), Error> {
        self.stack.push(load_gitignore_layer(canonical_dir)?);
        Ok(())
    }

    pub fn leave_dir(&mut self) {
        self.stack.pop();
    }

    pub fn should_ignore(&self, path: &Path, is_dir: bool) -> bool {
        for layer in self.layers().rev() {
            if layer.matcher.is_empty() {
                continue;
            }

            let relative = match path.strip_prefix(&layer.root) {
                Ok(rel) => rel,
                Err(_) => continue,
            };

            match layer.matcher.matched(relative, is_dir) {
                m if m.is_ignore() => return true,
                m if m.is_whitelist() => return false,
                _ => {}
            }
        }

        false
    }

    fn layers(&self) -> impl DoubleEndedIterator<Item = &GitignoreLayer> {
        self.parents.iter().chain(self.stack.iter())
    }
}

fn load_parent_gitignores(tree_root: &Path) -> Result<Vec<GitignoreLayer>, Error> {
    let mut parents = Vec::new();
    let mut current = tree_root.parent();

    while let Some(dir) = current {
        let layer = load_gitignore_layer(dir)?;
        if !layer.matcher.is_empty() {
            parents.push(layer);
        }
        current = dir.parent();
    }

    parents.reverse();
    Ok(parents)
}

fn load_gitignore_layer(dir: &Path) -> Result<GitignoreLayer, Error> {
    let gitignore_path = dir.join(".gitignore");
    if !gitignore_path.is_file() {
        return Ok(GitignoreLayer {
            root: dir.to_path_buf(),
            matcher: Gitignore::empty(),
        });
    }

    let mut builder = GitignoreBuilder::new(dir);
    if let Some(err) = builder.add(&gitignore_path) {
        return Err(map_ignore_error(&gitignore_path, err));
    }

    let matcher = builder
        .build()
        .map_err(|err| map_ignore_error(&gitignore_path, err))?;

    Ok(GitignoreLayer {
        root: dir.to_path_buf(),
        matcher,
    })
}

fn map_ignore_error(path: &Path, err: ignore::Error) -> Error {
    match err {
        ignore::Error::Io(io_err) => Error::ReadGitignore(path.display().to_string(), io_err),
        ignore::Error::Glob { glob, err } => Error::GitignoreParse {
            path: path.display().to_string(),
            detail: glob
                .map(|glob| format!("invalid glob `{glob}`: {err}"))
                .unwrap_or(err),
        },
        other => Error::GitignoreParse {
            path: path.display().to_string(),
            detail: other.to_string(),
        },
    }
}

/// `canonical_parent` must already be the canonical path of the directory whose entries are filtered.
pub fn filter_entries_by_config(
    canonical_parent: &Path,
    relative_parent: &Path,
    entries: Vec<DirEntry>,
    config: &Config,
    gitignore_filter: Option<&GitignoreFilter>,
) -> Result<Vec<DirEntry>, Error> {
    let mut filtered = Vec::new();

    for entry in entries {
        if !config.show_all && entry.file_name().to_string_lossy().starts_with('.') {
            continue;
        }

        let file_type = entry
            .file_type()
            .map_err(|e| Error::GetFileType(entry.path().display().to_string(), e))?;

        if config.show_dir_only && !file_type.is_dir() {
            continue;
        }

        let entry_path = canonical_parent.join(entry.file_name());
        let relative_path = relative_parent.join(entry.file_name());
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();

        if let Some(filter) = gitignore_filter
            && filter.should_ignore(&entry_path, file_type.is_dir())
        {
            continue;
        }

        if let Some(glob_filter) = &config.glob_filter {
            if glob_filter.is_excluded(&relative_path, &file_name_str) {
                continue;
            }

            if glob_filter.has_include_patterns() {
                if file_type.is_dir() {
                    if !glob_filter.matches_include(&relative_path, &file_name_str)
                        && !dir_contains_included_entry(
                            entry.path().as_path(),
                            &relative_path,
                            config,
                            gitignore_filter,
                            glob_filter,
                        )?
                    {
                        continue;
                    }
                } else if !glob_filter.matches_include(&relative_path, &file_name_str) {
                    continue;
                }
            }
        }

        filtered.push(entry);
    }

    Ok(filtered)
}

fn dir_contains_included_entry(
    dir: &Path,
    relative_dir: &Path,
    config: &Config,
    gitignore_filter: Option<&GitignoreFilter>,
    glob_filter: &GlobFilter,
) -> Result<bool, Error> {
    let canonical_dir = dir
        .canonicalize()
        .map_err(|e| Error::GetMetadata(dir.display().to_string(), e))?;

    let entries = dir
        .read_dir()
        .map_err(|e| Error::ReadDir(dir.display().to_string(), e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| Error::ReadDirEntry(dir.display().to_string(), e))?;

    for entry in entries {
        if !config.show_all && entry.file_name().to_string_lossy().starts_with('.') {
            continue;
        }

        let file_type = entry
            .file_type()
            .map_err(|e| Error::GetFileType(entry.path().display().to_string(), e))?;

        if config.show_dir_only && !file_type.is_dir() {
            continue;
        }

        let entry_path = canonical_dir.join(entry.file_name());
        let relative_path = relative_dir.join(entry.file_name());
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();

        if let Some(filter) = gitignore_filter
            && filter.should_ignore(&entry_path, file_type.is_dir())
        {
            continue;
        }

        if glob_filter.is_excluded(&relative_path, &file_name_str) {
            continue;
        }

        if file_type.is_dir() {
            if glob_filter.matches_include(&relative_path, &file_name_str)
                || dir_contains_included_entry(
                    entry.path().as_path(),
                    &relative_path,
                    config,
                    gitignore_filter,
                    glob_filter,
                )?
            {
                return Ok(true);
            }
        } else if glob_filter.matches_include(&relative_path, &file_name_str) {
            return Ok(true);
        }
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ptree_filter_{name}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn gitignore_matches_root_rule() {
        let dir = temp_dir("root_rule");
        fs::write(dir.join(".gitignore"), "/target\n").unwrap();
        fs::create_dir_all(dir.join("target")).unwrap();

        let mut filter = GitignoreFilter::new(&dir).unwrap();
        filter.enter_dir(&dir).unwrap();

        assert!(filter.should_ignore(&dir.join("target"), true));
        assert!(!filter.should_ignore(&dir.join("main.rs"), false));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn gitignore_matches_wildcard_extension() {
        let dir = temp_dir("wildcard");
        fs::write(dir.join(".gitignore"), "*.o\n").unwrap();

        let mut filter = GitignoreFilter::new(&dir).unwrap();
        filter.enter_dir(&dir).unwrap();

        assert!(filter.should_ignore(&dir.join("foo.o"), false));
        assert!(!filter.should_ignore(&dir.join("foo.rs"), false));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn gitignore_dir_only_rule() {
        let dir = temp_dir("dir_only");
        fs::write(dir.join(".gitignore"), "build/\n").unwrap();

        let mut filter = GitignoreFilter::new(&dir).unwrap();
        filter.enter_dir(&dir).unwrap();

        assert!(filter.should_ignore(&dir.join("build"), true));
        assert!(!filter.should_ignore(&dir.join("build"), false));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn nested_gitignore_applies_in_subdirectory() {
        let dir = temp_dir("nested");
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(dir.join("src/.gitignore"), "*.generated\n").unwrap();

        let mut filter = GitignoreFilter::new(&dir).unwrap();
        filter.enter_dir(&dir).unwrap();
        filter.enter_dir(&dir.join("src")).unwrap();

        assert!(filter.should_ignore(&dir.join("src/out.generated"), false));
        assert!(!filter.should_ignore(&dir.join("src/main.rs"), false));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn nested_gitignore_whitelist_unignores_parent_rule() {
        let dir = temp_dir("whitelist");
        fs::write(dir.join(".gitignore"), "*.log\n").unwrap();
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(dir.join("src/.gitignore"), "!important.log\n").unwrap();

        let mut filter = GitignoreFilter::new(&dir).unwrap();
        filter.enter_dir(&dir).unwrap();
        filter.enter_dir(&dir.join("src")).unwrap();

        assert!(!filter.should_ignore(&dir.join("src/important.log"), false));
        assert!(filter.should_ignore(&dir.join("src/debug.log"), false));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn parent_gitignore_applies_to_subdirectory_root() {
        let dir = temp_dir("parent");
        fs::write(dir.join(".gitignore"), "ignored/\n").unwrap();
        fs::create_dir_all(dir.join("nested")).unwrap();

        let nested = dir.join("nested");
        let canonical_nested = nested.canonicalize().unwrap();

        let mut filter = GitignoreFilter::new(&nested).unwrap();
        filter.enter_dir(&canonical_nested).unwrap();

        assert!(filter.should_ignore(&canonical_nested.join("ignored"), true));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn glob_exclude_hides_matching_entries() {
        let dir = temp_dir("glob_exclude");
        fs::write(dir.join("keep.txt"), "").unwrap();
        fs::write(dir.join("skip.log"), "").unwrap();

        let glob_filter = GlobFilter::from_pattern_args(&[], &["*.log".to_string()])
            .unwrap()
            .unwrap();
        let config = crate::config::test_config_with_glob(glob_filter);

        let canonical = dir.canonicalize().unwrap();
        let entries = fs::read_dir(&dir)
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let filtered =
            filter_entries_by_config(&canonical, Path::new(""), entries, &config, None).unwrap();

        let names: Vec<_> = filtered
            .iter()
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, vec!["keep.txt"]);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn glob_include_shows_matching_files_and_parent_dirs() {
        let dir = temp_dir("glob_include");
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(dir.join("src/main.rs"), "").unwrap();
        fs::write(dir.join("src/readme.txt"), "").unwrap();
        fs::write(dir.join("notes.txt"), "").unwrap();

        let glob_filter = GlobFilter::from_pattern_args(&["*.rs".to_string()], &[])
            .unwrap()
            .unwrap();
        let config = crate::config::test_config_with_glob(glob_filter);

        let canonical = dir.canonicalize().unwrap();
        let entries = fs::read_dir(&dir)
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let filtered =
            filter_entries_by_config(&canonical, Path::new(""), entries, &config, None).unwrap();

        let names: Vec<_> = filtered
            .iter()
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, vec!["src"]);
        let _ = fs::remove_dir_all(&dir);
    }
}
