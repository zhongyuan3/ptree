use std::io::ErrorKind;
use std::path::Path;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;

use crate::color::BuiltinColorTheme;
use crate::error::Error;
use crate::format::print_trees;
use crate::icon::BuiltinIconTheme;
use crate::settings::EffectiveOptions;
use crate::sort::SortMode;

#[derive(Parser)]
#[command(version, name = "ptree")]
pub struct App {
    #[command(subcommand)]
    pub command: Option<AppCommand>,

    #[command(flatten)]
    pub args: CmdArgs,
}

#[derive(Subcommand)]
pub enum AppCommand {
    /// Generate shell completion scripts
    Completions {
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[derive(Parser)]
pub struct CmdArgs {
    #[arg(value_name = "PATH", default_values_t = vec![String::from(".")])]
    pub targets: Vec<String>,

    #[arg(short = 'L', long = "level", help = "Maximum depth of the tree")]
    pub level: Option<usize>,

    #[arg(
        short = 'a',
        long = "all",
        default_value_t = false,
        help = "Show all entries"
    )]
    pub all: bool,

    #[arg(
        short = 'd',
        long = "dir-only",
        default_value_t = false,
        help = "Show only directories"
    )]
    pub dir_only: bool,

    #[arg(
        long = "no-color",
        default_value_t = false,
        help = "Disable color output (also disabled when stdout is not color-capable)"
    )]
    pub no_color: bool,

    #[arg(long, default_value_t = false, help = "Use only ASCII characters")]
    pub ascii: bool,

    #[arg(
        long = "show-gitignore",
        default_value_t = false,
        help = "Show entries matched by .gitignore (default: hide them)"
    )]
    pub show_gitignore: bool,

    #[arg(
        long = "no-icons",
        default_value_t = false,
        help = "Disable icon output"
    )]
    pub no_icons: bool,

    #[arg(long = "icon-theme", help = "Icon set to use")]
    pub icon_theme: Option<BuiltinIconTheme>,

    #[arg(long = "color-theme", help = "Color theme: dark, light, or auto")]
    pub color_theme: Option<BuiltinColorTheme>,

    #[arg(
        long = "no-report",
        default_value_t = false,
        help = "Omit the directory/file count summary at the end"
    )]
    pub no_report: bool,

    #[arg(
        long = "no-dirsfirst",
        default_value_t = false,
        help = "Do not sort directories before files (directories-first is the default)"
    )]
    pub no_dirsfirst: bool,

    #[arg(
        long = "sort",
        value_enum,
        help = "Sort entries by name, extension, or leave unsorted"
    )]
    pub sort: Option<SortMode>,

    #[arg(
        long = "reverse",
        default_value_t = false,
        help = "Reverse the sort order"
    )]
    pub reverse: bool,

    #[arg(long = "filelimit", help = "Stop after listing this many entries")]
    pub filelimit: Option<usize>,

    #[arg(
        long = "parallel",
        default_value_t = false,
        help = "Use parallel sorting for directories with many entries"
    )]
    pub parallel: bool,

    #[arg(
        short = 'I',
        long = "ignore",
        action = clap::ArgAction::Append,
        help = "Exclude entries matching glob pattern(s); repeat flag or use `|` within a value"
    )]
    pub ignore_patterns: Vec<String>,

    #[arg(
        short = 'P',
        long = "pattern",
        action = clap::ArgAction::Append,
        help = "Include only matching entries (parent dirs of matches are shown); repeat or use `|`"
    )]
    pub include_patterns: Vec<String>,

    #[arg(
        long = "paths",
        default_value_t = false,
        conflicts_with = "json",
        help = "Print one relative path per line (plain output for pipes and fzf)"
    )]
    pub paths: bool,

    #[arg(
        long = "full-path",
        default_value_t = false,
        help = "With --paths, print absolute paths instead of paths relative to PATH"
    )]
    pub full_path: bool,

    #[arg(
        short = 'J',
        long = "json",
        default_value_t = false,
        conflicts_with = "paths",
        help = "Print the tree as JSON"
    )]
    pub json: bool,

    #[arg(
        short = 's',
        long = "size",
        default_value_t = false,
        help = "Show file sizes in bytes"
    )]
    pub size: bool,

    #[arg(
        long = "human-size",
        default_value_t = false,
        help = "Show human-readable file sizes (implies --size)"
    )]
    pub human_size: bool,

    #[arg(
        long = "config",
        help = "Load settings from an additional TOML config file"
    )]
    pub config: Option<std::path::PathBuf>,
}

impl App {
    pub fn cli_command() -> clap::Command {
        Self::command()
    }
}

pub fn generate_completions(shell: Shell) {
    let mut command = App::cli_command();
    let name = command.get_name().to_string();
    clap_complete::generate(shell, &mut command, name, &mut std::io::stdout());
}

fn validate_path(path: &Path) -> Result<(), Error> {
    path.symlink_metadata().map_err(|e| {
        if e.kind() == ErrorKind::NotFound {
            Error::PathNotFound(path.display().to_string())
        } else {
            Error::AccessPath(path.display().to_string(), e)
        }
    })?;
    Ok(())
}

pub fn exec_cmd(args: &CmdArgs) -> Result<(), Error> {
    if args.targets.is_empty() {
        return Err(Error::PathNotFound("no path provided".to_string()));
    }

    let primary = Path::new(args.targets[0].as_str());
    validate_path(primary)?;
    let options = EffectiveOptions::from_args(args, primary)?;

    let paths: Vec<_> = args
        .targets
        .iter()
        .map(|target| Path::new(target.as_str()))
        .collect();

    for path in &paths[1..] {
        validate_path(path)?;
    }

    print_trees(&paths, &options)
}
