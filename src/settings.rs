use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::color::BuiltinColorTheme;
use crate::command::CmdArgs;
use crate::error::Error;
use crate::icon::BuiltinIconTheme;
use crate::sort::SortMode;

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
struct FileConfig {
    level: Option<usize>,
    all: Option<bool>,
    dir_only: Option<bool>,
    no_color: Option<bool>,
    ascii: Option<bool>,
    show_gitignore: Option<bool>,
    no_icons: Option<bool>,
    icon_theme: Option<BuiltinIconTheme>,
    color_theme: Option<BuiltinColorTheme>,
    no_report: Option<bool>,
    no_dirsfirst: Option<bool>,
    ignore: Vec<String>,
    pattern: Vec<String>,
    paths: Option<bool>,
    full_path: Option<bool>,
    size: Option<bool>,
    human_size: Option<bool>,
    json: Option<bool>,
    sort: Option<SortMode>,
    reverse: Option<bool>,
    filelimit: Option<usize>,
    parallel: Option<bool>,
}

#[derive(Debug)]
pub struct EffectiveOptions {
    pub level: Option<usize>,
    pub all: bool,
    pub dir_only: bool,
    pub no_color: bool,
    pub ascii: bool,
    pub show_gitignore: bool,
    pub no_icons: bool,
    pub icon_theme: BuiltinIconTheme,
    pub color_theme: BuiltinColorTheme,
    pub no_report: bool,
    pub no_dirsfirst: bool,
    pub ignore_patterns: Vec<String>,
    pub include_patterns: Vec<String>,
    pub paths: bool,
    pub full_path: bool,
    pub show_size: bool,
    pub human_size: bool,
    pub json: bool,
    pub sort_mode: SortMode,
    pub sort_reverse: bool,
    pub file_limit: Option<usize>,
    pub parallel: bool,
}

impl EffectiveOptions {
    pub fn from_args(args: &CmdArgs, tree_root: &Path) -> Result<Self, Error> {
        let file = load_merged_file_config(args, tree_root)?;

        Ok(Self {
            level: args.level.or(file.level),
            all: pick_bool(args.all, file.all, false),
            dir_only: pick_bool(args.dir_only, file.dir_only, false),
            no_color: pick_bool(args.no_color, file.no_color, false),
            ascii: pick_bool(args.ascii, file.ascii, false),
            show_gitignore: pick_bool(args.show_gitignore, file.show_gitignore, false),
            no_icons: pick_bool(args.no_icons, file.no_icons, false),
            icon_theme: args.icon_theme.or(file.icon_theme).unwrap_or_default(),
            color_theme: args.color_theme.or(file.color_theme).unwrap_or_default(),
            no_report: pick_bool(args.no_report, file.no_report, false),
            no_dirsfirst: pick_bool(args.no_dirsfirst, file.no_dirsfirst, false),
            ignore_patterns: if args.ignore_patterns.is_empty() {
                file.ignore
            } else {
                args.ignore_patterns.clone()
            },
            include_patterns: if args.include_patterns.is_empty() {
                file.pattern
            } else {
                args.include_patterns.clone()
            },
            paths: pick_bool(args.paths, file.paths, false),
            full_path: pick_bool(args.full_path, file.full_path, false),
            show_size: pick_bool(args.size, file.size, false)
                || pick_bool(args.human_size, file.human_size, false),
            human_size: pick_bool(args.human_size, file.human_size, false),
            json: pick_bool(args.json, file.json, false),
            sort_mode: args.sort.or(file.sort).unwrap_or(SortMode::Name),
            sort_reverse: pick_bool(args.reverse, file.reverse, false),
            file_limit: args.filelimit.or(file.filelimit),
            parallel: pick_bool(args.parallel, file.parallel, false),
        })
    }
}

fn pick_bool(cli: bool, file: Option<bool>, default: bool) -> bool {
    if cli { true } else { file.unwrap_or(default) }
}

fn load_merged_file_config(args: &CmdArgs, tree_root: &Path) -> Result<FileConfig, Error> {
    let mut merged = FileConfig::default();

    if let Some(path) = user_config_path()
        && path.is_file()
    {
        merge_file_into(&mut merged, &path)?;
    }

    if let Some(path) = discover_project_config(tree_root)
        && path.is_file()
    {
        merge_file_into(&mut merged, &path)?;
    }

    if let Some(path) = &args.config {
        if path.is_file() {
            merge_file_into(&mut merged, path)?;
        } else {
            return Err(Error::ConfigNotFound(path.display().to_string()));
        }
    }

    Ok(merged)
}

fn merge_file_into(merged: &mut FileConfig, path: &Path) -> Result<(), Error> {
    let contents = fs::read_to_string(path)
        .map_err(|err| Error::ReadConfig(path.display().to_string(), err))?;
    let next: FileConfig = toml::from_str(&contents)
        .map_err(|err| Error::ParseConfig(path.display().to_string(), err.to_string()))?;
    merge_file_config(merged, next);
    Ok(())
}

fn merge_file_config(base: &mut FileConfig, overlay: FileConfig) {
    if let Some(level) = overlay.level {
        base.level = Some(level);
    }
    merge_option_bool(&mut base.all, overlay.all);
    merge_option_bool(&mut base.dir_only, overlay.dir_only);
    merge_option_bool(&mut base.no_color, overlay.no_color);
    merge_option_bool(&mut base.ascii, overlay.ascii);
    merge_option_bool(&mut base.show_gitignore, overlay.show_gitignore);
    merge_option_bool(&mut base.no_icons, overlay.no_icons);
    if overlay.icon_theme.is_some() {
        base.icon_theme = overlay.icon_theme;
    }
    if overlay.color_theme.is_some() {
        base.color_theme = overlay.color_theme;
    }
    merge_option_bool(&mut base.no_report, overlay.no_report);
    merge_option_bool(&mut base.no_dirsfirst, overlay.no_dirsfirst);
    if !overlay.ignore.is_empty() {
        base.ignore = overlay.ignore;
    }
    if !overlay.pattern.is_empty() {
        base.pattern = overlay.pattern;
    }
    merge_option_bool(&mut base.paths, overlay.paths);
    merge_option_bool(&mut base.full_path, overlay.full_path);
    merge_option_bool(&mut base.size, overlay.size);
    merge_option_bool(&mut base.human_size, overlay.human_size);
    merge_option_bool(&mut base.json, overlay.json);
    if overlay.sort.is_some() {
        base.sort = overlay.sort;
    }
    merge_option_bool(&mut base.reverse, overlay.reverse);
    if overlay.filelimit.is_some() {
        base.filelimit = overlay.filelimit;
    }
    merge_option_bool(&mut base.parallel, overlay.parallel);
}

fn merge_option_bool(target: &mut Option<bool>, value: Option<bool>) {
    if value.is_some() {
        *target = value;
    }
}

fn user_config_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("PTREE_CONFIG") {
        return Some(PathBuf::from(path));
    }

    if let Ok(path) = std::env::var("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(path).join("ptree").join("config.toml"));
    }

    home_dir().map(|home| home.join(".config").join("ptree").join("config.toml"))
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

fn discover_project_config(tree_root: &Path) -> Option<PathBuf> {
    let mut current = tree_root.canonicalize().ok()?;
    loop {
        let candidate = current.join(".ptree.toml");
        if candidate.is_file() {
            return Some(candidate);
        }
        if !current.pop() {
            break;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("ptree_settings_{name}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn project_config_sets_defaults_when_cli_not_provided() {
        let dir = temp_dir("project_config");
        fs::write(
            dir.join(".ptree.toml"),
            "level = 2\nno-icons = true\nignore = [\"vendor\"]\n",
        )
        .unwrap();

        let args = CmdArgs {
            targets: vec![dir.display().to_string()],
            level: None,
            all: false,
            dir_only: false,
            no_color: false,
            ascii: false,
            show_gitignore: false,
            no_icons: false,
            icon_theme: None,
            color_theme: None,
            no_report: false,
            no_dirsfirst: false,
            ignore_patterns: Vec::new(),
            include_patterns: Vec::new(),
            paths: false,
            full_path: false,
            size: false,
            human_size: false,
            json: false,
            sort: None,
            reverse: false,
            filelimit: None,
            parallel: false,
            config: None,
        };

        let effective = EffectiveOptions::from_args(&args, &dir).unwrap();
        assert_eq!(effective.level, Some(2));
        assert!(effective.no_icons);
        assert_eq!(effective.ignore_patterns, vec!["vendor"]);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn cli_overrides_project_config() {
        let dir = temp_dir("cli_override");
        fs::write(dir.join(".ptree.toml"), "level = 5\n").unwrap();

        let args = CmdArgs {
            targets: vec![dir.display().to_string()],
            level: Some(1),
            all: false,
            dir_only: false,
            no_color: false,
            ascii: false,
            show_gitignore: false,
            no_icons: false,
            icon_theme: None,
            color_theme: None,
            no_report: false,
            no_dirsfirst: false,
            ignore_patterns: Vec::new(),
            include_patterns: Vec::new(),
            paths: false,
            full_path: false,
            size: false,
            human_size: false,
            json: false,
            sort: None,
            reverse: false,
            filelimit: None,
            parallel: false,
            config: None,
        };

        let effective = EffectiveOptions::from_args(&args, &dir).unwrap();
        assert_eq!(effective.level, Some(1));
        let _ = fs::remove_dir_all(&dir);
    }
}
