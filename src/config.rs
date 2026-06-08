use crate::color::{Colors, configure_color_output, resolve_use_color};
use crate::command::CmdArgs;
use crate::icon::Icons;

pub struct Config {
    pub max_depth: Option<usize>,
    pub show_all: bool,
    pub show_dir_only: bool,
    pub use_color: bool,
    pub use_only_ascii: bool,
    pub show_gitignore: bool,
    pub show_icons: bool,
    pub show_report: bool,
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
    pub fn build_from_args(args: &CmdArgs) -> Self {
        let use_color = resolve_use_color(args.no_color);
        configure_color_output(use_color);

        Self {
            max_depth: args.level,
            show_all: args.all,
            show_dir_only: args.dir_only,
            use_color,
            use_only_ascii: args.ascii,
            show_gitignore: args.show_gitignore,
            show_icons: !args.no_icons,
            show_report: !args.no_report,
            icons: Icons::new(args.icon_theme),
            colors: Colors::with_theme(args.color_theme),
        }
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
