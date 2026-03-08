# batmd

A terminal markdown viewer and editor — bat meets vim.

View rendered markdown with syntax highlighting, then drop into a raw editor to make changes. Built with Rust using ratatui.

## Install

Requires Rust toolchain. Install via [rustup](https://rustup.rs/) if needed.

```bash
git clone <repo-url>
cd batmde
cargo build --release
```

The binary will be at `target/release/batmd`. Copy it somewhere on your `$PATH`:

```bash
cp target/release/batmd ~/.local/bin/
```

## Usage

```bash
batmd README.md
```

Opens the file in view mode. If the file doesn't exist, it will be created.

## Keybindings

### View Mode

| Key | Action |
|-----|--------|
| `q` | Quit |
| `e` / `i` | Enter edit mode |
| `j` / `k` or arrows | Scroll up/down |
| `g` / `G` | Jump to top/bottom |
| `Ctrl-T` / `Ctrl-B` | Jump to top/bottom |
| `Page Up` / `Page Down` | Scroll by page |
| `/` | Search |
| `n` / `N` | Next/previous search match |
| `Esc` | Clear search |
| `r` / `R` | Reload file |
| `Ctrl-C` / `Ctrl-Q` | Quit |

### Edit Mode

| Key | Action |
|-----|--------|
| `Esc` | Save and return to view mode |
| `Ctrl-A` | Move to start of line |
| `Ctrl-E` | Move to end of line |
| `Ctrl-K` | Kill to end of line |
| `Ctrl-D` | Delete character forward |
| `Ctrl-T` | Jump to top of document |
| `Ctrl-B` | Jump to bottom of document |
| `Tab` | Insert 4 spaces |
| `Ctrl-C` / `Ctrl-Q` | Quit |

## Features

- Rendered markdown view with headings, code blocks, lists, links, blockquotes
- Raw markdown editor with syntax highlighting and line numbers
- Case-insensitive search with match highlighting
- Auto-save on exiting edit mode
- External file change detection with conflict resolution
- Read-only file detection
- Word wrap for long lines in view mode
- Full Unicode support

## License

MIT
