# CirPath

**CirPath** is a simple, functional TUI text editor written in Rust, inspired by **GNU nano**.

It consists of three main views:

* **Editor** — Edit and save text files
* **File Browser** — Navigate directories and open files
* **Menu** — Configure CirPath

Switch between views using `Ctrl+E` and `Ctrl+Q`.

CirPath avoids traditional window borders and instead uses inverted title bars and status bars, creating a minimal interface inspired by nano.

---

## Features

* Fast and lightweight TUI written in Rust
* nano-inspired text editing
* Built-in file browser
* Built-in settings menu
* Direct configuration file editing
* UTF-8 text support
* Hidden file toggle
* Configurable editor and browser behavior
* Simple modular architecture

---


| File         | Description                                               |
| ------------ | --------------------------------------------------------- |
| `main.rs`    | Entry point, terminal initialization, and main event loop |
| `app.rs`     | Application state, view management, and rendering         |
| `editor.rs`  | nano-style text buffer editing, loading, and saving       |
| `browser.rs` | File browser, directory navigation, and path input        |
| `menu.rs`    | Settings UI with category and detail panels               |
| `config.rs`  | Configuration loading and saving                          |

The configuration file is stored at:

```text
~/.config/cirpath/config.toml
```

CirPath is designed to be modular. New functionality can generally be added to the corresponding module:

* `editor.rs` — Editor features
* `browser.rs` — File browser features
* `menu.rs` — Menu and settings UI

New configuration options can be added to `Config` and exposed through `items_for` and `apply_selection` in `menu.rs`.

---

## Build

### Requirements

* Rust
* Cargo

Build the release version:

```bash
cargo build --release
```

Run CirPath:

```bash
./target/release/cirpath
```

Alternatively:

```bash
cargo run --release
```

---

## Controls

### Global

| Key      | Action                                           |
| -------- | ------------------------------------------------ |
| `Ctrl+E` | Switch to the next view: Editor → Browser → Menu |
| `Ctrl+Q` | Switch to the previous view                      |

### Editor

| Key                   | Action                                   |
| --------------------- | ---------------------------------------- |
| Character input       | Insert text at the cursor                |
| `Enter`               | Insert a new line                        |
| `Backspace`           | Delete the previous character            |
| `Delete`              | Delete the next character                |
| `↑` `↓` `←` `→`       | Move the cursor                          |
| `Home` / `End`        | Move to the beginning or end of the line |
| `PageUp` / `PageDown` | Move through the document                |
| `Ctrl+S`              | Save the current file                    |
| `Ctrl+X`              | Exit CirPath                             |
| `Ctrl+K`              | Delete the current line                  |

If there are unsaved changes, press `Ctrl+X` again to confirm exit.

IME input, including Japanese input, is handled by the operating system and terminal environment.

### File Browser

| Key                     | Action                             |
| ----------------------- | ---------------------------------- |
| `↑` / `↓`               | Select an item                     |
| `Enter` / `→`           | Enter a directory or open a file   |
| `Backspace` / `←` / `u` | Move to the parent directory       |
| `g`                     | Enter an absolute or relative path |
| `Ctrl+H`                | Toggle hidden files                |

The file browser can navigate outside the directory where CirPath was launched.

By default, opening a file automatically switches to the editor. This behavior can be changed from the **File Browser** settings.

### Menu

| Key       | Action                                                 |
| --------- | ------------------------------------------------------ |
| `←` / `→` | Switch between the category and detail panels          |
| `↑` / `↓` | Move through items                                     |
| `Enter`   | Open a category, select an option, or toggle a setting |

The configuration file itself can also be opened directly from the **Configuration File** category and edited using CirPath.

Changes are stored in:

```text
~/.config/cirpath/config.toml
```

---

## Dependencies

CirPath is built with:

* `ratatui` — Terminal user interface
* `crossterm` — Terminal input and control
* `serde` — Configuration serialization
* `toml` — TOML configuration support
* `dirs` — Platform-specific configuration paths

---

## License

See the `LICENSE` file for details.
