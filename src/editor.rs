use crate::i18n::{self, Lang};
use crate::status::StatusMsg;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use std::fs;
use std::path::PathBuf;
use unicode_width::UnicodeWidthStr;

/// 現在エディタが待ち受けている入力モード。
/// nano の下部プロンプト(検索/置換/別名保存/行ジャンプ/ファイル挿入)を再現する。
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Normal,
    Search,
    ReplaceFrom,
    ReplaceTo,
    GotoLine,
    SaveAs,
    InsertFile,
}

pub enum EditorAction {
    None,
    Quit,
    /// Ctrl+G (ヘルプ): メニュー画面のキー操作一覧へ切り替えてほしいという合図
    ShowHelp,
}

/// nano を参考にしたテキストエディタ本体。
/// 日本語などのIME入力はOS側に任せ、ここでは確定済み文字を1文字ずつ受け取る。
pub struct Editor {
    pub lines: Vec<String>,
    pub path: Option<PathBuf>,
    pub cursor_row: usize,
    pub cursor_col: usize, // 文字単位(バイトではなくchar数)のカーソル位置
    pub scroll_row: usize,
    pub dirty: bool,
    pub message: Option<StatusMsg>,
    /// Ctrl+X (終了) の2段階確認用フラグ
    confirm_quit: bool,
    mode: Mode,
    input_buffer: String,
    /// 置換機能で「検索文字列」を一時保持するためのバッファ
    replace_from: String,
    /// Ctrl+W で最後に検索した文字列(Enterのみで再検索するため)
    last_search: String,
    /// Ctrl+K で切り取った行を保持するカット&ペースト用バッファ
    cut_buffer: Vec<String>,
    /// 直前の操作がCtrl+Kだったか(連続カットで1つのバッファにまとめるため)
    last_was_cut: bool,
}

impl Editor {
    pub fn new() -> Self {
        Editor {
            lines: vec![String::new()],
            path: None,
            cursor_row: 0,
            cursor_col: 0,
            scroll_row: 0,
            dirty: false,
            message: None,
            confirm_quit: false,
            mode: Mode::Normal,
            input_buffer: String::new(),
            replace_from: String::new(),
            last_search: String::new(),
            cut_buffer: Vec::new(),
            last_was_cut: false,
        }
    }

    pub fn open_file(&mut self, path: PathBuf, lang: Lang) {
        match fs::read_to_string(&path) {
            Ok(content) => {
                let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
                if lines.is_empty() {
                    lines.push(String::new());
                }
                self.lines = lines;
                self.path = Some(path);
                self.cursor_row = 0;
                self.cursor_col = 0;
                self.scroll_row = 0;
                self.dirty = false;
                self.message = Some(StatusMsg::info(i18n::ed_loaded(lang)));
            }
            Err(e) => {
                // ファイルが存在しない場合は新規作成として空バッファで開く
                self.lines = vec![String::new()];
                self.path = Some(path);
                self.cursor_row = 0;
                self.cursor_col = 0;
                self.scroll_row = 0;
                self.dirty = false;
                self.message = Some(StatusMsg::error(i18n::ed_new_file_read_error(lang, &e.to_string())));
            }
        }
    }

    pub fn save(&mut self, lang: Lang) {
        let Some(path) = self.path.clone() else {
            // 保存先未指定の場合は別名保存プロンプトへ
            self.mode = Mode::SaveAs;
            self.input_buffer.clear();
            return;
        };
        self.write_to(path, lang);
    }

    fn write_to(&mut self, path: PathBuf, lang: Lang) {
        let mut content = self.lines.join("\n");
        content.push('\n');
        match fs::write(&path, content) {
            Ok(_) => {
                self.dirty = false;
                let display = path.display().to_string();
                self.path = Some(path);
                self.message = Some(StatusMsg::info(i18n::ed_saved(lang, &display)));
            }
            Err(e) => {
                self.message = Some(StatusMsg::error(i18n::ed_save_failed(lang, &e.to_string())));
            }
        }
    }

    fn current_line_char_len(&self) -> usize {
        self.lines[self.cursor_row].chars().count()
    }

    fn insert_str_at_cursor(&mut self, text: &str) {
        let mut inserted_lines: Vec<&str> = text.split('\n').collect();
        if inserted_lines.is_empty() {
            inserted_lines.push("");
        }
        let line = self.lines[self.cursor_row].clone();
        let byte_idx = char_to_byte_idx(&line, self.cursor_col);
        let (left, right) = line.split_at(byte_idx);

        if inserted_lines.len() == 1 {
            let mut new_line = String::new();
            new_line.push_str(left);
            new_line.push_str(inserted_lines[0]);
            new_line.push_str(right);
            self.lines[self.cursor_row] = new_line;
            self.cursor_col += inserted_lines[0].chars().count();
        } else {
            let last_idx = inserted_lines.len() - 1;
            let mut first = String::new();
            first.push_str(left);
            first.push_str(inserted_lines[0]);
            let mut last = String::new();
            last.push_str(inserted_lines[last_idx]);
            last.push_str(right);

            let mut new_lines: Vec<String> = Vec::new();
            new_lines.push(first);
            for mid in &inserted_lines[1..last_idx] {
                new_lines.push(mid.to_string());
            }
            new_lines.push(last);

            self.lines.splice(self.cursor_row..=self.cursor_row, new_lines);
            self.cursor_row += last_idx;
            self.cursor_col = inserted_lines[last_idx].chars().count();
        }
        self.dirty = true;
    }

    /// 現在位置から前方検索を行い、見つかれば true を返してカーソルを移動する。
    fn search_forward(&mut self, needle: &str) -> bool {
        if needle.is_empty() {
            return false;
        }
        let total = self.lines.len();
        // 現在行は現在カーソルより後ろから、以降の行は先頭から、最後にwrapして現在行の先頭〜カーソルまで
        for offset in 0..=total {
            let row = (self.cursor_row + offset) % total;
            let line = &self.lines[row];
            let start_char = if offset == 0 { self.cursor_col + 1 } else { 0 };
            let chars: Vec<char> = line.chars().collect();
            if start_char > chars.len() {
                continue;
            }
            let substr: String = chars[start_char..].iter().collect();
            if let Some(pos) = substr.find(needle) {
                let char_pos = substr[..pos].chars().count() + start_char;
                self.cursor_row = row;
                self.cursor_col = char_pos;
                return true;
            }
            if offset == total {
                // 完全に一周し、現在行の先頭からカーソルまでも確認する
                let head: String = chars[..=self.cursor_col.min(chars.len().saturating_sub(1))]
                    .iter()
                    .collect();
                if let Some(pos) = head.find(needle) {
                    let char_pos = head[..pos].chars().count();
                    self.cursor_row = row;
                    self.cursor_col = char_pos;
                    return true;
                }
            }
        }
        false
    }

    fn replace_all(&mut self, from: &str, to: &str) -> usize {
        if from.is_empty() {
            return 0;
        }
        let mut count = 0;
        for line in self.lines.iter_mut() {
            let matches = line.matches(from).count();
            if matches > 0 {
                *line = line.replace(from, to);
                count += matches;
            }
        }
        if count > 0 {
            self.dirty = true;
        }
        count
    }

    fn cut_line(&mut self) {
        if self.lines.len() > 1 {
            let removed = self.lines.remove(self.cursor_row);
            if self.last_was_cut {
                self.cut_buffer.push(removed);
            } else {
                self.cut_buffer = vec![removed];
            }
            if self.cursor_row >= self.lines.len() {
                self.cursor_row = self.lines.len() - 1;
            }
        } else {
            let removed = std::mem::take(&mut self.lines[0]);
            if self.last_was_cut {
                self.cut_buffer.push(removed);
            } else {
                self.cut_buffer = vec![removed];
            }
        }
        self.cursor_col = 0;
        self.dirty = true;
        self.last_was_cut = true;
    }

    fn paste_cut_buffer(&mut self, lang: Lang) {
        if self.cut_buffer.is_empty() {
            self.message = Some(StatusMsg::error(i18n::ed_no_clipboard(lang)));
            return;
        }
        let text = self.cut_buffer.join("\n");
        self.insert_str_at_cursor(&text);
        self.message = Some(StatusMsg::info(i18n::ed_pasted(lang)));
    }

    /// モード中(検索/置換/別名保存等)の入力プロンプトの処理。
    fn handle_mode_key(&mut self, key: KeyEvent, lang: Lang) -> EditorAction {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.input_buffer.clear();
                self.message = Some(StatusMsg::info(i18n::ed_cancelled(lang)));
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
            }
            KeyCode::Enter => {
                let value = self.input_buffer.clone();
                self.input_buffer.clear();
                match self.mode {
                    Mode::Search => {
                        self.mode = Mode::Normal;
                        let needle = if value.is_empty() { self.last_search.clone() } else { value };
                        if needle.is_empty() {
                            self.message = Some(StatusMsg::error(i18n::ed_search_empty(lang)));
                        } else {
                            self.last_search = needle.clone();
                            if self.search_forward(&needle) {
                                self.message = Some(StatusMsg::info(i18n::ed_search_found(lang, &needle)));
                            } else {
                                self.message = Some(StatusMsg::error(i18n::ed_search_not_found(lang, &needle)));
                            }
                        }
                    }
                    Mode::ReplaceFrom => {
                        if value.is_empty() {
                            self.mode = Mode::Normal;
                            self.message = Some(StatusMsg::error(i18n::ed_search_empty(lang)));
                        } else {
                            self.replace_from = value;
                            self.mode = Mode::ReplaceTo;
                        }
                    }
                    Mode::ReplaceTo => {
                        self.mode = Mode::Normal;
                        let from = self.replace_from.clone();
                        let count = self.replace_all(&from, &value);
                        self.message = Some(StatusMsg::info(i18n::ed_replaced(lang, &from, &value, count)));
                    }
                    Mode::GotoLine => {
                        self.mode = Mode::Normal;
                        match value.trim().parse::<usize>() {
                            Ok(n) if n >= 1 && n <= self.lines.len() => {
                                self.cursor_row = n - 1;
                                self.cursor_col = 0;
                                self.message = Some(StatusMsg::info(i18n::ed_goto_ok(lang, n)));
                            }
                            _ => {
                                self.message =
                                    Some(StatusMsg::error(i18n::ed_goto_invalid(lang, self.lines.len())));
                            }
                        }
                    }
                    Mode::SaveAs => {
                        self.mode = Mode::Normal;
                        if value.trim().is_empty() {
                            self.message = Some(StatusMsg::error(i18n::ed_filename_empty(lang)));
                        } else {
                            self.write_to(PathBuf::from(value.trim()), lang);
                        }
                    }
                    Mode::InsertFile => {
                        self.mode = Mode::Normal;
                        let path = PathBuf::from(value.trim());
                        match fs::read_to_string(&path) {
                            Ok(content) => {
                                self.insert_str_at_cursor(&content);
                                self.message =
                                    Some(StatusMsg::info(i18n::ed_inserted(lang, &path.display().to_string())));
                            }
                            Err(e) => {
                                self.message =
                                    Some(StatusMsg::error(i18n::ed_insert_failed(lang, &e.to_string())));
                            }
                        }
                    }
                    Mode::Normal => {}
                }
            }
            _ => {}
        }
        EditorAction::None
    }

    pub fn handle_key(&mut self, key: KeyEvent, lang: Lang) -> EditorAction {
        if self.mode != Mode::Normal {
            return self.handle_mode_key(key, lang);
        }

        // Ctrl+X 以外を押したら終了確認はキャンセル
        if key.code != KeyCode::Char('x') {
            self.confirm_quit = false;
        }
        // Ctrl+K 以外を押したら連続カットの連結をリセット
        if key.code != KeyCode::Char('k') {
            self.last_was_cut = false;
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('s') => {
                    self.save(lang);
                    return EditorAction::None;
                }
                KeyCode::Char('o') => {
                    // 別名で保存 (Write Out)
                    self.mode = Mode::SaveAs;
                    self.input_buffer = self
                        .path
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_default();
                    return EditorAction::None;
                }
                KeyCode::Char('r') => {
                    // 指定ファイルをカーソル位置に挿入 (Read File)
                    self.mode = Mode::InsertFile;
                    self.input_buffer.clear();
                    return EditorAction::None;
                }
                KeyCode::Char('w') => {
                    // 検索 (Where Is)
                    self.mode = Mode::Search;
                    self.input_buffer.clear();
                    return EditorAction::None;
                }
                KeyCode::Char('\\') | KeyCode::Char('4') => {
                    // 置換 (Replace / nano の Ctrl+\ 相当。端末はFSバイトを送るため '4' としても解釈される)
                    self.mode = Mode::ReplaceFrom;
                    self.input_buffer.clear();
                    return EditorAction::None;
                }
                KeyCode::Char('_') | KeyCode::Char('7') => {
                    // 指定行へジャンプ (Go To Line / nano の Ctrl+_ 相当。端末はUSバイトを送るため '7' としても解釈される)
                    self.mode = Mode::GotoLine;
                    self.input_buffer.clear();
                    return EditorAction::None;
                }
                KeyCode::Char('g') => {
                    return EditorAction::ShowHelp;
                }
                KeyCode::Char('c') => {
                    self.message = Some(StatusMsg::info(i18n::ed_cursor_pos(
                        lang,
                        self.cursor_row + 1,
                        self.cursor_col + 1,
                        self.lines.len(),
                    )));
                    return EditorAction::None;
                }
                KeyCode::Char('x') => {
                    if !self.dirty || self.confirm_quit {
                        return EditorAction::Quit;
                    } else {
                        self.confirm_quit = true;
                        self.message = Some(StatusMsg::error(i18n::ed_confirm_quit(lang)));
                        return EditorAction::None;
                    }
                }
                KeyCode::Char('k') => {
                    self.cut_line();
                    return EditorAction::None;
                }
                KeyCode::Char('u') => {
                    self.paste_cut_buffer(lang);
                    return EditorAction::None;
                }
                _ => return EditorAction::None,
            }
        }

        // 通常のキー入力ではメッセージは都度クリアする(古いエラーが残り続けるのを防ぐ)
        self.message = None;

        match key.code {
            KeyCode::Char(c) => {
                let line = &mut self.lines[self.cursor_row];
                let byte_idx = char_to_byte_idx(line, self.cursor_col);
                line.insert(byte_idx, c);
                self.cursor_col += 1;
                self.dirty = true;
            }
            KeyCode::Enter => {
                let line = self.lines[self.cursor_row].clone();
                let byte_idx = char_to_byte_idx(&line, self.cursor_col);
                let (left, right) = line.split_at(byte_idx);
                self.lines[self.cursor_row] = left.to_string();
                self.lines.insert(self.cursor_row + 1, right.to_string());
                self.cursor_row += 1;
                self.cursor_col = 0;
                self.dirty = true;
            }
            KeyCode::Backspace => {
                if self.cursor_col > 0 {
                    let line = &mut self.lines[self.cursor_row];
                    let byte_idx = char_to_byte_idx(line, self.cursor_col - 1);
                    line.remove(byte_idx);
                    self.cursor_col -= 1;
                    self.dirty = true;
                } else if self.cursor_row > 0 {
                    let cur = self.lines.remove(self.cursor_row);
                    self.cursor_row -= 1;
                    self.cursor_col = self.current_line_char_len();
                    self.lines[self.cursor_row].push_str(&cur);
                    self.dirty = true;
                }
            }
            KeyCode::Delete => {
                let len = self.current_line_char_len();
                if self.cursor_col < len {
                    let line = &mut self.lines[self.cursor_row];
                    let byte_idx = char_to_byte_idx(line, self.cursor_col);
                    line.remove(byte_idx);
                    self.dirty = true;
                } else if self.cursor_row + 1 < self.lines.len() {
                    let next = self.lines.remove(self.cursor_row + 1);
                    self.lines[self.cursor_row].push_str(&next);
                    self.dirty = true;
                }
            }
            KeyCode::Left => {
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                } else if self.cursor_row > 0 {
                    self.cursor_row -= 1;
                    self.cursor_col = self.current_line_char_len();
                }
            }
            KeyCode::Right => {
                let len = self.current_line_char_len();
                if self.cursor_col < len {
                    self.cursor_col += 1;
                } else if self.cursor_row + 1 < self.lines.len() {
                    self.cursor_row += 1;
                    self.cursor_col = 0;
                }
            }
            KeyCode::Up => {
                if self.cursor_row > 0 {
                    self.cursor_row -= 1;
                    self.cursor_col = self.cursor_col.min(self.current_line_char_len());
                }
            }
            KeyCode::Down => {
                if self.cursor_row + 1 < self.lines.len() {
                    self.cursor_row += 1;
                    self.cursor_col = self.cursor_col.min(self.current_line_char_len());
                }
            }
            KeyCode::Home => self.cursor_col = 0,
            KeyCode::End => self.cursor_col = self.current_line_char_len(),
            KeyCode::PageUp => {
                self.cursor_row = self.cursor_row.saturating_sub(10);
                self.cursor_col = self.cursor_col.min(self.current_line_char_len());
            }
            KeyCode::PageDown => {
                self.cursor_row = (self.cursor_row + 10).min(self.lines.len() - 1);
                self.cursor_col = self.cursor_col.min(self.current_line_char_len());
            }
            KeyCode::Tab => {
                let line = &mut self.lines[self.cursor_row];
                let byte_idx = char_to_byte_idx(line, self.cursor_col);
                line.insert(byte_idx, '\t');
                self.cursor_col += 1;
                self.dirty = true;
            }
            _ => {}
        }
        EditorAction::None
    }

    fn adjust_scroll(&mut self, visible_rows: usize) {
        if self.cursor_row < self.scroll_row {
            self.scroll_row = self.cursor_row;
        } else if self.cursor_row >= self.scroll_row + visible_rows {
            self.scroll_row = self.cursor_row + 1 - visible_rows;
        }
    }

    fn mode_prompt_label(&self, lang: Lang) -> Option<&'static str> {
        match self.mode {
            Mode::Normal => None,
            Mode::Search => Some(i18n::ed_prompt_search(lang)),
            Mode::ReplaceFrom => Some(i18n::ed_prompt_replace_from(lang)),
            Mode::ReplaceTo => Some(i18n::ed_prompt_replace_to(lang)),
            Mode::GotoLine => Some(i18n::ed_prompt_goto(lang)),
            Mode::SaveAs => Some(i18n::ed_prompt_saveas(lang)),
            Mode::InsertFile => Some(i18n::ed_prompt_insertfile(lang)),
        }
    }

    pub fn draw(&mut self, f: &mut Frame, area: Rect, show_line_numbers: bool, lang: Lang) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1), Constraint::Length(2)])
            .split(area);

        let text_area = chunks[0];
        let visible_rows = text_area.height as usize;
        self.adjust_scroll(visible_rows.max(1));

        let gutter_width = if show_line_numbers {
            self.lines.len().to_string().len().max(3) + 1
        } else {
            0
        };

        let mut lines: Vec<Line> = Vec::new();
        for (i, raw) in self
            .lines
            .iter()
            .enumerate()
            .skip(self.scroll_row)
            .take(visible_rows)
        {
            let rendered = raw.replace('\t', "    ");
            if show_line_numbers {
                let num = format!("{:>width$} ", i + 1, width = gutter_width - 1);
                lines.push(Line::from(vec![
                    Span::styled(num, Style::default().fg(Color::DarkGray)),
                    Span::raw(rendered),
                ]));
            } else {
                lines.push(Line::from(Span::raw(rendered)));
            }
        }

        let paragraph = Paragraph::new(lines);
        f.render_widget(paragraph, text_area);

        // カーソル位置を実際の端末カーソルに反映(プロンプト表示中は下部の入力位置へ)
        if self.mode == Mode::Normal {
            let cursor_screen_row = (self.cursor_row - self.scroll_row) as u16;
            let cursor_screen_col = gutter_width as u16
                + display_width(&self.lines[self.cursor_row], self.cursor_col);
            f.set_cursor(
                text_area.x + cursor_screen_col,
                text_area.y + cursor_screen_row,
            );
        }

        // ステータス行 (nano風、反転表示。エラー時は赤)
        let status_bar = if let Some(label) = self.mode_prompt_label(lang) {
            Paragraph::new(Line::from(Span::styled(
                format!(" {}: {}", label, self.input_buffer),
                Style::default().fg(Color::Black).bg(Color::White),
            )))
        } else {
            let name = self
                .path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| i18n::ed_new_buffer(lang).to_string());
            let dirty_mark = if self.dirty { i18n::ed_dirty_mark(lang) } else { "" };
            let msg = self.message.clone().unwrap_or_else(|| {
                StatusMsg::info(i18n::ed_status_line(
                    lang,
                    &name,
                    dirty_mark,
                    self.cursor_row + 1,
                    self.lines.len(),
                ))
            });
            Paragraph::new(Line::from(Span::styled(format!(" {} ", msg.text), msg.style())))
        };
        f.render_widget(status_bar, chunks[1]);

        // ショートカット一覧 (nano/pico風、2行x6列のグリッド)
        draw_shortcut_grid(f, chunks[2], lang);
    }
}

/// nano/pico を参考にした、下部2行x6列のキーショートカット一覧を描画する。
fn draw_shortcut_grid(f: &mut Frame, area: Rect, lang: Lang) {
    let entries = i18n::ed_shortcut_grid(lang);
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 6); 6])
        .split(area);

    for (col_idx, col_area) in cols.iter().enumerate() {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(1)])
            .split(*col_area);

        for row_idx in 0..2 {
            let entry_idx = col_idx * 2 + row_idx;
            if let Some((key, label)) = entries.get(entry_idx) {
                if key.is_empty() {
                    continue;
                }
                let line = Line::from(vec![
                    Span::styled(format!("{}", key), Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(format!(" {}", label)),
                ]);
                f.render_widget(Paragraph::new(line), rows[row_idx]);
            }
        }
    }
}

fn char_to_byte_idx(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(b, _)| b)
        .unwrap_or(s.len())
}

fn display_width(s: &str, char_idx: usize) -> u16 {
    let prefix: String = s.chars().take(char_idx).collect();
    let prefix = prefix.replace('\t', "    ");
    UnicodeWidthStr::width(prefix.as_str()) as u16
}
