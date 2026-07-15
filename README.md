# CirPath

CirPath is a Rust TUI (terminal user interface) text editor inspired by `nano` / `pico`.
It combines three windows in a single app - **Editor**, **File Browser**, and **Menu** -
that you switch between with `Ctrl+E` / `Ctrl+Q`. There are no boxed borders; like `nano`,
the interface is built from a plain title bar and reverse-video status bars only.

The UI avoids platform/environment-dependent characters (no emoji, no special "gaiji"
symbols). All on-screen text is either plain ASCII or standard Japanese (JIS-range)
characters, and the app fully supports both **English** and **Japanese** display
languages, switchable at runtime from the Menu screen.


To add a new feature, extend the relevant screen module (`editor.rs` / `browser.rs` /
`menu.rs`), add any new setting to `Config`, and wire it into `menu.rs`'s `items_for` /
`apply_selection`. Each module is self-contained, so new features generally don't require
touching the other screens.

## Building & Running

```bash
cargo build --release
./target/release/cirpath
```

### Toolchain note

This project targets a minimum Rust toolchain of **rustc 1.75 / cargo 1.75** and pins
exact dependency versions in `Cargo.toml` (`ratatui = "=0.26.3"`, `crossterm = "=0.27.0"`,
`toml = "=0.5.11"`, `unicode-width = "=0.1.13"`) to stay compatible with that toolchain.
If you use a newer Rust toolchain, these pins are safe to relax with `cargo update`.

## Controls

### Global (available on every screen)

| Key | Action |
|---|---|
| `Ctrl+E` | Switch to the next window (cycles Editor, Browser, Menu, Editor, ...) |
| `Ctrl+Q` | Switch to the previous window |
| `Ctrl+G` | Help - jumps straight to the "Key Bindings" category in the Menu |

### Editor screen (nano feature set)

The bottom of the editor always shows a 2-row by 6-column shortcut grid, in the style of
classic `nano` / `UW PICO`, so the available commands are visible at a glance.

| Key | Action |
|---|---|
| Printable characters | Insert at the cursor (IME composition for Japanese and other input methods is handled entirely by the OS) |
| `Enter` | Insert a newline |
| `Backspace` / `Delete` | Delete the character before/at the cursor (merges across line boundaries) |
| Arrow keys / `Home` / `End` / `Page Up` / `Page Down` | Move the cursor |
| `Ctrl+S` | Save |
| `Ctrl+O` | Save As - prompts for a filename (Write Out) |
| `Ctrl+R` | Insert the contents of another file at the cursor (Read File) |
| `Ctrl+W` | Search (Where Is). Press Enter on an empty prompt to repeat the last search |
| `Ctrl+\` | Replace - prompts for a search string, then a replacement, and replaces every match |
| `Ctrl+_` | Go to a specific line number |
| `Ctrl+K` | Cut the current line (repeated presses accumulate into one clipboard buffer) |
| `Ctrl+U` | Paste (UnCut) the last cut text at the cursor |
| `Ctrl+C` | Show the current cursor position (line/column) and total line count |
| `Ctrl+X` | Quit - press again to confirm if there are unsaved changes |
| `Esc` | Cancel the current prompt (search / replace / save-as / go-to-line / insert-file) |

### File Browser screen

| Key | Action |
|---|---|
| Up / Down | Move the selection |
| `Enter` / Right | Enter the selected directory, or open the selected file |
| `Backspace` / Left / `u` | Go up to the parent directory |
| `g` | Type an absolute or relative path to jump directly to it (works outside the launch directory) |
| `n` | Create a new file in the current directory |
| `N` (Shift+n) | Create a new folder in the current directory |
| `r` | Rename the selected file or folder |
| `d` | Delete the selected file or folder (asks for `y`/`n` confirmation first) |
| `Ctrl+H` | Toggle visibility of hidden (dot) files |
| `Esc` | Cancel the current prompt (path entry / new file / new folder / rename) |

The visible list scrolls automatically so the selection is always kept on screen, even
with directories containing far more entries than fit on one page.

If **"Auto-switch to editor when a file is selected"** is enabled in the Menu (this is
the default), opening a file from the browser immediately switches focus to the Editor.

### Menu screen (Word-style two-pane settings)

| Key | Action |
|---|---|
| Left / Right | Switch focus between the category panel (left) and the detail panel (right) |
| Up / Down | Move the selection within the focused panel |
| `Enter` | From the left panel, moves into the right panel; from the right panel, applies/toggles the selected setting |

Categories: **General** (includes the language switch), **Editor**, **File Browser**,
**Key Bindings** (a full reference list), **Config File** (opens `config.toml` directly
in the Editor for manual editing), and **About**.

All settings changes take effect immediately and are persisted to
`~/.config/cirpath/config.toml`.

## Language support

CirPath ships with complete English and Japanese translations for every visible string:
window titles, status messages, error messages, prompts, the Menu's categories/items, the
key-binding reference, and the Editor's PICO-style shortcut grid. Switch languages from
**Menu -> General -> "Switch display language"**; the change applies instantly across all
three windows and is saved to the config file for the next launch.

## Error display

Errors - an invalid path, a failed save, a search with no match, and so on - are shown
in the status bar with a **red background** so they're unmistakable. Any subsequent
keypress automatically clears the message, so stale errors never linger on screen.

## Color palette

The UI intentionally avoids blue/cyan tones (which are hard to read against a black
terminal background in many color schemes). The palette is limited to:

- White background + black text - normal status bars, prompts, and the title bar
- Red background + white text - error messages and the delete-confirmation prompt
- Yellow (bold) - directory entries in the File Browser
- Dark gray - line numbers and unfocused Menu panels

## Known limitations

The following `nano`/full-editor features are **not** implemented yet:

- Multiple buffers / tabs
- Syntax highlighting
- Undo / redo
- Text selection (marking) for copy - only whole-line cut/paste (`Ctrl+K` / `Ctrl+U`) is supported

These are straightforward to add on top of the existing `editor.rs` structure if needed.
