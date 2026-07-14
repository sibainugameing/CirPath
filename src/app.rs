use crate::browser::{Browser, BrowserAction};
use crate::config::Config;
use crate::editor::{Editor, EditorAction};
use crate::i18n::{self, Lang};
use crate::menu::{Menu, MenuAction};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Focus {
    Editor,
    Browser,
    Menu,
}

impl Focus {
    fn next(self) -> Self {
        match self {
            Focus::Editor => Focus::Browser,
            Focus::Browser => Focus::Menu,
            Focus::Menu => Focus::Editor,
        }
    }
    fn prev(self) -> Self {
        match self {
            Focus::Editor => Focus::Menu,
            Focus::Browser => Focus::Editor,
            Focus::Menu => Focus::Browser,
        }
    }
    fn label(self, lang: Lang) -> &'static str {
        match self {
            Focus::Editor => i18n::focus_editor(lang),
            Focus::Browser => i18n::focus_browser(lang),
            Focus::Menu => i18n::focus_menu(lang),
        }
    }
}

pub struct App {
    pub focus: Focus,
    pub editor: Editor,
    pub browser: Browser,
    pub menu: Menu,
    pub config: Config,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        let config = Config::load();
        App {
            focus: Focus::Editor,
            editor: Editor::new(),
            browser: Browser::new(config.show_hidden_files),
            menu: Menu::new(),
            config,
            should_quit: false,
        }
    }

    fn lang(&self) -> Lang {
        Lang::from_str(&self.config.language)
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        // グローバルなウィンドウ切替 (Ctrl+Q / Ctrl+E)
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('e') => {
                    self.focus = self.focus.next();
                    return;
                }
                KeyCode::Char('q') => {
                    self.focus = self.focus.prev();
                    return;
                }
                _ => {}
            }
        }

        let lang = self.lang();

        match self.focus {
            Focus::Editor => match self.editor.handle_key(key, lang) {
                EditorAction::Quit => self.should_quit = true,
                EditorAction::ShowHelp => {
                    // キー操作一覧カテゴリ(索引3)を開いてメニューへ切り替える
                    self.menu.selected_category = 3;
                    self.menu.selected_item = 0;
                    self.focus = Focus::Menu;
                }
                EditorAction::None => {}
            },
            Focus::Browser => match self.browser.handle_key(key, lang) {
                BrowserAction::OpenFile(path) => {
                    self.editor.open_file(path, lang);
                    if self.config.auto_open_on_select {
                        self.focus = Focus::Editor;
                    }
                }
                BrowserAction::None => {}
            },
            Focus::Menu => {
                // 隠しファイル設定が変化したらブラウザに反映
                let before_hidden = self.config.show_hidden_files;
                match self.menu.handle_key(key, &mut self.config) {
                    MenuAction::OpenConfigFile => {
                        let lang_after = self.lang();
                        self.editor.open_file(Config::config_path(), lang_after);
                        self.focus = Focus::Editor;
                    }
                    MenuAction::None => {}
                }
                if before_hidden != self.config.show_hidden_files {
                    self.browser.show_hidden = self.config.show_hidden_files;
                    self.browser.refresh();
                }
            }
        }
    }

    pub fn draw(&mut self, f: &mut Frame) {
        let lang = self.lang();
        let area = f.size();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1)])
            .split(area);

        let title = i18n::title_bar(lang, self.focus.label(lang));
        let title_bar = Paragraph::new(Line::from(Span::styled(
            title,
            Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD),
        )));
        f.render_widget(title_bar, chunks[0]);

        match self.focus {
            Focus::Editor => self.editor.draw(f, chunks[1], self.config.show_line_numbers, lang),
            Focus::Browser => self.browser.draw(f, chunks[1], lang),
            Focus::Menu => self.menu.draw(f, chunks[1], &self.config),
        }
    }
}
