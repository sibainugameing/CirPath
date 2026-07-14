use crate::config::Config;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState, Paragraph},
    Frame,
};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Panel {
    Left,
    Right,
}

pub enum MenuAction {
    None,
    /// 設定ファイル自体をエディタで開いてそちらへフォーカスを移す
    OpenConfigFile,
}

/// Microsoft Word のメニューのように、左でカテゴリ、右で詳細項目を選ぶ設定画面。
pub struct Menu {
    pub categories: Vec<&'static str>,
    pub selected_category: usize,
    pub selected_item: usize,
    pub panel: Panel,
    pub message: Option<String>,
}

impl Menu {
    pub fn new() -> Self {
        Menu {
            categories: vec!["一般", "エディタ", "ファイルブラウザ", "キー操作一覧", "設定ファイル", "このアプリについて"],
            selected_category: 0,
            selected_item: 0,
            panel: Panel::Left,
            message: None,
        }
    }

    fn items_for(&self, category: usize) -> Vec<String> {
        match category {
            0 => vec!["設定はこのメニューから即座に反映されます".to_string()],
            1 => vec![
                "行番号を表示する".to_string(),
                "タブ幅を変更する (現在の幅から+1、9で1に戻る)".to_string(),
                "保存時に末尾改行を保証する".to_string(),
            ],
            2 => vec![
                "ファイル選択時に自動でエディタへ切り替える".to_string(),
                "隠しファイル(ドットファイル)を表示する".to_string(),
            ],
            3 => vec![
                "Ctrl+E : 次のウィンドウへ切り替え".to_string(),
                "Ctrl+Q : 前のウィンドウへ切り替え".to_string(),
                "Ctrl+G : ヘルプ(このキー操作一覧を表示)".to_string(),
                "[エディタ] Ctrl+S : 保存  Ctrl+O : 別名で保存".to_string(),
                "[エディタ] Ctrl+X : 終了 (未保存時は2回押し)".to_string(),
                "[エディタ] Ctrl+W : 検索  Ctrl+\\ : 置換".to_string(),
                "[エディタ] Ctrl+K : 行を切り取り  Ctrl+U : 貼り付け".to_string(),
                "[エディタ] Ctrl+_ : 指定行へ移動".to_string(),
                "[エディタ] Ctrl+R : 指定ファイルをカーソル位置へ挿入".to_string(),
                "[エディタ] Ctrl+C : カーソル位置(行/列)を表示".to_string(),
                "[ブラウザ] up/down : 選択移動".to_string(),
                "[ブラウザ] enter/right : 開く   backspace/left/u : 上の階層へ".to_string(),
                "[ブラウザ] g : 絶対/相対パスを直接入力して移動".to_string(),
                "[ブラウザ] n : 新規ファイル作成   N : 新規フォルダ作成".to_string(),
                "[ブラウザ] r : 名前変更   d : 削除(確認あり)".to_string(),
                "[ブラウザ] Ctrl+H : 隠しファイル表示切替".to_string(),
                "[メニュー] left/right : 左右パネル切替   up/down : 項目選択   Enter : 決定/切替".to_string(),
            ],
            4 => vec!["設定ファイルをエディタで開いて直接編集する".to_string()],
            5 => vec![
                "CirPath - nano風 TUI テキストエディタ".to_string(),
                "エディタ / ファイルブラウザ / メニューの3画面構成".to_string(),
                "Rust + ratatui + crossterm で実装".to_string(),
            ],
            _ => vec![],
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent, config: &mut Config) -> MenuAction {
        match key.code {
            KeyCode::Left => self.panel = Panel::Left,
            KeyCode::Right => self.panel = Panel::Right,
            KeyCode::Up => match self.panel {
                Panel::Left => {
                    if self.selected_category > 0 {
                        self.selected_category -= 1;
                        self.selected_item = 0;
                    }
                }
                Panel::Right => {
                    if self.selected_item > 0 {
                        self.selected_item -= 1;
                    }
                }
            },
            KeyCode::Down => match self.panel {
                Panel::Left => {
                    if self.selected_category + 1 < self.categories.len() {
                        self.selected_category += 1;
                        self.selected_item = 0;
                    }
                }
                Panel::Right => {
                    let max = self.items_for(self.selected_category).len();
                    if max > 0 && self.selected_item + 1 < max {
                        self.selected_item += 1;
                    }
                }
            },
            KeyCode::Enter => {
                if self.panel == Panel::Left {
                    self.panel = Panel::Right;
                    return MenuAction::None;
                }
                return self.apply_selection(config);
            }
            _ => {}
        }
        MenuAction::None
    }

    fn apply_selection(&mut self, config: &mut Config) -> MenuAction {
        match self.selected_category {
            1 => match self.selected_item {
                0 => {
                    config.show_line_numbers = !config.show_line_numbers;
                    self.message = Some(format!("行番号表示: {}", on_off(config.show_line_numbers)));
                }
                1 => {
                    config.tab_width = if config.tab_width >= 8 { 1 } else { config.tab_width + 1 };
                    self.message = Some(format!("タブ幅: {}", config.tab_width));
                }
                2 => {
                    config.ensure_trailing_newline = !config.ensure_trailing_newline;
                    self.message = Some(format!(
                        "末尾改行を保証: {}",
                        on_off(config.ensure_trailing_newline)
                    ));
                }
                _ => {}
            },
            2 => match self.selected_item {
                0 => {
                    config.auto_open_on_select = !config.auto_open_on_select;
                    self.message = Some(format!(
                        "自動でエディタへ切替: {}",
                        on_off(config.auto_open_on_select)
                    ));
                }
                1 => {
                    config.show_hidden_files = !config.show_hidden_files;
                    self.message = Some(format!("隠しファイル表示: {}", on_off(config.show_hidden_files)));
                }
                _ => {}
            },
            4 => {
                config.save();
                return MenuAction::OpenConfigFile;
            }
            _ => {}
        }
        config.save();
        MenuAction::None
    }

    pub fn draw(&mut self, f: &mut Frame, area: Rect, config: &Config) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(area);

        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(chunks[0]);

        let left_items: Vec<ListItem> = self
            .categories
            .iter()
            .map(|c| ListItem::new(*c))
            .collect();
        let mut left_state = ListState::default();
        left_state.select(Some(self.selected_category));
        let left_style = if self.panel == Panel::Left {
            Style::default().bg(Color::White).fg(Color::Black).add_modifier(Modifier::BOLD)
        } else {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        };
        let left_list = List::new(left_items).highlight_style(left_style);
        f.render_stateful_widget(left_list, cols[0], &mut left_state);

        let right_raw = self.items_for(self.selected_category);
        let right_items: Vec<ListItem> = right_raw
            .iter()
            .enumerate()
            .map(|(i, text)| {
                let label = decorate_with_state(self.selected_category, i, text, config);
                ListItem::new(label)
            })
            .collect();
        let mut right_state = ListState::default();
        if !right_raw.is_empty() {
            right_state.select(Some(self.selected_item));
        }
        let right_style = if self.panel == Panel::Right {
            Style::default().bg(Color::White).fg(Color::Black).add_modifier(Modifier::BOLD)
        } else {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        };
        let right_list = List::new(right_items).highlight_style(right_style);
        f.render_stateful_widget(right_list, cols[1], &mut right_state);

        let hint = self.message.clone().unwrap_or_else(|| {
            "left/right:パネル切替  up/down:項目選択  Enter:決定/切替".to_string()
        });
        let bottom = Paragraph::new(Line::from(Span::styled(
            format!(" {}", hint),
            Style::default().fg(Color::Black).bg(Color::White),
        )));
        f.render_widget(bottom, chunks[1]);
    }
}

fn on_off(b: bool) -> &'static str {
    if b {
        "ON"
    } else {
        "OFF"
    }
}

fn decorate_with_state(category: usize, item: usize, text: &str, config: &Config) -> String {
    let state = match (category, item) {
        (1, 0) => Some(config.show_line_numbers),
        (1, 2) => Some(config.ensure_trailing_newline),
        (2, 0) => Some(config.auto_open_on_select),
        (2, 1) => Some(config.show_hidden_files),
        _ => None,
    };
    match state {
        Some(b) => format!("[{}] {}", on_off(b), text),
        None => text.to_string(),
    }
}
