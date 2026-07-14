use crate::config::Config;
use crate::i18n::{self, Lang};
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
    pub selected_category: usize,
    pub selected_item: usize,
    pub panel: Panel,
    pub message: Option<String>,
}

impl Menu {
    pub fn new() -> Self {
        Menu {
            selected_category: 0,
            selected_item: 0,
            panel: Panel::Left,
            message: None,
        }
    }

    fn items_for(&self, category: usize, lang: Lang) -> Vec<String> {
        match category {
            0 => vec![
                i18n::menu_item_general_hint(lang).to_string(),
                i18n::menu_item_language(lang).to_string(),
            ],
            1 => vec![
                i18n::menu_item_line_numbers(lang).to_string(),
                i18n::menu_item_tab_width(lang).to_string(),
                i18n::menu_item_trailing_newline(lang).to_string(),
            ],
            2 => vec![
                i18n::menu_item_auto_open(lang).to_string(),
                i18n::menu_item_show_hidden(lang).to_string(),
            ],
            3 => i18n::menu_keybinds(lang),
            4 => vec![i18n::menu_item_open_config(lang).to_string()],
            5 => i18n::menu_about(lang),
            _ => vec![],
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent, config: &mut Config) -> MenuAction {
        let lang = Lang::from_str(&config.language);
        let categories_len = i18n::menu_categories(lang).len();
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
                    if self.selected_category + 1 < categories_len {
                        self.selected_category += 1;
                        self.selected_item = 0;
                    }
                }
                Panel::Right => {
                    let max = self.items_for(self.selected_category, lang).len();
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
                return self.apply_selection(config, lang);
            }
            _ => {}
        }
        MenuAction::None
    }

    fn apply_selection(&mut self, config: &mut Config, lang: Lang) -> MenuAction {
        match self.selected_category {
            0 => {
                if self.selected_item == 1 {
                    let new_lang = lang.toggle();
                    config.language = new_lang.as_str().to_string();
                    self.message = Some(format!(
                        "{}: {}",
                        i18n::menu_item_language(new_lang),
                        new_lang.label()
                    ));
                }
            }
            1 => match self.selected_item {
                0 => {
                    config.show_line_numbers = !config.show_line_numbers;
                    self.message = Some(format!(
                        "{}: {}",
                        i18n::menu_item_line_numbers(lang),
                        i18n::on_off(lang, config.show_line_numbers)
                    ));
                }
                1 => {
                    config.tab_width = if config.tab_width >= 8 { 1 } else { config.tab_width + 1 };
                    self.message = Some(format!("{}: {}", i18n::menu_item_tab_width(lang), config.tab_width));
                }
                2 => {
                    config.ensure_trailing_newline = !config.ensure_trailing_newline;
                    self.message = Some(format!(
                        "{}: {}",
                        i18n::menu_item_trailing_newline(lang),
                        i18n::on_off(lang, config.ensure_trailing_newline)
                    ));
                }
                _ => {}
            },
            2 => match self.selected_item {
                0 => {
                    config.auto_open_on_select = !config.auto_open_on_select;
                    self.message = Some(format!(
                        "{}: {}",
                        i18n::menu_item_auto_open(lang),
                        i18n::on_off(lang, config.auto_open_on_select)
                    ));
                }
                1 => {
                    config.show_hidden_files = !config.show_hidden_files;
                    self.message = Some(format!(
                        "{}: {}",
                        i18n::menu_item_show_hidden(lang),
                        i18n::on_off(lang, config.show_hidden_files)
                    ));
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
        let lang = Lang::from_str(&config.language);
        let categories = i18n::menu_categories(lang);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(area);

        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(chunks[0]);

        let left_items: Vec<ListItem> = categories.iter().map(|c| ListItem::new(*c)).collect();
        let mut left_state = ListState::default();
        left_state.select(Some(self.selected_category));
        let left_style = if self.panel == Panel::Left {
            Style::default().bg(Color::White).fg(Color::Black).add_modifier(Modifier::BOLD)
        } else {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        };
        let left_list = List::new(left_items).highlight_style(left_style);
        f.render_stateful_widget(left_list, cols[0], &mut left_state);

        let right_raw = self.items_for(self.selected_category, lang);
        let right_items: Vec<ListItem> = right_raw
            .iter()
            .enumerate()
            .map(|(i, text)| {
                let label = decorate_with_state(self.selected_category, i, text, config, lang);
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

        let hint = self.message.clone().unwrap_or_else(|| i18n::menu_hint(lang).to_string());
        let bottom = Paragraph::new(Line::from(Span::styled(
            format!(" {}", hint),
            Style::default().fg(Color::Black).bg(Color::White),
        )));
        f.render_widget(bottom, chunks[1]);
    }
}

fn decorate_with_state(category: usize, item: usize, text: &str, config: &Config, lang: Lang) -> String {
    let state = match (category, item) {
        (1, 0) => Some(config.show_line_numbers),
        (1, 2) => Some(config.ensure_trailing_newline),
        (2, 0) => Some(config.auto_open_on_select),
        (2, 1) => Some(config.show_hidden_files),
        _ => None,
    };
    match state {
        Some(b) => format!("[{}] {}", i18n::on_off(lang, b), text),
        None if category == 0 && item == 1 => {
            format!("[{}] {}", lang.label(), text)
        }
        None => text.to_string(),
    }
}
