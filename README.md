# ptree

A `tree`-like directory listing tool written in Rust, with syntax-highlighted names, Nerd Font icons, and `.gitignore` awareness.

## Install

```bash
cargo install --path .
```

Or run directly:

```bash
cargo run -- .
```

## Usage

```bash
ptree [OPTIONS] [PATH]...
```

`PATH` defaults to the current directory (`.`).

### Options

| Flag                    | Description                                                               |
| ----------------------- | ------------------------------------------------------------------------- |
| `-L`, `--level <N>`     | Maximum depth (root is level 1)                                           |
| `-a`, `--all`           | Show hidden entries (names starting with `.`)                             |
| `-d`, `--dir-only`      | Show only directories                                                     |
| `--no-color`            | Disable color (also off when stdout is not color-capable)                 |
| `--ascii`               | Use ASCII tree characters (`\|--`, `` `-- ``) instead of box-drawing      |
| `--show-gitignore`      | Show entries matched by `.gitignore` (hidden by default)                  |
| `--no-icons`            | Disable icon output                                                       |
| `--icon-theme <THEME>`  | Icon set: `nerd-font` (default), `ascii`                                  |
| `--color-theme <THEME>` | Color theme: `dark`, `light`, or `auto`                                   |
| `--no-report`           | Omit the directory/file count summary at the end                          |
| `--no-dirsfirst`        | List entries in plain name order (default: directories before files)      |
| `-I`, `--ignore`        | Exclude entries matching glob pattern(s); repeat or use `\|` in one value |
| `-P`, `--pattern`       | Include only matching entries (parent dirs of matches are shown)          |
| `--paths`               | Print one relative path per line (for pipes, `fzf`, `xargs`)              |
| `--full-path`           | With `--paths`, print absolute paths                                      |
| `-J`, `--json`          | Print the tree as JSON (conflicts with `--paths`)                         |
| `-s`, `--size`          | Show file sizes in bytes beside file names                                |
| `--human-size`          | Show human-readable file sizes (implies `--size`)                         |
| `--config <FILE>`       | Load an additional TOML config file                                       |
| `--sort <MODE>`         | Sort by `name`, `ext`, or `none` (default: `name`)                        |
| `--reverse`             | Reverse the sort order                                                    |
| `--filelimit <N>`       | Stop after listing `N` entries                                            |
| `--parallel`            | Parallel sort for directories with many entries                           |

`PATH` accepts multiple paths: `ptree src tests Cargo.toml`.

### Configuration file

`ptree` loads settings from (later sources override earlier ones):

1. `~/.config/ptree/config.toml` (or `$XDG_CONFIG_HOME/ptree/config.toml`)
2. `.ptree.toml` in `PATH` or any parent directory
3. `--config <FILE>` if provided

CLI flags always override file settings. Example `~/.config/ptree/config.toml`:

```toml
level = 3
no-icons = false
ignore = ["target", "node_modules"]
color-theme = "dark"
```

Set `PTREE_CONFIG` to use a specific file instead of the default user config path.

`color-theme = "auto"` picks `light` or `dark` from the terminal background (`$COLORFGBG`).

### Shell completions

```bash
ptree completions bash > ~/.local/share/bash-completion/completions/ptree
ptree completions fish > ~/.config/fish/completions/ptree.fish
ptree completions zsh  > _ptree
```

### Examples

```bash
# Current directory, default styling
ptree

# Two levels deep, ASCII tree, no color (for scripts / CI)
ptree -L 2 --ascii --no-color

# Directories only, respect .gitignore
ptree -d src

# Show ignored files too
ptree --show-gitignore

# Only Rust sources; exclude build artifacts
ptree -P '*.rs' -I 'target|*.o' src

# One path per line for fzf
ptree --paths src | fzf

# Plain name order instead of directories-first
ptree --no-dirsfirst

# JSON for tooling
ptree -J src | jq '.children[].name'

# Show file sizes
ptree --human-size src

# Use a config file
ptree --config ./ptree.toml

# Light theme or follow terminal background
ptree --color-theme light
ptree --color-theme auto

# Sort by extension; cap output size
ptree --sort ext --filelimit 50

# Multiple roots
ptree src tests
```

## Behavior notes

### `.gitignore`

By default, entries matched by `.gitignore` rules are hidden. Rules from ancestor directories apply when listing a subdirectory. Use `--show-gitignore` to include them.

### Symbolic links

- **Directory symlinks** are followed when traversing and classifying entries: they appear as directories and their contents are listed.
- **File symlinks** (and broken symlinks) are shown as symlinks and are not descended into.
- **Cycles** (e.g. `a → b → a`) are detected via canonical path tracking; the second visit is skipped to avoid infinite loops.

### Summary

After the tree, `ptree` prints a blank line followed by counts of listed entries, for example:

```text
2 directories, 3 files
```

Only entries that appear in the output are counted (respecting depth limits, `.gitignore`, and `--dir-only`). Use `--no-report` to suppress the summary.

### Color

Color follows the [NO_COLOR](https://no-color.org/) convention and common `CLICOLOR*` / `FORCE_COLOR` environment variables. Use `--no-color` to force plain output.

## Development

```bash
cargo test
cargo clippy -- -D warnings
```

## License

MIT — see [LICENSE](LICENSE).
