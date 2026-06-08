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
ptree [OPTIONS] [PATH]
```

`PATH` defaults to the current directory (`.`).

### Options

| Flag                    | Description                                                          |
| ----------------------- | -------------------------------------------------------------------- |
| `-L`, `--level <N>`     | Maximum depth (root is level 1)                                      |
| `-a`, `--all`           | Show hidden entries (names starting with `.`)                        |
| `-d`, `--dir-only`      | Show only directories                                                |
| `--no-color`            | Disable color (also off when stdout is not color-capable)            |
| `--ascii`               | Use ASCII tree characters (`\|--`, `` `-- ``) instead of box-drawing |
| `--show-gitignore`      | Show entries matched by `.gitignore` (hidden by default)             |
| `--no-icons`            | Disable icon output                                                  |
| `--icon-theme <THEME>`  | Icon set: `nerd-font` (default), `ascii`                             |
| `--color-theme <THEME>` | Color theme: `dark` (default)                                        |
| `--no-report`           | Omit the directory/file count summary at the end                     |

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
