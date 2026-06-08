use std::collections::HashSet;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::config::IndentType;
use crate::error::Error;
use crate::filter::GitignoreFilter;
use crate::filter::filter_entries_by_config;
use crate::icon::detect_entry_kind;

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

struct TraverseState<'a> {
    indent_symbols: Vec<&'static str>,
    is_last: bool,
    current_depth: usize,
    config: &'a Config,
    gitignore_filter: Option<&'a mut GitignoreFilter>,
    visited_dirs: HashSet<DirIdentity>,
    stats: TreeStats,
}

impl<'a> TraverseState<'a> {
    pub fn new(config: &'a Config, gitignore_filter: Option<&'a mut GitignoreFilter>) -> Self {
        Self {
            indent_symbols: Vec::new(),
            is_last: true,
            current_depth: 1,
            config,
            gitignore_filter,
            visited_dirs: HashSet::new(),
            stats: TreeStats::default(),
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

fn print_path_name(path: &Path, state: &mut TraverseState) -> Result<(), Error> {
    state.print_current_indent();

    let kind = detect_entry_kind(path)?;
    let name = path.file_name().unwrap_or(path.as_os_str());
    let name_str = name.to_string_lossy();

    if state.config.use_color {
        if state.config.show_icons {
            let icon = state.config.icons.icon(kind, path);
            println!("{}{name_str}", state.config.colors.paint(kind, path, icon),);
        } else {
            println!("{}", state.config.colors.paint(kind, path, &name_str));
        }
    } else if state.config.show_icons {
        let icon = state.config.icons.icon(kind, path);
        println!("{icon}{name_str}");
    } else {
        println!("{name_str}");
    }

    Ok(())
}

fn print_tree_recur(path: &Path, state: &mut TraverseState) -> Result<(), Error> {
    if let Some(max_depth) = state.config.max_depth
        && state.current_depth > max_depth
    {
        return Ok(());
    }

    print_path_name(path, state)?;
    state.stats.record_entry(path);

    if !path.is_dir() {
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
        entries,
        state.config,
        state.gitignore_filter.as_deref(),
    )?;

    filtered.sort_by_key(|a| a.file_name());

    for (index, entry) in filtered.iter().enumerate() {
        let is_last = index == filtered.len() - 1;
        state.push_indent(is_last);
        print_tree_recur(entry.path().as_path(), state)?;
        state.pop_indent();
    }

    state.current_depth -= 1;

    if let Some(filter) = state.gitignore_filter.as_mut() {
        filter.leave_dir();
    }

    Ok(())
}

fn print_summary(stats: TreeStats) {
    println!();
    println!("{}", stats.format_summary());
}

pub fn print_tree(path: &Path, config: &Config) -> Result<(), Error> {
    let mut filter = if !config.show_gitignore {
        Some(GitignoreFilter::new(path)?)
    } else {
        None
    };

    let mut state = TraverseState::new(config, filter.as_mut());
    print_tree_recur(path, &mut state)?;
    if config.show_report {
        print_summary(state.stats);
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
}
