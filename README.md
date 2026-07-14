# CirPath

CirPath is a simple, functionally elegant TUI text editor written in Rust and inspired by nano.

It consists of three main windows—**Editor**, **File Browser**, and **Menu**—which can be switched using `Ctrl+E` and `Ctrl+Q`.

Instead of using borders, the interface follows nano's minimal design, consisting only of a title bar and an inverted-color status bar.

The interface avoids environment-dependent characters such as emojis and platform-specific symbols. It uses only ASCII characters and Japanese characters within the standard JIS character set.

The display language can be switched between **Japanese** and **English** from **General** in the menu. Language changes are applied immediately across the entire interface.

When adding new features, logic can be added to the corresponding screen module (`editor.rs`, `browser.rs`, or `menu.rs`). Configuration options can be added to `Config`, with the corresponding choices added to `items_for` and `apply_selection` in `menu.rs`.

The architecture is designed so that features can be extended without affecting unrelated modules.

## Build and Run

```bash
cargo build --release
./target/release/cirpath
```

## Controls

### Global Controls

| Key      | Action                                                                    |
| -------- | ------------------------------------------------------------------------- |
| `Ctrl+E` | Switch to the next window (cycles through Editor, File Browser, and Menu) |
| `Ctrl+Q` | Switch to the previous window                                             |
| `Ctrl+G` | Open Help (directly opens the menu's keyboard shortcut list)              |

### Editor

The editor provides a comprehensive set of nano-style features.

| Key                                                 | Action                                                                                         |
| --------------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| Character input                                     | Insert text at the cursor position (IME input, including Japanese, is handled by the OS)       |
| `Enter`                                             | Insert a new line                                                                              |
| `Backspace` / `Delete`                              | Delete the previous or next character, including across line boundaries                        |
| Arrow keys / `Home` / `End` / `PageUp` / `PageDown` | Move the cursor                                                                                |
| `Ctrl+S`                                            | Save                                                                                           |
| `Ctrl+O`                                            | Save As (Write Out; prompts for a file name)                                                   |
| `Ctrl+R`                                            | Insert the contents of a specified file at the cursor position (Read File)                     |
| `Ctrl+W`                                            | Search (Where Is). Press `Enter` with an empty query to repeat the previous search             |
| `Ctrl+\`                                            | Replace. Enter the search string followed by the replacement string to replace all occurrences |
| `Ctrl+_`                                            | Jump to a specified line number (Go To Line)                                                   |
| `Ctrl+K`                                            | Cut the current line. Repeated cuts are accumulated into a single buffer                       |
| `Ctrl+U`                                            | Paste the cut text at the cursor position (Uncut)                                              |
| `Ctrl+C`                                            | Display the current cursor position (line/column) and total number of lines                    |
| `Ctrl+X`                                            | Exit. If there are unsaved changes, press again to confirm                                     |
| `Esc`                                               | Cancel input prompts such as search, replace, and file name entry                              |

### File Browser

| Key                      | Action                                                                                                                    |
| ------------------------ | ------------------------------------------------------------------------------------------------------------------------- |
| Up / Down                | Select an item                                                                                                            |
| `Enter` / Right          | Enter the selected directory or open the selected file                                                                    |
| `Backspace` / Left / `u` | Move to the parent directory                                                                                              |
| `g`                      | Enter an absolute or relative path directly and navigate to it, including locations outside the current working directory |
| `n`                      | Create a new file                                                                                                         |
| `N` (`Shift+n`)          | Create a new directory                                                                                                    |
| `r`                      | Rename the selected item                                                                                                  |
| `d`                      | Delete the selected item with a `y/n` confirmation to prevent accidental deletion                                         |
| `Ctrl+H`                 | Toggle hidden files (dotfiles)                                                                                            |
| `Esc`                    | Cancel path entry, creation, or rename prompts                                                                            |

When a file is selected, CirPath automatically switches to the Editor if **Automatically switch to the editor when a file is selected** is enabled in the **File Browser** settings. This option is enabled by default.

### Menu

The menu uses a Word-style two-panel layout.

| Key          | Action                                                                                                  |
| ------------ | ------------------------------------------------------------------------------------------------------- |
| Left / Right | Switch between the left category panel and the right detail panel                                       |
| Up / Down    | Move through items in the selected panel                                                                |
| `Enter`      | From the left panel, move to the right panel; from the right panel, select or toggle the current option |

From the **Configuration File** category in the menu, you can open the TOML configuration file itself in the editor and edit it directly.

Changes are saved to:

```text
~/.config/cirpath/config.toml
```

## Error Messages

Errors such as navigating to a nonexistent path, failing to save a file, or failing to find a search term are displayed in the status bar with a red background.

The message is automatically cleared when the next key is pressed, preventing outdated error messages from remaining on the screen.
