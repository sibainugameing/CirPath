use crate::app::{App, Window};
use crate::browser::BrowserPrompt;
use crate::editor::Prompt;
use crate::menu::MenuFocus;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

const TAB_TITLES: [&str; 3] = ["エディター", "ファイルブラウザ", "メニュー"];

pub fn draw(f: &mut Frame, app: &mut App) {
    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // タブバー
            Constraint::Min(3),    // メインコンテンツ
            Constraint::Length(1), // グローバルヘルプ行
        ])
        .split(size);

    draw_tab_bar(f, app, chunks[0]);

    match app.active {
        Window::Editor => draw_editor(f, app, chunks[1]),
        Window::Browser => draw_browser(f, app, chunks[1]),
        Window::Menu => draw_menu(f, app, chunks[1]),
    }

    let help = Paragraph::new(app.global_message.clone())
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[2]);
}

fn draw_tab_bar(f: &mut Frame, app: &App, area: Rect) {
    let mut spans = Vec::new();
    for (i, title) in TAB_TITLES.iter().enumerate() {
        let is_active = matches!(
            (i, app.active),
            (0, Window::Editor) | (1, Window::Browser) | (2, Window::Menu)
        );
        let style = if is_active {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        spans.push(Span::styled(format!(" {} ", title), style));
        spans.push(Span::raw(" "));
    }
    let line = Line::from(spans);
    f.render_widget(Paragraph::new(line), area);
}

fn draw_editor(f: &mut Frame, app: &mut App, area: Rect) {
    let title = match &app.editor.file_path {
        Some(p) => format!(
            " CirPath Editor - {}{} ",
            p.display(),
            if app.editor.dirty { " [変更あり]" } else { "" }
        ),
        None => " CirPath Editor - [新規バッファ] ".to_string(),
    };
    let block = Block::default().borders(Borders::ALL).title(title);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);

    let text_area = chunks[0];
    let viewport_height = text_area.height as usize;
    app.editor.adjust_scroll(viewport_height.max(1));

    let gutter_width = if app.editor.show_line_numbers {
        app.editor.lines.len().to_string().len().max(3) + 1
    } else {
        0
    };

    let mut lines: Vec<Line> = Vec::new();
    for (i, line) in app
        .editor
        .lines
        .iter()
        .enumerate()
        .skip(app.editor.scroll_row)
        .take(viewport_height)
    {
        if app.editor.show_line_numbers {
            let num = format!("{:>width$} ", i + 1, width = gutter_width - 1);
            lines.push(Line::from(vec![
                Span::styled(num, Style::default().fg(Color::DarkGray)),
                Span::raw(line.clone()),
            ]));
        } else {
            lines.push(Line::from(line.clone()));
        }
    }
    f.render_widget(Paragraph::new(lines), text_area);

    // カーソル位置を設定
    let cursor_screen_row = app.editor.cursor_row.saturating_sub(app.editor.scroll_row);
    let cursor_screen_col = gutter_width + app.editor.cursor_col;
    f.set_cursor(
        text_area.x + cursor_screen_col as u16,
        text_area.y + cursor_screen_row as u16,
    );

    let status_line = match &app.editor.prompt {
        Prompt::SaveAs(buf) => format!("保存先のパスを入力 (Enter確定 / Esc中止): {}", buf),
        Prompt::Search(buf) => format!("検索文字列を入力 (Enter確定 / Esc中止): {}", buf),
        Prompt::None => app.editor.status_message.clone(),
    };
    let status = Paragraph::new(status_line).style(Style::default().fg(Color::Yellow));
    f.render_widget(status, chunks[1]);
}

fn draw_browser(f: &mut Frame, app: &mut App, area: Rect) {
    let title = format!(" ファイルブラウザ - {} ", app.browser.current_dir.display());
    let block = Block::default().borders(Borders::ALL).title(title);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);

    let list_area = chunks[0];
    let viewport_height = list_area.height as usize;
    app.browser.ensure_visible(viewport_height.max(1));

    let items: Vec<ListItem> = app
        .browser
        .entries
        .iter()
        .enumerate()
        .skip(app.browser.scroll)
        .take(viewport_height)
        .map(|(i, entry)| {
            let icon = if entry.is_dir { "📁" } else { "📄" };
            let label = format!("{} {}", icon, entry.name);
            let style = if i == app.browser.selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if entry.is_dir {
                Style::default().fg(Color::LightBlue)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(label).style(style)
        })
        .collect();

    f.render_widget(List::new(items), list_area);

    let status_line = match &app.browser.prompt {
        BrowserPrompt::GotoPath(buf) => {
            format!("移動先パスを入力 (相対/絶対) (Enter確定 / Esc中止): {}", buf)
        }
        BrowserPrompt::None => {
            let selected_info = app
                .browser
                .selected_entry()
                .map(|e| {
                    if e.is_dir {
                        format!("[DIR] {}", e.name)
                    } else {
                        format!("[FILE] {}", e.name)
                    }
                })
                .unwrap_or_else(|| "(空のディレクトリ)".to_string());
            format!("{}  |  {}", selected_info, app.browser.status_message)
        }
    };
    let status = Paragraph::new(status_line).style(Style::default().fg(Color::Yellow));
    f.render_widget(status, chunks[1]);
}

fn draw_menu(f: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" メニュー ");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(inner);

    let cat_items: Vec<ListItem> = app
        .menu
        .categories
        .iter()
        .enumerate()
        .map(|(i, cat)| {
            let selected = i == app.menu.selected_category;
            let focused = selected && app.menu.focus == MenuFocus::Category;
            let style = if focused {
                Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else if selected {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(format!(" {}", cat.name)).style(style)
        })
        .collect();
    let cat_block = Block::default().borders(Borders::ALL).title("カテゴリ");
    f.render_widget(List::new(cat_items).block(cat_block), chunks[0]);

    let current_cat = &app.menu.categories[app.menu.selected_category];
    let item_items: Vec<ListItem> = current_cat
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let selected = i == app.menu.selected_item;
            let focused = selected && app.menu.focus == MenuFocus::Item;
            let style = if focused {
                Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD)
            } else if selected {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(format!(" {}", item.label)).style(style)
        })
        .collect();
    let item_block = Block::default().borders(Borders::ALL).title("項目 (Enterで実行)");
    f.render_widget(List::new(item_items).block(item_block), chunks[1]);

    let _ = Alignment::Left;
}
