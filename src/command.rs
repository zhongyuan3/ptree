use std::io::ErrorKind;
use std::path::Path;

use clap::Parser;

use crate::color::BuiltinColorTheme;
use crate::config::Config;
use crate::error::Error;
use crate::format::print_tree;
use crate::icon::BuiltinIconTheme;

#[derive(Parser)]
#[command(version)]
pub struct CmdArgs {
    #[arg(default_value_t = String::from("."))]
    pub path: String,

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

    #[arg(
        long = "icon-theme",
        default_value = "nerd-font",
        help = "Icon set to use"
    )]
    pub icon_theme: BuiltinIconTheme,

    #[arg(
        long = "color-theme",
        default_value = "dark",
        help = "Color theme to use"
    )]
    pub color_theme: BuiltinColorTheme,

    #[arg(
        long = "no-report",
        default_value_t = false,
        help = "Omit the directory/file count summary at the end"
    )]
    pub no_report: bool,
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
    let path = Path::new(args.path.as_str());
    validate_path(path)?;
    let config = Config::build_from_args(args);
    print_tree(path, &config)
}
