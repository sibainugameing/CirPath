use crate::browser::{Browser, BrowserPrompt};
use crate::config::Config;
use crate::editor::{Editor, Prompt};
use crate::menu::{Menu, MenuAction, MenuFocus};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Window {
    Editor,
    Browser,
    Menu,
}

impl Window {
    pub fn next(self) -> Window {
        match self {
            Window::Editor => Window::Browser,
            Window::Browser => Window::Menu,
            Window::Menu => Window::Editor,
        }
    }
    pub fn prev(self) -> Window {
        match self {
            Window::Editor => Window::Menu,
            Window::Browser => Window::Editor,
            Window::Menu => Window::Browser,
        }
    }
}

pub struct App {
    pub config: Config,
    pub editor: Editor,
    pub browser: Browser,
    pub menu: Menu,
    pub active: Window,
    pub should_quit: bool,
    pub global_message: String,
}

impl App {
    pub fn new() -> Self {
        let config = Config::load();
        let start_dir = if config.start_dir.is_empty() {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        } else {
            PathBuf::from(&config.start_dir)
        };
        App {
            editor: Editor::new(config.tab_width, config.show_line_numbers),
            browser: Browser::new(start_dir),
            menu: Menu::new(),
            active: Window::Browser,
            should_quit: false,
            global_message: "Ctrl+Q / Ctrl+E: ウィンドウ切替 | Ctrl+C: 終了".to_string(),
            config,
        }
    }

    pub fn open_path_in_editor(&mut self, path: PathBuf) {
        match self.editor.open_file(&path) {
            Ok(_) => {
                if self.config.auto_jump_to_editor {
                    self.active = Window::Editor;
                }
            }
            Err(e) => {
                self.global_message = format!("開けませんでした: {}", e);
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        // グローバルショートカット (どのウィンドウにいても有効)
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    self.active = self.active.next();
                    return;
                }
                KeyCode::Char('e') | KeyCode::Char('E') => {
                    self.active = self.active.prev();
                    return;
                }
                KeyCode::Char('c') | KeyCode::Char('C') => {
                    self.should_quit = true;
                    return;
                }
                _ => {}
            }
        }

        match self.active {
            Window::Editor => self.handle_editor_key(key),
            Window::Browser => self.handle_browser_key(key),
            Window::Menu => self.handle_menu_key(key),
        }
    }

    fn handle_editor_key(&mut self, key: KeyEvent) {
        // プロンプト表示中 (保存名の入力 / 検索文字列の入力)
        match self.editor.prompt.clone() {
            Prompt::SaveAs(mut buf) => {
                match key.code {
                    KeyCode::Enter => {
                        let path = PathBuf::from(buf.clone());
                        self.editor.prompt = Prompt::None;
                        if !buf.trim().is_empty() {
                            let _ = self.editor.save_to(path);
                        }
                    }
                    KeyCode::Esc => {
                        self.editor.prompt = Prompt::None;
                        self.editor.status_message = "保存をキャンセルしました".to_string();
                    }
                    KeyCode::Backspace => {
                        buf.pop();
                        self.editor.prompt = Prompt::SaveAs(buf);
                    }
                    KeyCode::Char(c) => {
                        buf.push(c);
                        self.editor.prompt = Prompt::SaveAs(buf);
                    }
                    _ => {}
                }
                return;
            }
            Prompt::Search(mut buf) => {
                match key.code {
                    KeyCode::Enter => {
                        self.editor.prompt = Prompt::None;
                        self.editor.search_next(&buf.clone());
                    }
                    KeyCode::Esc => {
                        self.editor.prompt = Prompt::None;
                    }
                    KeyCode::Backspace => {
                        buf.pop();
                        self.editor.prompt = Prompt::Search(buf);
                    }
                    KeyCode::Char(c) => {
                        buf.push(c);
                        self.editor.prompt = Prompt::Search(buf);
                    }
                    _ => {}
                }
                return;
            }
            Prompt::None => {}
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('s') | KeyCode::Char('S') => {
                    let _ = self.editor.save();
                }
                KeyCode::Char('o') | KeyCode::Char('O') => {
                    self.active = Window::Browser;
                }
                KeyCode::Char('w') | KeyCode::Char('W') => {
                    self.editor.prompt = Prompt::Search(String::new());
                }
                KeyCode::Char('k') | KeyCode::Char('K') => {
                    self.editor.cut_line();
                }
                KeyCode::Char('u') | KeyCode::Char('U') => {
                    self.editor.uncut();
                }
                _ => {}
            }
            return;
        }

        match key.code {
            KeyCode::Char(c) => self.editor.insert_char(c),
            KeyCode::Enter => self.editor.insert_newline(),
            KeyCode::Backspace => self.editor.backspace(),
            KeyCode::Delete => self.editor.delete_forward(),
            KeyCode::Tab => self.editor.insert_tab(),
            KeyCode::Left => self.editor.move_left(),
            KeyCode::Right => self.editor.move_right(),
            KeyCode::Up => self.editor.move_up(),
            KeyCode::Down => self.editor.move_down(),
            KeyCode::Home => self.editor.move_home(),
            KeyCode::End => self.editor.move_end(),
            KeyCode::PageUp => self.editor.page_up(10),
            KeyCode::PageDown => self.editor.page_down(10),
            _ => {}
        }
    }

    fn handle_browser_key(&mut self, key: KeyEvent) {
        if let BrowserPrompt::GotoPath(mut buf) = self.browser.prompt.clone() {
            match key.code {
                KeyCode::Enter => {
                    self.browser.prompt = BrowserPrompt::None;
                    self.browser.goto_path(&buf.clone());
                }
                KeyCode::Esc => {
                    self.browser.prompt = BrowserPrompt::None;
                }
                KeyCode::Backspace => {
                    buf.pop();
                    self.browser.prompt = BrowserPrompt::GotoPath(buf);
                }
                KeyCode::Char(c) => {
                    buf.push(c);
                    self.browser.prompt = BrowserPrompt::GotoPath(buf);
                }
                _ => {}
            }
            return;
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => self.browser.move_up(),
            KeyCode::Down | KeyCode::Char('j') => self.browser.move_down(),
            KeyCode::Char('g') => {
                self.browser.prompt = BrowserPrompt::GotoPath(String::new());
            }
            KeyCode::Char('h') | KeyCode::Backspace | KeyCode::Left => self.browser.go_parent(),
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                if let Some(file_path) = self.browser.enter_selected() {
                    self.open_path_in_editor(file_path);
                }
            }
            KeyCode::Char('r') => self.browser.refresh(),
            KeyCode::Home => self.browser.go_top(),
            KeyCode::End => self.browser.go_bottom(),
            _ => {}
        }
    }

    fn handle_menu_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => self.menu.move_up(),
            KeyCode::Down => self.menu.move_down(),
            KeyCode::Right | KeyCode::Enter => {
                let action = self.menu.activate();
                self.apply_menu_action(action);
            }
            KeyCode::Left => self.menu.focus_left(),
            _ => {
                if self.menu.focus == MenuFocus::Category {
                    // 何もしない
                }
            }
        }
    }

    fn apply_menu_action(&mut self, action: MenuAction) {
        match action {
            MenuAction::None => {}
            MenuAction::ToggleAutoJump => {
                self.config.auto_jump_to_editor = !self.config.auto_jump_to_editor;
                let _ = self.config.save();
                self.global_message = format!(
                    "自動移動: {}",
                    if self.config.auto_jump_to_editor { "ON" } else { "OFF" }
                );
            }
            MenuAction::ToggleLineNumbers => {
                self.config.show_line_numbers = !self.config.show_line_numbers;
                self.editor.show_line_numbers = self.config.show_line_numbers;
                let _ = self.config.save();
            }
            MenuAction::IncreaseTabWidth => {
                self.config.tab_width += 1;
                self.editor.tab_width = self.config.tab_width;
                let _ = self.config.save();
            }
            MenuAction::DecreaseTabWidth => {
                if self.config.tab_width > 1 {
                    self.config.tab_width -= 1;
                    self.editor.tab_width = self.config.tab_width;
                    let _ = self.config.save();
                }
            }
            MenuAction::OpenConfigFile => {
                let path = Config::ensure_and_path();
                self.open_path_in_editor(path);
            }
            MenuAction::SetThemeDark => {
                self.config.theme = "dark".to_string();
                let _ = self.config.save();
            }
            MenuAction::SetThemeLight => {
                self.config.theme = "light".to_string();
                let _ = self.config.save();
            }
            MenuAction::Quit => {
                self.should_quit = true;
            }
        }
    }
}
