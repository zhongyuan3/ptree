use crate::color::{Colors, configure_color_output, resolve_use_color};
use crate::error::Error;
use crate::filter::GlobFilter;
use crate::icon::Icons;
use crate::settings::EffectiveOptions;
use crate::sort::SortMode;

pub struct Config {
    pub max_depth: Option<usize>,
    pub show_all: bool,
    pub show_dir_only: bool,
    pub use_color: bool,
    pub use_only_ascii: bool,
    pub show_gitignore: bool,
    pub show_icons: bool,
    pub show_report: bool,
    pub dirsfirst: bool,
    pub sort_mode: SortMode,
    pub sort_reverse: bool,
    pub file_limit: Option<usize>,
    pub parallel: bool,
    pub paths_mode: bool,
    pub full_path: bool,
    pub show_size: bool,
    pub human_size: bool,
    pub glob_filter: Option<GlobFilter>,
    pub icons: Icons,
    pub colors: Colors,
}

pub enum IndentType {
    Space,
    Vertical,
    Last,
    Continue,
}

impl Config {
    pub fn from_options(options: &EffectiveOptions) -> Result<Self, Error> {
        let machine_output = options.paths || options.json;
        let use_color = resolve_use_color(options.no_color) && !machine_output;
        configure_color_output(use_color);

        let glob_filter =
            GlobFilter::from_pattern_args(&options.include_patterns, &options.ignore_patterns)?;

        Ok(Self {
            max_depth: options.level,
            show_all: options.all,
            show_dir_only: options.dir_only,
            use_color,
            use_only_ascii: options.ascii,
            show_gitignore: options.show_gitignore,
            show_icons: !options.no_icons && !machine_output,
            show_report: !options.no_report && !machine_output,
            dirsfirst: !options.no_dirsfirst,
            sort_mode: options.sort_mode,
            sort_reverse: options.sort_reverse,
            file_limit: options.file_limit,
            parallel: options.parallel,
            paths_mode: options.paths,
            full_path: options.full_path,
            show_size: options.show_size,
            human_size: options.human_size,
            glob_filter,
            icons: Icons::new(options.icon_theme),
            colors: Colors::with_theme(options.color_theme),
        })
    }

    pub fn get_indent(&self, indent_type: IndentType) -> &'static str {
        match indent_type {
            IndentType::Space => self.get_indent_space(),
            IndentType::Vertical => self.get_indent_vertical(),
            IndentType::Last => self.get_indent_last(),
            IndentType::Continue => self.get_indent_continue(),
        }
    }

    fn get_indent_space(&self) -> &'static str {
        "    "
    }

    fn get_indent_vertical(&self) -> &'static str {
        if self.use_only_ascii {
            "|   "
        } else {
            "│   "
        }
    }

    fn get_indent_last(&self) -> &'static str {
        if self.use_only_ascii {
            "`-- "
        } else {
            "└── "
        }
    }

    fn get_indent_continue(&self) -> &'static str {
        if self.use_only_ascii {
            "|-- "
        } else {
            "├── "
        }
    }
}

#[cfg(test)]
pub(crate) fn test_config_with_glob(glob_filter: GlobFilter) -> Config {
    test_config(SortMode::Name, true, false).with_glob_filter(glob_filter)
}

#[cfg(test)]
pub(crate) fn test_config(sort_mode: SortMode, dirsfirst: bool, parallel: bool) -> Config {
    Config {
        max_depth: None,
        show_all: false,
        show_dir_only: false,
        use_color: false,
        use_only_ascii: false,
        show_gitignore: true,
        show_icons: false,
        show_report: false,
        dirsfirst,
        sort_mode,
        sort_reverse: false,
        file_limit: None,
        parallel,
        paths_mode: false,
        full_path: false,
        show_size: false,
        human_size: false,
        glob_filter: None,
        icons: Icons::default(),
        colors: Colors::default(),
    }
}

#[cfg(test)]
impl Config {
    fn with_glob_filter(mut self, glob_filter: GlobFilter) -> Self {
        self.glob_filter = Some(glob_filter);
        self
    }
}
