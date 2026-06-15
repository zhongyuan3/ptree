use std::cmp::Ordering;
use std::ffi::OsString;
use std::fs::DirEntry;

use clap::ValueEnum;
use rayon::prelude::*;
use serde::Deserialize;

use crate::config::Config;
use crate::error::Error;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SortMode {
    #[default]
    Name,
    Ext,
    None,
}

pub fn sort_entries(entries: &mut Vec<DirEntry>, config: &Config) -> Result<(), Error> {
    if entries.is_empty() {
        return Ok(());
    }

    if config.sort_mode == SortMode::None && !config.dirsfirst {
        return Ok(());
    }

    let mut keyed = entries
        .drain(..)
        .enumerate()
        .map(|(index, entry)| {
            let file_type = entry
                .file_type()
                .map_err(|e| Error::GetFileType(entry.path().display().to_string(), e))?;
            Ok(SortableEntry {
                index,
                entry,
                is_dir: file_type.is_dir(),
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;

    let sorter = |a: &SortableEntry, b: &SortableEntry| compare_entries(a, b, config);

    if config.parallel && keyed.len() >= 64 {
        keyed.par_sort_by(sorter);
    } else {
        keyed.sort_by(sorter);
    }

    if config.sort_reverse {
        keyed.reverse();
    }

    entries.extend(keyed.into_iter().map(|item| item.entry));
    Ok(())
}

struct SortableEntry {
    index: usize,
    entry: DirEntry,
    is_dir: bool,
}

fn compare_entries(a: &SortableEntry, b: &SortableEntry, config: &Config) -> Ordering {
    if config.dirsfirst {
        match (a.is_dir, b.is_dir) {
            (true, false) => return Ordering::Less,
            (false, true) => return Ordering::Greater,
            _ => {}
        }
    }

    let ordering = match config.sort_mode {
        SortMode::None => a.index.cmp(&b.index),
        SortMode::Name => a.entry.file_name().cmp(&b.entry.file_name()),
        SortMode::Ext => {
            ext_sort_key(&a.entry.file_name()).cmp(&ext_sort_key(&b.entry.file_name()))
        }
    };

    ordering.then_with(|| a.entry.file_name().cmp(&b.entry.file_name()))
}

fn ext_sort_key(name: &std::ffi::OsStr) -> (String, OsString) {
    let name_str = name.to_string_lossy();
    if let Some((stem, ext)) = name_str.rsplit_once('.')
        && !stem.is_empty()
    {
        (ext.to_ascii_lowercase(), name.to_os_string())
    } else {
        (String::new(), name.to_os_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ptree_sort_{name}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn ext_sort_groups_by_extension() {
        let dir = temp_dir("ext_sort");
        fs::write(dir.join("b.rs"), "").unwrap();
        fs::write(dir.join("a.txt"), "").unwrap();
        fs::write(dir.join("c.rs"), "").unwrap();

        let entries = fs::read_dir(&dir)
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let mut entries = entries;
        let config = crate::config::test_config(SortMode::Ext, true, false);
        sort_entries(&mut entries, &config).unwrap();

        let names: Vec<_> = entries
            .iter()
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, vec!["b.rs", "c.rs", "a.txt"]);
        let _ = fs::remove_dir_all(&dir);
    }
}
