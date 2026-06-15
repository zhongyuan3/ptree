use std::collections::HashSet;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::config::Config;
use crate::config::IndentType;
use crate::error::Error;
use crate::filter::GitignoreFilter;
use crate::filter::filter_entries_by_config;
use crate::icon::EntryKind;
use crate::icon::detect_entry_kind;
use crate::settings::EffectiveOptions;
use crate::sort::sort_entries;

type DirIdentity = PathBuf;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct TreeStats {
    directories: usize,
    files: usize,
}

impl TreeStats {
    fn record_entry(&mut self, path: &Path) {
        if path.is_dir() {
            self.directories += 1;
        } else {
            self.files += 1;
        }
    }

    fn format_summary(&self) -> String {
        let dir_word = if self.directories == 1 {
            "directory"
        } else {
            "directories"
        };
        let file_word = if self.files == 1 { "file" } else { "files" };
        format!(
            "{} {}, {} {}",
            self.directories, dir_word, self.files, file_word
        )
    }
}

#[derive(Debug, Serialize)]
struct JsonNode {
    name: String,
    path: String,
    #[serde(rename = "type")]
    kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    children: Option<Vec<JsonNode>>,
}

struct TraverseState<'a> {
    indent_symbols: Vec<&'static str>,
    is_last: bool,
    current_depth: usize,
    relative_path: PathBuf,
    is_root: bool,
    config: &'a Config,
    gitignore_filter: Option<&'a mut GitignoreFilter>,
    visited_dirs: HashSet<DirIdentity>,
    stats: TreeStats,
    entries_listed: usize,
    limit_reached: bool,
}

impl<'a> TraverseState<'a> {
    pub fn new(config: &'a Config, gitignore_filter: Option<&'a mut GitignoreFilter>) -> Self {
        Self {
            indent_symbols: Vec::new(),
            is_last: true,
            current_depth: 1,
            relative_path: PathBuf::new(),
            is_root: true,
            config,
            gitignore_filter,
            visited_dirs: HashSet::new(),
            stats: TreeStats::default(),
            entries_listed: 0,
            limit_reached: false,
        }
    }

    pub fn prepare_parent_indent(&mut self) {
        if !self.indent_symbols.is_empty() {
            self.indent_symbols.pop();
            self.indent_symbols.push(if self.is_last {
                self.config.get_indent(IndentType::Space)
            } else {
                self.config.get_indent(IndentType::Vertical)
            });
        }
    }

    pub fn push_indent(&mut self, is_last: bool) {
        self.is_last = is_last;
        self.indent_symbols.push(if is_last {
            self.config.get_indent(IndentType::Last)
        } else {
            self.config.get_indent(IndentType::Continue)
        });
    }

    pub fn pop_indent(&mut self) {
        self.indent_symbols.pop();
    }

    pub fn print_current_indent(&self) {
        for s in &self.indent_symbols {
            print!("{s}");
        }
    }
}

fn read_all_entries(path: &Path) -> Result<Vec<DirEntry>, Error> {
    let path_str = path.display().to_string();
    path.read_dir()
        .map_err(|e| Error::ReadDir(path_str.clone(), e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| Error::ReadDirEntry(path_str, e))
}

fn entry_within_limit(state: &TraverseState) -> bool {
    state
        .config
        .file_limit
        .is_none_or(|limit| state.entries_listed < limit)
}

fn mark_entry_listed(state: &mut TraverseState) {
    state.entries_listed += 1;
    if let Some(limit) = state.config.file_limit
        && state.entries_listed >= limit
    {
        state.limit_reached = true;
    }
}

fn format_output_path(relative: &Path) -> String {
    if relative.as_os_str().is_empty() {
        ".".to_string()
    } else {
        relative.to_string_lossy().replace('\\', "/")
    }
}

fn entry_file_size(path: &Path) -> Result<Option<u64>, Error> {
    if path.is_dir() {
        return Ok(None);
    }

    let metadata = path
        .symlink_metadata()
        .map_err(|e| Error::GetMetadata(path.display().to_string(), e))?;
    Ok(Some(metadata.len()))
}

fn format_size(size: u64, human: bool) -> String {
    if human {
        format_human_size(size)
    } else {
        size.to_string()
    }
}

fn format_human_size(size: u64) -> String {
    const UNITS: [&str; 5] = ["B", "K", "M", "G", "T"];
    let mut value = size as f64;
    let mut unit = 0;

    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }

    if unit == 0 {
        format!("{}{}", size, UNITS[unit])
    } else {
        format!("{:.1}{}", value, UNITS[unit])
    }
}

fn json_kind(kind: EntryKind) -> &'static str {
    match kind {
        EntryKind::Directory => "directory",
        EntryKind::Symlink => "symlink",
        EntryKind::Executable => "executable",
        EntryKind::ReadOnly => "readonly",
        EntryKind::File => "file",
    }
}

fn print_path_line(path: &Path, state: &TraverseState) -> Result<(), Error> {
    let output = if state.config.full_path {
        path.canonicalize()
            .map_err(|e| Error::GetMetadata(path.display().to_string(), e))?
            .display()
            .to_string()
    } else {
        format_output_path(&state.relative_path)
    };
    println!("{output}");
    Ok(())
}

fn print_path_name(path: &Path, state: &mut TraverseState) -> Result<(), Error> {
    state.print_current_indent();

    let kind = detect_entry_kind(path)?;
    let name = path.file_name().unwrap_or(path.as_os_str());
    let name_str = name.to_string_lossy();
    let size_suffix = if state.config.show_size {
        entry_file_size(path)?
            .map(|size| format!("  {}", format_size(size, state.config.human_size)))
    } else {
        None
    };

    if state.config.use_color && state.config.show_icons {
        let icon = state.config.icons.icon(kind, path);
        let painted_icon = state.config.colors.paint(kind, path, icon);
        match size_suffix {
            Some(suffix) => println!("{painted_icon}{name_str}{suffix}"),
            None => println!("{painted_icon}{name_str}"),
        }
    } else if state.config.show_icons {
        let icon = state.config.icons.icon(kind, path);
        match size_suffix {
            Some(suffix) => println!("{icon}{name_str}{suffix}"),
            None => println!("{icon}{name_str}"),
        }
    } else {
        match size_suffix {
            Some(suffix) => println!("{name_str}{suffix}"),
            None => println!("{name_str}"),
        }
    }

    Ok(())
}

fn build_json_node(path: &Path, state: &mut TraverseState) -> Result<JsonNode, Error> {
    if let Some(max_depth) = state.config.max_depth
        && state.current_depth > max_depth
    {
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string());
        return Ok(JsonNode {
            name,
            path: format_output_path(&state.relative_path),
            kind: "unknown",
            size: None,
            children: None,
        });
    }

    if !entry_within_limit(state) {
        state.limit_reached = true;
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string());
        return Ok(JsonNode {
            name,
            path: format_output_path(&state.relative_path),
            kind: "unknown",
            size: None,
            children: None,
        });
    }

    let kind = detect_entry_kind(path)?;
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string());
    let size = if state.config.show_size {
        entry_file_size(path)?
    } else {
        None
    };

    state.stats.record_entry(path);
    state.is_root = false;
    mark_entry_listed(state);

    if !path.is_dir() {
        return Ok(JsonNode {
            name,
            path: format_output_path(&state.relative_path),
            kind: json_kind(kind),
            size,
            children: None,
        });
    }

    if state.limit_reached {
        return Ok(JsonNode {
            name,
            path: format_output_path(&state.relative_path),
            kind: json_kind(kind),
            size,
            children: None,
        });
    }

    let canonical = path
        .canonicalize()
        .map_err(|e| Error::GetMetadata(path.display().to_string(), e))?;

    if !state.visited_dirs.insert(canonical.clone()) {
        return Ok(JsonNode {
            name,
            path: format_output_path(&state.relative_path),
            kind: json_kind(kind),
            size,
            children: None,
        });
    }

    if let Some(filter) = state.gitignore_filter.as_mut() {
        filter.enter_dir(&canonical)?;
    }

    state.current_depth += 1;

    let entries = read_all_entries(path)?;
    let mut filtered = filter_entries_by_config(
        &canonical,
        &state.relative_path,
        entries,
        state.config,
        state.gitignore_filter.as_deref(),
    )?;
    sort_entries(&mut filtered, state.config)?;

    let mut children = Vec::new();
    for entry in filtered {
        if state.limit_reached {
            break;
        }
        state.relative_path.push(entry.file_name().as_os_str());
        children.push(build_json_node(entry.path().as_path(), state)?);
        state.relative_path.pop();
        if state.limit_reached {
            break;
        }
    }

    state.current_depth -= 1;

    if let Some(filter) = state.gitignore_filter.as_mut() {
        filter.leave_dir();
    }

    Ok(JsonNode {
        name,
        path: format_output_path(&state.relative_path),
        kind: json_kind(kind),
        size,
        children: if children.is_empty() {
            None
        } else {
            Some(children)
        },
    })
}

fn print_tree_recur(path: &Path, state: &mut TraverseState) -> Result<(), Error> {
    if let Some(max_depth) = state.config.max_depth
        && state.current_depth > max_depth
    {
        return Ok(());
    }

    if !entry_within_limit(state) {
        state.limit_reached = true;
        return Ok(());
    }

    if state.config.paths_mode {
        if !(state.is_root && path.is_dir()) {
            print_path_line(path, state)?;
        }
    } else {
        print_path_name(path, state)?;
    }

    state.stats.record_entry(path);
    state.is_root = false;
    mark_entry_listed(state);

    if !path.is_dir() || state.limit_reached {
        return Ok(());
    }

    let canonical = path
        .canonicalize()
        .map_err(|e| Error::GetMetadata(path.display().to_string(), e))?;

    if !state.visited_dirs.insert(canonical.clone()) {
        return Ok(());
    }

    if let Some(filter) = state.gitignore_filter.as_mut() {
        filter.enter_dir(&canonical)?;
    }

    state.current_depth += 1;

    state.prepare_parent_indent();

    let entries = read_all_entries(path)?;

    let mut filtered = filter_entries_by_config(
        &canonical,
        &state.relative_path,
        entries,
        state.config,
        state.gitignore_filter.as_deref(),
    )?;

    sort_entries(&mut filtered, state.config)?;

    for (index, entry) in filtered.iter().enumerate() {
        if state.limit_reached {
            break;
        }
        let is_last = index == filtered.len() - 1;
        state.push_indent(is_last);
        state.relative_path.push(entry.file_name().as_os_str());
        print_tree_recur(entry.path().as_path(), state)?;
        state.relative_path.pop();
        state.pop_indent();
        if state.limit_reached {
            break;
        }
    }

    state.current_depth -= 1;

    if let Some(filter) = state.gitignore_filter.as_mut() {
        filter.leave_dir();
    }

    Ok(())
}

fn print_summary(stats: TreeStats, limit_reached: bool, file_limit: Option<usize>) {
    println!();
    if limit_reached && let Some(limit) = file_limit {
        println!("[file limit of {limit} entries reached]");
        println!();
    }
    println!("{}", stats.format_summary());
}

fn print_tree(path: &Path, config: &Config) -> Result<(), Error> {
    let mut filter = if !config.show_gitignore {
        Some(GitignoreFilter::new(path)?)
    } else {
        None
    };

    let mut state = TraverseState::new(config, filter.as_mut());
    print_tree_recur(path, &mut state)?;
    if config.show_report {
        print_summary(state.stats, state.limit_reached, config.file_limit);
    } else if state.limit_reached
        && let Some(limit) = config.file_limit
    {
        println!();
        println!("[file limit of {limit} entries reached]");
    }
    Ok(())
}

fn build_json_tree(path: &Path, config: &Config) -> Result<JsonNode, Error> {
    let mut filter = if !config.show_gitignore {
        Some(GitignoreFilter::new(path)?)
    } else {
        None
    };

    let mut state = TraverseState::new(config, filter.as_mut());
    build_json_node(path, &mut state)
}

pub fn print_trees(paths: &[&Path], options: &EffectiveOptions) -> Result<(), Error> {
    if paths.is_empty() {
        return Ok(());
    }

    if options.json {
        let mut trees = Vec::with_capacity(paths.len());
        for path in paths {
            let config = Config::from_options(options)?;
            trees.push(build_json_tree(path, &config)?);
        }

        let output = if trees.len() == 1 {
            serde_json::to_string_pretty(&trees[0])?
        } else {
            serde_json::to_string_pretty(&trees)?
        };
        println!("{output}");
        return Ok(());
    }

    for (index, path) in paths.iter().enumerate() {
        let config = Config::from_options(options)?;
        if index > 0 && !config.paths_mode {
            println!();
        }
        print_tree(path, &config)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_pluralizes_directory_and_file_labels() {
        assert_eq!(
            TreeStats {
                directories: 1,
                files: 1,
            }
            .format_summary(),
            "1 directory, 1 file"
        );
        assert_eq!(
            TreeStats {
                directories: 2,
                files: 3,
            }
            .format_summary(),
            "2 directories, 3 files"
        );
    }

    #[test]
    fn format_output_path_uses_forward_slashes() {
        assert_eq!(format_output_path(Path::new("src/main.rs")), "src/main.rs");
        assert_eq!(format_output_path(Path::new("")), ".");
    }

    #[test]
    fn format_human_size_uses_binary_units() {
        assert_eq!(format_human_size(512), "512B");
        assert_eq!(format_human_size(1024), "1.0K");
        assert_eq!(format_human_size(1536), "1.5K");
    }
}
