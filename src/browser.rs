use crate::i18n::{self, Lang};
use crate::status::StatusMsg;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState, Paragraph},
    Frame,
};
use std::fs;
use std::path::PathBuf;

pub enum BrowserAction {
    None,
    /// ファイルを開く(自動でエディタへ切り替えるかは呼び出し側のconfig次第)
    OpenFile(PathBuf),
}

/// ブラウザが待ち受けている入力モード。
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Normal,
    /// 'g' : 絶対/相対パスを直接入力して移動
    GotoPath,
    /// 'n' : 新規ファイル作成
    NewFile,
    /// 'N' : 新規フォルダ作成
    NewFolder,
    /// 'r' : 選択中の項目をリネーム
    Rename,
    /// 'd' : 選択中の項目を削除 (確認プロンプト、y/nのみ受け付け)
    ConfirmDelete,
}

/// VSCodeのサイドバーやFinderを意識したキーボード操作オンリーのファイルブラウザ。
/// 実行ディレクトリの外にも絶対・相対パスで自由に移動できる。
pub struct Browser {
    pub current_dir: PathBuf,
    pub entries: Vec<PathBuf>,
    pub selected: usize,
    pub show_hidden: bool,
    mode: Mode,
    input_buffer: String,
    pub message: Option<StatusMsg>,
    /// 表示スクロール位置(先頭に表示しているエントリのインデックス)
    scroll: usize,
}

impl Browser {
    pub fn new(show_hidden: bool) -> Self {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut b = Browser {
            current_dir,
            entries: Vec::new(),
            selected: 0,
            show_hidden,
            mode: Mode::Normal,
            input_buffer: String::new(),
            message: None,
            scroll: 0,
        };
        b.refresh();
        b
    }

    pub fn refresh(&mut self) {
        let mut entries: Vec<PathBuf> = Vec::new();
        if let Ok(read) = fs::read_dir(&self.current_dir) {
            for entry in read.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                if !self.show_hidden && name.starts_with('.') {
                    continue;
                }
                entries.push(path);
            }
        }
        entries.sort_by(|a, b| {
            let a_dir = a.is_dir();
            let b_dir = b.is_dir();
            match (a_dir, b_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_lowercase()
                    .cmp(&b.file_name().unwrap_or_default().to_string_lossy().to_lowercase()),
            }
        });
        self.entries = entries;
        if self.selected >= self.entries.len() {
            self.selected = self.entries.len().saturating_sub(1);
        }
    }

    fn go_to(&mut self, path: PathBuf) {
        if let Ok(canonical) = path.canonicalize() {
            self.current_dir = canonical;
        } else {
            self.current_dir = path;
        }
        self.selected = 0;
        self.scroll = 0;
        self.refresh();
    }

    fn resolve(&self, raw: &str) -> PathBuf {
        let candidate = PathBuf::from(shellexpand_home(raw));
        if candidate.is_absolute() {
            candidate
        } else {
            self.current_dir.join(candidate)
        }
    }

    fn handle_mode_key(&mut self, key: KeyEvent, lang: Lang) -> BrowserAction {
        if self.mode == Mode::ConfirmDelete {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    self.mode = Mode::Normal;
                    if let Some(target) = self.entries.get(self.selected).cloned() {
                        let result = if target.is_dir() {
                            fs::remove_dir_all(&target)
                        } else {
                            fs::remove_file(&target)
                        };
                        match result {
                            Ok(_) => {
                                self.message = Some(StatusMsg::info(i18n::br_deleted(lang)));
                                self.refresh();
                            }
                            Err(e) => {
                                self.message =
                                    Some(StatusMsg::error(i18n::br_delete_failed(lang, &e.to_string())));
                            }
                        }
                    }
                }
                _ => {
                    self.mode = Mode::Normal;
                    self.message = Some(StatusMsg::info(i18n::br_delete_cancelled(lang)));
                }
            }
            return BrowserAction::None;
        }

        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.input_buffer.clear();
                self.message = Some(StatusMsg::info(i18n::br_cancelled(lang)));
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
            }
            KeyCode::Enter => {
                let raw = self.input_buffer.trim().to_string();
                let mode = self.mode;
                self.mode = Mode::Normal;
                self.input_buffer.clear();

                if raw.is_empty() {
                    self.message = Some(StatusMsg::error(i18n::br_input_empty(lang)));
                    return BrowserAction::None;
                }

                match mode {
                    Mode::GotoPath => {
                        let target = self.resolve(&raw);
                        if target.is_dir() {
                            self.go_to(target);
                            self.message = Some(StatusMsg::info(i18n::br_moved(lang)));
                        } else if target.is_file() {
                            return BrowserAction::OpenFile(target);
                        } else {
                            self.message = Some(StatusMsg::error(i18n::br_path_not_found(
                                lang,
                                &target.display().to_string(),
                            )));
                        }
                    }
                    Mode::NewFile => {
                        let target = self.current_dir.join(&raw);
                        if target.exists() {
                            self.message = Some(StatusMsg::error(i18n::br_file_exists(lang)));
                        } else {
                            match fs::write(&target, "") {
                                Ok(_) => {
                                    self.message = Some(StatusMsg::info(i18n::br_created_file(lang, &raw)));
                                    self.refresh();
                                }
                                Err(e) => {
                                    self.message =
                                        Some(StatusMsg::error(i18n::br_create_failed(lang, &e.to_string())));
                                }
                            }
                        }
                    }
                    Mode::NewFolder => {
                        let target = self.current_dir.join(&raw);
                        if target.exists() {
                            self.message = Some(StatusMsg::error(i18n::br_folder_exists(lang)));
                        } else {
                            match fs::create_dir_all(&target) {
                                Ok(_) => {
                                    self.message = Some(StatusMsg::info(i18n::br_created_folder(lang, &raw)));
                                    self.refresh();
                                }
                                Err(e) => {
                                    self.message =
                                        Some(StatusMsg::error(i18n::br_create_failed(lang, &e.to_string())));
                                }
                            }
                        }
                    }
                    Mode::Rename => {
                        if let Some(target) = self.entries.get(self.selected).cloned() {
                            let new_path = self.current_dir.join(&raw);
                            match fs::rename(&target, &new_path) {
                                Ok(_) => {
                                    self.message = Some(StatusMsg::info(i18n::br_renamed(lang)));
                                    self.refresh();
                                }
                                Err(e) => {
                                    self.message =
                                        Some(StatusMsg::error(i18n::br_rename_failed(lang, &e.to_string())));
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        BrowserAction::None
    }

    pub fn handle_key(&mut self, key: KeyEvent, lang: Lang) -> BrowserAction {
        if self.mode != Mode::Normal {
            return self.handle_mode_key(key, lang);
        }

        // 通常のキー入力ではメッセージは都度クリアする(古いエラーが残り続けるのを防ぐ)
        self.message = None;

        match key.code {
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            KeyCode::Down => {
                if self.selected + 1 < self.entries.len() {
                    self.selected += 1;
                }
            }
            KeyCode::Char('g') => {
                self.mode = Mode::GotoPath;
                self.input_buffer.clear();
            }
            KeyCode::Char('n') => {
                self.mode = Mode::NewFile;
                self.input_buffer.clear();
            }
            KeyCode::Char('N') => {
                self.mode = Mode::NewFolder;
                self.input_buffer.clear();
            }
            KeyCode::Char('r') => {
                if let Some(target) = self.entries.get(self.selected) {
                    self.input_buffer = target
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    self.mode = Mode::Rename;
                } else {
                    self.message = Some(StatusMsg::error(i18n::br_no_selection(lang)));
                }
            }
            KeyCode::Char('d') => {
                if self.entries.get(self.selected).is_some() {
                    self.mode = Mode::ConfirmDelete;
                } else {
                    self.message = Some(StatusMsg::error(i18n::br_no_selection(lang)));
                }
            }
            KeyCode::Char('h') | KeyCode::Char('H')
                if key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                self.show_hidden = !self.show_hidden;
                self.refresh();
            }
            KeyCode::Backspace | KeyCode::Left | KeyCode::Char('u') => {
                if let Some(parent) = self.current_dir.parent() {
                    let parent = parent.to_path_buf();
                    self.go_to(parent);
                }
            }
            KeyCode::Enter | KeyCode::Right => {
                if let Some(path) = self.entries.get(self.selected).cloned() {
                    if path.is_dir() {
                        self.go_to(path);
                    } else {
                        return BrowserAction::OpenFile(path);
                    }
                }
            }
            _ => {}
        }
        BrowserAction::None
    }

    fn adjust_scroll(&mut self, visible_rows: usize) {
        if visible_rows == 0 {
            return;
        }
        if self.selected < self.scroll {
            self.scroll = self.selected;
        } else if self.selected >= self.scroll + visible_rows {
            self.scroll = self.selected + 1 - visible_rows;
        }
        // エントリ数が減った場合などにスクロール位置が末尾を超えないよう補正
        if self.entries.len() <= visible_rows {
            self.scroll = 0;
        } else if self.scroll > self.entries.len() - visible_rows {
            self.scroll = self.entries.len() - visible_rows;
        }
    }

    pub fn draw(&mut self, f: &mut Frame, area: Rect, lang: Lang) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(area);

        let visible_rows = chunks[0].height as usize;
        self.adjust_scroll(visible_rows);

        let items: Vec<ListItem> = self
            .entries
            .iter()
            .skip(self.scroll)
            .take(visible_rows)
            .map(|p| {
                let name = p
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| p.display().to_string());
                if p.is_dir() {
                    ListItem::new(format!("[DIR]  {}", name))
                        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                } else {
                    ListItem::new(format!("[FILE] {}", name))
                }
            })
            .collect();

        let mut state = ListState::default();
        if !self.entries.is_empty() {
            state.select(Some(self.selected - self.scroll));
        }

        let list = List::new(items).highlight_style(
            Style::default()
                .bg(Color::White)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );
        f.render_stateful_widget(list, chunks[0], &mut state);

        let bottom = if let Some(label) = self.prompt_label(lang) {
            Paragraph::new(Line::from(Span::styled(
                format!(" {}: {}", label, self.input_buffer),
                Style::default().fg(Color::Black).bg(Color::White),
            )))
        } else if self.mode == Mode::ConfirmDelete {
            Paragraph::new(Line::from(Span::styled(
                i18n::br_confirm_delete(lang),
                Style::default().fg(Color::White).bg(Color::Red),
            )))
        } else {
            let msg = self
                .message
                .clone()
                .unwrap_or_else(|| StatusMsg::info(i18n::br_hint(lang)));
            let path_line = format!(" {}  |  {}", self.current_dir.display(), msg.text);
            Paragraph::new(Line::from(Span::styled(path_line, msg.style())))
        };
        f.render_widget(bottom, chunks[1]);
    }

    fn prompt_label(&self, lang: Lang) -> Option<&'static str> {
        match self.mode {
            Mode::Normal | Mode::ConfirmDelete => None,
            Mode::GotoPath => Some(i18n::br_prompt_goto(lang)),
            Mode::NewFile => Some(i18n::br_prompt_new_file(lang)),
            Mode::NewFolder => Some(i18n::br_prompt_new_folder(lang)),
            Mode::Rename => Some(i18n::br_prompt_rename(lang)),
        }
    }
}

fn shellexpand_home(s: &str) -> String {
    if let Some(rest) = s.strip_prefix('~') {
        if let Some(home) = dirs::home_dir() {
            return format!("{}{}", home.display(), rest);
        }
    }
    s.to_string()
}
