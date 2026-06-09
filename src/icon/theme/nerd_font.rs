use phf::Map;
use phf_macros::phf_map;

use super::IconTheme;

static KIND_ICONS: Map<&str, &str> = phf_map! {
    "directory" => "󰉋 ",
    "symlink" => "󰌷 ",
    "executable" => "󰡯 ",
    "readonly" => "󰌾 ",
    "file" => "󰈔 ",
};

static EXTENSION_ICONS: Map<&str, &str> = phf_map! {
    // C / C++
    "c" => " ",
    "C" => " ",
    "h" => " ",
    "cpp" => " ",
    "cc" => " ",
    "cxx" => " ",
    "hpp" => " ",
    "hxx" => " ",

    // Systems / native
    "rs" => " ",
    "go" => " ",
    "zig" => " ",
    "nim" => " ",
    "v" => " ",
    "asm" => " ",
    "s" => " ",
    "S" => " ",
    "wasm" => " ",

    // JVM / .NET
    "java" => "󰬷 ",
    "kt" => "󱈙 ",
    "kts" => "󱈙 ",
    "scala" => " ",
    "sc" => " ",
    "groovy" => " ",
    "cs" => "󰌛 ",
    "fs" => " ",
    "fsx" => " ",

    // Scripting
    "py" => "󰌠 ",
    "pyw" => "󰌠 ",
    "rb" => "󰴭 ",
    "php" => "󰌟 ",
    "lua" => "󰢱 ",
    "pl" => "󰛄 ",
    "pm" => "󰛄 ",
    "r" => "󰟔 ",
    "R" => "󰟔 ",
    "sh" => " ",
    "bash" => " ",
    "zsh" => " ",
    "fish" => " ",
    "ps1" => " ",
    "psm1" => " ",

    // Web frontend
    "js" => "󰌞 ",
    "mjs" => "󰌞 ",
    "cjs" => "󰌞 ",
    "jsx" => "󰌞 ",
    "ts" => "󰛦 ",
    "tsx" => "󰛦 ",
    "vue" => "󰡄 ",
    "svelte" => " ",
    "html" => "󰌝 ",
    "htm" => "󰌝 ",
    "css" => " ",
    "scss" => " ",
    "sass" => " ",
    "less" => " ",
    "svg" => "󰜡 ",

    // Functional
    "hs" => "󰲒 ",
    "lhs" => "󰲒 ",
    "ex" => " ",
    "exs" => " ",
    "heex" => " ",
    "erl" => " ",
    "hrl" => " ",
    "clj" => " ",
    "cljs" => " ",
    "cljc" => " ",
    "edn" => " ",

    // Mobile
    "swift" => " ",
    "dart" => " ",
    "m" => " ",
    "mm" => " ",

    // other languages
    "vala" => " ",

    // Data / config
    "json" => "󰘦 ",
    "jsonc" => "󰘦 ",
    "yaml" => " ",
    "yml" => " ",
    "toml" => " ",
    "xml" => "󰗀 ",
    "ini" => "󰒓 ",
    "cfg" => "󰒓 ",
    "conf" => "󰒓 ",
    "env" => "󰇘 ",
    "sql" => "󰆼 ",
    "graphql" => " ",
    "gql" => " ",
    "tf" => "󱁢 ",
    "tfvars" => "󱁢 ",
    "prisma" => " ",

    // Markup / docs
    "md" => "󰍔 ",
    "mdx" => "󰍔 ",
    "txt" => "󰈙 ",
    "tex" => " ",
    "org" => " ",

    // Office / documents
    "pdf" => " ",
    "doc" => " ",
    "docx" => " ",
    "xls" => "󱎏 ",
    "xlsx" => "󱎏 ",
    "csv" => " ",
    "ppt" => "󱎐 ",
    "pptx" => "󱎐 ",

    // Images
    "png" => "󰈟 ",
    "jpg" => "󰈟 ",
    "jpeg" => "󰈟 ",
    "gif" => "󰈟 ",
    "webp" => "󰈟 ",
    "bmp" => "󰈟 ",
    "ico" => "󰈟 ",
    "tiff" => "󰈟 ",
    "tif" => "󰈟 ",

    // Audio / video
    "mp3" => "󰈣 ",
    "wav" => "󰈣 ",
    "flac" => "󰈣 ",
    "ogg" => "󰈣 ",
    "aac" => "󰈣 ",
    "mp4" => "󰈫 ",
    "mkv" => "󰈫 ",
    "avi" => "󰈫 ",
    "mov" => "󰈫 ",
    "webm" => "󰈫 ",

    // Archives
    "zip" => "󰗄 ",
    "tar" => "󰗄 ",
    "gz" => "󰗄 ",
    "bz2" => "󰗄 ",
    "xz" => "󰗄 ",
    "7z" => "󰗄 ",
    "rar" => "󰗄 ",
    "tgz" => "󰗄 ",

    // Lock / patch / diff
    "lock" => " ",
    "patch" => "󰏫 ",
    "diff" => "󰏫 ",

    // Misc
    "o" => " ",
    "out" => " ",
    "obj" => " ",
    "exe" => " ",
    "mk" => " ",
    "cmake" => " ",
};

/// Exact filename matches for well-known project and config files.
static SPECIAL_ICONS: Map<&str, &str> = phf_map! {
    // Build systems
    "Makefile" => " ",
    "makefile" => " ",
    "GNUmakefile" => " ",
    "CMakeLists.txt" => " ",

    // Containers
    "Dockerfile" => "󰡨 ",
    "Containerfile" => "󰡨 ",
    "docker-compose.yml" => "󰡨 ",
    "docker-compose.yaml" => "󰡨 ",
    "compose.yml" => "󰡨 ",
    "compose.yaml" => "󰡨 ",

    // Documentation
    "README" => "󰂺 ",
    "README.md" => "󰂺 ",
    "README.rst" => "󰂺 ",
    "README.txt" => "󰂺 ",
    "CHANGELOG" => "󰋚 ",
    "CHANGELOG.md" => "󰋚 ",
    "CONTRIBUTING" => "󰅍 ",
    "CONTRIBUTING.md" => "󰅍 ",
    "AUTHORS" => "󰀄 ",
    "CODEOWNERS" => "󰀄 ",

    // License
    "LICENSE" => "󰿃 ",
    "LICENSE.md" => "󰿃 ",
    "LICENSE.txt" => "󰿃 ",
    "COPYING" => "󰿃 ",
    "UNLICENSE" => "󰿃 ",

    // Git / editor
    ".gitignore" => "󰊢 ",
    ".gitattributes" => "󰊢 ",
    ".gitmodules" => "󰊢 ",
    ".editorconfig" => " ",
    ".prettierrc" => " ",
    ".prettierrc.js" => " ",
    ".prettierrc.json" => " ",
    ".eslintrc" => " ",
    ".eslintrc.js" => " ",
    ".eslintrc.json" => " ",
    ".eslintignore" => " ",

    // Environment
    ".env" => "󰇘 ",
    ".env.local" => "󰇘 ",
    ".env.example" => "󰇘 ",
    ".envrc" => "󰇘 ",

    // Rust
    "Cargo.toml" => "󰒓 ",
    "Cargo.lock" => " ",

    // Node / JS
    "package.json" => " ",
    "package-lock.json" => " ",
    "yarn.lock" => " ",
    "pnpm-lock.yaml" => " ",
    "tsconfig.json" => "󰛦 ",
    "jsconfig.json" => "󰌞 ",
    "vite.config.js" => " ",
    "vite.config.ts" => " ",
    "webpack.config.js" => " ",
    "rollup.config.js" => " ",

    // Go
    "go.mod" => " ",
    "go.sum" => " ",

    // Python
    "requirements.txt" => "󰌠 ",
    "Pipfile" => "󰌠 ",
    "Pipfile.lock" => "󰌠 ",
    "pyproject.toml" => "󰌠 ",
    "setup.py" => "󰌠 ",
    "setup.cfg" => "󰌠 ",

    // Ruby / PHP / Java
    "Gemfile" => "󰴭 ",
    "Gemfile.lock" => "󰴭 ",
    "Rakefile" => "󰴭 ",
    "composer.json" => "󰌟 ",
    "composer.lock" => "󰌟 ",
    "build.gradle" => " ",
    "build.gradle.kts" => "󱈙 ",
    "pom.xml" => " ",
    "settings.gradle" => " ",

    // Infrastructure / misc
    ".clang-format" => "󰒓 ",
    ".clangd" => "󰒓 ",
    ".vscode" => " ",
    "flake.nix" => "󱄅 ",
    "shell.nix" => "󱄅 ",
    "default.nix" => "󱄅 ",
    "Vagrantfile" => " ",
    "Procfile" => " ",
    "Gruntfile.js" => " ",
    "Gulpfile.js" => " ",
};

pub const THEME: IconTheme = IconTheme {
    name: "nerd-font",
    kinds: &KIND_ICONS,
    extensions: &EXTENSION_ICONS,
    special: &SPECIAL_ICONS,
    fallback: "󰈔 ",
};
